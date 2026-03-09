use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::middleware::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::update_profile;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub nickname: Option<String>,
}

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    user: AuthenticatedUser,
    body: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let input = update_profile::Input {
        user_id: user.user_id,
        name: body.name.clone(),
        nickname: body.nickname.clone(),
    };

    let result: ApiResponse<_> = update_profile::command(sqlite.get_ref().clone(), input)
        .await
        .into();
    result
}
