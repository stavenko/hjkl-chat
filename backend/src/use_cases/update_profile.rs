use crate::providers::sqlite::SQLiteProvider;
use std::sync::Arc;
use uuid::Uuid;

use super::get_profile;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
    #[error("Get profile error: {0}")]
    GetProfile(#[from] get_profile::Error),
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

pub struct Input {
    pub user_id: Uuid,
    pub name: Option<String>,
    pub nickname: Option<String>,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    input: Input,
) -> Result<get_profile::Output, Error> {
    let rows = sqlite.execute_with_params(
        "UPDATE users SET name = ?, nickname = ? WHERE id = ?",
        rusqlite::params![input.name, input.nickname, input.user_id],
    )?;

    if rows == 0 {
        return Err(Error::UserNotFound);
    }

    let profile = get_profile::command(sqlite, input.user_id).await?;
    Ok(profile)
}
