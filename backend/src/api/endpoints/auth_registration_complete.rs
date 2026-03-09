use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::registration_complete;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationCompleteRequest {
    pub session_id: String,
    pub password: String,
    pub password_confirm: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    body: web::Json<RegistrationCompleteRequest>,
) -> impl Responder {
    let session_id = match uuid::Uuid::parse_str(&body.session_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiResponse::Err(crate::api::Error {
                code: "InvalidSessionId".to_string(),
                message: "Invalid session_id format".to_string(),
            });
        }
    };

    let input = registration_complete::Input {
        session_id,
        password: body.password.clone(),
        password_confirm: body.password_confirm.clone(),
    };

    registration_complete::command(sqlite.get_ref().clone(), input)
        .await
        .into()
}
