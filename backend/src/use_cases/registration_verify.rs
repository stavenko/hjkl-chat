use crate::models::registration::{
    RegistrationSession, RegistrationVerifyError, RegistrationVerifyResponse,
};
use crate::providers::sqlite::SQLiteProvider;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

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