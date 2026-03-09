use actix_web::{web, Responder};
use std::sync::Arc;

use crate::api::middleware::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::sqlite::SQLiteProvider;
use crate::use_cases::get_profile;

pub async fn handler(
    sqlite: web::Data<Arc<SQLiteProvider>>,
    user: AuthenticatedUser,
) -> impl Responder {
    let result: ApiResponse<_> = get_profile::command(sqlite.get_ref().clone(), user.user_id)
        .await
        .into();
    result
}
