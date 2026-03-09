use leptos::*;
use leptos_router::*;

use crate::components::*;
use crate::services::auth_service;

use super::helpers::start_resend_countdown;

pub(crate) fn registration_verify_signals() -> (RwSignal<String>, RwSignal<bool>, Signal<bool>) {
    let code = create_rw_signal(String::new());
    let submitted = create_rw_signal(false);

    create_effect(move |_| {
        code.get();
        submitted.set(false);
    });

    let disabled = Signal::derive(move || code.get().is_empty() || submitted.get());

    (code, submitted, disabled)
}

#[component]
pub fn RegistrationVerifyCodePage() -> impl IntoView {
    let query = use_query_map();
    let session_id = create_rw_signal(
        query.with_untracked(|q| q.get("session_id").cloned().unwrap_or_default()),
    );
    let email = store_value(
        query.with_untracked(|q| q.get("email").cloned().unwrap_or_default()),
    );
    let resend_at: f64 = query
        .with_untracked(|q| q.get("resend_at").cloned().unwrap_or_default())
        .parse()
        .unwrap_or(0.0);

    let countdown_remaining = create_rw_signal(0i32);
    let error = create_rw_signal(None::<String>);
    let navigate = store_value(use_navigate());

    let (code, submitted, disabled) = registration_verify_signals();

    start_resend_countdown(resend_at, countdown_remaining);

    let error_signal: Signal<Option<String>> = Signal::derive(move || error.get());

    let on_resend = move || {
        error.set(None);
        let email_val = email.get_value();
        spawn_local(async move {
            match auth_service::registration_init(&email_val).await {
                Ok(response) => {
                    session_id.set(response.session_id);
                    let epoch_secs = js_sys::Date::parse(&response.resend_available_at) / 1000.0;
                    start_resend_countdown(epoch_secs, countdown_remaining);
                }
                Err(msg) => error.set(Some(msg)),
            }
        });
    };

    let on_submit = move || {
        submitted.set(true);
        error.set(None);
        let sid = session_id.get_untracked();
        let code_val = code.get_untracked();
        let email_val = email.get_value();
        spawn_local(async move {
            match auth_service::registration_verify(&sid, &code_val).await {
                Ok(response) => {
                    let url = format!(
                        "/register/password?session_id={}&email={}",
                        response.session_id, email_val
                    );
                    navigate.with_value(|nav| {
                        nav(&url, NavigateOptions::default());
                    });
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
                <FormHeader text="Enter Code"/>
                <FormDescription text="We sent a verification code to your email"/>
                <TextInput
                    label="Code"
                    placeholder=""
                    value=code
                    error=error_signal
                />
                <Button
                    label="Verify"
                    disabled=disabled
                    on_click=on_submit
                />
                {move || {
                    let r = countdown_remaining.get();
                    if r > 0 {
                        view! {
                            <p class="form-description">
                                {format!("Resend code in {}s", r)}
                            </p>
                        }.into_view()
                    } else {
                        view! {
                            <p class="form-description">
                                "Didn't receive a code? "
                                <a
                                    class="auth-link"
                                    href="#"
                                    on:click=move |ev| {
                                        ev.prevent_default();
                                        on_resend();
                                    }
                                >
                                    "Resend"
                                </a>
                            </p>
                        }.into_view()
                    }
                }}
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
    fn verify_disabled_when_empty() {
        let runtime = create_runtime();
        let (_code, _submitted, disabled) = registration_verify_signals();
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn verify_enabled_when_code_filled() {
        let runtime = create_runtime();
        let (code, _submitted, disabled) = registration_verify_signals();
        code.set("123456".into());
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    fn verify_disabled_after_submit() {
        let runtime = create_runtime();
        let (code, submitted, disabled) = registration_verify_signals();
        code.set("123456".into());
        submitted.set(true);
        assert!(disabled.get_untracked());
        runtime.dispose();
    }

    #[wasm_bindgen_test]
    async fn verify_reenabled_after_code_change() {
        let runtime = create_runtime();
        let (code, submitted, disabled) = registration_verify_signals();
        code.set("123456".into());
        submitted.set(true);
        assert!(disabled.get_untracked());

        code.set("654321".into());
        tick().await;
        assert!(!disabled.get_untracked());
        runtime.dispose();
    }
}
