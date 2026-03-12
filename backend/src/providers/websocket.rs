use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use crate::models::chat::{ChatId, MessageId, UserId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LlmTokenKind {
    Thinking,
    Content,
}

#[derive(Debug, Clone)]
pub struct LlmToken {
    pub kind: LlmTokenKind,
    pub text: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum WsOutMessage {
    Token {
        chat_id: ChatId,
        message_id: MessageId,
        kind: LlmTokenKind,
        text: String,
    },
    MessageComplete {
        chat_id: ChatId,
        message_id: MessageId,
    },
    Error {
        chat_id: ChatId,
        message: String,
    },
    SyncAvailable {
        version: u64,
    },
}

pub struct WebSocketProvider {
    connections: Arc<RwLock<HashMap<UserId, Vec<mpsc::UnboundedSender<WsOutMessage>>>>>,
}

impl WebSocketProvider {
    pub fn new() -> Self {
        WebSocketProvider {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(
        &self,
        user_id: UserId,
        sender: mpsc::UnboundedSender<WsOutMessage>,
    ) {
        let mut conns = self.connections.write().await;
        conns.entry(user_id).or_default().push(sender);
    }

    pub async fn unregister(&self, user_id: UserId, sender: &mpsc::UnboundedSender<WsOutMessage>) {
        let mut conns = self.connections.write().await;
        if let Some(senders) = conns.get_mut(&user_id) {
            senders.retain(|s| !s.same_channel(sender));
            if senders.is_empty() {
                conns.remove(&user_id);
            }
        }
    }

    pub async fn send_to_user(&self, user_id: UserId, msg: WsOutMessage) {
        let conns = self.connections.read().await;
        if let Some(senders) = conns.get(&user_id) {
            for sender in senders {
                let _ = sender.send(msg.clone());
            }
        }
    }
}
