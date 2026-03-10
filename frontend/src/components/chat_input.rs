use leptos::*;

#[component]
pub fn ChatInput(
    value: RwSignal<String>,
    #[prop(into, optional)] disabled: MaybeSignal<bool>,
    on_send: impl Fn() + 'static,
) -> impl IntoView {
    let on_send = std::rc::Rc::new(on_send);
    let on_send_clone = on_send.clone();

    view! {
        <div class="chat-input">
            <textarea
                class="chat-input__textarea"
                placeholder="Type a message..."
                rows="1"
                prop:value=move || value.get()
                prop:disabled=move || disabled.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                }
                on:keydown=move |ev| {
                    if ev.key() == "Enter" && !ev.shift_key() {
                        ev.prevent_default();
                        if !disabled.get() && !value.get().trim().is_empty() {
                            on_send_clone();
                        }
                    }
                }
            />
            <button
                class=move || {
                    if disabled.get() || value.get().trim().is_empty() {
                        "chat-input__send chat-input__send--disabled"
                    } else {
                        "chat-input__send"
                    }
                }
                disabled=move || disabled.get() || value.get().trim().is_empty()
                on:click=move |_| {
                    if !disabled.get() && !value.get().trim().is_empty() {
                        on_send();
                    }
                }
            >
                "Send"
            </button>
        </div>
    }
}
