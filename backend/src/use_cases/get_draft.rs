use crate::models::chat::{ChatId, ChatMessage, MessageId};
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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub draft: ChatMessage,
}

pub async fn command(
    storage: &PersonalizedChatStorage,
    chat_id: ChatId,
    input: Input,
) -> Result<Output, Error> {
    let chat_storage = storage.get_chat_storage(chat_id);
    let draft = chat_storage.get_draft(input.message_id).await?;
    Ok(Output { draft })
}
