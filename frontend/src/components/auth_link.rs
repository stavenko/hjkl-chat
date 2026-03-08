use leptos::*;

#[component]
pub fn AuthLink(
    #[prop(into)] text: String,
    #[prop(into)] href: String,
) -> impl IntoView {
    view! {
        <a class="auth-link" href=href>{text}</a>
    }
}
