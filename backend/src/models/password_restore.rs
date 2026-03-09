use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRestoreSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub verification_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub resend_available_at: DateTime<Utc>,
}

impl PasswordRestoreSession {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(PasswordRestoreSession {
            id: row.get("id")?,
            user_id: row.get("user_id")?,
            email: row.get("email")?,
            verification_code: row.get("verification_code")?,
            created_at: row.get("created_at")?,
            expires_at: row.get("expires_at")?,
            resend_available_at: row.get("resend_available_at")?,
        })
    }
}
