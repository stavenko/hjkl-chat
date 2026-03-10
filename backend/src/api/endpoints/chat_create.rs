use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::s3::S3Provider;
use crate::use_cases::create_chat;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateChatRequest {
    pub model: String,
}

pub async fn handler(
    s3: web::Data<Arc<S3Provider>>,
    user: AuthenticatedUser,
    body: web::Json<CreateChatRequest>,
) -> impl Responder {
    let input = create_chat::Input {
        user_id: user.user_id,
        model: body.model.clone(),
    };

    let result: ApiResponse<_> = create_chat::command(s3.get_ref().clone(), input)
        .await
        .into();
    result
}
