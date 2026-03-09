use crate::models::password_restore::PasswordRestoreSession;
use crate::providers::sqlite::SQLiteProvider;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid verification code")]
    InvalidCode,
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
            Error::InvalidCode => crate::api::Error {
                code: "InvalidCode".to_string(),
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
    pub email: String,
    pub code: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub session_id: Uuid,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    input: Input,
) -> Result<Output, Error> {
    let session = sqlite
        .query_one(
            "SELECT * FROM password_restore_sessions WHERE email = ?",
            &[input.email.as_str().into()],
            PasswordRestoreSession::from_row,
        )?
        .ok_or(Error::SessionNotFound)?;

    let now = Utc::now();
    if now > session.expires_at {
        return Err(Error::ExpiredSession);
    }

    if session.verification_code != input.code {
        return Err(Error::InvalidCode);
    }

    Ok(Output {
        session_id: session.id,
    })
}
