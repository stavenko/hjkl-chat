use leptos::*;
use leptos_router::*;

use crate::components::ThemeToggle;
use crate::pages::*;
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
                    <Route path="/register" view=RegistrationPage/>
                    <Route path="/register/success" view=RegistrationSuccessPage/>
                    <Route path="/password/restore" view=PlaceholderPage/>
                    <Route path="/" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    if !auth_service::is_authenticated() {
        let navigate = use_navigate();
        navigate("/login", NavigateOptions {
            replace: true,
            ..Default::default()
        });
        return ().into_view();
    }
    view! { <p>"Welcome to hjkl-chat"</p> }.into_view()
}

#[component]
fn PlaceholderPage() -> impl IntoView {
    view! { <p>"Coming soon"</p> }
}
