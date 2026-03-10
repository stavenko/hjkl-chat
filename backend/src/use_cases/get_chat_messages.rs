use crate::models::chat::{ChatId, ChatMessage};
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

pub struct Input {
    pub chat_id: ChatId,
    pub last_n: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub messages: Vec<ChatMessage>,
}

pub async fn command(
    storage: &PersonalizedChatStorage,
    input: Input,
) -> Result<Output, Error> {
    let chat_storage = storage.get_chat_storage(input.chat_id);
    let messages = match input.last_n {
        Some(n) => chat_storage.get_last_n(n).await?,
        None => chat_storage.get_all_messages().await?,
    };
    Ok(Output { messages })
}
