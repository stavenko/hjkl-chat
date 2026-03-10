use leptos::*;
use leptos_router::*;

use crate::components::ThemeToggle;
use crate::pages::chat::ChatPage;
use crate::pages::login::LoginPage;
use crate::pages::registration_input_email::RegistrationInputEmailPage;
use crate::pages::registration_set_password::RegistrationSetPasswordPage;
use crate::pages::registration_success::RegistrationSuccessPage;
use crate::pages::registration_verify_code::RegistrationVerifyCodePage;
use crate::pages::restore_input_email::RestoreInputEmailPage;
use crate::pages::restore_set_password::RestoreSetPasswordPage;
use crate::pages::restore_success::RestoreSuccessPage;
use crate::pages::restore_verify_code::RestoreVerifyCodePage;
use crate::services::{auth_service, get_features};

#[component]
pub fn App() -> impl IntoView {
    let show_theme_toggle = get_features().is_enabled("debug-light-dark-switch");

    view! {
        <Router>
            <main>
                {show_theme_toggle.then(|| view! { <ThemeToggle/> })}
                <Routes>
                    <Route path="/login" view=LoginPage/>
                    <Route path="/register" view=RegistrationInputEmailPage/>
                    <Route path="/register/verify" view=RegistrationVerifyCodePage/>
                    <Route path="/register/password" view=RegistrationSetPasswordPage/>
                    <Route path="/register/success" view=RegistrationSuccessPage/>
                    <Route path="/password/restore" view=RestoreInputEmailPage/>
                    <Route path="/password/restore/verify" view=RestoreVerifyCodePage/>
                    <Route path="/password/restore/password" view=RestoreSetPasswordPage/>
                    <Route path="/password/restore/success" view=RestoreSuccessPage/>
                    <Route path="/chat" view=ChatPage/>
                    <Route path="/chat/:id" view=ChatPage/>
                    <Route path="/" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let navigate = use_navigate();
    if auth_service::is_authenticated() {
        navigate("/chat", NavigateOptions {
            replace: true,
            ..Default::default()
        });
    } else {
        navigate("/login", NavigateOptions {
            replace: true,
            ..Default::default()
        });
    }
    ().into_view()
}

#[component]
fn PlaceholderPage() -> impl IntoView {
    view! { <p>"Coming soon"</p> }
}
