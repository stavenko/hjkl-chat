use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, RequestInit, RequestMode};

use crate::services::get_api_base_url;
use crate::services::auth_service;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ApiError {
    pub fn is_version_conflict(&self) -> bool {
        self.status == 409 || self.code == "VersionConflict"
    }
}

async fn request_json_typed<Resp: DeserializeOwned>(
    method: &str,
    path: &str,
    body: Option<String>,
) -> Result<Resp, ApiError> {
    let base_url = get_api_base_url();
    let url = format!("{}{}", base_url, path);

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::SameOrigin);

    if let Some(ref b) = body {
        opts.set_body(&wasm_bindgen::JsValue::from_str(b));
    }

    let headers =
        Headers::new().map_err(|e| ApiError { status: 0, code: "ClientError".into(), message: format!("Failed to create headers: {:?}", e) })?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| ApiError { status: 0, code: "ClientError".into(), message: format!("Failed to set header: {:?}", e) })?;

    if let Some(token) = auth_service::get_token() {
        headers
            .set("Authorization", &format!("Bearer {}", token))
            .map_err(|e| ApiError { status: 0, code: "ClientError".into(), message: format!("Failed to set auth header: {:?}", e) })?;
    }

    opts.set_headers(&headers);

    let window = web_sys::window().expect("no window");
    let resp_value =
        JsFuture::from(window.fetch_with_str_and_init(&url, &opts))
            .await
            .map_err(|e| ApiError { status: 0, code: "NetworkError".into(), message: format!("Network error: {:?}", e) })?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|e| ApiError { status: 0, code: "ClientError".into(), message: format!("Failed to convert response: {:?}", e) })?;

    let status = resp.status();

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| ApiError { status, code: "ClientError".into(), message: format!("Failed to read body: {:?}", e) })?,
    )
    .await
    .map_err(|e| ApiError { status, code: "ClientError".into(), message: format!("Failed to await body: {:?}", e) })?;

    let text_str = text
        .as_string()
        .ok_or_else(|| ApiError { status, code: "ClientError".into(), message: "Response body is not a string".into() })?;

    if resp.ok() {
        serde_json::from_str::<Resp>(&text_str)
            .map_err(|e| ApiError { status, code: "ParseError".into(), message: format!("Failed to parse response: {}", e) })
    } else {
        if status == 401 {
            auth_service::clear_token();
        }
        #[derive(Deserialize)]
        struct ErrorResponse {
            #[serde(default)]
            code: String,
            message: String,
        }
        match serde_json::from_str::<ErrorResponse>(&text_str) {
            Ok(err) => Err(ApiError { status, code: err.code, message: err.message }),
            Err(_) => Err(ApiError { status, code: "Unknown".into(), message: format!("Request failed with status {}", status) }),
        }
    }
}

async fn request_json<Resp: DeserializeOwned>(
    method: &str,
    path: &str,
    body: Option<String>,
) -> Result<Resp, String> {
    request_json_typed(method, path, body).await.map_err(|e| e.message)
}

pub async fn post_json<Req: Serialize, Resp: DeserializeOwned>(
    path: &str,
    payload: &Req,
) -> Result<Resp, String> {
    let body = serde_json::to_string(payload)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    request_json("POST", path, Some(body)).await
}

pub async fn post_json_typed<Req: Serialize, Resp: DeserializeOwned>(
    path: &str,
    payload: &Req,
) -> Result<Resp, ApiError> {
    let body = serde_json::to_string(payload)
        .map_err(|e| ApiError { status: 0, code: "SerializeError".into(), message: format!("Failed to serialize request: {}", e) })?;
    request_json_typed("POST", path, Some(body)).await
}

async fn get_json<Resp: DeserializeOwned>(path: &str) -> Result<Resp, String> {
    request_json("GET", path, None).await
}

// --- Types ---

#[derive(Debug, Clone, Deserialize)]
pub struct ChatSummary {
    pub id: String,
    pub model: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub reasoning: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

// --- API functions ---

#[derive(Deserialize)]
pub struct ListChatsResponse {
    pub status: String,
    pub chats: Vec<ChatSummary>,
}

pub async fn list_chats() -> Result<ListChatsResponse, String> {
    post_json("/api/chat/list", &serde_json::json!({})).await
}

#[derive(Serialize)]
struct SendMessageRequest {
    message_id: String,
    model: String,
}

#[derive(Deserialize)]
pub struct SendMessageResponse {
    pub status: String,
    pub assistant_message_id: String,
}

pub async fn send_message(
    chat_id: &str,
    message_id: &str,
    model: &str,
) -> Result<SendMessageResponse, String> {
    post_json(
        &format!("/api/chat/{}/send-message", chat_id),
        &SendMessageRequest {
            message_id: message_id.to_string(),
            model: model.to_string(),
        },
    )
    .await
}

#[derive(Deserialize)]
pub struct ListModelsResponse {
    pub status: String,
    pub models: Vec<ModelInfo>,
}

pub async fn list_models() -> Result<ListModelsResponse, String> {
    get_json("/api/chat/models").await
}
