use leptos::*;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum BubblePhase {
    /// Waiting for first token from LLM
    Querying,
    /// Receiving thinking tokens
    Thinking,
    /// Receiving or finished receiving content tokens
    Content,
    /// Loaded from storage — no streaming state
    #[default]
    Done,
}

#[component]
pub fn ChatBubble(
    #[prop(into)] role: String,
    #[prop(into)] content: Signal<String>,
    #[prop(into, optional)] reasoning: Signal<Option<String>>,
    #[prop(into, optional)] thinking_count: Signal<usize>,
    #[prop(into, optional)] phase: Signal<BubblePhase>,
) -> impl IntoView {
    let is_user = role == "User";
    let bubble_class = if is_user {
        "chat-bubble chat-bubble--user"
    } else {
        "chat-bubble chat-bubble--assistant"
    };

    let thinking_expanded = create_rw_signal(false);

    view! {
        <div class=bubble_class>
            // Thinking badge on top border — only after thinking phase ends (Content/Done)
            {move || {
                if is_user {
                    return ().into_view();
                }
                let count = thinking_count.get();
                let p = phase.get();
                if count == 0 {
                    return ().into_view();
                }
                match p {
                    // During thinking: no badge, text is shown inline below
                    BubblePhase::Thinking | BubblePhase::Querying => ().into_view(),
                    // After thinking: collapsed badge on border
                    _ => {
                        view! {
                            <div
                                class="chat-bubble__thinking-badge"
                                on:click=move |_| thinking_expanded.update(|v| *v = !*v)
                            >
                                {move || format!("Thinking: {} tok", thinking_count.get())}
                            </div>
                        }.into_view()
                    }
                }
            }}

            // Thinking — inline collapsible during Thinking phase,
            // or expandable via badge after content arrives
            {move || {
                if is_user {
                    return ().into_view();
                }
                let p = phase.get();
                match p {
                    BubblePhase::Thinking => {
                        // Inside bubble: collapsed label, expandable on click
                        view! {
                            <div
                                class="chat-bubble__thinking-inline chat-bubble__thinking-inline--active"
                                on:click=move |_| thinking_expanded.update(|v| *v = !*v)
                            >
                                {move || format!("Thinking: {} tok", thinking_count.get())}
                            </div>
                            <pre
                                class=move || {
                                    if thinking_expanded.get() {
                                        "chat-bubble__reasoning-content chat-bubble__reasoning-content--live chat-bubble__reasoning-content--clickable"
                                    } else {
                                        "chat-bubble__reasoning-content chat-bubble__reasoning-content--live chat-bubble__reasoning-content--clickable chat-bubble__content--hidden"
                                    }
                                }
                                on:click=move |_| thinking_expanded.set(false)
                            >
                                {move || reasoning.get().unwrap_or_default()}
                            </pre>
                        }.into_view()
                    }
                    _ => {
                        // After thinking: expandable via badge click
                        view! {
                            {move || {
                                if !thinking_expanded.get() {
                                    return ().into_view();
                                }
                                match reasoning.get() {
                                    Some(text) if !text.is_empty() => {
                                        view! {
                                            <pre
                                                class="chat-bubble__reasoning-content chat-bubble__reasoning-content--clickable"
                                                on:click=move |_| thinking_expanded.set(false)
                                            >{text}</pre>
                                            <div class="chat-bubble__thinking-delimiter">
                                                <span on:click=move |_| thinking_expanded.set(false)>"close thinking"</span>
                                            </div>
                                        }.into_view()
                                    }
                                    _ => ().into_view()
                                }
                            }}
                        }.into_view()
                    }
                }
            }}

            // Main content area
            <div class=move || {
                match phase.get() {
                    BubblePhase::Querying => "chat-bubble__content chat-bubble__content--querying",
                    BubblePhase::Thinking => "chat-bubble__content chat-bubble__content--hidden",
                    _ => "chat-bubble__content",
                }
            }>
                {move || {
                    let p = phase.get();
                    let c = content.get();
                    match p {
                        BubblePhase::Querying => "Querying...".to_string(),
                        _ => c,
                    }
                }}
            </div>
        </div>
    }
}
