use leptos::*;

use crate::components::*;
use crate::services::auth_service;

pub(crate) fn is_valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return false;
    }
    let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let domain_parts: Vec<&str> = domain.split('.').collect();
    domain_parts.len() >= 2 && domain_parts.iter().all(|p| !p.is_empty())
}

pub(crate) fn registration_init_signals() -> (
    RwSignal<String>,
    RwSignal<bool>,
    RwSignal<bool>,
    Signal<Option<String>>,
    Signal<bool>,
) {
    let email = create_rw_signal(String::new());
    let touched = create_rw_signal(false);
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        email.get();
        submitted.set(false);
    });

    let email_error: Signal<Option<String>> = Signal::derive(move || {
        if !touched.get() {
            return None;
        }
        let val = email.get();
        if val.trim().is_empty() {
            return None;
        }
        if !is_valid_email(&val) {
            return Some("Invalid email address".to_string());
        }
        None
    });

    let disabled = Signal::derive(move || {
        !is_valid_email(&email.get()) || submitted.get()
    });

    (email, touched, submitted, email_error, disabled)
}

#[component]
pub fn RegistrationPage() -> impl IntoView {
    let (email, touched, submitted, email_error, disabled) = registration_init_signals();
    let server_error = create_rw_signal(None::<String>);

    let on_submit = move || {
        submitted.set(true);
        let email_val = email.get_untracked();

        spawn_local(async move {
            match auth_service::registration_init(&email_val).await {
                Ok(_response) => {
                    // TODO: navigate to verification step with session_id
                    leptos::logging::log!("Registration init success: {:?}", _response);
                }
                Err(msg) => {
                    server_error.set(Some(msg));
                }
            }
        });
    };

    let combined_error: Signal<Option<String>> = Signal::derive(move || {
        email_error.get().or_else(|| server_error.get())
    });

    view! {
        <div class="auth-page">
            <Logo/>
            <p class="auth-page__tagline">"Create your account"</p>
            <Surface>
                <div on:input=move |_| { touched.set(true); }>
                    <TextInput
                        label="Email"
                        placeholder="you@example.com"
                        value=email
                        error=combined_error
                    />
                </div>
                <Button
                    label="Continue"
                    disabled=disabled
                    on_click=on_submit
                />
                <AuthLink text="Already have an account? Sign in" href="/login"/>
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
    fn disabled_when_email_empty() {
        let runtime = create_runtime();
        let (_email, _touched, _submitted, _email_error, disabled) =
            registration_init_signals();

        assert!(disabled.get_untracked(), "button must be disabled when email is empty");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_when_email_invalid() {
        let runtime = create_runtime();
        let (email, _touched, _submitted, _email_error, disabled) =
            registration_init_signals();

        email.set("not-an-email".into());

        assert!(disabled.get_untracked(), "button must be disabled for invalid email");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn enabled_when_email_valid() {
        let runtime = create_runtime();
        let (email, _touched, _submitted, _email_error, disabled) =
            registration_init_signals();

        email.set("user@example.com".into());

        assert!(!disabled.get_untracked(), "button must be enabled for valid email");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_after_submit() {
        let runtime = create_runtime();
        let (email, _touched, submitted, _email_error, disabled) =
            registration_init_signals();

        email.set("user@example.com".into());
        assert!(!disabled.get_untracked());

        submitted.set(true);
        assert!(disabled.get_untracked(), "button must be disabled after submit");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn reenabled_after_email_change() {
        let runtime = create_runtime();
        let (email, _touched, submitted, _email_error, disabled) =
            registration_init_signals();

        email.set("user@example.com".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        email.set("other@example.com".into());
        tick().await;
        assert!(!disabled.get_untracked(), "button must re-enable after email change");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn no_error_when_untouched() {
        let runtime = create_runtime();
        let (email, _touched, _submitted, email_error, _disabled) =
            registration_init_signals();

        email.set("bad".into());
        assert!(email_error.get_untracked().is_none(), "no error before user touches input");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn error_shown_when_touched_and_invalid() {
        let runtime = create_runtime();
        let (email, touched, _submitted, email_error, _disabled) =
            registration_init_signals();

        email.set("bad".into());
        touched.set(true);
        assert!(email_error.get_untracked().is_some(), "error shown for invalid email after touch");

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn no_error_when_touched_and_valid() {
        let runtime = create_runtime();
        let (email, touched, _submitted, email_error, _disabled) =
            registration_init_signals();

        email.set("user@example.com".into());
        touched.set(true);
        assert!(email_error.get_untracked().is_none(), "no error for valid email");

        runtime.dispose();
    }

    // -- is_valid_email tests --

    #[wasm_bindgen_test]
    fn valid_email_accepted() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("a@b.co"));
        assert!(is_valid_email("name+tag@domain.org"));
    }

    #[wasm_bindgen_test]
    fn empty_email_rejected() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("   "));
    }

    #[wasm_bindgen_test]
    fn email_without_at_rejected() {
        assert!(!is_valid_email("userexample.com"));
    }

    #[wasm_bindgen_test]
    fn email_without_domain_dot_rejected() {
        assert!(!is_valid_email("user@localhost"));
    }

    #[wasm_bindgen_test]
    fn email_with_empty_parts_rejected() {
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@.com"));
        assert!(!is_valid_email("user@example."));
    }
}
