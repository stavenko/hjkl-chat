use crate::providers::pipes::{ModelInfo, PipesProvider};

#[derive(Debug, Clone, serde::Serialize)]
pub struct Output {
    pub models: Vec<ModelInfo>,
}

pub fn command(pipes: &PipesProvider) -> Output {
    let models = pipes.list_models();
    Output { models }
}
