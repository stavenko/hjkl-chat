use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::api::auth_extractor::AuthenticatedUser;
use crate::providers::websocket::WebSocketProvider;

pub async fn handler(
    req: HttpRequest,
    stream: web::Payload,
    user: AuthenticatedUser,
    ws_provider: web::Data<Arc<WebSocketProvider>>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let user_id = user.user_id;
    let ws_provider_clone = ws_provider.get_ref().clone();
    let tx_clone = tx.clone();

    ws_provider.register(user_id, tx.clone()).await;

    actix_web::rt::spawn(async move {
        let mut session_clone = session.clone();

        let send_task = actix_web::rt::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let json = serde_json::to_string(&msg).unwrap_or_default();
                if session_clone.text(json).await.is_err() {
                    break;
                }
            }
        });

        let recv_task = actix_web::rt::spawn(async move {
            while let Some(Ok(msg)) = msg_stream.next().await {
                match msg {
                    Message::Ping(bytes) => {
                        if session.pong(&bytes).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        });

        let _ = futures::future::select(
            Box::pin(send_task),
            Box::pin(recv_task),
        )
        .await;

        ws_provider_clone.unregister(user_id, &tx_clone).await;
    });

    Ok(response)
}
