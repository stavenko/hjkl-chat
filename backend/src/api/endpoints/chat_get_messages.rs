use actix_web::{web, Responder};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::use_cases::get_chat_messages;

#[derive(Debug, Clone, Deserialize)]
pub struct GetMessagesQuery {
    pub last_n: Option<usize>,
}

pub async fn handler(
    storage: PersonalizedChatStorage,
    path: web::Path<Uuid>,
    query: web::Query<GetMessagesQuery>,
) -> impl Responder {
    let chat_id = path.into_inner();

    let input = get_chat_messages::Input {
        chat_id,
        last_n: query.last_n,
    };

    let result: ApiResponse<_> = get_chat_messages::command(&storage, input).await.into();
    result
}
