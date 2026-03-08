use leptos::*;

#[component]
pub fn Button(
    #[prop(into)] label: String,
    #[prop(optional)] disabled: bool,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let class = if disabled {
        "btn btn--disabled"
    } else {
        "btn"
    };

    view! {
        <button class=class disabled=disabled on:click=move |_| {
            if !disabled {
                on_click();
            }
        }>
            {label}
        </button>
    }
}
