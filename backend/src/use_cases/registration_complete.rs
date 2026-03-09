use crate::models::registration::RegistrationSession;
use crate::providers::sqlite::SQLiteProvider;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Session has expired")]
    ExpiredSession,
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::PasswordMismatch => crate::api::Error {
                code: "PasswordMismatch".to_string(),
                message: value.to_string(),
            },
            Error::SessionNotFound => crate::api::Error {
                code: "SessionNotFound".to_string(),
                message: value.to_string(),
            },
            Error::ExpiredSession => crate::api::Error {
                code: "ExpiredSession".to_string(),
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
    pub session_id: Uuid,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub message: String,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    input: Input,
) -> Result<Output, Error> {
    if input.password != input.password_confirm {
        return Err(Error::PasswordMismatch);
    }

    let session = sqlite
        .query_one_with_params(
            "SELECT * FROM registration_sessions WHERE id = ?",
            rusqlite::params![input.session_id],
            RegistrationSession::from_row,
        )?
        .ok_or(Error::SessionNotFound)?;

    let now = Utc::now();
    if now > session.expires_at {
        return Err(Error::ExpiredSession);
    }

    let password_hash = bcrypt::hash(&input.password, bcrypt::DEFAULT_COST)
        .map_err(|e| Error::DatabaseError(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(e),
        )))?;

    let user_id = Uuid::new_v4();

    sqlite.execute_with_params(
        "INSERT INTO users (id, password_hash, created_at) VALUES (?, ?, ?)",
        rusqlite::params![user_id, password_hash, now],
    )?;

    sqlite.execute_with_params(
        "INSERT INTO emails (email, user_id, is_verified) VALUES (?, ?, 1)",
        rusqlite::params![session.email, user_id],
    )?;

    sqlite.execute_with_params(
        "DELETE FROM registration_sessions WHERE id = ?",
        rusqlite::params![input.session_id],
    )?;

    Ok(Output {
        message: "Registration complete".to_string(),
    })
}

