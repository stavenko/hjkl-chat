use crate::models::chat::{Chat, ChatId, ChatMessage, MessageId, MessageRole};
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

    fn chat_meta_path(&self) -> String {
        format!("chats/{}/chat.yaml", self.chat_id)
    }

    fn messages_path(&self) -> String {
        format!("chats/{}/messages.yaml", self.chat_id)
    }

    fn draft_path(&self, message_id: &MessageId) -> String {
        format!("chats/{}/drafts/{}.yaml", self.chat_id, message_id)
    }

    pub async fn get_or_create_chat(&self, model: &str) -> ChatStorageResult<Chat> {
        let meta_path = self.chat_meta_path();
        if self
            .file_storage
            .exists(&meta_path)
            .await
            .map_err(ChatStorageError::S3)?
        {
            let data = self
                .file_storage
                .get(&meta_path)
                .await
                .map_err(ChatStorageError::S3)?;
            let yaml_str = String::from_utf8(data)?;
            let chat: Chat = serde_yaml::from_str(&yaml_str)?;
            Ok(chat)
        } else {
            let chat = Chat {
                id: self.chat_id,
                user_id: self.file_storage.user_id(),
                title: "New Chat".to_string(),
                model: model.to_string(),
                created_at: Utc::now(),
                messages: Vec::new(),
            };
            let yaml = serde_yaml::to_string(&chat)?;
            self.file_storage
                .put(&meta_path, yaml.as_bytes())
                .await
                .map_err(ChatStorageError::S3)?;
            Ok(chat)
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
        self.file_storage
            .put(&path, yaml.as_bytes())
            .await
            .map_err(ChatStorageError::S3)?;
        Ok(())
    }

    pub async fn send_message(&self, message_id: MessageId) -> ChatStorageResult<ChatMessage> {
        let draft_path = self.draft_path(&message_id);
        if !self
            .file_storage
            .exists(&draft_path)
            .await
            .map_err(ChatStorageError::S3)?
        {
            return Err(ChatStorageError::DraftNotFound);
        }

        let data = self
            .file_storage
            .get(&draft_path)
            .await
            .map_err(ChatStorageError::S3)?;
        let yaml_str = String::from_utf8(data)?;
        let draft: ChatMessage = serde_yaml::from_str(&yaml_str)?;

        let mut messages = self.load_messages().await?;
        messages.push(draft.clone());
        self.save_messages(&messages).await?;

        self.update_title_if_needed(&messages).await?;

        self.file_storage
            .delete(&draft_path)
            .await
            .map_err(ChatStorageError::S3)?;

        Ok(draft)
    }

    pub async fn save_assistant_message(
        &self,
        message: ChatMessage,
    ) -> ChatStorageResult<()> {
        let mut messages = self.load_messages().await?;
        messages.push(message);
        self.save_messages(&messages).await?;
        Ok(())
    }

    pub async fn get_last_n(&self, n: usize) -> ChatStorageResult<Vec<ChatMessage>> {
        let messages = self.load_messages().await?;
        if n >= messages.len() {
            Ok(messages)
        } else {
            Ok(messages[messages.len() - n..].to_vec())
        }
    }

    pub async fn get_all_messages(&self) -> ChatStorageResult<Vec<ChatMessage>> {
        self.load_messages().await
    }

    pub async fn get_message_count(&self) -> ChatStorageResult<usize> {
        let messages = self.load_messages().await?;
        Ok(messages.len())
    }

    async fn load_messages(&self) -> ChatStorageResult<Vec<ChatMessage>> {
        let path = self.messages_path();
        if !self
            .file_storage
            .exists(&path)
            .await
            .map_err(ChatStorageError::S3)?
        {
            return Ok(Vec::new());
        }
        let data = self
            .file_storage
            .get(&path)
            .await
            .map_err(ChatStorageError::S3)?;
        let yaml_str = String::from_utf8(data)?;
        let messages: Vec<ChatMessage> = serde_yaml::from_str(&yaml_str)?;
        Ok(messages)
    }

    async fn save_messages(&self, messages: &[ChatMessage]) -> ChatStorageResult<()> {
        let yaml = serde_yaml::to_string(messages)?;
        let path = self.messages_path();
        self.file_storage
            .put(&path, yaml.as_bytes())
            .await
            .map_err(ChatStorageError::S3)?;
        Ok(())
    }

    async fn update_title_if_needed(
        &self,
        messages: &[ChatMessage],
    ) -> ChatStorageResult<()> {
        let meta_path = self.chat_meta_path();
        if !self
            .file_storage
            .exists(&meta_path)
            .await
            .map_err(ChatStorageError::S3)?
        {
            return Ok(());
        }

        let data = self
            .file_storage
            .get(&meta_path)
            .await
            .map_err(ChatStorageError::S3)?;
        let yaml_str = String::from_utf8(data)?;
        let mut chat: Chat = serde_yaml::from_str(&yaml_str)?;

        if chat.title == "New Chat" {
            if let Some(first_user_msg) = messages
                .iter()
                .find(|m| matches!(m.role, MessageRole::User))
            {
                let title: String = first_user_msg.content.chars().take(50).collect();
                chat.title = if title.len() < first_user_msg.content.len() {
                    format!("{}...", title)
                } else {
                    title
                };
                let yaml = serde_yaml::to_string(&chat)?;
                self.file_storage
                    .put(&meta_path, yaml.as_bytes())
                    .await
                    .map_err(ChatStorageError::S3)?;
            }
        }
        Ok(())
    }
}
