use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub email: String,
    pub user_id: Uuid,
    pub is_verified: bool,
}

impl Email {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Email {
            email: row.get("email")?,
            user_id: row.get("user_id")?,
            is_verified: row.get("is_verified")?,
        })
    }
}
