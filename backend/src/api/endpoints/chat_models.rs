use actix_web::{web, Responder};
use std::sync::Arc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::llm::LlmProvider;
use crate::use_cases::list_models;

pub async fn handler(
    llm: web::Data<Arc<LlmProvider>>,
    _user: AuthenticatedUser,
) -> impl Responder {
    let result: ApiResponse<_> = list_models::command(llm.get_ref().as_ref())
        .await
        .into();
    result
}
