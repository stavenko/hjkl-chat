use leptos::*;
use leptos_router::*;

use crate::components::*;
use crate::services::auth_service;

pub(crate) fn restore_complete_signals() -> (
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
pub fn RestoreSetPasswordPage() -> impl IntoView {
    let query = use_query_map();
    let session_id = query.with_untracked(|q| q.get("session_id").cloned().unwrap_or_default());

    let error = create_rw_signal(None::<String>);
    let navigate = use_navigate();

    let (password, confirm, submitted, disabled, strength, mismatch_error) =
        restore_complete_signals();

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());

    let on_submit = move || {
        submitted.set(true);
        error.set(None);
        let sid = session_id.clone();
        let pwd = password.get_untracked();
        let conf = confirm.get_untracked();
        let navigate = navigate.clone();
        spawn_local(async move {
            match auth_service::password_restore_complete(&sid, &pwd, &conf).await {
                Ok(_) => {
                    navigate("/password/restore/success", NavigateOptions::default());
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
                <FormHeader text="Set New Password"/>
                <PasswordWithStrengthInput
                    label="New Password"
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
                    label="Set Password"
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

    // --- compute_password_strength ---

    #[wasm_bindgen_test]
    fn strength_none_when_empty() {
        assert_eq!(compute_password_strength(""), PasswordStrength::None);
    }

    #[wasm_bindgen_test]
    fn strength_weak_when_short() {
        assert_eq!(compute_password_strength("Ab1"), PasswordStrength::Weak);
    }

    #[wasm_bindgen_test]
    fn strength_weak_when_long_but_one_class() {
        assert_eq!(compute_password_strength("abcdefgh"), PasswordStrength::Weak);
    }

    #[wasm_bindgen_test]
    fn strength_medium_with_two_classes() {
        assert_eq!(compute_password_strength("abcdefgH"), PasswordStrength::Medium);
    }

    #[wasm_bindgen_test]
    fn strength_medium_with_lower_and_digit() {
        assert_eq!(compute_password_strength("abcdefg1"), PasswordStrength::Medium);
    }

    #[wasm_bindgen_test]
    fn strength_strong_with_all_classes() {
        assert_eq!(compute_password_strength("Abcdefg1"), PasswordStrength::Strong);
    }

    // --- restore_complete_signals ---

    #[wasm_bindgen_test]
    fn complete_disabled_when_empty() {
        let runtime = create_runtime();
        let (_password, _confirm, _submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_enabled_when_weak_but_matching() {
        let runtime = create_runtime();
        let (password, confirm, _submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        password.set("short".into());
        confirm.set("short".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_disabled_when_mismatch() {
        let runtime = create_runtime();
        let (password, confirm, _submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        confirm.set("Abcdefg2".into());
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_enabled_when_medium_and_matching() {
        let runtime = create_runtime();
        let (password, confirm, _submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        password.set("abcdefgH".into());
        confirm.set("abcdefgH".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_enabled_when_strong_and_matching() {
        let runtime = create_runtime();
        let (password, confirm, _submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_strength_tracks_password() {
        let runtime = create_runtime();
        let (password, _confirm, _submitted, _disabled, strength, _mismatch) =
            restore_complete_signals();

        assert_eq!(strength.get_untracked(), PasswordStrength::None);

        password.set("abc".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Weak);

        password.set("abcdefgH".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Medium);

        password.set("Abcdefg1".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Strong);

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_mismatch_when_different() {
        let runtime = create_runtime();
        let (password, confirm, _submitted, _disabled, _strength, mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        confirm.set("Abcdefg2".into());
        assert!(mismatch.get_untracked().is_some());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_no_mismatch_when_matching() {
        let runtime = create_runtime();
        let (password, confirm, _submitted, _disabled, _strength, mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(mismatch.get_untracked().is_none());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_no_mismatch_when_confirm_empty() {
        let runtime = create_runtime();
        let (password, _confirm, _submitted, _disabled, _strength, mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        assert!(mismatch.get_untracked().is_none());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn complete_disabled_after_submit() {
        let runtime = create_runtime();
        let (password, confirm, submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(!disabled.get_untracked());

        submitted.set(true);
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn complete_reenabled_after_password_change() {
        let runtime = create_runtime();
        let (password, confirm, submitted, disabled, _strength, _mismatch) =
            restore_complete_signals();
        password.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        password.set("Xbcdefg1".into());
        tick().await;
        // now passwords mismatch, so still disabled for that reason
        confirm.set("Xbcdefg1".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }
}
