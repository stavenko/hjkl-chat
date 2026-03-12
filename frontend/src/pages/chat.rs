use leptos::*;
use leptos_router::*;
use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use wasm_bindgen::JsCast;

use crate::components::icons::IconMenu;
use crate::components::{ChatBackground, ChatBubble, ChatInput, ModelSelector, ProfileModal, UserIcon};
use crate::services::local_storage::{LocalChatMessage, LocalDb, LocalDraftEntry};
use crate::services::sync_engine::SyncEngine;
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
    let profile_open = create_rw_signal(false);
    let user_name = create_rw_signal(String::new());
    let wallpaper_ref = create_node_ref::<html::Div>();
    let sync_status = create_rw_signal(String::new());

    // Shared LocalDb handle
    let local_db: Rc<RefCell<Option<Rc<LocalDb>>>> = Rc::new(RefCell::new(None));

    let scroll_to_bottom = move || {
        if let Some(el) = wallpaper_ref.get() {
            el.set_scroll_top(el.scroll_height());
        }
    };

    // Load user info for the avatar
    spawn_local({
        let user_name = user_name;
        async move {
            if let Ok(me) = auth_service::get_me().await {
                let display = me.nickname
                    .or(me.name)
                    .unwrap_or_else(|| me.emails.first().map(|e| e.email.clone()).unwrap_or_default());
                user_name.set(display);
            }
        }
    });

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

    // Initialize LocalDB, load from IndexedDB first, then sync
    let local_db_init = local_db.clone();
    spawn_local({
        let params = params;
        let chat_id = chat_id;
        let messages = messages;
        let sync_status = sync_status;
        async move {
            let p = params.get_untracked();
            let id = p.get("id").expect("id param must exist at this point");
            chat_id.set(id.clone());

            // Open IndexedDB
            match LocalDb::open().await {
                Ok(db) => {
                    let db = Rc::new(db);
                    *local_db_init.borrow_mut() = Some(db.clone());

                    // Step 1: Load from IndexedDB (instant)
                    match db.get_messages_for_chat(id).await {
                        Ok(local_msgs) if !local_msgs.is_empty() => {
                            let bubbles: Vec<MessageBubble> = local_msgs
                                .into_iter()
                                .map(|m| MessageBubble {
                                    id: m.id,
                                    role: m.role,
                                    content: create_rw_signal(m.content),
                                    reasoning: create_rw_signal(m.reasoning),
                                })
                                .collect();
                            messages.set(bubbles);
                            request_animation_frame(scroll_to_bottom);
                        }
                        _ => {}
                    }

                    // Step 2: Background sync pull from server
                    sync_status.set("Syncing...".to_string());
                    let engine = SyncEngine::new(db.clone());
                    match engine.pull().await {
                        Ok(had_changes) => {
                            sync_status.set(String::new());
                            if had_changes {
                                // Reload messages from IndexedDB after sync
                                if let Ok(local_msgs) = db.get_messages_for_chat(id).await {
                                    let bubbles: Vec<MessageBubble> = local_msgs
                                        .into_iter()
                                        .map(|m| MessageBubble {
                                            id: m.id,
                                            role: m.role,
                                            content: create_rw_signal(m.content),
                                            reasoning: create_rw_signal(m.reasoning),
                                        })
                                        .collect();
                                    messages.set(bubbles);
                                    request_animation_frame(scroll_to_bottom);
                                }
                            }
                        }
                        Err(e) => {
                            sync_status.set(String::new());
                            web_sys::console::warn_1(
                                &format!("Sync pull failed, using local data: {}", e).into(),
                            );
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("IndexedDB unavailable: {:?}", e).into(),
                    );
                }
            }
        }
    });

    // Connect WebSocket — handle SyncAvailable
    let local_db_ws = local_db.clone();
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
                        ws_service::WsEvent::MessageComplete { chat_id: event_chat_id, message_id } => {
                            sending.set(false);

                            // Save completed assistant message to IndexedDB
                            let db_ref = local_db_ws.borrow().clone();
                            if let Some(db) = db_ref {
                                let msgs = messages.get_untracked();
                                if let Some(bubble) = msgs.iter().find(|m| m.id == message_id) {
                                    let local_msg = LocalChatMessage {
                                        id: message_id.clone(),
                                        chat_id: event_chat_id,
                                        role: bubble.role.clone(),
                                        content: bubble.content.get_untracked(),
                                        reasoning: bubble.reasoning.get_untracked(),
                                        created_at: String::new(),
                                        version: 0,
                                    };
                                    let db = db.clone();
                                    spawn_local(async move {
                                        let _ = db.put_message(&local_msg).await;
                                    });
                                }
                            }
                        }
                        ws_service::WsEvent::Error { message, .. } => {
                            error_msg.set(Some(message));
                            sending.set(false);
                        }
                        ws_service::WsEvent::SyncAvailable { .. } => {
                            // New data available on server — trigger a sync pull
                            let db_ref = local_db_ws.borrow().clone();
                            if let Some(db) = db_ref {
                                let cid = chat_id.get_untracked();
                                spawn_local(async move {
                                    let engine = SyncEngine::new(db.clone());
                                    if let Ok(true) = engine.pull().await {
                                        // Reload messages from IndexedDB
                                        if let Ok(local_msgs) = db.get_messages_for_chat(&cid).await {
                                            let bubbles: Vec<MessageBubble> = local_msgs
                                                .into_iter()
                                                .map(|m| MessageBubble {
                                                    id: m.id,
                                                    role: m.role,
                                                    content: create_rw_signal(m.content),
                                                    reasoning: create_rw_signal(m.reasoning),
                                                })
                                                .collect();
                                            messages.set(bubbles);
                                            request_animation_frame(scroll_to_bottom);
                                        }
                                    }
                                });
                            }
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

    // Debounced draft saving — write to IndexedDB first, then background push to server
    let local_db_draft = local_db.clone();
    let draft_timeout_clone = draft_timeout.clone();
    create_effect(move |_| {
        let text = input_text.get();
        let cid = chat_id.get_untracked();
        let model = selected_model.get_untracked();

        if text.trim().is_empty() || cid.is_empty() || model.is_empty() {
            return;
        }

        let mid = current_message_id.get_untracked().unwrap_or_else(|| {
            let new_mid = Uuid::new_v4().to_string();
            current_message_id.set(Some(new_mid.clone()));
            new_mid
        });

        if let Some(timeout_id) = draft_timeout_clone.borrow_mut().take() {
            let window = web_sys::window().expect("no window");
            window.clear_timeout_with_handle(timeout_id);
        }

        let db_ref = local_db_draft.borrow().clone();
        let window = web_sys::window().expect("no window");
        let cb = wasm_bindgen::closure::Closure::once(move || {
            spawn_local(async move {
                // Save to IndexedDB first
                if let Some(db) = db_ref {
                    let draft = LocalDraftEntry {
                        id: mid.clone(),
                        chat_id: cid.clone(),
                        content: text.clone(),
                        model: model.clone(),
                        version: 0,
                    };
                    let _ = db.put_draft(&draft).await;
                }

                // Draft stays in IndexedDB only — will be pushed on send
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

    let local_db_send = local_db.clone();
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

        if let Some(timeout_id) = draft_timeout.borrow_mut().take() {
            let window = web_sys::window().expect("no window");
            window.clear_timeout_with_handle(timeout_id);
        }

        sending.set(true);
        error_msg.set(None);

        let message_id = current_message_id
            .get_untracked()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        current_message_id.set(None);

        let user_bubble = MessageBubble {
            id: message_id.clone(),
            role: "User".to_string(),
            content: create_rw_signal(text.clone()),
            reasoning: create_rw_signal(None),
        };
        messages.update(|m| m.push(user_bubble));
        input_text.set(String::new());
        request_animation_frame(scroll_to_bottom);

        let db_ref = local_db_send.borrow().clone();

        spawn_local({
            let chat_id = chat_id;
            let model = model;
            let text = text;
            let messages = messages;
            let error_msg = error_msg;
            let sending = sending;
            async move {
                let cid = chat_id.get_untracked();

                // Ensure draft is in IndexedDB before pushing
                if let Some(ref db) = db_ref {
                    let draft = LocalDraftEntry {
                        id: message_id.clone(),
                        chat_id: cid.clone(),
                        content: text.clone(),
                        model: model.clone(),
                        version: 0,
                    };
                    let _ = db.put_draft(&draft).await;
                }

                // Push draft to server via sync before sending
                if let Some(ref db) = db_ref {
                    let engine = SyncEngine::new(db.clone());
                    if let Err(e) = engine.push_drafts(&cid).await {
                        error_msg.set(Some(format!("Failed to sync draft: {}", e)));
                        sending.set(false);
                        return;
                    }
                }

                // Save user message to IndexedDB and remove draft
                if let Some(ref db) = db_ref {
                    let local_msg = LocalChatMessage {
                        id: message_id.clone(),
                        chat_id: cid.clone(),
                        role: "User".to_string(),
                        content: text.clone(),
                        reasoning: None,
                        created_at: String::new(),
                        version: 0,
                    };
                    let _ = db.put_message(&local_msg).await;
                    let _ = db.delete_draft(&message_id).await;
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
                        request_animation_frame(scroll_to_bottom);
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
            // Fade overlays — above messages, below floating UI
            <div class="chat-page__fade-top"></div>
            <div class="chat-page__fade-bottom"></div>

            // Fixed gradient
            <ChatBackground/>

            // Scrollable messages area
            <div class="chat-page__wallpaper" node_ref=wallpaper_ref>
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
            </div>

            // Floating header bar
            <div class="chat-page__header">
                <UserIcon
                    label=Signal::derive(move || user_name.get())
                    on_click=Box::new(move || profile_open.set(true))
                />
                <div class="chat-page__header-title">
                    <span>"Chat"</span>
                    {move || {
                        let sync = sync_status.get();
                        if !sync.is_empty() {
                            Some(view! {
                                <span class="chat-page__header-syncing">{sync}</span>
                            })
                        } else if !ws_connected.get() {
                            Some(view! {
                                <span class="chat-page__header-disconnected">"Disconnected"</span>
                            })
                        } else {
                            None
                        }
                    }}
                </div>
                <button class="chat-page__menu-btn" title="Menu">
                    <IconMenu/>
                </button>
            </div>

            // Model selector below avatar
            <div class="chat-page__model-bar">
                <ModelSelector
                    models=Signal::derive(move || models.get())
                    selected=selected_model
                />
            </div>

            // Error banner
            {move || error_msg.get().map(|msg| {
                view! { <div class="chat-page__error">{msg}</div> }
            })}

            // Floating input bar
            <div class="chat-page__footer">
                <ChatInput
                    value=input_text
                    disabled=MaybeSignal::derive(move || sending.get())
                    on_send=on_send
                />
            </div>

            // Profile modal
            <ProfileModal open=profile_open/>
        </div>
    }
    .into_view()
}
