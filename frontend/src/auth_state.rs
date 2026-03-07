use leptos::*;

fn get_from_storage(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|win| win.local_storage().ok().flatten())
        .and_then(|storage| storage.get(key).ok().flatten())
}

fn set_in_storage(key: &str, value: &str) -> Result<(), wasm_bindgen::JsValue> {
    let storage = web_sys::window()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?
        .local_storage()?
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no storage"))?;
    storage.set(key, value)?;
    Ok(())
}

#[allow(dead_code)]
fn remove_from_storage(key: &str) -> Result<(), wasm_bindgen::JsValue> {
    let storage = web_sys::window()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no window"))?
        .local_storage()?
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("no storage"))?;
    storage.remove_item(key)?;
    Ok(())
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct AuthState {
    pub access_token: ReadSignal<Option<String>>,
    pub refresh_token: ReadSignal<Option<String>>,
    pub user_id: ReadSignal<Option<String>>,
    pub user_email: ReadSignal<Option<String>>,
    access_token_write: WriteSignal<Option<String>>,
    refresh_token_write: WriteSignal<Option<String>>,
    user_id_write: WriteSignal<Option<String>>,
    user_email_write: WriteSignal<Option<String>>,
}

#[allow(dead_code)]
impl AuthState {
    pub fn new() -> Self {
        let (access_token, access_token_write) = create_signal(get_from_storage("access_token"));
        let (refresh_token, refresh_token_write) = create_signal(get_from_storage("refresh_token"));
        let (user_id, user_id_write) = create_signal(get_from_storage("user_id"));
        let (user_email, user_email_write) = create_signal(get_from_storage("user_email"));

        Self {
            access_token,
            refresh_token,
            user_id,
            user_email,
            access_token_write,
            refresh_token_write,
            user_id_write,
            user_email_write,
        }
    }

    pub fn save_tokens(&self, access: &str, refresh: &str, user_id: &str, user_email: &str) {
        set_in_storage("access_token", access).ok();
        set_in_storage("refresh_token", refresh).ok();
        set_in_storage("user_id", user_id).ok();
        set_in_storage("user_email", user_email).ok();
        self.access_token_write.set(Some(access.to_string()));
        self.refresh_token_write.set(Some(refresh.to_string()));
        self.user_id_write.set(Some(user_id.to_string()));
        self.user_email_write.set(Some(user_email.to_string()));
    }

    pub fn clear_tokens(&self) {
        remove_from_storage("access_token").ok();
        remove_from_storage("refresh_token").ok();
        remove_from_storage("user_id").ok();
        remove_from_storage("user_email").ok();
        self.access_token_write.set(None);
        self.refresh_token_write.set(None);
        self.user_id_write.set(None);
        self.user_email_write.set(None);
    }

    pub fn is_authenticated(&self) -> bool {
        self.access_token.get().is_some()
    }
}
