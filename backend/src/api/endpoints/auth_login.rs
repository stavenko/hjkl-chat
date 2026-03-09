use actix_web::{web, Responder};
use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::login;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    body: web::Json<LoginRequest>,
) -> impl Responder {
    let input = login::Input {
        email: body.email.clone(),
        password: body.password.clone(),
    };

    let result: ApiResponse<_> = login::command(sqlite.get_ref().clone(), input)
        .await
        .into();
    result
}
