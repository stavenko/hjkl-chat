use leptos::*;

use crate::auth_state::AuthState;
use crate::components::authentication_button::AuthenticationButton;
use crate::components::authentication_input::AuthenticationInput;

#[component]
pub fn LoginForm(on_success: Callback<()>) -> impl IntoView {
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error: RwSignal<Option<String>> = create_rw_signal(None);

    let can_submit = create_memo(move |_| !email.get().is_empty() && !password.get().is_empty());
    let disabled = create_rw_signal(!can_submit.get());

    create_effect(move |_| {
        disabled.set(!can_submit.get());
    });

    let on_submit = {
        move |_| {
            let email_val = email.get();
            let password_val = password.get();

            leptos::spawn_local(async move {
                match crate::auth_service::login(&email_val, &password_val).await {
                    Ok(response) => {
                        if response.status == "ok" {
                            if let Some(auth_state) = use_context::<AuthState>() {
                                auth_state.save_tokens(
                                    &response.access_token,
                                    &response.refresh_token,
                                    &response.user.id,
                                    &response.user.email,
                                );
                            }
                            on_success.call(());
                        }
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }
            });
        }
    };

    view! {
        <form class="login-form" on:submit=on_submit>
            <div on:input=move |_| error.set(None)>
                <AuthenticationInput
                    label="Email".to_string()
                    value=email
                    error=error
                    input_type="email".to_string()
                />
            </div>
            <div on:input=move |_| error.set(None)>
                <AuthenticationInput
                    label="Password".to_string()
                    value=password
                    error=error
                    input_type="password".to_string()
                />
            </div>
            <AuthenticationButton
                disabled=disabled
                label="Sign In".to_string()
                on_click=move |_| {}
            />
        </form>
    }
}