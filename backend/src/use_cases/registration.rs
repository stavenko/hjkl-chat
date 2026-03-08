use crate::models::auth::UserInfo;
use crate::models::registration::{
    RegistrationCompleteResponse, RegistrationError, RegistrationInitResponse,
    RegistrationSession, RegistrationVerifyError, RegistrationVerifyResponse,
};
use crate::providers::smtp::{SMTPProvider, SmtpClient};
use crate::providers::sqlite::SQLiteProvider;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Header, EncodingKey};
use rand::Rng;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
struct JwtClaims {
    sub: String,
    email: String,
    exp: usize,
    iat: usize,
}

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
    #[allow(dead_code)]
    pub fn new_for_test(sqlite: Arc<SQLiteProvider>, smtp: Arc<S>) -> Self {
        Self { sqlite, smtp }
    }

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

pub struct RegistrationVerifyUseCase {
    sqlite: Arc<SQLiteProvider>,
}

impl RegistrationVerifyUseCase {
    pub fn new(sqlite: Arc<SQLiteProvider>) -> Self {
        Self { sqlite }
    }

    pub async fn verify_registration(
        &self,
        session_id: Uuid,
        code: &str,
    ) -> Result<RegistrationVerifyResponse, RegistrationVerifyError> {
        let session = self
            .sqlite
            .query_one(
                "SELECT * FROM registration_sessions WHERE id = ?",
                &[session_id.to_string().as_str().into()],
                RegistrationSession::from_row,
            )?
            .ok_or(RegistrationVerifyError::SessionNotFound)?;

        let now = Utc::now();
        if now > session.expires_at {
            return Err(RegistrationVerifyError::ExpiredSession);
        }

        if session.verification_code != code {
            return Err(RegistrationVerifyError::InvalidCode);
        }

        Ok(RegistrationVerifyResponse {
            status: "ok".to_string(),
            session_id: session.id.to_string(),
            expires_at: session.expires_at.to_rfc3339(),
        })
    }
}

pub struct RegistrationCompleteUseCase {
    sqlite: Arc<SQLiteProvider>,
    jwt_secret: String,
}

impl RegistrationCompleteUseCase {
    pub fn new(sqlite: Arc<SQLiteProvider>, jwt_secret: String) -> Self {
        Self { sqlite, jwt_secret }
    }

    pub async fn complete_registration(
        &self,
        session_id: Uuid,
        password: &str,
        password_confirm: &str,
    ) -> Result<RegistrationCompleteResponse, RegistrationError> {
        if password != password_confirm {
            return Err(RegistrationError::PasswordMismatch);
        }

        if !is_password_strong(password) {
            return Err(RegistrationError::WeakPassword);
        }

        let session = self
            .sqlite
            .query_one(
                "SELECT * FROM registration_sessions WHERE id = ?",
                &[session_id.to_string().as_str().into()],
                RegistrationSession::from_row,
            )?
            .ok_or(RegistrationError::SessionNotFound)?;

        let now = Utc::now();
        if now > session.expires_at {
            return Err(RegistrationError::ExpiredSession);
        }

        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| RegistrationError::DatabaseError(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            )))?;

        let user_id = Uuid::new_v4();
        let user_email = session.email.clone();

        self.sqlite.execute_with_params(
            "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
            rusqlite::params![user_id, user_email, password_hash, now],
        )?;

        let access_expires = now + Duration::hours(1);
        let refresh_expires = now + Duration::days(7);

        let access_claims = JwtClaims {
            sub: user_id.to_string(),
            email: user_email.clone(),
            exp: access_expires.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| {
            RegistrationError::DatabaseError(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))
        })?;

        let refresh_claims = JwtClaims {
            sub: user_id.to_string(),
            email: user_email.clone(),
            exp: refresh_expires.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| {
            RegistrationError::DatabaseError(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))
        })?;

        let session_id_new = Uuid::new_v4();
        self.sqlite.execute_with_params(
            "INSERT INTO sessions (id, user_id, access_token, refresh_token, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                session_id_new,
                user_id,
                access_token.clone(),
                refresh_token.clone(),
                refresh_expires,
                now,
            ],
        )?;

        self.sqlite.execute_with_params(
            "DELETE FROM registration_sessions WHERE id = ?",
            rusqlite::params![session_id],
        )?;

        Ok(RegistrationCompleteResponse {
            status: "ok".to_string(),
            user: UserInfo {
                id: user_id,
                email: user_email,
            },
            access_token,
            refresh_token,
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

fn is_password_strong(password: &str) -> bool {
    if password.len() < 8 {
        return false;
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());

    has_uppercase && has_lowercase && has_digit
}