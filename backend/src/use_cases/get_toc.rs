use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::providers::chat_storage::ChatStorageError;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::models::chat::{ChatIndex, ChatMeta};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(#[from] ChatStorageError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        crate::api::Error {
            code: "InternalServerError".to_string(),
            message: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TocEntry {
    pub path: String,
    pub title: String,
    pub file_type: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub entries: Vec<TocEntry>,
}

pub async fn command(
    storage: &PersonalizedFileStorage,
) -> Result<Output, Error> {
    let keys = storage
        .list("chats/")
        .await
        .map_err(ChatStorageError::S3)?;

    let mut entries = Vec::new();
    for key in &keys {
        if !key.ends_with("/chat-meta.yaml") {
            continue;
        }

        let chat_id = match key
            .strip_prefix("chats/")
            .and_then(|s| s.strip_suffix("/chat-meta.yaml"))
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            Some(id) => id,
            None => continue,
        };

        let meta_data = storage.get(key).await.map_err(ChatStorageError::S3)?;
        let meta_str = String::from_utf8(meta_data).map_err(ChatStorageError::from)?;
        let meta: ChatMeta =
            serde_yaml::from_str(&meta_str).map_err(ChatStorageError::from)?;

        let index_key = format!("chats/{}/chat.yaml", chat_id);
        let index_data = storage.get(&index_key).await.map_err(ChatStorageError::S3)?;
        let index_str = String::from_utf8(index_data).map_err(ChatStorageError::from)?;
        let index: ChatIndex =
            serde_yaml::from_str(&index_str).map_err(ChatStorageError::from)?;

        // Build title from first message content or fall back to chat ID
        let title = if let Some(first_msg_id) = index.message_ids.first() {
            let msg_key = format!("chats/{}/messages/{}.yaml", chat_id, first_msg_id);
            match storage.get(&msg_key).await {
                Ok(msg_data) => {
                    let msg_str = String::from_utf8(msg_data).unwrap_or_default();
                    let msg: crate::models::chat::ChatMessage =
                        serde_yaml::from_str(&msg_str).unwrap_or_else(|_| {
                            crate::models::chat::ChatMessage {
                                id: *first_msg_id,
                                role: crate::models::chat::MessageRole::User,
                                content: String::new(),
                                reasoning: None,
                                created_at: Utc::now(),
                                version: 0,
                            }
                        });
                    let content = msg.content.trim().to_string();
                    if content.len() > 80 {
                        format!("{}...", &content[..77])
                    } else if content.is_empty() {
                        format!("Chat {}", &chat_id.to_string()[..8])
                    } else {
                        content
                    }
                }
                Err(_) => format!("Chat {}", &chat_id.to_string()[..8]),
            }
        } else {
            format!("Chat {}", &chat_id.to_string()[..8])
        };

        entries.push(TocEntry {
            path: format!("chats/{}", chat_id),
            title,
            file_type: "chat".to_string(),
            updated_at: meta.created_at,
        });
    }

    entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(Output { entries })
}
