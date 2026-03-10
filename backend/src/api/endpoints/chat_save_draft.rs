use actix_web::{web, Responder};
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::use_cases::save_draft;

pub async fn handler(
    storage: PersonalizedChatStorage,
    path: web::Path<Uuid>,
    body: web::Json<save_draft::Input>,
) -> impl Responder {
    let chat_id = path.into_inner();
    let result: ApiResponse<_> = save_draft::command(&storage, chat_id, body.into_inner()).await.into();
    result
}
