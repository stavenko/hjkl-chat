use actix_web::{web, Responder};
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::use_cases::get_chat_messages;

pub async fn handler(
    storage: PersonalizedChatStorage,
    path: web::Path<Uuid>,
    query: web::Query<get_chat_messages::Input>,
) -> impl Responder {
    let chat_id = path.into_inner();
    let result: ApiResponse<_> = get_chat_messages::command(&storage, chat_id, query.into_inner()).await.into();
    result
}
