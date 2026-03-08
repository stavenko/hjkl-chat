use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use std::sync::Arc;

use crate::models::registration::{
    RegistrationVerifyRequest, RegistrationVerifyError,
};
use crate::use_cases::registration_verify::RegistrationVerifyUseCase;

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationVerifyApiResponse {
    pub status: String,
    pub session_id: String,
    pub expires_at: String,
}

pub async fn registration_verify(
    use_case: web::Data<Arc<RegistrationVerifyUseCase>>,
    body: web::Json<RegistrationVerifyRequest>,
) -> impl Responder {
    let session_id = match uuid::Uuid::parse_str(&body.session_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": "Invalid session_id format"
            }));
        }
    };

    match use_case.verify_registration(session_id, &body.code).await {
        Ok(response) => HttpResponse::Ok().json(RegistrationVerifyApiResponse {
            status: response.status,
            session_id: response.session_id,
            expires_at: response.expires_at,
        }),
        Err(e) => {
            tracing::error!("Registration verify error: {}", e);
            match e {
                RegistrationVerifyError::InvalidCode => HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Invalid verification code"
                })),
                RegistrationVerifyError::ExpiredSession => HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Session has expired"
                })),
                RegistrationVerifyError::SessionNotFound => HttpResponse::NotFound().json(serde_json::json!({
                    "status": "error",
                    "message": "Session not found"
                })),
                _ => HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": e.to_string()
                })),
            }
        }
    }
}