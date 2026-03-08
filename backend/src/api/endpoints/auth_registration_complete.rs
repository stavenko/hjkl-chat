use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::auth::UserInfo;
use crate::models::registration::RegistrationError;
use crate::use_cases::registration_complete::RegistrationCompleteUseCase;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationCompleteRequest {
    pub session_id: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationCompleteResponse {
    pub status: String,
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
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
        Ok(response) => HttpResponse::Ok().json(RegistrationCompleteResponse {
            status: response.status,
            user: response.user,
            access_token: response.access_token,
            refresh_token: response.refresh_token,
        }),
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