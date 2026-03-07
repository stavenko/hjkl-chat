use leptos::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

#[component]
pub fn AuthenticationInput(
    label: String,
    value: RwSignal<String>,
    error: RwSignal<Option<String>>,
    input_type: String,
) -> impl IntoView {
    view! {
        <div class="authentication-input">
            <label class="label">{label}</label>
            <div class="input-container">
                <input
                    type={input_type}
                    value=move || value.get()
                    on:change=move |ev| {
                        let target: HtmlInputElement = ev.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
                        let val = target.value();
                        value.set(val.clone());
                    }
                    class=move || {
                        let mut classes = vec!["input"];
                        if error.get().is_some() {
                            classes.push("input-error");
                        }
                        classes.join(" ")
                    }
                />
            </div>
            <Show when=move || error.get().is_some() fallback=|| view! {}>
                <div class="error-text">{move || error.get().unwrap()}</div>
            </Show>
        </div>
    }
}
