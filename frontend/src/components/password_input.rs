use leptos::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordStrength {
    None,
    Weak,
    Medium,
    Strong,
}

#[component]
pub fn PasswordInput(
    #[prop(into)] label: String,
    #[prop(into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(into)] strength: Signal<PasswordStrength>,
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

    let strength_class = move |segment: u8| {
        let s = strength.get();
        let active = match s {
            PasswordStrength::None => 0,
            PasswordStrength::Weak => 1,
            PasswordStrength::Medium => 2,
            PasswordStrength::Strong => 3,
        };
        if segment <= active && active > 0 {
            match s {
                PasswordStrength::Weak => "strength-bar__segment strength-bar__segment--weak",
                PasswordStrength::Medium => "strength-bar__segment strength-bar__segment--medium",
                PasswordStrength::Strong => "strength-bar__segment strength-bar__segment--strong",
                PasswordStrength::None => "strength-bar__segment",
            }
        } else {
            "strength-bar__segment"
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
                    {move || if visible.get() { "\u{1F441}" } else { "\u{25C9}" }}
                </button>
            </div>
            <div class="strength-bar">
                <div class=move || strength_class(1)></div>
                <div class=move || strength_class(2)></div>
                <div class=move || strength_class(3)></div>
            </div>
            {move || {
                error.get().map(|msg| {
                    view! { <span class="text-input__error">{msg}</span> }
                })
            }}
        </div>
    }
}
