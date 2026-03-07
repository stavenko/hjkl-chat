use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Session {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Session {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            access_token: row.get("access_token")?,
            refresh_token: row.get("refresh_token")?,
            expires_at: row.get("expires_at")?,
            created_at: row.get("created_at")?,
        })
    }
}
