use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

use crate::components::*;
use crate::services::auth_service;

#[component]
pub fn ProfileModal(
    open: RwSignal<bool>,
) -> impl IntoView {
    let name = create_rw_signal(String::new());
    let nickname = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);
    let success = create_rw_signal(None::<String>);
    let loading = create_rw_signal(true);
    let emails = create_rw_signal(Vec::<auth_service::EmailInfo>::new());

    create_effect(move |_| {
        name.get();
        nickname.get();
        submitted.set(false);
    });

    let disabled = Signal::derive(move || submitted.get());

    // Load profile when modal opens
    create_effect(move |prev: Option<bool>| {
        let is_open = open.get();
        if is_open && prev != Some(true) {
            loading.set(true);
            error.set(None);
            success.set(None);
            spawn_local(async move {
                match auth_service::get_me().await {
                    Ok(me) => {
                        name.set(me.name.unwrap_or_default());
                        nickname.set(me.nickname.unwrap_or_default());
                        emails.set(me.emails);
                        loading.set(false);
                    }
                    Err(msg) => {
                        error.set(Some(msg));
                        loading.set(false);
                    }
                }
            });
        }
        is_open
    });

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());
    let no_error: Signal<Option<String>> = Signal::derive(|| None);

    let on_save = move || {
        submitted.set(true);
        error.set(None);
        success.set(None);
        let name_val = name.get_untracked();
        let nickname_val = nickname.get_untracked();
        spawn_local(async move {
            let name_opt = if name_val.is_empty() { None } else { Some(name_val) };
            let nickname_opt = if nickname_val.is_empty() { None } else { Some(nickname_val) };
            match auth_service::update_profile(name_opt, nickname_opt).await {
                Ok(me) => {
                    name.set(me.name.unwrap_or_default());
                    nickname.set(me.nickname.unwrap_or_default());
                    emails.set(me.emails);
                    success.set(Some("Profile updated".to_string()));
                }
                Err(msg) => {
                    error.set(Some(msg));
                }
            }
        });
    };

    let on_logout = move || {
        auth_service::clear_token();
        let navigate = use_navigate();
        navigate("/login", NavigateOptions {
            replace: true,
            ..Default::default()
        });
    };

    let close = move |_| open.set(false);
    let close_on_backdrop = move |ev: web_sys::MouseEvent| {
        // Only close if clicking the backdrop itself, not the modal content
        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                if el.class_list().contains("profile-modal__backdrop") {
                    open.set(false);
                }
            }
        }
    };

    view! {
        <Show when=move || open.get()>
            <div class="profile-modal__backdrop" on:click=close_on_backdrop>
                <div class="profile-modal">
                    <div class="profile-modal__header">
                        <h2 class="profile-modal__title">"Profile"</h2>
                        <button class="profile-modal__close" on:click=close>"×"</button>
                    </div>
                    <div class="profile-modal__body">
                        {move || {
                            if loading.get() {
                                view! { <p class="form-description">"Loading..."</p> }.into_view()
                            } else {
                                view! {
                                    <div class="profile-emails">
                                        <span class="text-input__label">"Email"</span>
                                        <For
                                            each=move || emails.get()
                                            key=|e| e.email.clone()
                                            children=move |email| {
                                                view! {
                                                    <p class="profile-email">{email.email}</p>
                                                }
                                            }
                                        />
                                    </div>
                                    <TextInput
                                        label="Name"
                                        placeholder="Your name"
                                        value=name
                                        error=no_error
                                    />
                                    <TextInput
                                        label="Nickname"
                                        placeholder="Your nickname"
                                        value=nickname
                                        error=error_signal
                                    />
                                    {move || success.get().map(|msg| view! {
                                        <p class="profile-success">{msg}</p>
                                    })}
                                    <Button
                                        label="Save"
                                        disabled=disabled
                                        on_click=on_save.clone()
                                    />
                                    <Button
                                        label="Log out"
                                        disabled=Signal::derive(|| false)
                                        on_click=on_logout.clone()
                                    />
                                }.into_view()
                            }
                        }}
                    </div>
                </div>
            </div>
        </Show>
    }
}
