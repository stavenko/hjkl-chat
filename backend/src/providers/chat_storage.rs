use crate::models::chat::{
    ChatFacts, ChatId, ChatIndex, ChatMessage, ChatMeta, MessageId, MessageRole, SyncAction,
    SyncEntityType,
};
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::providers::sync_ledger::SyncLedgerStorage;
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
    #[error("Version conflict: expected {expected}, server has {actual}")]
    VersionConflict { expected: u64, actual: u64 },
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

    fn sync_ledger(&self) -> SyncLedgerStorage {
        SyncLedgerStorage::new(self.file_storage.clone())
    }

    fn chat_index_path(&self) -> String {
        format!("chats/{}/chat.yaml", self.chat_id)
    }

    fn chat_meta_path(&self) -> String {
        format!("chats/{}/chat-meta.yaml", self.chat_id)
    }

    fn chat_facts_path(&self) -> String {
        format!("chats/{}/context.yaml", self.chat_id)
    }

    pub async fn get_chat_facts(&self) -> ChatStorageResult<Option<ChatFacts>> {
        let path = self.chat_facts_path();
        if !self.file_storage.exists(&path).await? {
            return Ok(None);
        }
        let data = self.file_storage.get(&path).await?;
        let yaml_str = String::from_utf8(data)?;
        let facts: ChatFacts = serde_yaml::from_str(&yaml_str)?;
        Ok(Some(facts))
    }

    pub async fn save_chat_facts(&self, facts: &ChatFacts) -> ChatStorageResult<()> {
        let path = self.chat_facts_path();
        let yaml = serde_yaml::to_string(facts)?;
        self.file_storage.put(&path, yaml.as_bytes()).await?;
        Ok(())
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
            let ledger = self.sync_ledger();
            let version = ledger
                .record(
                    SyncEntityType::Chat,
                    format!("chats/{}", self.chat_id),
                    SyncAction::Created,
                )
                .await?;

            let index = ChatIndex {
                id: self.chat_id,
                model: model.to_string(),
                message_ids: Vec::new(),
                version,
            };
            let yaml = serde_yaml::to_string(&index)?;
            self.file_storage.put(&index_path, yaml.as_bytes()).await?;

            let meta = ChatMeta {
                created_at: Utc::now(),
            };
            let meta_yaml = serde_yaml::to_string(&meta)?;
            let meta_path = self.chat_meta_path();
            self.file_storage
                .put(&meta_path, meta_yaml.as_bytes())
                .await?;

            Ok(index)
        }
    }

    pub async fn save_draft(
        &self,
        message_id: MessageId,
        content: &str,
    ) -> ChatStorageResult<u64> {
        let ledger = self.sync_ledger();
        let entity_path = format!("chats/{}/drafts/{}", self.chat_id, message_id);

        let action = if self.file_storage.exists(&self.draft_path(&message_id)).await? {
            SyncAction::Updated
        } else {
            SyncAction::Created
        };

        let version = ledger
            .record(SyncEntityType::Draft, entity_path, action)
            .await?;

        let draft = ChatMessage {
            id: message_id,
            role: MessageRole::User,
            content: content.to_string(),
            reasoning: None,
            created_at: Utc::now(),
            version,
        };
        let yaml = serde_yaml::to_string(&draft)?;
        let path = self.draft_path(&message_id);
        self.file_storage.put(&path, yaml.as_bytes()).await?;
        Ok(version)
    }

    pub async fn get_draft(&self, message_id: MessageId) -> ChatStorageResult<ChatMessage> {
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
        let mut draft: ChatMessage = serde_yaml::from_str(&yaml_str)?;

        let ledger = self.sync_ledger();
        let version = ledger
            .record(
                SyncEntityType::Message,
                format!("chats/{}/messages/{}", self.chat_id, message_id),
                SyncAction::Created,
            )
            .await?;
        draft.version = version;

        // Save as individual message file
        let msg_path = self.message_path(&message_id);
        let msg_yaml = serde_yaml::to_string(&draft)?;
        self.file_storage
            .put(&msg_path, msg_yaml.as_bytes())
            .await?;

        // Append to chat index
        self.append_message_id(message_id).await?;

        // Record draft deletion
        ledger
            .record(
                SyncEntityType::Draft,
                format!("chats/{}/drafts/{}", self.chat_id, message_id),
                SyncAction::Deleted,
            )
            .await?;

        // Delete the draft
        self.file_storage.delete(&draft_path).await?;

        Ok(draft)
    }

    pub async fn update_keywords(&self) -> ChatStorageResult<()> {
        let messages = self.get_all_messages().await?;
        let all_content: String = messages
            .iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let tokens = keyword_extractor::tokenize(&all_content);
        let tf = keyword_extractor::term_frequencies(&tokens);

        let term_doc_counts: std::collections::HashMap<String, usize> =
            tf.iter().map(|(t, _)| (t.clone(), 1)).collect();

        let keywords = keyword_extractor::extract_keywords(&tf, 1, &term_doc_counts, 20);
        let serialized = keyword_extractor::serialize_keywords(&keywords);

        let kw_path = format!("chats/{}.kw.txt", self.chat_id);
        self.file_storage
            .put(&kw_path, serialized.as_bytes())
            .await?;

        Ok(())
    }

    pub async fn save_assistant_message(
        &self,
        mut message: ChatMessage,
    ) -> ChatStorageResult<u64> {
        let ledger = self.sync_ledger();
        let version = ledger
            .record(
                SyncEntityType::Message,
                format!("chats/{}/messages/{}", self.chat_id, message.id),
                SyncAction::Created,
            )
            .await?;
        message.version = version;

        let msg_path = self.message_path(&message.id);
        let yaml = serde_yaml::to_string(&message)?;
        self.file_storage.put(&msg_path, yaml.as_bytes()).await?;

        self.append_message_id(message.id).await?;

        // Update keywords after each assistant response
        let _ = self.update_keywords().await;

        Ok(version)
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

    pub async fn load_index(&self) -> ChatStorageResult<ChatIndex> {
        let path = self.chat_index_path();
        if !self.file_storage.exists(&path).await? {
            return Ok(ChatIndex {
                id: self.chat_id,
                model: String::new(),
                message_ids: Vec::new(),
                version: 0,
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

        let ledger = self.sync_ledger();
        let version = ledger
            .record(
                SyncEntityType::Chat,
                format!("chats/{}", self.chat_id),
                SyncAction::Updated,
            )
            .await?;
        index.version = version;

        let yaml = serde_yaml::to_string(&index)?;
        let path = self.chat_index_path();
        self.file_storage.put(&path, yaml.as_bytes()).await?;
        Ok(())
    }
}
