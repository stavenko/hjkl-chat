use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::providers::pipes::PipesProvider;
use crate::providers::websocket::WebSocketProvider;
use crate::use_cases::send_message;

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub message_id: Uuid,
    pub model: String,
}

pub async fn handler(
    storage: PersonalizedChatStorage,
    file_storage: PersonalizedFileStorage,
    pipes: web::Data<Arc<PipesProvider>>,
    ws: web::Data<Arc<WebSocketProvider>>,
    path: web::Path<Uuid>,
    body: web::Json<SendMessageRequest>,
) -> impl Responder {
    let chat_id = path.into_inner();
    let user_id = file_storage.user_id();

    let input = send_message::Input {
        chat_id,
        message_id: body.message_id,
        model: body.model.clone(),
    };

    let result: ApiResponse<_> = send_message::command(
        &storage,
        pipes.get_ref().clone(),
        ws.get_ref().clone(),
        user_id,
        input,
    )
    .await
    .into();
    result
}
