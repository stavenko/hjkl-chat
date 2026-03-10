use actix_web::{web, Responder};
use std::sync::Arc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::pipes::PipesProvider;
use crate::use_cases::list_models;

pub async fn handler(
    pipes: web::Data<Arc<PipesProvider>>,
    _user: AuthenticatedUser,
) -> impl Responder {
    let result: ApiResponse<_> = ApiResponse::Ok(list_models::command(pipes.get_ref().as_ref()));
    result
}
