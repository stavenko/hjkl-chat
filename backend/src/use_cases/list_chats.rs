use crate::models::chat::ChatSummary;
use crate::providers::chat_storage;
use crate::providers::s3::S3Provider;
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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub chats: Vec<ChatSummary>,
}

pub async fn command(
    s3: Arc<S3Provider>,
    input: Input,
) -> Result<Output, Error> {
    let chats = chat_storage::list_chats(&s3, &input.user_id).await?;
    Ok(Output { chats })
}
