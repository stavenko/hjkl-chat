use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, RequestMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginResponse {
    pub status: String,
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Server error: {0}")]
    Server(String),
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        ApiError::Parse(value.to_string())
    }
}

impl From<wasm_bindgen::JsValue> for ApiError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        ApiError::Network(format!("{:?}", value))
    }
}

pub async fn login(email: &str, password: &str) -> Result<LoginResponse, ApiError> {
    let api_base_url = crate::services::get_api_base_url();
    let url = format!("{}/api/auth/login", api_base_url);

    let request_body = LoginRequest {
        email: email.to_string(),
        password: password.to_string(),
    };

    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_mode(RequestMode::Cors);
    let headers = web_sys::Headers::new().unwrap();
    headers.append("Content-Type", "application/json").unwrap();
    opts.set_headers(&headers);
    
    let body = serde_json::to_string(&request_body)?;
    opts.set_body(&wasm_bindgen::JsValue::from_str(&body));

    let window = web_sys::window().expect("no global window exists");
    let response = JsFuture::from(window.fetch_with_str_and_init(&url, &opts))
        .await?;

    let response: web_sys::Response = response.dyn_into()?;

    let text = JsFuture::from(response.text()?)
        .await?
        .as_string()
        .unwrap_or_default();

    if response.ok() {
        let result: LoginResponse = serde_json::from_str(&text)?;
        Ok(result)
    } else {
        let error: ErrorResponse = serde_json::from_str(&text)?;
        Err(ApiError::Server(error.message))
    }
}