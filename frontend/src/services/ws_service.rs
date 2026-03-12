use serde::Deserialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use crate::services::get_api_base_url;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    Token {
        chat_id: String,
        message_id: String,
        kind: String,
        text: String,
    },
    MessageComplete {
        chat_id: String,
        message_id: String,
    },
    Error {
        chat_id: String,
        message: String,
    },
    SyncAvailable {
        version: u64,
    },
}

pub struct WsConnection {
    ws: WebSocket,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_close: Closure<dyn FnMut()>,
    _on_error: Closure<dyn FnMut(web_sys::ErrorEvent)>,
}

impl WsConnection {
    pub fn close(&self) {
        let _ = self.ws.close();
    }
}

impl Drop for WsConnection {
    fn drop(&mut self) {
        let _ = self.ws.close();
    }
}

pub fn connect(
    token: &str,
    on_event: impl Fn(WsEvent) + 'static,
    on_connected: impl Fn() + 'static,
    on_disconnected: impl Fn() + 'static,
) -> WsConnection {
    let base_url = get_api_base_url();

    let ws_url = if base_url.is_empty() {
        let window = web_sys::window().expect("no window");
        let location = window.location();
        let protocol = location.protocol().unwrap_or_default();
        let host = location.host().unwrap_or_default();
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        format!("{}//{}/api/ws?token={}", ws_protocol, host, token)
    } else {
        let ws_base = base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        format!("{}/api/ws?token={}", ws_base, token)
    };

    let ws = WebSocket::new(&ws_url).expect("failed to create WebSocket");
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
            let text_str: String = text.into();
            match serde_json::from_str::<WsEvent>(&text_str) {
                Ok(event) => on_event(event),
                Err(err) => {
                    web_sys::console::warn_1(
                        &format!("Failed to parse WS message: {}", err).into(),
                    );
                }
            }
        }
    }) as Box<dyn FnMut(MessageEvent)>);

    let on_open = Closure::wrap(Box::new(move || {
        on_connected();
    }) as Box<dyn FnMut()>);

    let on_close = Closure::wrap(Box::new(move || {
        on_disconnected();
    }) as Box<dyn FnMut()>);

    let on_error = Closure::wrap(Box::new(move |_e: web_sys::ErrorEvent| {
        web_sys::console::error_1(&"WebSocket error".into());
    }) as Box<dyn FnMut(web_sys::ErrorEvent)>);

    ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    on_open.forget();

    WsConnection {
        ws,
        _on_message: on_message,
        _on_close: on_close,
        _on_error: on_error,
    }
}
