use actix_web::{web, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::session::Session;
use crate::providers::sqlite::SQLiteProvider;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let result = (|| {
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized(
                        serde_json::json!({"code": "InvalidCredentials", "message": "Missing authorization header"}),
                    )
                })?;

            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized(
                        serde_json::json!({"code": "InvalidCredentials", "message": "Invalid authorization header format"}),
                    )
                })?;

            let sqlite = req
                .app_data::<web::Data<Arc<SQLiteProvider>>>()
                .ok_or_else(|| {
                    actix_web::error::ErrorInternalServerError("Database not configured")
                })?;

            let session = sqlite
                .query_one(
                    "SELECT * FROM sessions WHERE token = ?",
                    &[token.into()],
                    Session::from_row,
                )
                .map_err(|_| {
                    actix_web::error::ErrorInternalServerError("Database error")
                })?
                .ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized(
                        serde_json::json!({"code": "SessionNotFound", "message": "Session not found"}),
                    )
                })?;

            let now = chrono::Utc::now();
            if now > session.expires_at {
                return Err(actix_web::error::ErrorUnauthorized(
                    serde_json::json!({"code": "ExpiredSession", "message": "Session has expired"}),
                ));
            }

            Ok(AuthenticatedUser {
                user_id: session.user_id,
            })
        })();

        ready(result)
    }
}
