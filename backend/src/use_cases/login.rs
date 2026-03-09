use crate::models::auth::UserInfo;
use crate::models::email::Email;
use crate::models::session::Session;
use crate::models::user::User;
use crate::providers::sqlite::SQLiteProvider;
use chrono::Utc;
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid email or password")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Password hash error: {0}")]
    PasswordHashError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidCredentials | Error::UserNotFound => crate::api::Error {
                code: "InvalidCredentials".to_string(),
                message: "Invalid email or password".to_string(),
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
    pub password: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub user: UserInfo,
    pub token: String,
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    input: Input,
) -> Result<Output, Error> {
    let email_record = sqlite
        .query_one(
            "SELECT * FROM emails WHERE email = ?",
            &[input.email.as_str().into()],
            Email::from_row,
        )?
        .ok_or(Error::UserNotFound)?;

    let user = sqlite
        .query_one_with_params(
            "SELECT * FROM users WHERE id = ?",
            rusqlite::params![email_record.user_id],
            User::from_row,
        )?
        .ok_or(Error::UserNotFound)?;

    let password_valid = bcrypt::verify(&input.password, &user.password_hash)
        .map_err(|e| Error::PasswordHashError(Box::new(e)))?;

    if !password_valid {
        return Err(Error::InvalidCredentials);
    }

    let now = Utc::now();
    let expires = now + chrono::Duration::days(30);
    let token = generate_token();

    let session = Session {
        id: Uuid::new_v4(),
        user_id: user.id,
        token: token.clone(),
        expires_at: expires,
        created_at: now,
    };

    sqlite.execute_with_params(
        "INSERT INTO sessions (id, user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
        rusqlite::params![
            session.id,
            session.user_id,
            session.token,
            session.expires_at,
            session.created_at,
        ],
    )?;

    Ok(Output {
        user: UserInfo {
            id: user.id,
            email: email_record.email,
        },
        token,
    })
}
