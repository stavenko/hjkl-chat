use arti_pipes::llm_executors::openai::OpenAI;

use crate::config::PipesConfig;

pub fn create_executor(config: &PipesConfig, model_id: &str) -> OpenAI {
    OpenAI::builder()
        .api_base(&config.api_base_url)
        .api_key(config.api_key.as_deref().unwrap_or(""))
        .model(model_id)
        .build()
}
