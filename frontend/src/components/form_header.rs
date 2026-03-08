use leptos::*;

#[component]
pub fn FormHeader(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <h2 class="form-header">{text}</h2>
    }
}
