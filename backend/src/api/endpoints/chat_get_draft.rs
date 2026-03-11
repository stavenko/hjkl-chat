use actix_web::{web, Responder};
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::use_cases::get_draft;

pub async fn handler(
    storage: PersonalizedChatStorage,
    path: web::Path<Uuid>,
    query: web::Query<get_draft::Input>,
) -> impl Responder {
    let chat_id = path.into_inner();
    let result: ApiResponse<_> = get_draft::command(&storage, chat_id, query.into_inner()).await.into();
    result
}
