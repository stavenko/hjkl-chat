use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::restore_verify;

#[derive(Debug, Clone, Deserialize)]
pub struct RestoreVerifyRequest {
    pub email: String,
    pub code: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    body: web::Json<RestoreVerifyRequest>,
) -> impl Responder {
    let input = restore_verify::Input {
        email: body.email.clone(),
        code: body.code.clone(),
    };

    let result: ApiResponse<_> = restore_verify::command(sqlite.get_ref().clone(), input)
        .await
        .into();
    result
}
