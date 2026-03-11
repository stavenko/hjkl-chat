use leptos::*;
use wasm_bindgen::JsCast;

use crate::components::icons::{IconPaperclip, IconSend};

fn auto_resize(el: &web_sys::HtmlTextAreaElement) {
    el.style().set_property("height", "auto").ok();
    let scroll_h = el.scroll_height();
    el.style()
        .set_property("height", &format!("{}px", scroll_h))
        .ok();

    // Single-line check: compare scrollHeight to one line height
    let one_line = 40; // --size-input-height
    if scroll_h > one_line {
        el.class_list().add_1("chat-input__textarea--multiline").ok();
    } else {
        el.class_list().remove_1("chat-input__textarea--multiline").ok();
    }
}

#[component]
pub fn ChatInput(
    value: RwSignal<String>,
    #[prop(into, optional)] disabled: MaybeSignal<bool>,
    on_send: impl Fn() + 'static,
) -> impl IntoView {
    let on_send = std::rc::Rc::new(on_send);
    let on_send_clone = on_send.clone();

    let can_send = Signal::derive(move || !disabled.get() && !value.get().trim().is_empty());

    view! {
        <div class="chat-input">
            <button class="chat-input__attach" title="Attach file">
                <IconPaperclip/>
            </button>
            <textarea
                class="chat-input__textarea"
                placeholder="Type a message..."
                rows="1"
                prop:value=move || value.get()
                prop:disabled=move || disabled.get()
                on:input=move |ev| {
                    value.set(event_target_value(&ev));
                    if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()) {
                        auto_resize(&el);
                    }
                }
                on:keydown=move |ev| {
                    if ev.key() == "Enter" && !ev.shift_key() {
                        ev.prevent_default();
                        if can_send.get_untracked() {
                            on_send_clone();
                            if let Some(el) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()) {
                                el.style().set_property("height", "auto").ok();
                                el.class_list().remove_1("chat-input__textarea--multiline").ok();
                            }
                        }
                    }
                }
            />
            <button
                class=move || {
                    if can_send.get() {
                        "chat-input__send"
                    } else {
                        "chat-input__send chat-input__send--disabled"
                    }
                }
                disabled=move || !can_send.get()
                on:click=move |_| {
                    if can_send.get_untracked() {
                        on_send();
                    }
                }
            >
                <IconSend/>
            </button>
        </div>
    }
}
