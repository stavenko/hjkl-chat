use actix_web::{web, Responder};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::s3::S3Provider;
use crate::use_cases::get_chat;

pub async fn handler(
    s3: web::Data<Arc<S3Provider>>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> impl Responder {
    let chat_id = path.into_inner();

    let input = get_chat::Input {
        user_id: user.user_id,
        chat_id,
    };

    let result: ApiResponse<_> = get_chat::command(s3.get_ref().clone(), input)
        .await
        .into();
    result
}
