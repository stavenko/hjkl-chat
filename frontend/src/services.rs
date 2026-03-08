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
    
    "http://localhost:8080".to_string()
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
    let default_url = "http://localhost:8080".to_string();
    
    let window = match web_sys::window() {
        Some(w) => w,
        None => return default_url,
    };
    
    let promise = window.fetch_with_str("/config.json");
    
    let response_value = match JsFuture::from(promise).await {
        Ok(v) => v,
        Err(_) => return default_url,
    };
    
    let response: web_sys::Response = match response_value.dyn_into() {
        Ok(r) => r,
        Err(_) => return default_url,
    };
    
    let text_promise = match response.text() {
        Ok(p) => p,
        Err(_) => return default_url,
    };
    
    let text = match JsFuture::from(text_promise).await {
        Ok(t) => t,
        Err(_) => return default_url,
    };
    
    let text_str = match text.as_string() {
        Some(s) => s,
        None => return default_url,
    };
    
    let config: serde_json::Value = match serde_json::from_str(&text_str) {
        Ok(c) => c,
        Err(_) => return default_url,
    };
    
    config["api_base_url"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| default_url)
}