use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div>"Hello from Leptos!"</div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

fn main() {
    run();
}
