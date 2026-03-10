use actix_web::{web, Responder};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::ApiResponse;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::personalized_file_storage::PersonalizedFileStorage;
use crate::providers::websocket::WebSocketProvider;
use crate::use_cases::send_message;

pub async fn handler(
    storage: PersonalizedChatStorage,
    file_storage: PersonalizedFileStorage,
    ws: web::Data<Arc<WebSocketProvider>>,
    path: web::Path<Uuid>,
    body: web::Json<send_message::Input>,
) -> impl Responder {
    let chat_id = path.into_inner();
    let user_id = file_storage.user_id();

    let result: ApiResponse<_> = send_message::command(
        &storage,
        ws.get_ref().clone(),
        user_id,
        chat_id,
        body.into_inner(),
    )
    .await
    .into();
    result
}
