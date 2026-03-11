use leptos::*;
use leptos_router::*;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::JsCast;

use crate::components::{ChatBubble, ChatInput, ModelSelector};
use crate::services::{auth_service, chat_service, ws_service};

#[derive(Clone)]
struct MessageBubble {
    id: String,
    role: String,
    content: RwSignal<String>,
    reasoning: RwSignal<Option<String>>,
}

#[component]
pub fn ChatPage() -> impl IntoView {
    if !auth_service::is_authenticated() {
        let navigate = use_navigate();
        navigate(
            "/login",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
        return view! { <div/> }.into_view();
    }

    let params = use_params_map();

    // If no chat ID in URL, generate one and redirect before creating any signals
    {
        let p = params.get_untracked();
        let id_param = p.get("id").cloned().unwrap_or_default();
        if id_param.is_empty() {
            let new_id = Uuid::new_v4().to_string();
            let navigate = use_navigate();
            navigate(
                &format!("/chat/{}", new_id),
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
            return view! { <div/> }.into_view();
        }
    }

    let chat_id = create_rw_signal(String::new());
    let messages: RwSignal<Vec<MessageBubble>> = create_rw_signal(Vec::new());
    let input_text = create_rw_signal(String::new());
    let sending = create_rw_signal(false);
    let models: RwSignal<Vec<(String, String)>> = create_rw_signal(Vec::new());
    let selected_model = create_rw_signal(String::new());
    let ws_connected = create_rw_signal(false);
    let error_msg: RwSignal<Option<String>> = create_rw_signal(None);
    let ws_conn: Rc<RefCell<Option<ws_service::WsConnection>>> = Rc::new(RefCell::new(None));
    let current_message_id: RwSignal<Option<String>> = create_rw_signal(None);
    let draft_timeout: Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));

    // Load models on mount
    spawn_local({
        let models = models;
        let selected_model = selected_model;
        async move {
            match chat_service::list_models().await {
                Ok(resp) => {
                    let model_list: Vec<(String, String)> = resp
                        .models
                        .into_iter()
                        .map(|m| (m.id.clone(), m.name))
                        .collect();
                    if !model_list.is_empty() && selected_model.get_untracked().is_empty() {
                        selected_model.set(model_list[0].0.clone());
                    }
                    models.set(model_list);
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to load models: {}", e).into());
                }
            }
        }
    });

    // Connect WebSocket
    {
        let ws_conn = ws_conn.clone();
        spawn_local(async move {
            if let Some(token) = auth_service::get_token() {
                let conn = ws_service::connect(
                    &token,
                    move |event| match event {
                        ws_service::WsEvent::Token {
                            message_id,
                            kind,
                            text,
                            ..
                        } => {
                            let msgs = messages.get_untracked();
                            if let Some(bubble) = msgs.iter().find(|m| m.id == message_id) {
                                if kind == "Thinking" {
                                    bubble.reasoning.update(|r| {
                                        let current = r.get_or_insert_with(String::new);
                                        current.push_str(&text);
                                    });
                                } else {
                                    bubble.content.update(|c| c.push_str(&text));
                                }
                            }
                        }
                        ws_service::WsEvent::MessageComplete { .. } => {
                            sending.set(false);
                        }
                        ws_service::WsEvent::Error { message, .. } => {
                            error_msg.set(Some(message));
                            sending.set(false);
                        }
                    },
                    move || {
                        ws_connected.set(true);
                    },
                    move || {
                        ws_connected.set(false);
                    },
                );
                *ws_conn.borrow_mut() = Some(conn);
            }
        });
    }

    // Initialize chat_id and load existing messages
    // (no-ID case is handled by the early return above)
    spawn_local({
        let params = params;
        let chat_id = chat_id;
        let messages = messages;
        async move {
            let p = params.get_untracked();
            let id = p.get("id").expect("id param must exist at this point");
            chat_id.set(id.clone());
            match chat_service::get_chat_messages(id, None).await {
                Ok(resp) => {
                    let bubbles: Vec<MessageBubble> = resp
                        .messages
                        .into_iter()
                        .map(|m| MessageBubble {
                            id: m.id,
                            role: m.role,
                            content: create_rw_signal(m.content),
                            reasoning: create_rw_signal(m.reasoning),
                        })
                        .collect();
                    messages.set(bubbles);
                }
                Err(_e) => {
                    // Chat might not exist yet, that's ok for new chats
                }
            }
        }
    });

    // Debounced draft saving on input change
    let draft_timeout_clone = draft_timeout.clone();
    create_effect(move |_| {
        let text = input_text.get();
        let cid = chat_id.get_untracked();
        let model = selected_model.get_untracked();

        if text.trim().is_empty() || cid.is_empty() || model.is_empty() {
            return;
        }

        // Generate message_id if we don't have one yet
        let mid = current_message_id.get_untracked().unwrap_or_else(|| {
            let new_mid = Uuid::new_v4().to_string();
            current_message_id.set(Some(new_mid.clone()));
            new_mid
        });

        // Clear existing timeout
        if let Some(timeout_id) = draft_timeout_clone.borrow_mut().take() {
            let window = web_sys::window().expect("no window");
            window.clear_timeout_with_handle(timeout_id);
        }

        // Set new debounced timeout (500ms)
        let window = web_sys::window().expect("no window");
        let cb = wasm_bindgen::closure::Closure::once(move || {
            spawn_local(async move {
                let _ = chat_service::save_draft(&cid, &mid, &text, &model).await;
            });
        });
        let timeout_id = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                500,
            )
            .expect("failed to set timeout");
        cb.forget();
        *draft_timeout_clone.borrow_mut() = Some(timeout_id);
    });

    let on_send = move || {
        let text = input_text.get_untracked().trim().to_string();
        if text.is_empty() || sending.get_untracked() {
            return;
        }

        let model = selected_model.get_untracked();
        if model.is_empty() {
            error_msg.set(Some("No model selected".to_string()));
            return;
        }

        // Cancel any pending draft timeout
        if let Some(timeout_id) = draft_timeout.borrow_mut().take() {
            let window = web_sys::window().expect("no window");
            window.clear_timeout_with_handle(timeout_id);
        }

        sending.set(true);
        error_msg.set(None);

        // Use existing message_id or generate one
        let message_id = current_message_id
            .get_untracked()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Reset current_message_id for next message
        current_message_id.set(None);

        let user_bubble = MessageBubble {
            id: message_id.clone(),
            role: "User".to_string(),
            content: create_rw_signal(text.clone()),
            reasoning: create_rw_signal(None),
        };
        messages.update(|m| m.push(user_bubble));
        input_text.set(String::new());

        spawn_local({
            let chat_id = chat_id;
            let model = model;
            let text = text;
            let messages = messages;
            let error_msg = error_msg;
            let sending = sending;
            async move {
                let cid = chat_id.get_untracked();

                // Save draft first (in case debounce didn't fire yet)
                if let Err(e) =
                    chat_service::save_draft(&cid, &message_id, &text, &model).await
                {
                    error_msg.set(Some(format!("Failed to save draft: {}", e)));
                    sending.set(false);
                    return;
                }

                match chat_service::send_message(&cid, &message_id, &model).await {
                    Ok(resp) => {
                        let assistant_bubble = MessageBubble {
                            id: resp.assistant_message_id,
                            role: "Assistant".to_string(),
                            content: create_rw_signal(String::new()),
                            reasoning: create_rw_signal(None),
                        };
                        messages.update(|m| m.push(assistant_bubble));
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Failed to send message: {}", e)));
                        sending.set(false);
                    }
                }
            }
        });
    };

    {
        let ws_conn = ws_conn.clone();
        on_cleanup(move || {
            if let Some(conn) = ws_conn.borrow().as_ref() {
                conn.close();
            }
        });
    }

    view! {
        <div class="chat-page">
            <div class="chat-page__header">
                <ModelSelector
                    models=Signal::derive(move || models.get())
                    selected=selected_model
                />
                <div class="chat-page__status">
                    {move || if ws_connected.get() {
                        view! { <span class="chat-page__status--connected">"Connected"</span> }.into_view()
                    } else {
                        view! { <span class="chat-page__status--disconnected">"Disconnected"</span> }.into_view()
                    }}
                </div>
            </div>

            {move || error_msg.get().map(|msg| {
                view! { <div class="chat-page__error">{msg}</div> }
            })}

            <div class="chat-messages">
                <For
                    each=move || messages.get()
                    key=|m| m.id.clone()
                    children=move |m| {
                        let role = m.role.clone();
                        let content = Signal::derive(move || m.content.get());
                        let reasoning = Signal::derive(move || m.reasoning.get());
                        view! {
                            <ChatBubble
                                role=role
                                content=content
                                reasoning=reasoning
                            />
                        }
                    }
                />
            </div>

            <ChatInput
                value=input_text
                disabled=MaybeSignal::derive(move || sending.get())
                on_send=on_send
            />
        </div>
    }
    .into_view()
}
