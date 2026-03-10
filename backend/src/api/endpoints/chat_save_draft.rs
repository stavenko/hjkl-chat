use actix_web::{web, Responder};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::use_cases::save_draft;

#[derive(Debug, Clone, Deserialize)]
pub struct SaveDraftRequest {
    pub message_id: Uuid,
    pub content: String,
    pub model: String,
}

pub async fn handler(
    storage: PersonalizedChatStorage,
    path: web::Path<Uuid>,
    body: web::Json<SaveDraftRequest>,
) -> impl Responder {
    let chat_id = path.into_inner();

    let input = save_draft::Input {
        chat_id,
        message_id: body.message_id,
        content: body.content.clone(),
        model: body.model.clone(),
    };

    let result: ApiResponse<_> = save_draft::command(&storage, input).await.into();
    result
}
