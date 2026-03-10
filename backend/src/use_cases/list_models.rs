use crate::providers::llm::{LlmProvider, ModelInfo};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("LLM provider error: {0}")]
    Llm(#[from] crate::providers::llm::LlmError),
}

impl From<Error> for crate::api::Error {
    fn from(value: Error) -> Self {
        crate::api::Error {
            code: "InternalServerError".to_string(),
            message: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub models: Vec<ModelInfo>,
}

pub async fn command(
    llm: &LlmProvider,
) -> Result<Output, Error> {
    let models = llm.list_models().await?;
    Ok(Output { models })
}
