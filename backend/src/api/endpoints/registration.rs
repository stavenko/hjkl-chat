use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::registration::{
    RegistrationError, RegistrationInitResponse as ModelInitResponse,
    RegistrationVerifyRequest, RegistrationVerifyResponse as ModelVerifyResponse,
    RegistrationVerifyError,
};
use crate::providers::smtp::SMTPProvider;
use crate::use_cases::registration::{
    RegistrationCompleteUseCase, RegistrationUseCase, RegistrationVerifyUseCase,
};

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

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationVerifyResponse {
    pub status: String,
    pub session_id: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationCompleteRequest {
    pub session_id: String,
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
        Ok(response) => HttpResponse::Ok().json(convert_verify_response(response)),
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

pub async fn registration_complete(
    use_case: web::Data<Arc<RegistrationCompleteUseCase>>,
    body: web::Json<RegistrationCompleteRequest>,
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

    match use_case
        .complete_registration(session_id, &body.password, &body.password_confirm)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(convert_complete_response(response)),
        Err(e) => {
            tracing::error!("Registration complete error: {}", e);
            match e {
                RegistrationError::PasswordMismatch => HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Passwords do not match"
                })),
                RegistrationError::WeakPassword => HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Password is too weak"
                })),
                RegistrationError::ExpiredSession => HttpResponse::BadRequest().json(serde_json::json!({
                    "status": "error",
                    "message": "Session has expired"
                })),
                RegistrationError::SessionNotFound => HttpResponse::NotFound().json(serde_json::json!({
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

fn convert_init_response(response: ModelInitResponse) -> RegistrationInitResponse {
    RegistrationInitResponse {
        status: response.status,
        message: response.message,
        session_id: response.session_id.to_string(),
        resend_available_at: response.resend_available_at.to_rfc3339(),
    }
}

fn convert_verify_response(response: ModelVerifyResponse) -> RegistrationVerifyResponse {
    RegistrationVerifyResponse {
        status: response.status,
        session_id: response.session_id,
        expires_at: response.expires_at,
    }
}

fn convert_complete_response(
    response: crate::models::registration::RegistrationCompleteResponse,
) -> RegistrationCompleteResponse {
    RegistrationCompleteResponse {
        status: response.status,
        user: response.user,
        access_token: response.access_token,
        refresh_token: response.refresh_token,
    }
}