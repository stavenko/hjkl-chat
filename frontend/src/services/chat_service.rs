use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, RequestInit, RequestMode};

use crate::services::get_api_base_url;
use crate::services::auth_service;

async fn request_json<Resp: DeserializeOwned>(
    method: &str,
    path: &str,
    body: Option<String>,
) -> Result<Resp, String> {
    let base_url = get_api_base_url();
    let url = format!("{}{}", base_url, path);

    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::SameOrigin);

    if let Some(ref b) = body {
        opts.set_body(&wasm_bindgen::JsValue::from_str(b));
    }

    let headers =
        Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set header: {:?}", e))?;

    if let Some(token) = auth_service::get_token() {
        headers
            .set("Authorization", &format!("Bearer {}", token))
            .map_err(|e| format!("Failed to set auth header: {:?}", e))?;
    }

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
        serde_json::from_str::<Resp>(&text_str)
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        if resp.status() == 401 {
            auth_service::clear_token();
        }
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

async fn post_json<Req: Serialize, Resp: DeserializeOwned>(
    path: &str,
    payload: &Req,
) -> Result<Resp, String> {
    let body = serde_json::to_string(payload)
        .map_err(|e| format!("Failed to serialize request: {}", e))?;
    request_json("POST", path, Some(body)).await
}

async fn get_json<Resp: DeserializeOwned>(path: &str) -> Result<Resp, String> {
    request_json("GET", path, None).await
}

// --- Types ---

#[derive(Debug, Clone, Deserialize)]
pub struct ChatSummary {
    pub id: String,
    pub title: String,
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
    get_json("/api/chat").await
}

#[derive(Serialize)]
struct SaveDraftRequest {
    message_id: String,
    content: String,
    model: String,
}

#[derive(Deserialize)]
pub struct SaveDraftResponse {
    pub status: String,
    pub message_id: String,
}

pub async fn save_draft(
    chat_id: &str,
    message_id: &str,
    content: &str,
    model: &str,
) -> Result<SaveDraftResponse, String> {
    post_json(
        &format!("/api/chat/{}/draft", chat_id),
        &SaveDraftRequest {
            message_id: message_id.to_string(),
            content: content.to_string(),
            model: model.to_string(),
        },
    )
    .await
}

#[derive(Deserialize)]
pub struct GetChatMessagesResponse {
    pub status: String,
    pub messages: Vec<ChatMessage>,
}

pub async fn get_chat_messages(
    chat_id: &str,
    last_n: Option<usize>,
) -> Result<GetChatMessagesResponse, String> {
    let path = match last_n {
        Some(n) => format!("/api/chat/{}/messages?last_n={}", chat_id, n),
        None => format!("/api/chat/{}/messages", chat_id),
    };
    get_json(&path).await
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
        &format!("/api/chat/{}/send", chat_id),
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
