use std::collections::HashMap;

use crate::providers::chat_storage::{ChatStorage, ChatStorageError};
use crate::providers::personalized_file_storage::PersonalizedFileStorage;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(#[from] ChatStorageError),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
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
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub path: String,
    pub keywords: Vec<String>,
}

pub async fn command(
    storage: &PersonalizedFileStorage,
    input: &Input,
) -> Result<Output, Error> {
    let kw_path = format!("{}.kw.txt", input.path);

    // Try loading existing keywords
    if storage.exists(&kw_path).await.map_err(ChatStorageError::S3)? {
        let data = storage.get(&kw_path).await.map_err(ChatStorageError::S3)?;
        let text = String::from_utf8(data).map_err(ChatStorageError::Utf8)?;
        let keywords = keyword_extractor::deserialize_keywords(&text);
        return Ok(Output {
            path: input.path.clone(),
            keywords,
        });
    }

    // Compute keywords on-the-fly
    let chat_id = input
        .path
        .strip_prefix("chats/")
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .ok_or_else(|| Error::InvalidPath(input.path.clone()))?;

    let chat_storage = ChatStorage::new(storage.clone(), chat_id);
    let messages = chat_storage.get_all_messages().await?;

    let all_content: String = messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    let tokens = keyword_extractor::tokenize(&all_content);
    let tf = keyword_extractor::term_frequencies(&tokens);

    // Use single-document IDF approximation
    let term_doc_counts: HashMap<String, usize> =
        tf.iter().map(|(t, _)| (t.clone(), 1)).collect();

    let keywords = keyword_extractor::extract_keywords(&tf, 1, &term_doc_counts, 20);

    // Store for next time
    let serialized = keyword_extractor::serialize_keywords(&keywords);
    let _ = storage.put(&kw_path, serialized.as_bytes()).await;

    Ok(Output {
        path: input.path.clone(),
        keywords,
    })
}
