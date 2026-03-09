use leptos::*;
use leptos_router::*;

use crate::components::*;
use crate::services::auth_service;

pub(crate) fn change_password_signals() -> (
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<String>,
    RwSignal<bool>,
    Signal<bool>,
    Signal<PasswordStrength>,
    Signal<Option<String>>,
) {
    let old_password = create_rw_signal(String::new());
    let new_password = create_rw_signal(String::new());
    let confirm = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        old_password.get();
        new_password.get();
        confirm.get();
        submitted.set(false);
    });

    let strength = Signal::derive(move || compute_password_strength(&new_password.get()));

    let mismatch_error = Signal::derive(move || {
        let c = confirm.get();
        if c.is_empty() || new_password.get() == c {
            None
        } else {
            Some("Passwords do not match".to_string())
        }
    });

    let disabled = Signal::derive(move || {
        let o = old_password.get();
        let n = new_password.get();
        let c = confirm.get();
        if o.is_empty() || n.is_empty() || c.is_empty() || submitted.get() {
            return true;
        }
        n != c
    });

    (old_password, new_password, confirm, submitted, disabled, strength, mismatch_error)
}

#[component]
pub fn ChangePasswordPage() -> impl IntoView {
    if !auth_service::is_authenticated() {
        let navigate = use_navigate();
        navigate("/login", NavigateOptions {
            replace: true,
            ..Default::default()
        });
        return ().into_view();
    }

    let (old_password, new_password, confirm, submitted, disabled, strength, mismatch_error) =
        change_password_signals();

    let error = create_rw_signal(None::<String>);
    let navigate = use_navigate();

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());
    let no_error: Signal<Option<String>> = Signal::derive(|| None);

    let on_submit = move || {
        submitted.set(true);
        error.set(None);
        let old = old_password.get_untracked();
        let new = new_password.get_untracked();
        let conf = confirm.get_untracked();
        let navigate = navigate.clone();
        spawn_local(async move {
            match auth_service::change_password(&old, &new, &conf).await {
                Ok(_) => {
                    navigate("/me", NavigateOptions::default());
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
            <p class="auth-page__tagline">"Change your password"</p>
            <Surface>
                <FormHeader text="Change Password"/>
                <PasswordLoginInput
                    label="Current Password"
                    placeholder=""
                    value=old_password
                    error=error_signal
                />
                <PasswordWithStrengthInput
                    label="New Password"
                    placeholder=""
                    value=new_password
                    error=no_error
                    strength=strength
                />
                <PasswordLoginInput
                    label="Confirm New Password"
                    placeholder=""
                    value=confirm
                    error=mismatch_error
                />
                <Button
                    label="Change Password"
                    disabled=disabled
                    on_click=on_submit
                />
                <AuthLink text="Back to profile" href="/me/"/>
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
    fn disabled_when_all_empty() {
        let runtime = create_runtime();
        let (_old, _new, _confirm, _submitted, disabled, _strength, _mismatch) =
            change_password_signals();
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_when_old_password_empty() {
        let runtime = create_runtime();
        let (_old, new_pw, confirm, _submitted, disabled, _strength, _mismatch) =
            change_password_signals();
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_when_new_passwords_mismatch() {
        let runtime = create_runtime();
        let (old, new_pw, confirm, _submitted, disabled, _strength, _mismatch) =
            change_password_signals();
        old.set("oldpass".into());
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg2".into());
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn enabled_when_all_filled_and_matching() {
        let runtime = create_runtime();
        let (old, new_pw, confirm, _submitted, disabled, _strength, _mismatch) =
            change_password_signals();
        old.set("oldpass".into());
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn disabled_after_submit() {
        let runtime = create_runtime();
        let (old, new_pw, confirm, submitted, disabled, _strength, _mismatch) =
            change_password_signals();
        old.set("oldpass".into());
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(!disabled.get_untracked());

        submitted.set(true);
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn reenabled_after_field_change() {
        let runtime = create_runtime();
        let (old, new_pw, confirm, submitted, disabled, _strength, _mismatch) =
            change_password_signals();
        old.set("oldpass".into());
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        old.set("newoldpass".into());
        tick().await;
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn strength_tracks_new_password() {
        let runtime = create_runtime();
        let (_old, new_pw, _confirm, _submitted, _disabled, strength, _mismatch) =
            change_password_signals();

        assert_eq!(strength.get_untracked(), PasswordStrength::None);

        new_pw.set("abc".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Weak);

        new_pw.set("abcdefgH".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Medium);

        new_pw.set("Abcdefg1".into());
        assert_eq!(strength.get_untracked(), PasswordStrength::Strong);

        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn mismatch_shown_when_different() {
        let runtime = create_runtime();
        let (_old, new_pw, confirm, _submitted, _disabled, _strength, mismatch) =
            change_password_signals();
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg2".into());
        assert!(mismatch.get_untracked().is_some());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn no_mismatch_when_matching() {
        let runtime = create_runtime();
        let (_old, new_pw, confirm, _submitted, _disabled, _strength, mismatch) =
            change_password_signals();
        new_pw.set("Abcdefg1".into());
        confirm.set("Abcdefg1".into());
        assert!(mismatch.get_untracked().is_none());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn no_mismatch_when_confirm_empty() {
        let runtime = create_runtime();
        let (_old, new_pw, _confirm, _submitted, _disabled, _strength, mismatch) =
            change_password_signals();
        new_pw.set("Abcdefg1".into());
        assert!(mismatch.get_untracked().is_none());
        runtime.dispose();
    }
}
