use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn App() -> impl IntoView {
    let (is_dark, set_is_dark) = create_signal(false);

    let toggle_theme = move |_| {
        let next = !is_dark.get_untracked();
        set_is_dark.set(next);

        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let html = document
            .document_element()
            .expect("no <html>")
            .unchecked_into::<web_sys::HtmlElement>();

        let theme = if next { "dark" } else { "light" };
        html.dataset().set("theme", theme).expect("failed to set data-theme");
    };

    view! {
        <div class="page">
            <button class="theme-toggle" on:click=toggle_theme>
                <span class="theme-toggle__icon">
                    {move || if is_dark.get() { "☀︎" } else { "☾" }}
                </span>
                {move || if is_dark.get() { "Light" } else { "Dark" }}
            </button>
            <h1 class="page__greeting">"Hello!"</h1>
        </div>
    }
}
