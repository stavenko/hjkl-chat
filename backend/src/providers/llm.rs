use crate::config::LlmConfig;
use crate::models::chat::ChatMessage;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmTokenKind {
    Thinking,
    Content,
}

#[derive(Debug, Clone)]
pub struct LlmToken {
    pub kind: LlmTokenKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum LlmError {
    #[error("HTTP request error: {0}")]
    Http(String),
    #[error("Model list parse error: {0}")]
    Parse(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

pub type LlmResult<T> = Result<T, LlmError>;

#[derive(Debug, Clone, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Clone, Deserialize)]
struct OpenAiModel {
    id: String,
}

pub struct LlmProvider {
    config: LlmConfig,
    http_client: reqwest::Client,
}

impl LlmProvider {
    pub fn new(config: LlmConfig) -> Self {
        LlmProvider {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn list_models(&self) -> LlmResult<Vec<ModelInfo>> {
        let mut req = self.http_client.get(&self.config.models_url);
        if let Some(ref key) = self.config.api_key {
            if !key.is_empty() {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
        }

        let response = req
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        let models_resp: OpenAiModelsResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        Ok(models_resp
            .data
            .into_iter()
            .map(|m| ModelInfo {
                name: m.id.clone(),
                id: m.id,
            })
            .collect())
    }

    pub fn execute_prompt(
        &self,
        model: &str,
        messages: &[ChatMessage],
    ) -> mpsc::UnboundedReceiver<LlmToken> {
        let (tx, rx) = mpsc::unbounded_channel();

        let api_base_url = self.config.api_base_url.clone();
        let api_key = self.config.api_key.clone();
        let model = model.to_string();
        let messages = messages.to_vec();

        let executor = arti_pipes::llm_executors::GptOss::builder()
            .api_base(&api_base_url)
            .api_key(api_key.as_deref().unwrap_or(""))
            .model(&model)
            .reasoning_effort("high")
            .build();

        let prompt_text = build_prompt_text(&messages);

        tokio::spawn(async move {
            let result = match arti_pipes::executor::PromptExecutor::execute_raw(&executor, prompt_text).await {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("LLM execution error: {:?}", e);
                    return;
                }
            };

            let mut thinking_stream = result.thinking_stream;
            let mut content_stream = result.content_stream;

            let tx_thinking = tx.clone();
            let thinking_task = tokio::spawn(async move {
                while let Some(Ok(token)) = thinking_stream.next().await {
                    let _ = tx_thinking.send(LlmToken {
                        kind: LlmTokenKind::Thinking,
                        text: token.content,
                    });
                }
            });

            let tx_content = tx.clone();
            let content_task = tokio::spawn(async move {
                while let Some(Ok(token)) = content_stream.next().await {
                    let _ = tx_content.send(LlmToken {
                        kind: LlmTokenKind::Content,
                        text: token.content,
                    });
                }
            });

            let _ = thinking_task.await;
            let _ = content_task.await;
        });

        rx
    }
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
