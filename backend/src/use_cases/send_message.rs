use crate::models::chat::{ChatId, ChatMessage, MessageId, MessageRole, UserId};
use crate::providers::chat_storage::ChatStorageError;
use crate::providers::echo_executor::EchoExecutor;
use crate::providers::personalized_chat_storage::PersonalizedChatStorage;
use crate::providers::websocket::{LlmTokenKind, WebSocketProvider, WsOutMessage};
use arti_pipes::executor::PromptExecutor;
use chrono::Utc;
use futures::StreamExt;
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

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    pub message_id: MessageId,
    pub model: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub assistant_message_id: MessageId,
}

pub async fn command(
    storage: &PersonalizedChatStorage,
    ws: Arc<WebSocketProvider>,
    user_id: UserId,
    chat_id: ChatId,
    input: Input,
) -> Result<Output, Error> {
    let chat_storage = storage.get_chat_storage(chat_id);

    let _sent_message = chat_storage.send_message(input.message_id).await?;

    let messages = chat_storage.get_all_messages().await?;

    let assistant_message_id = Uuid::new_v4();
    let _model = input.model;

    tokio::spawn(async move {
        // Allow the HTTP response to reach the client before streaming tokens,
        // so the client can set up the assistant message bubble first.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let executor = EchoExecutor;
        let prompt_text = build_prompt_text(&messages);

        let result = match executor.execute_raw(prompt_text).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("LLM execution error: {:?}", e);
                ws.send_to_user(
                    user_id,
                    WsOutMessage::Error {
                        chat_id,
                        message: format!("LLM execution failed: {}", e),
                    },
                )
                .await;
                return;
            }
        };

        let mut thinking_stream = result.thinking_stream;
        let mut content_stream = result.content_stream;

        let mut content_buf = String::new();
        let mut reasoning_buf = String::new();

        let ws_thinking = ws.clone();
        let thinking_task = tokio::spawn(async move {
            let mut buf = String::new();
            while let Some(Ok(token)) = thinking_stream.next().await {
                buf.push_str(&token.content);
                ws_thinking
                    .send_to_user(
                        user_id,
                        WsOutMessage::Token {
                            chat_id,
                            message_id: assistant_message_id,
                            kind: LlmTokenKind::Thinking,
                            text: token.content,
                        },
                    )
                    .await;
            }
            buf
        });

        let ws_content = ws.clone();
        let content_task = tokio::spawn(async move {
            let mut buf = String::new();
            while let Some(Ok(token)) = content_stream.next().await {
                buf.push_str(&token.content);
                ws_content
                    .send_to_user(
                        user_id,
                        WsOutMessage::Token {
                            chat_id,
                            message_id: assistant_message_id,
                            kind: LlmTokenKind::Content,
                            text: token.content,
                        },
                    )
                    .await;
            }
            buf
        });

        if let Ok(thinking) = thinking_task.await {
            reasoning_buf = thinking;
        }
        if let Ok(content) = content_task.await {
            content_buf = content;
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

fn build_prompt_text(messages: &[ChatMessage]) -> String {
    use crate::models::chat::MessageRole;

    let mut prompt = String::new();
    for msg in messages {
        match msg.role {
            MessageRole::User => {
                prompt.push_str(&format!("<|user|>\n{}\n", msg.content));
            }
            MessageRole::Assistant => {
                prompt.push_str(&format!("<|assistant|>\n{}\n", msg.content));
            }
        }
    }
    prompt.push_str("<|assistant|>\n");
    prompt
}
