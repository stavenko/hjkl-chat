pub mod app;
pub mod components;
pub mod services;

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::spawn_local(async {
        services::init_api_base_url().await;
        leptos::mount_to_body(app::App);
    });
}
