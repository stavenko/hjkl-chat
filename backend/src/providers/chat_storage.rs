use crate::models::chat::{ChatId, ChatIndex, ChatMessage, ChatMeta, MessageId, MessageRole};
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use chrono::Utc;

#[derive(thiserror::Error, Debug)]
pub enum ChatStorageError {
    #[error("S3 error: {0}")]
    S3(crate::providers::s3::S3ProviderError),
    #[error("YAML serialization error: {0}")]
    YamlSerialize(#[from] serde_yaml::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Chat not found")]
    NotFound,
    #[error("Draft not found")]
    DraftNotFound,
}

impl From<crate::providers::s3::S3ProviderError> for ChatStorageError {
    fn from(e: crate::providers::s3::S3ProviderError) -> Self {
        ChatStorageError::S3(e)
    }
}

pub type ChatStorageResult<T> = Result<T, ChatStorageError>;

pub struct ChatStorage {
    file_storage: PersonalizedFileStorage,
    chat_id: ChatId,
}

impl ChatStorage {
    pub fn new(file_storage: PersonalizedFileStorage, chat_id: ChatId) -> Self {
        ChatStorage {
            file_storage,
            chat_id,
        }
    }

    pub fn get_id(&self) -> ChatId {
        self.chat_id
    }

    fn chat_index_path(&self) -> String {
        format!("chats/{}/chat.yaml", self.chat_id)
    }

    fn chat_meta_path(&self) -> String {
        format!("chats/{}/chat-meta.yaml", self.chat_id)
    }

    fn message_path(&self, message_id: &MessageId) -> String {
        format!("chats/{}/messages/{}.yaml", self.chat_id, message_id)
    }

    fn draft_path(&self, message_id: &MessageId) -> String {
        format!("chats/{}/drafts/{}.yaml", self.chat_id, message_id)
    }

    pub async fn get_or_create_chat(&self, model: &str) -> ChatStorageResult<ChatIndex> {
        let index_path = self.chat_index_path();
        if self.file_storage.exists(&index_path).await? {
            let data = self.file_storage.get(&index_path).await?;
            let yaml_str = String::from_utf8(data)?;
            let index: ChatIndex = serde_yaml::from_str(&yaml_str)?;
            Ok(index)
        } else {
            let index = ChatIndex {
                id: self.chat_id,
                model: model.to_string(),
                message_ids: Vec::new(),
            };
            let yaml = serde_yaml::to_string(&index)?;
            self.file_storage.put(&index_path, yaml.as_bytes()).await?;

            let meta = ChatMeta {
                created_at: Utc::now(),
            };
            let meta_yaml = serde_yaml::to_string(&meta)?;
            let meta_path = self.chat_meta_path();
            self.file_storage.put(&meta_path, meta_yaml.as_bytes()).await?;

            Ok(index)
        }
    }

    pub async fn save_draft(
        &self,
        message_id: MessageId,
        content: &str,
    ) -> ChatStorageResult<()> {
        let draft = ChatMessage {
            id: message_id,
            role: MessageRole::User,
            content: content.to_string(),
            reasoning: None,
            created_at: Utc::now(),
        };
        let yaml = serde_yaml::to_string(&draft)?;
        let path = self.draft_path(&message_id);
        self.file_storage.put(&path, yaml.as_bytes()).await?;
        Ok(())
    }

    pub async fn get_draft(
        &self,
        message_id: MessageId,
    ) -> ChatStorageResult<ChatMessage> {
        let path = self.draft_path(&message_id);
        if !self.file_storage.exists(&path).await? {
            return Err(ChatStorageError::DraftNotFound);
        }
        let data = self.file_storage.get(&path).await?;
        let yaml_str = String::from_utf8(data)?;
        let draft: ChatMessage = serde_yaml::from_str(&yaml_str)?;
        Ok(draft)
    }

    pub async fn send_message(&self, message_id: MessageId) -> ChatStorageResult<ChatMessage> {
        let draft_path = self.draft_path(&message_id);
        if !self.file_storage.exists(&draft_path).await? {
            return Err(ChatStorageError::DraftNotFound);
        }

        let data = self.file_storage.get(&draft_path).await?;
        let yaml_str = String::from_utf8(data)?;
        let draft: ChatMessage = serde_yaml::from_str(&yaml_str)?;

        // Save as individual message file
        let msg_path = self.message_path(&message_id);
        let msg_yaml = serde_yaml::to_string(&draft)?;
        self.file_storage.put(&msg_path, msg_yaml.as_bytes()).await?;

        // Append to chat index
        self.append_message_id(message_id).await?;

        // Delete the draft
        self.file_storage.delete(&draft_path).await?;

        Ok(draft)
    }

    pub async fn save_assistant_message(
        &self,
        message: ChatMessage,
    ) -> ChatStorageResult<()> {
        let msg_path = self.message_path(&message.id);
        let yaml = serde_yaml::to_string(&message)?;
        self.file_storage.put(&msg_path, yaml.as_bytes()).await?;

        self.append_message_id(message.id).await?;

        Ok(())
    }

    pub async fn get_last_n(&self, n: usize) -> ChatStorageResult<Vec<ChatMessage>> {
        let index = self.load_index().await?;
        let ids = &index.message_ids;
        let start = if n >= ids.len() { 0 } else { ids.len() - n };
        let mut messages = Vec::new();
        for id in &ids[start..] {
            let msg = self.load_message(id).await?;
            messages.push(msg);
        }
        Ok(messages)
    }

    pub async fn get_all_messages(&self) -> ChatStorageResult<Vec<ChatMessage>> {
        let index = self.load_index().await?;
        let mut messages = Vec::new();
        for id in &index.message_ids {
            let msg = self.load_message(id).await?;
            messages.push(msg);
        }
        Ok(messages)
    }

    async fn load_index(&self) -> ChatStorageResult<ChatIndex> {
        let path = self.chat_index_path();
        if !self.file_storage.exists(&path).await? {
            return Ok(ChatIndex {
                id: self.chat_id,
                model: String::new(),
                message_ids: Vec::new(),
            });
        }
        let data = self.file_storage.get(&path).await?;
        let yaml_str = String::from_utf8(data)?;
        let index: ChatIndex = serde_yaml::from_str(&yaml_str)?;
        Ok(index)
    }

    async fn load_message(&self, message_id: &MessageId) -> ChatStorageResult<ChatMessage> {
        let path = self.message_path(message_id);
        let data = self.file_storage.get(&path).await?;
        let yaml_str = String::from_utf8(data)?;
        let msg: ChatMessage = serde_yaml::from_str(&yaml_str)?;
        Ok(msg)
    }

    async fn append_message_id(&self, message_id: MessageId) -> ChatStorageResult<()> {
        let mut index = self.load_index().await?;
        index.message_ids.push(message_id);
        let yaml = serde_yaml::to_string(&index)?;
        let path = self.chat_index_path();
        self.file_storage.put(&path, yaml.as_bytes()).await?;
        Ok(())
    }
}
