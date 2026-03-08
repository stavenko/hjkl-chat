use crate::models::registration::{RegistrationError, RegistrationInitResponse, RegistrationSession};
use crate::providers::smtp::{SMTPProvider, SmtpClient};
use crate::providers::sqlite::SQLiteProvider;
use chrono::{Duration, Utc};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

pub struct RegistrationUseCase<S: SmtpClient + Send + Sync + 'static> {
    sqlite: Arc<SQLiteProvider>,
    smtp: Arc<S>,
}

impl RegistrationUseCase<SMTPProvider> {
    pub fn new(sqlite: Arc<SQLiteProvider>, smtp: Arc<SMTPProvider>) -> Self {
        Self { sqlite, smtp }
    }
}

impl<S: SmtpClient + Send + Sync + 'static> RegistrationUseCase<S> {
    pub async fn init_registration(
        &self,
        email: &str,
    ) -> Result<RegistrationInitResponse, RegistrationError> {
        if email.is_empty() || !email.contains('@') || !email.contains('.') {
            return Err(RegistrationError::InvalidEmail);
        }

        let now = Utc::now();
        let expires_at = now + Duration::minutes(15);
        let resend_available_at = now + Duration::seconds(60);

        let verification_code = generate_verification_code();
        let session_id = Uuid::new_v4();

        let session = RegistrationSession {
            id: session_id,
            email: email.to_string(),
            verification_code: verification_code.clone(),
            created_at: now,
            expires_at,
            resend_available_at,
        };

        self.sqlite.execute_with_params(
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

        self.smtp
            .send_email(&session.email, subject, &body)
            .await?;

        Ok(RegistrationInitResponse {
            status: "ok".to_string(),
            message: "Verification email sent".to_string(),
            session_id: session.id,
            resend_available_at: session.resend_available_at,
        })
    }
}

fn generate_verification_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| rng.gen_range(0..10))
        .map(|d| d.to_string())
        .collect()
}