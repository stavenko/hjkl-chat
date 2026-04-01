use crate::services::chat_service;
use crate::services::local_storage::{LocalDb, LocalFileKeywords};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub file_type: String,
    pub score: f64,
}

pub fn search_local(
    all_keywords: &[LocalFileKeywords],
    query: &str,
) -> Vec<SearchResult> {
    let query_tokens = keyword_extractor::tokenize(query);
    if query_tokens.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    for entry in all_keywords {
        let mut score = 0.0;
        for (pos, kw) in entry.keywords.iter().enumerate() {
            for qt in &query_tokens {
                if kw == qt {
                    score += 1.0 / (pos as f64 + 1.0);
                }
            }
        }
        if score > 0.0 {
            results.push(SearchResult {
                path: entry.path.clone(),
                title: entry.title.clone(),
                file_type: entry.file_type.clone(),
                score,
            });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

pub async fn search_remote(query: &str) -> Result<Vec<SearchResult>, String> {
    let resp = chat_service::search_files(query).await?;
    Ok(resp
        .results
        .into_iter()
        .map(|r| SearchResult {
            path: r.path.clone(),
            title: r.path, // title unavailable from search endpoint
            file_type: "chat".to_string(),
            score: r.score,
        })
        .collect())
}

pub async fn rebuild_index(db: &LocalDb) -> Result<(), String> {
    let toc = chat_service::get_toc()
        .await
        .map_err(|e| format!("Failed to fetch TOC: {}", e))?;

    for entry in &toc.entries {
        let kw_resp = chat_service::get_file_keywords(&entry.path)
            .await
            .map_err(|e| format!("Failed to fetch keywords for {}: {}", entry.path, e))?;

        let local_entry = LocalFileKeywords {
            path: entry.path.clone(),
            title: entry.title.clone(),
            file_type: entry.file_type.clone(),
            keywords: kw_resp.keywords,
        };

        db.put_file_keywords(&local_entry)
            .await
            .map_err(|e| format!("Failed to store keywords: {:?}", e))?;
    }

    Ok(())
}

pub async fn needs_rebuild(db: &LocalDb) -> bool {
    match db.list_all_file_keywords().await {
        Ok(entries) => entries.is_empty(),
        Err(_) => true,
    }
}
