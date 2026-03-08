use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::registration::{
    RegistrationError, RegistrationInitResponse as ModelInitResponse,
};
use crate::providers::smtp::SMTPProvider;
use crate::use_cases::registration::RegistrationUseCase;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationInitRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationInitResponse {
    pub status: String,
    pub message: String,
    pub session_id: String,
    pub resend_available_at: String,
}

pub async fn registration_init(
    use_case: web::Data<Arc<RegistrationUseCase<SMTPProvider>>>,
    body: web::Json<RegistrationInitRequest>,
) -> impl Responder {
    let email = if body.email.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "status": "error",
            "message": "Missing email"
        }));
    } else {
        &body.email
    };

    match use_case.init_registration(email).await {
        Ok(response) => HttpResponse::Ok().json(convert_init_response(response)),
        Err(e) => {
            tracing::error!("Registration init error: {}", e);
            match e {
                RegistrationError::InvalidEmail => HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Invalid email format"
                })),
                RegistrationError::EmailAlreadyRegistered | RegistrationError::SessionAlreadyExists => {
                    HttpResponse::Conflict().json(serde_json::json!({
                        "status": "error",
                        "message": "Email already registered"
                    }))
                }
                _ => HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "message": e.to_string()
                })),
            }
        }
    }
}

fn convert_init_response(response: ModelInitResponse) -> RegistrationInitResponse {
    RegistrationInitResponse {
        status: response.status,
        message: response.message,
        session_id: response.session_id.to_string(),
        resend_available_at: response.resend_available_at.to_rfc3339(),
    }
}