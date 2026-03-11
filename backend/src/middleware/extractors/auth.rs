use actix_web::{web, FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::session::Session;
use crate::providers::sqlite::SQLiteProvider;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub token: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let token = extract_bearer_token(req);

        let sqlite = req.app_data::<web::Data<Arc<SQLiteProvider>>>().cloned();

        Box::pin(async move {
            let token = token.ok_or_else(|| {
                actix_web::error::ErrorUnauthorized("Missing authorization token")
            })?;

            let sqlite = sqlite.ok_or_else(|| {
                actix_web::error::ErrorInternalServerError("Database not available")
            })?;

            let session = sqlite
                .query_one_with_params(
                    "SELECT * FROM sessions WHERE token = ?",
                    rusqlite::params![token],
                    Session::from_row,
                )
                .map_err(|_| {
                    actix_web::error::ErrorInternalServerError("Database query failed")
                })?
                .ok_or_else(|| {
                    actix_web::error::ErrorUnauthorized("Invalid session token")
                })?;

            let now = chrono::Utc::now();
            if session.expires_at < now {
                return Err(actix_web::error::ErrorUnauthorized("Session expired"));
            }

            Ok(AuthenticatedUser {
                user_id: session.user_id,
                token: session.token,
            })
        })
    }
}

fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}
