use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div>"Hello from Leptos!"</div>
    }
}

mod services;

#[allow(clippy::main_recursion)]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::spawn_local(async {
        services::init_api_base_url().await;
        mount_to_body(App);
    });
}
