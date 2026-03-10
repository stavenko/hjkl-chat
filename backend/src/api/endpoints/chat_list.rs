use actix_web::{web, Responder};
use std::sync::Arc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::s3::S3Provider;
use crate::use_cases::list_chats;

pub async fn handler(
    s3: web::Data<Arc<S3Provider>>,
    user: AuthenticatedUser,
) -> impl Responder {
    let input = list_chats::Input {
        user_id: user.user_id,
    };

    let result: ApiResponse<_> = list_chats::command(s3.get_ref().clone(), input)
        .await
        .into();
    result
}
