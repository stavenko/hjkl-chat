use crate::config::PipesConfig;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub models: Vec<ModelInfo>,
}

pub fn command(config: &PipesConfig) -> Output {
    let models = config
        .models
        .iter()
        .map(|m| ModelInfo {
            id: m.id.clone(),
            name: m.name.clone(),
        })
        .collect();
    Output { models }
}
