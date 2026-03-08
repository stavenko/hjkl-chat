use leptos::*;

#[component]
pub fn TextInput(
    #[prop(into)] label: String,
    #[prop(into)] placeholder: String,
    value: RwSignal<String>,
    #[prop(into)] error: Signal<Option<String>>,
) -> impl IntoView {
    let field_class = move || {
        if error.get().is_some() {
            "text-input__field text-input__field--error"
        } else {
            "text-input__field"
        }
    };

    view! {
        <div class="text-input">
            <label class="text-input__label">{label}</label>
            <input
                class=field_class
                type="text"
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
            />
            {move || {
                error.get().map(|msg| {
                    view! { <span class="text-input__error">{msg}</span> }
                })
            }}
        </div>
    }
}
