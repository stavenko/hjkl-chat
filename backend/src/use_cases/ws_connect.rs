use crate::models::chat::UserId;
use crate::providers::websocket::WebSocketProvider;
use actix_ws::Message;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn command(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    user_id: UserId,
    ws_provider: Arc<WebSocketProvider>,
) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let tx_clone = tx.clone();
    let ws_provider_clone = ws_provider.clone();

    actix_web::rt::spawn(async move {
        ws_provider.register(user_id, tx.clone()).await;

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
}
