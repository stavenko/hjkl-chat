use leptos::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

#[derive(Clone)]
struct ApiBaseUrl(String);

pub fn get_api_base_url() -> String {
    expect_context::<ApiBaseUrl>().0.clone()
}

pub async fn init_api_base_url() {
    let window = web_sys::window().unwrap();
    let promise = window.fetch_with_str("/config.json");
    let response_value = JsFuture::from(promise).await.unwrap();
    let response: web_sys::Response = response_value.dyn_into().unwrap();
    let text_promise = response.text().unwrap();
    let text: wasm_bindgen::JsValue = JsFuture::from(text_promise).await.unwrap();
    let config: serde_json::Value =
        serde_json::from_str(text.as_string().unwrap().as_str()).unwrap();
    let api_base_url = config["api_base_url"]
        .as_str()
        .unwrap()
        .to_string();

    provide_context(ApiBaseUrl(api_base_url));
}