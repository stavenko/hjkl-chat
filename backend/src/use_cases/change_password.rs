use crate::models::user::User;
use crate::providers::sqlite::SQLiteProvider;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
    #[error("Password hash error: {0}")]
    PasswordHashError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidCredentials => crate::api::Error {
                code: "InvalidCredentials".to_string(),
                message: "Old password is incorrect".to_string(),
            },
            Error::PasswordMismatch => crate::api::Error {
                code: "PasswordMismatch".to_string(),
                message: value.to_string(),
            },
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
    pub old_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub message: String,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    input: Input,
) -> Result<Output, Error> {
    if input.new_password != input.new_password_confirm {
        return Err(Error::PasswordMismatch);
    }

    let user = sqlite
        .query_one_with_params(
            "SELECT * FROM users WHERE id = ?",
            rusqlite::params![input.user_id],
            User::from_row,
        )?
        .ok_or(Error::UserNotFound)?;

    let password_valid = bcrypt::verify(&input.old_password, &user.password_hash)
        .map_err(|e| Error::PasswordHashError(Box::new(e)))?;

    if !password_valid {
        return Err(Error::InvalidCredentials);
    }

    let new_hash = bcrypt::hash(&input.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| Error::PasswordHashError(Box::new(e)))?;

    sqlite.execute_with_params(
        "UPDATE users SET password_hash = ? WHERE id = ?",
        rusqlite::params![new_hash, input.user_id],
    )?;

    Ok(Output {
        message: "Password changed successfully".to_string(),
    })
}
