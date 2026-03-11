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
}

/// Stored in chats/{chat_id}/chat.yaml — ordered list of message IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatIndex {
    pub id: ChatId,
    pub model: String,
    pub message_ids: Vec<MessageId>,
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
