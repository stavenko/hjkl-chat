use actix_web::{web, Responder};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::api::ApiResponse;
use crate::providers::llm::LlmProvider;
use crate::providers::s3::S3Provider;
use crate::providers::websocket::WebSocketProvider;
use crate::use_cases::send_message;

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub model: String,
}

pub async fn handler(
    s3: web::Data<Arc<S3Provider>>,
    llm: web::Data<Arc<LlmProvider>>,
    ws: web::Data<Arc<WebSocketProvider>>,
    user: AuthenticatedUser,
    path: web::Path<Uuid>,
    body: web::Json<SendMessageRequest>,
) -> impl Responder {
    let chat_id = path.into_inner();

    let input = send_message::Input {
        user_id: user.user_id,
        chat_id,
        content: body.content.clone(),
        model: body.model.clone(),
    };

    let result: ApiResponse<_> = send_message::command(
        s3.get_ref().clone(),
        llm.get_ref().clone(),
        ws.get_ref().clone(),
        input,
    )
    .await
    .into();
    result
}
