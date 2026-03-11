use leptos::*;

#[component]
pub fn UserIcon(
    #[prop(into, optional)] label: MaybeSignal<String>,
    #[prop(optional)] on_click: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let initial = Signal::derive(move || {
        let l = label.get();
        if l.is_empty() {
            "?".to_string()
        } else {
            l.chars().next().unwrap_or('?').to_uppercase().to_string()
        }
    });

    let handle_click = move |_| {
        if let Some(ref cb) = on_click {
            cb();
        }
    };

    view! {
        <button class="user-icon" on:click=handle_click title="Profile">
            <span class="user-icon__initial">{initial}</span>
        </button>
    }
}
