use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::restore_complete;

#[derive(Debug, Clone, Deserialize)]
pub struct RestoreCompleteRequest {
    pub session_id: Uuid,
    pub password: String,
    pub password_confirm: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    body: web::Json<RestoreCompleteRequest>,
) -> impl Responder {
    let input = restore_complete::Input {
        session_id: body.session_id,
        password: body.password.clone(),
        password_confirm: body.password_confirm.clone(),
    };

    let result: ApiResponse<_> = restore_complete::command(sqlite.get_ref().clone(), input)
        .await
        .into();
    result
}
