use actix_web::{web, Responder};

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::use_cases::sync_push;

pub async fn handler(
    file_storage: PersonalizedFileStorage,
    chat_storage: PersonalizedChatStorage,
    body: web::Json<sync_push::Input>,
) -> impl Responder {
    let result: ApiResponse<_> =
        sync_push::command(&file_storage, &chat_storage, body.into_inner())
            .await
            .into();
    result
}
