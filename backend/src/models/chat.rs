use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub reasoning: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSummary {
    pub id: Uuid,
    pub title: String,
    pub model: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Chat> for ChatSummary {
    fn from(chat: &Chat) -> Self {
        ChatSummary {
            id: chat.id,
            title: chat.title.clone(),
            model: chat.model.clone(),
            created_at: chat.created_at,
        }
    }
}
