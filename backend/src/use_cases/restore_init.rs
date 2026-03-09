use crate::models::email::Email;
use crate::models::password_restore::PasswordRestoreSession;
use crate::providers::smtp::SmtpClient;
use crate::providers::sqlite::SQLiteProvider;
use chrono::{Duration, Utc};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("Email not found")]
    EmailNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    #[error("Database provider error: {0}")]
    DatabaseProvider(#[from] crate::providers::sqlite::SQLiteProviderError),
    #[error("SMTP error: {0}")]
    SmtpError(#[from] crate::providers::smtp::SMTPProviderError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidEmail => crate::api::Error {
                code: "InvalidEmail".to_string(),
                message: value.to_string(),
            },
            Error::EmailNotFound => crate::api::Error {
                code: "EmailNotFound".to_string(),
                message: "No account with this email".to_string(),
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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub message: String,
    pub resend_available_at: f64,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    smtp: &dyn SmtpClient,
    input: Input,
) -> Result<Output, Error> {
    if input.email.is_empty() || !input.email.contains('@') || !input.email.contains('.') {
        return Err(Error::InvalidEmail);
    }

    let email_record = sqlite
        .query_one(
            "SELECT * FROM emails WHERE email = ?",
            &[input.email.as_str().into()],
            Email::from_row,
        )?
        .ok_or(Error::EmailNotFound)?;

    // Delete any existing restore session for this email
    sqlite.execute_with_params(
        "DELETE FROM password_restore_sessions WHERE email = ?",
        rusqlite::params![&input.email],
    )?;

    let now = Utc::now();
    let expires_at = now + Duration::minutes(15);
    let resend_available_at = now + Duration::seconds(60);

    let verification_code = generate_verification_code();
    let session_id = Uuid::new_v4();

    let session = PasswordRestoreSession {
        id: session_id,
        user_id: email_record.user_id,
        email: input.email.clone(),
        verification_code: verification_code.clone(),
        created_at: now,
        expires_at,
        resend_available_at,
    };

    sqlite.execute_with_params(
        "INSERT INTO password_restore_sessions (id, user_id, email, verification_code, created_at, expires_at, resend_available_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            session.id,
            session.user_id,
            session.email,
            session.verification_code,
            session.created_at,
            session.expires_at,
            session.resend_available_at,
        ],
    )?;

    let subject = "Your Password Reset Code";
    let body = format!(
        "Hello,\n\nYour password reset code is: {}\n\nThis code will expire in 15 minutes.\n\nIf you did not request this code, please ignore this email.",
        verification_code
    );

    smtp.send_email(&session.email, subject, &body).await?;

    Ok(Output {
        message: "Verification email sent".to_string(),
        resend_available_at: session.resend_available_at.timestamp() as f64,
    })
}

fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| rng.gen_range(0..10))
        .map(|d| d.to_string())
        .collect()
}
