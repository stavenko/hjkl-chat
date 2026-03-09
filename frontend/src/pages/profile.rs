use leptos::*;
use leptos_router::*;

use crate::components::*;
use crate::services::auth_service;

pub(crate) fn profile_save_disabled_signals() -> (
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<bool>,
    Signal<bool>,
) {
    let name = create_rw_signal(String::new());
    let nickname = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        name.get();
        nickname.get();
        submitted.set(false);
    });

    let disabled = Signal::derive(move || submitted.get());

    (name, nickname, submitted, disabled)
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    if !auth_service::is_authenticated() {
        let navigate = use_navigate();
        navigate("/login", NavigateOptions {
            replace: true,
            ..Default::default()
        });
        return ().into_view();
    }

    let (name, nickname, submitted, disabled) = profile_save_disabled_signals();
    let error = create_rw_signal(None::<String>);
    let success = create_rw_signal(None::<String>);
    let loading = create_rw_signal(true);
    let emails = create_rw_signal(Vec::<auth_service::EmailInfo>::new());

    let navigate = use_navigate();

    // Load profile on mount
    spawn_local({
        let name = name;
        let nickname = nickname;
        let emails = emails;
        let loading = loading;
        let error = error;
        async move {
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
        }
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

    let on_logout = {
        let navigate = navigate.clone();
        move || {
            auth_service::clear_token();
            navigate("/login", NavigateOptions {
                replace: true,
                ..Default::default()
            });
        }
    };

    view! {
        <div class="auth-page">
            <Logo/>
            <p class="auth-page__tagline">"Your profile"</p>
            <Surface>
                <FormHeader text="Profile"/>
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
                            <AuthLink text="Change password" href="/me/change-password"/>
                            <Button
                                label="Log out"
                                disabled=Signal::derive(|| false)
                                on_click=on_logout.clone()
                            />
                        }.into_view()
                    }
                }}
            </Surface>
        </div>
    }.into_view()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn tick() {
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL))
            .await
            .unwrap();
    }

    #[wasm_bindgen_test]
    fn save_not_disabled_when_empty() {
        let runtime = create_runtime();
        let (_name, _nickname, _submitted, disabled) = profile_save_disabled_signals();
        assert!(!disabled.get_untracked(), "save should be enabled even with empty fields");
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn save_disabled_after_submit() {
        let runtime = create_runtime();
        let (_name, _nickname, submitted, disabled) = profile_save_disabled_signals();
        submitted.set(true);
        assert!(disabled.get_untracked(), "save must be disabled after submit");
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn save_reenabled_after_name_change() {
        let runtime = create_runtime();
        let (name, _nickname, submitted, disabled) = profile_save_disabled_signals();
        submitted.set(true);
        assert!(disabled.get_untracked());

        name.set("New Name".into());
        tick().await;
        assert!(!disabled.get_untracked(), "save must re-enable after name change");
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn save_reenabled_after_nickname_change() {
        let runtime = create_runtime();
        let (_name, nickname, submitted, disabled) = profile_save_disabled_signals();
        submitted.set(true);
        assert!(disabled.get_untracked());

        nickname.set("newnick".into());
        tick().await;
        assert!(!disabled.get_untracked(), "save must re-enable after nickname change");
        runtime.dispose();
    }
}
