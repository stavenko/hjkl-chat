use leptos::*;

#[component]
pub fn ModelSelector(
    models: Signal<Vec<(String, String)>>,
    selected: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="model-selector">
            <select
                class="model-selector__select"
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    selected.set(val);
                }
                prop:value=move || selected.get()
            >
                <For
                    each=move || models.get()
                    key=|m| m.0.clone()
                    children=move |m| {
                        let id = m.0.clone();
                        let name = m.1.clone();
                        view! {
                            <option value=id>{name}</option>
                        }
                    }
                />
            </select>
        </div>
    }
}
