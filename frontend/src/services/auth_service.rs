use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Headers, RequestInit, RequestMode};

use crate::services::get_api_base_url;

async fn post_json<Req: Serialize, Resp: DeserializeOwned>(
    path: &str,
    payload: &Req,
) -> Result<Resp, String> {
    let base_url = get_api_base_url();
    let url = format!("{}{}", base_url, path);

    let body = serde_json::to_string(payload)
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
        serde_json::from_str::<Resp>(&text_str)
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

// --- Login ---

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub status: String,
    pub user: UserInfo,
    pub token: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
}

pub async fn login(email: &str, password: &str) -> Result<LoginResponse, String> {
    post_json(
        "/api/auth/login",
        &LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        },
    )
    .await
}

// --- Password Restore ---

#[derive(Serialize)]
struct PasswordRestoreInitRequest {
    email: String,
}

#[derive(Deserialize)]
pub struct PasswordRestoreInitResponse {
    pub status: String,
    pub message: String,
    pub resend_available_at: f64,
}

#[derive(Serialize)]
struct PasswordRestoreVerifyRequest {
    email: String,
    code: String,
}

#[derive(Deserialize)]
pub struct PasswordRestoreVerifyResponse {
    pub status: String,
    pub session_id: String,
}

#[derive(Serialize)]
struct PasswordRestoreCompleteRequest {
    session_id: String,
    password: String,
    password_confirm: String,
}

#[derive(Deserialize)]
pub struct PasswordRestoreCompleteResponse {
    pub status: String,
    pub message: String,
}

pub async fn password_restore_init(
    email: &str,
) -> Result<PasswordRestoreInitResponse, String> {
    post_json(
        "/api/auth/password/restore/init",
        &PasswordRestoreInitRequest {
            email: email.to_string(),
        },
    )
    .await
}

pub async fn password_restore_verify(
    email: &str,
    code: &str,
) -> Result<PasswordRestoreVerifyResponse, String> {
    post_json(
        "/api/auth/password/restore/verify",
        &PasswordRestoreVerifyRequest {
            email: email.to_string(),
            code: code.to_string(),
        },
    )
    .await
}

pub async fn password_restore_complete(
    session_id: &str,
    password: &str,
    password_confirm: &str,
) -> Result<PasswordRestoreCompleteResponse, String> {
    post_json(
        "/api/auth/password/restore/complete",
        &PasswordRestoreCompleteRequest {
            session_id: session_id.to_string(),
            password: password.to_string(),
            password_confirm: password_confirm.to_string(),
        },
    )
    .await
}

// --- Token management ---

pub fn is_authenticated() -> bool {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage.get_item("token").ok().flatten().is_some()
}

pub fn store_token(token: &str) {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage
        .set_item("token", token)
        .expect("failed to store token");
}

pub fn get_token() -> Option<String> {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage.get_item("token").ok().flatten()
}

// --- Registration ---

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
    post_json(
        "/api/auth/registration/init",
        &RegistrationInitRequest {
            email: email.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
struct RegistrationVerifyRequest {
    session_id: String,
    code: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RegistrationVerifyResponse {
    pub status: String,
    pub session_id: String,
    pub expires_at: String,
}

pub async fn registration_verify(
    session_id: &str,
    code: &str,
) -> Result<RegistrationVerifyResponse, String> {
    post_json(
        "/api/auth/registration/verify",
        &RegistrationVerifyRequest {
            session_id: session_id.to_string(),
            code: code.to_string(),
        },
    )
    .await
}

#[derive(Serialize)]
struct RegistrationCompleteRequest {
    session_id: String,
    password: String,
    password_confirm: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RegistrationCompleteResponse {
    pub status: String,
    pub message: String,
}

pub async fn registration_complete(
    session_id: &str,
    password: &str,
    password_confirm: &str,
) -> Result<RegistrationCompleteResponse, String> {
    post_json(
        "/api/auth/registration/complete",
        &RegistrationCompleteRequest {
            session_id: session_id.to_string(),
            password: password.to_string(),
            password_confirm: password_confirm.to_string(),
        },
    )
    .await
}

pub fn clear_token() {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .expect("no local_storage")
        .expect("local_storage is None");
    storage
        .remove_item("token")
        .expect("failed to remove token");
}
