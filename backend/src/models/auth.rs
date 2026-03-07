use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub status: String,
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Password hash error: {0}")]
    PasswordHashError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Missing email")]
    MissingEmail,
    #[error("Missing password")]
    MissingPassword,
    #[error("Database error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
}
