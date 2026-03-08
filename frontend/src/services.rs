use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static API_BASE_URL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

#[derive(Clone)]
pub struct ApiBaseUrl(pub String);

pub fn get_api_base_url() -> String {
    if let Ok(url) = API_BASE_URL.lock() {
        if let Some(ref url) = *url {
            return url.clone();
        }
    }
    
    panic!("API base URL not initialized. Ensure init_api_base_url() was called before app mount.");
}

pub async fn init_api_base_url() {
    let api_base_url = fetch_config().await;
    
    {
        let mut url = API_BASE_URL.lock().unwrap();
        *url = Some(api_base_url.clone());
    }
    
    provide_context(ApiBaseUrl(api_base_url));
}

async fn fetch_config() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => panic!("Failed to get window object: Running outside browser context"),
    };
    
    let promise = window.fetch_with_str("/config.json");
    
    let response_value = match JsFuture::from(promise).await {
        Ok(v) => v,
        Err(e) => panic!("Failed to fetch config.json: {:?}", e),
    };
    
    let response: web_sys::Response = match response_value.dyn_into() {
        Ok(r) => r,
        Err(e) => panic!("Failed to convert fetch response: {:?}", e),
    };
    
    if !response.ok() {
        panic!("Failed to load config.json: HTTP {}", response.status());
    }
    
    let text_promise = match response.text() {
        Ok(p) => p,
        Err(e) => panic!("Failed to read config.json body: {:?}", e),
    };
    
    let text = match JsFuture::from(text_promise).await {
        Ok(t) => t,
        Err(e) => panic!("Failed to await config.json text: {:?}", e),
    };
    
    let text_str = match text.as_string() {
        Some(s) => s,
        None => panic!("Failed to convert config.json to string"),
    };
    
    let config: serde_json::Value = match serde_json::from_str(&text_str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse config.json as JSON: {}", e),
    };
    
    match config["api_base_url"].as_str() {
        Some(url) => url.to_string(),
        None => panic!("config.json missing required field 'api_base_url'"),
    }
}