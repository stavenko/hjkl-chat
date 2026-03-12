pub mod auth_service;
pub mod chat_service;
pub mod local_storage;
pub mod sync_engine;
pub mod ws_service;

use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static API_BASE_URL: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
static FEATURES: Lazy<Mutex<Option<Features>>> = Lazy::new(|| Mutex::new(None));

#[derive(Clone)]
pub struct ApiBaseUrl(pub String);

#[derive(Clone, Debug)]
pub struct Features(HashMap<String, bool>);

impl Features {
    pub fn is_enabled(&self, name: &str) -> bool {
        self.0.get(name).copied().unwrap_or(false)
    }
}

pub fn get_api_base_url() -> String {
    if let Ok(url) = API_BASE_URL.lock() {
        if let Some(ref url) = *url {
            return url.clone();
        }
    }

    panic!("API base URL not initialized. Ensure init_config() was called before app mount.");
}

pub fn get_features() -> Features {
    if let Ok(f) = FEATURES.lock() {
        if let Some(ref features) = *f {
            return features.clone();
        }
    }

    panic!("Features not initialized. Ensure init_config() was called before app mount.");
}

pub async fn init_config() {
    let (api_base_url, features) = fetch_config().await;

    {
        let mut url = API_BASE_URL.lock().unwrap();
        *url = Some(api_base_url.clone());
    }

    {
        let mut f = FEATURES.lock().unwrap();
        *f = Some(features);
    }
}

async fn fetch_config() -> (String, Features) {
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

    let api_base_url = match config["api_base_url"].as_str() {
        Some(url) => url.to_string(),
        None => panic!("config.json missing required field 'api_base_url'"),
    };

    let mut features_map = HashMap::new();
    if let Some(obj) = config["features"].as_object() {
        for (key, val) in obj {
            match val.as_bool() {
                Some(b) => { features_map.insert(key.clone(), b); }
                None => panic!("config.json features.{} must be a boolean", key),
            }
        }
    }

    (api_base_url, Features(features_map))
}
