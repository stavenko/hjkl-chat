use actix_web::Responder;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::use_cases::list_chats;

pub async fn handler(
    storage: PersonalizedChatStorage,
) -> impl Responder {
    let result: ApiResponse<_> = list_chats::command(&storage).await.into();
    result
}
