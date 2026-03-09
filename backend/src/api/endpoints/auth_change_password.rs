use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::middleware::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::change_password;

#[derive(Debug, Clone, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    pub new_password_confirm: String,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    user: AuthenticatedUser,
    body: web::Json<ChangePasswordRequest>,
) -> impl Responder {
    let input = change_password::Input {
        user_id: user.user_id,
        old_password: body.old_password.clone(),
        new_password: body.new_password.clone(),
        new_password_confirm: body.new_password_confirm.clone(),
    };

    let result: ApiResponse<_> = change_password::command(sqlite.get_ref().clone(), input)
        .await
        .into();
    result
}
