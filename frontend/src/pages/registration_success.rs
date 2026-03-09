use leptos::*;
use leptos_router::*;

use crate::components::*;

pub(crate) fn build_login_href(email: &str) -> String {
    if email.is_empty() {
        "/login".to_string()
    } else {
        format!("/login?email={}", email)
    }
}

#[component]
pub fn RegistrationSuccessPage() -> impl IntoView {
    let query = use_query_map();
    let email = move || {
        query.with(|q| q.get("email").cloned().unwrap_or_default())
    };
    let login_href = move || build_login_href(&email());

    view! {
        <div class="auth-page">
            <Logo/>
            <Surface>
                <FormHeader text="Account Created"/>
                <p class="form-description">
                    "You have created account \""
                    <strong>{email}</strong>
                    "\" successfully, you can now "
                    <A href=login_href class="auth-link auth-link--inline">"sign in"</A>
                    " with your email and password."
                </p>
            </Surface>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn login_href_includes_email() {
        assert_eq!(
            build_login_href("user@example.com"),
            "/login?email=user@example.com"
        );
    }

    #[wasm_bindgen_test]
    fn login_href_plain_when_empty() {
        assert_eq!(build_login_href(""), "/login");
    }
}
