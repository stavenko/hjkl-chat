use crate::models::chat::{ChatMessage, MessageRole};
use crate::providers::chat_storage;
use crate::providers::llm::{LlmProvider, LlmTokenKind};
use crate::providers::s3::S3Provider;
use crate::providers::websocket::{WebSocketProvider, WsOutMessage};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Chat storage error: {0}")]
    Storage(#[from] chat_storage::ChatStorageError),
    #[error("Chat not found")]
    NotFound,
    #[error("Access denied")]
    AccessDenied,
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::NotFound => crate::api::Error {
                code: "ChatNotFound".to_string(),
                message: "Chat not found".to_string(),
            },
            Error::AccessDenied => crate::api::Error {
                code: "AccessDenied".to_string(),
                message: "Access denied".to_string(),
            },
            e => crate::api::Error {
                code: "InternalServerError".to_string(),
                message: e.to_string(),
            },
        }
    }
}

pub struct Input {
    pub user_id: Uuid,
    pub chat_id: Uuid,
    pub content: String,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub message_id: Uuid,
}

pub async fn command(
    s3: Arc<S3Provider>,
    llm: Arc<LlmProvider>,
    ws: Arc<WebSocketProvider>,
    input: Input,
) -> Result<Output, Error> {
    let mut chat = chat_storage::load_chat(&s3, &input.user_id, &input.chat_id)
        .await
        .map_err(|e| match e {
            chat_storage::ChatStorageError::NotFound => Error::NotFound,
            other => Error::Storage(other),
        })?;

    if chat.user_id != input.user_id {
        return Err(Error::AccessDenied);
    }

    let user_message = ChatMessage {
        id: Uuid::new_v4(),
        role: MessageRole::User,
        content: input.content,
        reasoning: None,
        created_at: Utc::now(),
    };
    chat.messages.push(user_message);

    if chat.title == "New Chat" {
        if let Some(first_user_msg) = chat.messages.iter().find(|m| matches!(m.role, MessageRole::User)) {
            let title: String = first_user_msg.content.chars().take(50).collect();
            chat.title = if title.len() < first_user_msg.content.len() {
                format!("{}...", title)
            } else {
                title
            };
        }
    }

    chat_storage::save_chat(&s3, &chat).await?;

    let assistant_message_id = Uuid::new_v4();
    let chat_id = chat.id;
    let user_id = input.user_id;
    let model = input.model;
    let messages = chat.messages.clone();

    tokio::spawn(async move {
        let mut rx = llm.execute_prompt(&model, &messages);

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

        match chat_storage::load_chat(&s3, &user_id, &chat_id).await {
            Ok(mut updated_chat) => {
                updated_chat.messages.push(assistant_message);
                if let Err(e) = chat_storage::save_chat(&s3, &updated_chat).await {
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
            }
            Err(e) => {
                eprintln!("Failed to reload chat for saving assistant message: {}", e);
            }
        }
    });

    Ok(Output {
        message_id: assistant_message_id,
    })
}
