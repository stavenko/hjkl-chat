use actix_web::{web, HttpResponse, Responder};
use crate::models::auth::{AuthError, LoginRequest};
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::login as login_use_case;
use serde_json::json;
use std::sync::Arc;

pub async fn login(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    jwt_secret: web::Data<String>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let email = if body.email.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({
                "status": "error",
                "message": "Missing email"
            }));
    } else {
        &body.email
    };

    let password = if body.password.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({
                "status": "error",
                "message": "Missing password"
            }));
    } else {
        &body.password
    };

    match login_use_case(sqlite.get_ref().clone(), email, password, jwt_secret.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AuthError::InvalidCredentials | AuthError::UserNotFound) => {
            HttpResponse::Unauthorized()
                .json(json!({
                    "status": "error",
                    "message": "Invalid email or password"
                }))
        }
        Err(e) => {
            tracing::error!("Login error: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({
                    "status": "error",
                    "message": e.to_string()
                }))
        }
    }
}