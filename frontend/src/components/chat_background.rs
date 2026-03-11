use leptos::*;

#[component]
pub fn ChatBackground() -> impl IntoView {
    view! {
        <div class="chat-background">
            <img class="chat-background__gradient" src="/chat-bg-gradient.svg" alt="" />
        </div>
    }
}
