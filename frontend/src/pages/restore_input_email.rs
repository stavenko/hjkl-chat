use leptos::*;
use leptos_router::*;

use crate::components::*;
use crate::services::auth_service;

pub(crate) fn restore_request_signals() -> (RwSignal<String>, RwSignal<bool>, Signal<bool>) {
    let email = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        email.get();
        submitted.set(false);
    });

    let disabled = Signal::derive(move || email.get().is_empty() || submitted.get());

    (email, submitted, disabled)
}

#[component]
pub fn RestoreInputEmailPage() -> impl IntoView {
    let error = create_rw_signal(None::<String>);
    let navigate = use_navigate();

    let (email, submitted, disabled) = restore_request_signals();

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());

    let on_submit = move || {
        submitted.set(true);
        error.set(None);
        let email_val = email.get_untracked();
        let navigate = navigate.clone();
        spawn_local(async move {
            match auth_service::password_restore_init(&email_val).await {
                Ok(response) => {
                    let url = format!(
                        "/password/restore/verify?email={}&resend_at={}",
                        email_val, response.resend_available_at
                    );
                    navigate(&url, NavigateOptions::default());
                }
                Err(msg) => error.set(Some(msg)),
            }
        });
    };

    view! {
        <div class="auth-page">
            <Logo/>
            <p class="auth-page__tagline">"Restore access to your account"</p>
            <Surface>
                <FormHeader text="Restore Password"/>
                <TextInput
                    label="Email"
                    placeholder="user@example.com"
                    value=email
                    error=error_signal
                />
                <Button
                    label="Send Code"
                    disabled=disabled
                    on_click=on_submit
                />
                <AuthLink text="Back to Sign In" href="/login"/>
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

    async fn tick() {
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL))
            .await
            .unwrap();
    }

    #[wasm_bindgen_test]
    fn request_disabled_when_empty() {
        let runtime = create_runtime();
        let (_email, _submitted, disabled) = restore_request_signals();
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn request_enabled_when_email_filled() {
        let runtime = create_runtime();
        let (email, _submitted, disabled) = restore_request_signals();
        email.set("user@test.com".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn request_disabled_after_submit() {
        let runtime = create_runtime();
        let (email, submitted, disabled) = restore_request_signals();
        email.set("user@test.com".into());
        submitted.set(true);
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn request_reenabled_after_email_change() {
        let runtime = create_runtime();
        let (email, submitted, disabled) = restore_request_signals();
        email.set("user@test.com".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        email.set("other@test.com".into());
        tick().await;
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }
}
