use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::ApiResponse;
use crate::providers::smtp::SMTPProvider;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::restore_init;

#[derive(Debug, Clone, Deserialize)]
pub struct RestoreInitRequest {
    pub email: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    smtp: web::Data<Arc<SMTPProvider>>,
    body: web::Json<RestoreInitRequest>,
) -> impl Responder {
    let input = restore_init::Input {
        email: body.email.clone(),
    };

    let result: ApiResponse<_> = restore_init::command(sqlite.get_ref().clone(), smtp.get_ref().as_ref(), input)
        .await
        .into();
    result
}
