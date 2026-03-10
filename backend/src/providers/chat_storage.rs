use crate::models::chat::{Chat, ChatSummary};
use crate::providers::s3::S3Provider;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum ChatStorageError {
    #[error("S3 error: {0}")]
    S3(#[from] crate::providers::s3::S3ProviderError),
    #[error("YAML serialization error: {0}")]
    YamlSerialize(#[from] serde_yaml::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Chat not found")]
    NotFound,
}

pub type ChatStorageResult<T> = Result<T, ChatStorageError>;

fn chat_key(user_id: &Uuid, chat_id: &Uuid) -> String {
    format!("chats/{}/{}.yaml", user_id, chat_id)
}

fn chats_prefix(user_id: &Uuid) -> String {
    format!("chats/{}/", user_id)
}

pub async fn save_chat(s3: &Arc<S3Provider>, chat: &Chat) -> ChatStorageResult<()> {
    let yaml = serde_yaml::to_string(chat)?;
    let key = chat_key(&chat.user_id, &chat.id);
    s3.put_object(&key, yaml.as_bytes()).await?;
    Ok(())
}

pub async fn load_chat(
    s3: &Arc<S3Provider>,
    user_id: &Uuid,
    chat_id: &Uuid,
) -> ChatStorageResult<Chat> {
    let key = chat_key(user_id, chat_id);
    let exists = s3.object_exists(&key).await?;
    if !exists {
        return Err(ChatStorageError::NotFound);
    }
    let data = s3.get_object(&key).await?;
    let yaml_str = String::from_utf8(data)?;
    let chat: Chat = serde_yaml::from_str(&yaml_str)?;
    Ok(chat)
}

pub async fn list_chats(
    s3: &Arc<S3Provider>,
    user_id: &Uuid,
) -> ChatStorageResult<Vec<ChatSummary>> {
    let prefix = chats_prefix(user_id);
    let keys = s3.list_objects(&prefix).await?;

    let mut summaries = Vec::new();
    for key in keys {
        let data = s3.get_object(&key).await?;
        let yaml_str = String::from_utf8(data)?;
        let chat: Chat = serde_yaml::from_str(&yaml_str)?;
        summaries.push(ChatSummary::from(&chat));
    }

    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(summaries)
}

pub async fn delete_chat(
    s3: &Arc<S3Provider>,
    user_id: &Uuid,
    chat_id: &Uuid,
) -> ChatStorageResult<()> {
    let key = chat_key(user_id, chat_id);
    s3.delete_object(&key).await?;
    Ok(())
}
