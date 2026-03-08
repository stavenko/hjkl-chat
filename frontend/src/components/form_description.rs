use leptos::*;

#[component]
pub fn FormDescription(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <p class="form-description">{text}</p>
    }
}
