use leptos::*;
use leptos_router::*;

use crate::components::login_form::LoginForm;

#[component]
pub fn LoginPage() -> impl IntoView {
    let navigate = use_navigate();

    let on_success = Callback::new(move |_| {
        navigate("/", Default::default());
    });

    view! {
        <div class="login-page">
            <div class="logo">
                <h1>"hjkl-chat"</h1>
            </div>
            <p class="tagline">"Authenticate to see your projects"</p>
            <div class="form-card">
                <LoginForm on_success=on_success />
            </div>
            <div class="navigation-links">
                <a href="/password/restore" class="link">"Forgot password?"</a>
                <div class="separator">|</div>
                <a href="/register" class="link">"Don't have an account? Register"</a>
            </div>
        </div>
    }
}
