use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::registration_verify;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationVerifyRequest {
    pub session_id: String,
    pub code: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    body: web::Json<RegistrationVerifyRequest>,
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

    let input = registration_verify::Input {
        session_id,
        code: body.code.clone(),
    };

    registration_verify::command(sqlite.get_ref().clone(), input)
        .await
        .into()
}
