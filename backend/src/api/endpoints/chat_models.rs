use actix_web::{web, Responder};

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::config::PipesConfig;
use crate::use_cases::list_models;

pub async fn handler(
    config: web::Data<PipesConfig>,
    _user: AuthenticatedUser,
) -> impl Responder {
    let result: ApiResponse<_> = ApiResponse::Ok(list_models::command(config.get_ref()));
    result
}
