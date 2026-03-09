use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, RequestInit, RequestMode};

use crate::services::get_api_base_url;

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub status: String,
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
}

pub async fn login(email: &str, password: &str) -> Result<LoginResponse, String> {
    let base_url = get_api_base_url();
    let url = format!("{}/api/auth/login", base_url);

    let body = serde_json::to_string(&LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    })
    .map_err(|e| format!("Failed to serialize request: {}", e))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

    let headers =
        Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;
    opts.set_headers(&headers);

    let window = web_sys::window().expect("no window");
    let resp_value =
        JsFuture::from(window.fetch_with_str_and_init(&url, &opts))
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Failed to convert response: {:?}", e))?;

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("Failed to read body: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to await body: {:?}", e))?;

    let text_str = text
        .as_string()
        .ok_or_else(|| "Response body is not a string".to_string())?;

    if resp.ok() {
        serde_json::from_str::<LoginResponse>(&text_str)
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        #[derive(Deserialize)]
        struct ErrorResponse {
            message: String,
        }
        match serde_json::from_str::<ErrorResponse>(&text_str) {
            Ok(err) => Err(err.message),
            Err(_) => Err(format!("Request failed with status {}", resp.status())),
        }
    }
}

pub fn is_authenticated() -> bool {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage
        .get_item("access_token")
        .ok()
        .flatten()
        .is_some()
}

pub fn store_tokens(access_token: &str, refresh_token: &str) {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage
        .set_item("access_token", access_token)
        .expect("failed to store access_token");
    storage
        .set_item("refresh_token", refresh_token)
        .expect("failed to store refresh_token");
}

#[derive(Serialize)]
struct RegistrationInitRequest {
    email: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RegistrationInitResponse {
    pub status: String,
    pub message: String,
    pub session_id: String,
    pub resend_available_at: String,
}

pub async fn registration_init(email: &str) -> Result<RegistrationInitResponse, String> {
    let base_url = get_api_base_url();
    let url = format!("{}/api/auth/registration/init", base_url);

    let body = serde_json::to_string(&RegistrationInitRequest {
        email: email.to_string(),
    })
    .map_err(|e| format!("Failed to serialize request: {}", e))?;

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::SameOrigin);
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

    let headers =
        Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;
    opts.set_headers(&headers);

    let window = web_sys::window().expect("no window");
    let resp_value =
        JsFuture::from(window.fetch_with_str_and_init(&url, &opts))
            .await
            .map_err(|e| format!("Network error: {:?}", e))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|e| format!("Failed to convert response: {:?}", e))?;

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("Failed to read body: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to await body: {:?}", e))?;

    let text_str = text
        .as_string()
        .ok_or_else(|| "Response body is not a string".to_string())?;

    if resp.ok() {
        serde_json::from_str::<RegistrationInitResponse>(&text_str)
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        #[derive(Deserialize)]
        struct ErrorResponse {
            message: String,
        }
        match serde_json::from_str::<ErrorResponse>(&text_str) {
            Ok(err) => Err(err.message),
            Err(_) => Err(format!("Request failed with status {}", resp.status())),
        }
    }
}

pub fn clear_tokens() {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage
        .remove_item("access_token")
        .expect("failed to remove access_token");
    storage
        .remove_item("refresh_token")
        .expect("failed to remove refresh_token");
}
