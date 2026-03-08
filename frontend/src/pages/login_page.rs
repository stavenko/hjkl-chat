use leptos::*;
use leptos_router::*;

use crate::components::*;
use crate::services::auth_service;

/// Creates the login page disabled-button signal logic.
/// Extracted so it can be unit-tested without mounting the full component.
pub(crate) fn login_disabled_signals() -> (RwSignal<String>, RwSignal<String>, RwSignal<bool>, Signal<bool>) {
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        email.get();
        password.get();
        submitted.set(false);
    });

    let disabled = Signal::derive(move || {
        email.get().is_empty() || password.get().is_empty() || submitted.get()
    });

    (email, password, submitted, disabled)
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, password, submitted, disabled) = login_disabled_signals();
    let error = create_rw_signal(None::<String>);

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());
    let no_error: Signal<Option<String>> = Signal::derive(|| None);

    let navigate = use_navigate();

    let on_submit = move || {
        submitted.set(true);
        let email_val = email.get_untracked();
        let password_val = password.get_untracked();
        let navigate = navigate.clone();

        spawn_local(async move {
            match auth_service::login(&email_val, &password_val).await {
                Ok(response) => {
                    auth_service::store_tokens(
                        &response.access_token,
                        &response.refresh_token,
                    );
                    navigate("/", NavigateOptions::default());
                }
                Err(msg) => {
                    error.set(Some(msg));
                }
            }
        });
    };

    view! {
        <div class="auth-page">
            <Logo/>
            <p class="auth-page__tagline">"Authenticate to see your projects"</p>
            <Surface>
                <TextInput
                    label="Email"
                    placeholder="user@example.com"
                    value=email
                    error=no_error
                />
                <PasswordLoginInput
                    label="Password"
                    placeholder=""
                    value=password
                    error=error_signal
                />
                <Button
                    label="Sign In"
                    disabled=disabled
                    on_click=on_submit
                />
                <AuthLink text="Forgot password?" href="/password/restore"/>
                <AuthLink text="Don't have an account? Register" href="/register"/>
            </Surface>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Yield to the microtask queue so Leptos effects can execute.
    async fn tick() {
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL))
            .await
            .unwrap();
    }

    #[wasm_bindgen_test]
    fn disabled_when_fields_empty() {
        let runtime = create_runtime();
        let (_email, _password, _submitted, disabled) = login_disabled_signals();

        assert!(disabled.get_untracked(), "button must be disabled when both fields are empty");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_when_only_email_filled() {
        let runtime = create_runtime();
        let (email, _password, _submitted, disabled) = login_disabled_signals();

        email.set("user@test.com".into());

        assert!(disabled.get_untracked(), "button must be disabled when password is empty");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn enabled_when_both_fields_filled() {
        let runtime = create_runtime();
        let (email, password, _submitted, disabled) = login_disabled_signals();

        email.set("user@test.com".into());
        password.set("secret".into());

        assert!(!disabled.get_untracked(), "button must be enabled when both fields are filled");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_after_submit() {
        let runtime = create_runtime();
        let (email, password, submitted, disabled) = login_disabled_signals();

        email.set("user@test.com".into());
        password.set("secret".into());
        assert!(!disabled.get_untracked());

        submitted.set(true);
        assert!(disabled.get_untracked(), "button must be disabled after submit");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn reenabled_after_email_change() {
        let runtime = create_runtime();
        let (email, password, submitted, disabled) = login_disabled_signals();

        email.set("user@test.com".into());
        password.set("secret".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        email.set("other@test.com".into());
        tick().await;
        assert!(!disabled.get_untracked(), "button must re-enable after email change");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn reenabled_after_password_change() {
        let runtime = create_runtime();
        let (email, password, submitted, disabled) = login_disabled_signals();

        email.set("user@test.com".into());
        password.set("secret".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        password.set("new_secret".into());
        tick().await;
        assert!(!disabled.get_untracked(), "button must re-enable after password change");

        runtime.dispose();
    }
}
