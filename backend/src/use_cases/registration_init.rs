use crate::models::registration::RegistrationSession;
use crate::providers::smtp::SmtpClient;
use crate::providers::sqlite::SQLiteProvider;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
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
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::InvalidEmail => crate::api::Error {
                code: "InvalidEmail".to_string(),
                message: value.to_string(),
            },
            Error::EmailAlreadyRegistered => crate::api::Error {
                code: "EmailAlreadyRegistered".to_string(),
                message: "Email already registered".to_string(),
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
    pub session_id: Uuid,
    pub message: String,
    pub resend_available_at: DateTime<Utc>,
}

pub async fn command(
    sqlite: Arc<SQLiteProvider>,
    smtp: &dyn SmtpClient,
    input: Input,
) -> Result<Output, Error> {
    if input.email.is_empty() || !input.email.contains('@') || !input.email.contains('.') {
        return Err(Error::InvalidEmail);
    }

    let existing_email = sqlite.query_one(
        "SELECT email FROM emails WHERE email = ?",
        &[input.email.as_str().into()],
        |row| row.get::<_, String>("email"),
    )?;
    if existing_email.is_some() {
        return Err(Error::EmailAlreadyRegistered);
    }

    let now = Utc::now();
    let expires_at = now + Duration::minutes(15);
    let resend_available_at = now + Duration::seconds(60);

    let verification_code = generate_verification_code();
    let session_id = Uuid::new_v4();

    let session = RegistrationSession {
        id: session_id,
        email: input.email.clone(),
        verification_code: verification_code.clone(),
        created_at: now,
        expires_at,
        resend_available_at,
    };

    // Delete any existing registration session for this email (supports resend/retry)
    sqlite.execute_with_params(
        "DELETE FROM registration_sessions WHERE email = ?",
        rusqlite::params![&input.email],
    )?;

    sqlite.execute_with_params(
        "INSERT INTO registration_sessions (id, email, verification_code, created_at, expires_at, resend_available_at) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            session.id,
            session.email,
            session.verification_code,
            session.created_at,
            session.expires_at,
            session.resend_available_at,
        ],
    )?;

    let subject = "Your Registration Verification Code";
    let body = format!(
        "Hello,\n\nYour verification code is: {}\n\nThis code will expire in 15 minutes.\n\nIf you did not request this code, please ignore this email.",
        verification_code
    );

    smtp.send_email(&session.email, subject, &body).await?;

    Ok(Output {
        session_id: session.id,
        message: "Verification email sent".to_string(),
        resend_available_at: session.resend_available_at,
    })
}

fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| rng.gen_range(0..10))
        .map(|d| d.to_string())
        .collect()
}
