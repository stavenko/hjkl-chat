use leptos::*;

use super::icons::*;

#[component]
pub fn PasswordLoginInput(
    #[prop(into)] label: String,
    #[prop(into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(into)] error: Signal<Option<String>>,
) -> impl IntoView {
    let (visible, set_visible) = create_signal(false);

    let field_class = move || {
        if error.get().is_some() {
            "text-input__field password-input__field text-input__field--error"
        } else {
            "text-input__field password-input__field"
        }
    };

    let input_ref = create_node_ref::<leptos::html::Input>();

    let toggle_visibility = move |_| {
        let next = !visible.get_untracked();
        set_visible.set(next);
        if let Some(el) = input_ref.get_untracked() {
            let el: &web_sys::HtmlInputElement = el.as_ref();
            el.set_type(if next { "text" } else { "password" });
        }
    };

    view! {
        <div class="text-input">
            <label class="text-input__label">{label}</label>
            <div class="password-input__wrapper">
                <input
                    node_ref=input_ref
                    class=field_class
                    type="password"
                    placeholder=placeholder
                    prop:value=move || value.get()
                    on:input=move |ev| {
                        value.set(event_target_value(&ev));
                    }
                />
                <button
                    type="button"
                    class="password-input__eye"
                    on:click=toggle_visibility
                    tabindex=-1
                >
                    {move || if visible.get() {
                        view! { <IconEye/> }.into_view()
                    } else {
                        view! { <IconEyeOff/> }.into_view()
                    }}
                </button>
            </div>
            {move || {
                error.get().map(|msg| {
                    view! { <span class="text-input__error">{msg}</span> }
                })
            }}
        </div>
    }
}
