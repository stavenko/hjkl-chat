use crate::models::chat::{ChatId, MessageId};
use crate::providers::chat_storage::ChatStorageError;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] ChatStorageError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Storage(ChatStorageError::VersionConflict { expected, actual }) => {
                crate::api::Error {
                    code: "VersionConflict".to_string(),
                    message: format!(
                        "Version conflict: expected {}, server has {}",
                        expected, actual
                    ),
                }
            }
            other => crate::api::Error {
                code: "InternalServerError".to_string(),
                message: other.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    pub message_id: MessageId,
    pub content: String,
    pub model: String,
    pub expected_version: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub message_id: MessageId,
    pub version: u64,
}

pub async fn command(
    storage: &PersonalizedChatStorage,
    chat_id: ChatId,
    input: Input,
) -> Result<Output, Error> {
    let chat_storage = storage.get_chat_storage(chat_id);
    chat_storage.get_or_create_chat(&input.model).await?;

    let version = match input.expected_version {
        Some(expected) => {
            chat_storage
                .save_draft_versioned(input.message_id, &input.content, expected)
                .await?
        }
        None => {
            chat_storage
                .save_draft(input.message_id, &input.content)
                .await?
        }
    };

    Ok(Output {
        message_id: input.message_id,
        version,
    })
}
