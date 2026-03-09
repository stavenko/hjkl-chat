use crate::models::email::Email;
use crate::models::user::User;
use crate::providers::sqlite::SQLiteProvider;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::UserNotFound => crate::api::Error {
                code: "UserNotFound".to_string(),
                message: value.to_string(),
            },
            e => crate::api::Error {
                code: "InternalServerError".to_string(),
                message: e.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmailInfo {
    pub email: String,
    pub is_verified: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub id: Uuid,
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub emails: Vec<EmailInfo>,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    user_id: Uuid,
) -> Result<Output, Error> {
    let user = sqlite
        .query_one_with_params(
            "SELECT * FROM users WHERE id = ?",
            rusqlite::params![user_id],
            User::from_row,
        )?
        .ok_or(Error::UserNotFound)?;

    let conn = sqlite.get_connection()?;
    let mut stmt = conn.prepare("SELECT * FROM emails WHERE user_id = ?")?;
    let emails: Vec<EmailInfo> = stmt
        .query_map(rusqlite::params![user_id], |row| {
            Ok(Email::from_row(row)?)
        })?
        .filter_map(|r| r.ok())
        .map(|e| EmailInfo {
            email: e.email,
            is_verified: e.is_verified,
        })
        .collect();

    Ok(Output {
        id: user.id,
        name: user.name,
        nickname: user.nickname,
        emails,
    })
}
