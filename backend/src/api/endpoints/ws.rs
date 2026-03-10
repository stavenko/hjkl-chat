use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::Arc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::providers::websocket::WebSocketProvider;
use crate::use_cases::ws_connect;

pub async fn handler(
    req: HttpRequest,
    stream: web::Payload,
    user: AuthenticatedUser,
    ws_provider: web::Data<Arc<WebSocketProvider>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    ws_connect::command(session, msg_stream, user.user_id, ws_provider.get_ref().clone());

    Ok(response)
}
