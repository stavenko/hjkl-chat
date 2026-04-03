use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;
pub type ChatId = Uuid;
pub type MessageId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: String,
    pub reasoning: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub version: u64,
}

/// Stored in chats/{chat_id}/chat.yaml — ordered list of message IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIndex {
    pub id: ChatId,
    pub model: String,
    pub message_ids: Vec<MessageId>,
    #[serde(default)]
    pub version: u64,
}

/// Stored in chats/{chat_id}/chat-meta.yaml — metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMeta {
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSummary {
    pub id: ChatId,
    pub model: String,
    pub created_at: DateTime<Utc>,
}

// --- Context models ---

/// Stored at chats/{chat_id}/context.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatFacts {
    pub summary: String,
    pub facts: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

/// Stored at user-memory.yaml (per-user, not per-chat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMemory {
    pub facts: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

// --- Sync models ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncEntityType {
    Chat,
    Message,
    Draft,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncAction {
    Created,
    Updated,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntry {
    pub version: u64,
    pub entity_type: SyncEntityType,
    pub entity_path: String,
    pub action: SyncAction,
    pub timestamp: DateTime<Utc>,
}

/// Stored at sync-ledger.yaml per user
/// Tracks the global version counter for this user's data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLedger {
    pub global_version: u64,
    pub entries: Vec<SyncEntry>,
}
