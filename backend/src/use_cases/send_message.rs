use crate::models::chat::{ChatId, ChatMessage, MessageId, MessageRole, UserId};
use crate::providers::chat_storage::ChatStorageError;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::pipes::{LlmTokenKind, PipesProvider};
use crate::providers::websocket::{WebSocketProvider, WsOutMessage};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] ChatStorageError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        crate::api::Error {
            code: "InternalServerError".to_string(),
            message: value.to_string(),
        }
    }
}

pub struct Input {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub assistant_message_id: MessageId,
}

pub async fn command(
    storage: &PersonalizedChatStorage,
    pipes: Arc<PipesProvider>,
    ws: Arc<WebSocketProvider>,
    user_id: UserId,
    input: Input,
) -> Result<Output, Error> {
    let chat_storage = storage.get_chat_storage(input.chat_id);

    let _sent_message = chat_storage.send_message(input.message_id).await?;

    let messages = chat_storage.get_all_messages().await?;

    let assistant_message_id = Uuid::new_v4();
    let chat_id = input.chat_id;
    let model = input.model;

    tokio::spawn(async move {
        let mut rx = pipes.execute_prompt(&model, &messages);

        let mut content_buf = String::new();
        let mut reasoning_buf = String::new();

        while let Some(token) = rx.recv().await {
            match token.kind {
                LlmTokenKind::Thinking => {
                    reasoning_buf.push_str(&token.text);
                }
                LlmTokenKind::Content => {
                    content_buf.push_str(&token.text);
                }
            }

            ws.send_to_user(
                user_id,
                WsOutMessage::Token {
                    chat_id,
                    message_id: assistant_message_id,
                    kind: token.kind,
                    text: token.text,
                },
            )
            .await;
        }

        ws.send_to_user(
            user_id,
            WsOutMessage::MessageComplete {
                chat_id,
                message_id: assistant_message_id,
            },
        )
        .await;

        let assistant_message = ChatMessage {
            id: assistant_message_id,
            role: MessageRole::Assistant,
            content: content_buf,
            reasoning: if reasoning_buf.is_empty() {
                None
            } else {
                Some(reasoning_buf)
            },
            created_at: Utc::now(),
        };

        if let Err(e) = chat_storage.save_assistant_message(assistant_message).await {
            eprintln!("Failed to save assistant message: {}", e);
            ws.send_to_user(
                user_id,
                WsOutMessage::Error {
                    chat_id,
                    message: format!("Failed to save response: {}", e),
                },
            )
            .await;
        }
    });

    Ok(Output {
        assistant_message_id,
    })
}
