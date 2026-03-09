use leptos::*;
use leptos_router::*;

use crate::components::*;

use crate::services::auth_service;

pub(crate) fn registration_complete_signals() -> (
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<bool>,
    Signal<bool>,
    Signal<PasswordStrength>,
    Signal<Option<String>>,
) {
    let password = create_rw_signal(String::new());
    let confirm = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        password.get();
        confirm.get();
        submitted.set(false);
    });

    let strength = Signal::derive(move || compute_password_strength(&password.get()));

    let mismatch_error = Signal::derive(move || {
        let c = confirm.get();
        if c.is_empty() || password.get() == c {
            None
        } else {
            Some("Passwords do not match".to_string())
        }
    });

    let disabled = Signal::derive(move || {
        let p = password.get();
        let c = confirm.get();
        if p.is_empty() || c.is_empty() || submitted.get() {
            return true;
        }
        p != c
    });

    (password, confirm, submitted, disabled, strength, mismatch_error)
}

#[component]
pub fn RegistrationSetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let session_id = query.with_untracked(|q| q.get("session_id").cloned().unwrap_or_default());
    let email = query.with_untracked(|q| q.get("email").cloned().unwrap_or_default());

    let error = create_rw_signal(None::<String>);
    let navigate = use_navigate();

    let (password, confirm, submitted, disabled, strength, mismatch_error) =
        registration_complete_signals();

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());

    let on_submit = move || {
        submitted.set(true);
        error.set(None);
        let sid = session_id.clone();
        let email_val = email.clone();
        let pwd = password.get_untracked();
        let conf = confirm.get_untracked();
        let navigate = navigate.clone();
        spawn_local(async move {
            match auth_service::registration_complete(&sid, &pwd, &conf).await {
                Ok(_) => {
                    let url = format!("/register/success?email={}", email_val);
                    navigate(&url, NavigateOptions::default());
                }
                Err(msg) => error.set(Some(msg)),
            }
        });
    };

    view! {
        <div class="auth-page">
            <Logo/>
            <p class="auth-page__tagline">"Create your account"</p>
            <Surface>
                <FormHeader text="Set Password"/>
                <PasswordWithStrengthInput
                    label="Password"
                    placeholder=""
                    value=password
                    error=error_signal
                    strength=strength
                />
                <PasswordLoginInput
                    label="Confirm Password"
                    placeholder=""
                    value=confirm
                    error=mismatch_error
                />
                <Button
                    label="Create Account"
                    disabled=disabled
                    on_click=on_submit
                />
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
    fn complete_disabled_when_empty() {
        let runtime = create_runtime();
        let (_pw, _conf, _sub, disabled, _str, _mis) = registration_complete_signals();
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_enabled_when_weak_but_matching() {
        let runtime = create_runtime();
        let (pw, conf, _sub, disabled, _str, _mis) = registration_complete_signals();
        pw.set("short".into());
        conf.set("short".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_disabled_when_mismatch() {
        let runtime = create_runtime();
        let (pw, conf, _sub, disabled, _str, _mis) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        conf.set("Abcdefg2".into());
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_enabled_when_medium_and_matching() {
        let runtime = create_runtime();
        let (pw, conf, _sub, disabled, _str, _mis) = registration_complete_signals();
        pw.set("abcdefgH".into());
        conf.set("abcdefgH".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_enabled_when_strong_and_matching() {
        let runtime = create_runtime();
        let (pw, conf, _sub, disabled, _str, _mis) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        conf.set("Abcdefg1".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_strength_tracks_password() {
        let runtime = create_runtime();
        let (pw, _conf, _sub, _disabled, strength, _mis) = registration_complete_signals();

        assert_eq!(strength.get_untracked(), PasswordStrength::None);
        pw.set("abc".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Weak);
        pw.set("abcdefgH".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Medium);
        pw.set("Abcdefg1".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Strong);

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_mismatch_error_when_different() {
        let runtime = create_runtime();
        let (pw, conf, _sub, _disabled, _str, mismatch) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        conf.set("Abcdefg2".into());
        assert!(mismatch.get_untracked().is_some());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_no_mismatch_when_matching() {
        let runtime = create_runtime();
        let (pw, conf, _sub, _disabled, _str, mismatch) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        conf.set("Abcdefg1".into());
        assert!(mismatch.get_untracked().is_none());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_no_mismatch_when_confirm_empty() {
        let runtime = create_runtime();
        let (pw, _conf, _sub, _disabled, _str, mismatch) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        assert!(mismatch.get_untracked().is_none());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_disabled_after_submit() {
        let runtime = create_runtime();
        let (pw, conf, submitted, disabled, _str, _mis) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        conf.set("Abcdefg1".into());
        assert!(!disabled.get_untracked());
        submitted.set(true);
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn complete_reenabled_after_password_change() {
        let runtime = create_runtime();
        let (pw, conf, submitted, disabled, _str, _mis) = registration_complete_signals();
        pw.set("Abcdefg1".into());
        conf.set("Abcdefg1".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        pw.set("Xbcdefg1".into());
        tick().await;
        conf.set("Xbcdefg1".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }
}
