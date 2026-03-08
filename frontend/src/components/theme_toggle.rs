use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let (dark, set_dark) = create_signal(false);

    let toggle = move |_| {
        let next = !dark.get_untracked();
        set_dark.set(next);

        let document = web_sys::window().unwrap().document().unwrap();
        let html = document
            .document_element()
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        let dataset = html.dataset();
        dataset
            .set("theme", if next { "dark" } else { "light" })
            .unwrap();
    };

    view! {
        <button class="theme-toggle" on:click=toggle>
            <span class="theme-toggle__icon">
                {move || if dark.get() { "\u{263E}" } else { "\u{2600}" }}
            </span>
            {move || if dark.get() { "Dark" } else { "Light" }}
        </button>
    }
}
