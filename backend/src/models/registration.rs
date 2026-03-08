use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RegistrationInitRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationInitResponse {
    pub status: String,
    pub message: String,
    pub session_id: Uuid,
    pub resend_available_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationVerifyRequest {
    pub session_id: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationVerifyResponse {
    pub status: String,
    pub session_id: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct RegistrationCompleteRequest {
    pub session_id: Uuid,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationCompleteResponse {
    pub status: String,
    pub user: crate::models::auth::UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationSession {
    pub id: Uuid,
    pub email: String,
    pub verification_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub resend_available_at: DateTime<Utc>,
}

impl RegistrationSession {
    #[allow(dead_code)]
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(RegistrationSession {
            id: row.get("id")?,
            email: row.get("email")?,
            verification_code: row.get("verification_code")?,
            created_at: row.get("created_at")?,
            expires_at: row.get("expires_at")?,
            resend_available_at: row.get("resend_available_at")?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum RegistrationError {
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
    #[error("SMTP error: {0}")]
    SmtpError(#[from] crate::providers::smtp::SMTPProviderError),
    #[error("Email already registered")]
    EmailAlreadyRegistered,
    #[error("Session already exists")]
    SessionAlreadyExists,
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("Weak password")]
    WeakPassword,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Session has expired")]
    ExpiredSession,
}

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum RegistrationVerifyError {
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
