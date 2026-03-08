use leptos::*;

#[component]
pub fn Surface(children: Children) -> impl IntoView {
    view! {
        <div class="surface">{children()}</div>
    }
}
