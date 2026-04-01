use crate::providers::chat_storage::ChatStorageError;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;

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

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    pub query: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub path: String,
    pub score: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub results: Vec<SearchResult>,
}

pub async fn command(
    storage: &PersonalizedFileStorage,
    input: &Input,
) -> Result<Output, Error> {
    let query_tokens = keyword_extractor::tokenize(&input.query);
    if query_tokens.is_empty() {
        return Ok(Output { results: Vec::new() });
    }

    let keys = storage.list("chats/").await.map_err(ChatStorageError::S3)?;

    let kw_files: Vec<String> = keys
        .into_iter()
        .filter(|k| k.ends_with(".kw.txt"))
        .collect();

    let mut results = Vec::new();
    for kw_file in &kw_files {
        let data = match storage.get(kw_file).await {
            Ok(d) => d,
            Err(_) => continue,
        };
        let text = String::from_utf8(data).unwrap_or_default();
        let keywords = keyword_extractor::deserialize_keywords(&text);

        let mut score = 0.0;
        for (pos, kw) in keywords.iter().enumerate() {
            for qt in &query_tokens {
                if kw == qt {
                    // Higher score for keywords ranked higher (lower position)
                    score += 1.0 / (pos as f64 + 1.0);
                }
            }
        }

        if score > 0.0 {
            // Strip .kw.txt suffix to get the path
            let path = kw_file.strip_suffix(".kw.txt").unwrap_or(kw_file).to_string();
            results.push(SearchResult { path, score });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(Output { results })
}
