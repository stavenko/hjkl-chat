use leptos::ev::MouseEvent;
use leptos::*;

#[component]
pub fn AuthenticationButton(
    disabled: RwSignal<bool>,
    label: String,
    on_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <button
            type="submit"
            disabled=move || disabled.get()
            class=move || {
                let mut classes = vec!["authentication-button"];
                if disabled.get() {
                    classes.push("disabled");
                } else {
                    classes.push("primary");
                }
                classes.join(" ")
            }
            on:click=on_click
        >
            {label}
        </button>
    }
}
