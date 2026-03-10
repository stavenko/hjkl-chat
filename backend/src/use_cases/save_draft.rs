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
        crate::api::Error {
            code: "InternalServerError".to_string(),
            message: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    pub message_id: MessageId,
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub message_id: MessageId,
}

pub async fn command(
    storage: &PersonalizedChatStorage,
    chat_id: ChatId,
    input: Input,
) -> Result<Output, Error> {
    let chat_storage = storage.get_chat_storage(chat_id);
    chat_storage.get_or_create_chat(&input.model).await?;
    chat_storage
        .save_draft(input.message_id, &input.content)
        .await?;
    Ok(Output {
        message_id: input.message_id,
    })
}
