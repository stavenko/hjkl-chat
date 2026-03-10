use leptos::*;

#[component]
pub fn ChatBubble(
    #[prop(into)] role: String,
    #[prop(into)] content: Signal<String>,
    #[prop(into, optional)] reasoning: Signal<Option<String>>,
) -> impl IntoView {
    let is_user = role == "User";
    let bubble_class = if is_user {
        "chat-bubble chat-bubble--user"
    } else {
        "chat-bubble chat-bubble--assistant"
    };

    let reasoning_expanded = create_rw_signal(false);

    view! {
        <div class=bubble_class>
            {move || {
                let r = reasoning.get();
                if let Some(text) = r {
                    if !text.is_empty() {
                        return view! {
                            <div class="chat-bubble__reasoning">
                                <button
                                    class="chat-bubble__reasoning-toggle"
                                    on:click=move |_| reasoning_expanded.update(|v| *v = !*v)
                                >
                                    {move || if reasoning_expanded.get() { "Hide reasoning" } else { "Show reasoning" }}
                                </button>
                                {move || reasoning_expanded.get().then(|| {
                                    view! {
                                        <pre class="chat-bubble__reasoning-content">{reasoning.get().unwrap_or_default()}</pre>
                                    }
                                })}
                            </div>
                        }.into_view();
                    }
                }
                ().into_view()
            }}
            <div class="chat-bubble__content">
                {move || content.get()}
            </div>
        </div>
    }
}
