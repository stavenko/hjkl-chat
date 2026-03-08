use leptos::*;

#[component]
pub fn Button(
    #[prop(into)] label: String,
    #[prop(into, optional)] disabled: MaybeSignal<bool>,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let class = move || {
        if disabled.get() {
            "btn btn--disabled"
        } else {
            "btn"
        }
    };

    view! {
        <button class=class disabled=move || disabled.get() on:click=move |_| {
            if !disabled.get() {
                on_click();
            }
        }>
            {label}
        </button>
    }
}
