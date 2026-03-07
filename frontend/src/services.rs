use once_cell::sync::Lazy;
use std::sync::RwLock;

static API_BASE_URL: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

pub async fn init_api_base_url() {
    let resp = reqwest::get("/config.json")
        .await
        .expect("Failed to fetch config.json");
    let config: serde_json::Value = resp.json().await.expect("Failed to parse config.json");
    let api_base_url = config["api_base_url"]
        .as_str()
        .expect("config.json must contain api_base_url")
        .to_string();
    *API_BASE_URL.write().expect("Failed to write API_BASE_URL") = api_base_url;
}

#[allow(dead_code)]
pub fn get_api_base_url() -> String {
    API_BASE_URL.read().expect("Failed to read API_BASE_URL").clone()
}