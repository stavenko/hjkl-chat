use crate::models::chat::Chat;
use crate::providers::chat_storage;
use crate::providers::s3::S3Provider;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] chat_storage::ChatStorageError),
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
    pub user_id: Uuid,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub chat_id: Uuid,
}

pub async fn command(
    s3: Arc<S3Provider>,
    input: Input,
) -> Result<Output, Error> {
    let chat = Chat {
        id: Uuid::new_v4(),
        user_id: input.user_id,
        title: "New Chat".to_string(),
        model: input.model,
        created_at: Utc::now(),
        messages: Vec::new(),
    };

    chat_storage::save_chat(&s3, &chat).await?;

    Ok(Output { chat_id: chat.id })
}
