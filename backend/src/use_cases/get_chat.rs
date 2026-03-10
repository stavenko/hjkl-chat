use crate::models::chat::Chat;
use crate::providers::chat_storage;
use crate::providers::s3::S3Provider;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] chat_storage::ChatStorageError),
    #[error("Chat not found")]
    NotFound,
    #[error("Access denied")]
    AccessDenied,
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::NotFound => crate::api::Error {
                code: "ChatNotFound".to_string(),
                message: "Chat not found".to_string(),
            },
            Error::AccessDenied => crate::api::Error {
                code: "AccessDenied".to_string(),
                message: "Access denied".to_string(),
            },
            e => crate::api::Error {
                code: "InternalServerError".to_string(),
                message: e.to_string(),
            },
        }
    }
}

pub struct Input {
    pub user_id: Uuid,
    pub chat_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    #[serde(flatten)]
    pub chat: Chat,
}

pub async fn command(
    s3: Arc<S3Provider>,
    input: Input,
) -> Result<Output, Error> {
    let chat = chat_storage::load_chat(&s3, &input.user_id, &input.chat_id)
        .await
        .map_err(|e| match e {
            chat_storage::ChatStorageError::NotFound => Error::NotFound,
            other => Error::Storage(other),
        })?;

    if chat.user_id != input.user_id {
        return Err(Error::AccessDenied);
    }

    Ok(Output { chat })
}
