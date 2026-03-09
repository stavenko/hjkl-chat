use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub nickname: Option<String>,
    pub name: Option<String>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(User {
            id: row.get("id")?,
            nickname: row.get("nickname")?,
            name: row.get("name")?,
            password_hash: row.get("password_hash")?,
            created_at: row.get("created_at")?,
        })
    }
}
