use leptos::*;
use leptos_router::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::auth_state::AuthState;
use crate::pages::login_page::LoginPage;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="home-page">
            <h1>"Home"</h1>
            <p>"This is a protected page"</p>
        </div>
    }
}

#[component]
pub fn AuthGuard(children: ChildrenFn) -> impl IntoView {
    let auth_state = expect_context::<AuthState>();
    let navigate = use_navigate();
    let auth_state_for_effect = auth_state.clone();

    create_effect(move |_| {
        if !auth_state_for_effect.is_authenticated() {
            navigate("/login", Default::default());
        }
    });

    let is_authenticated = create_memo(move |_| auth_state.is_authenticated());

    let children_rc: RwSignal<Option<Rc<RefCell<Fragment>>>> =
        RwSignal::new(Some(Rc::new(RefCell::new(children()))));

    view! {
        <Suspense fallback=|| view! { <div>"Loading..."</div> } >
            <Show when=move || is_authenticated.get() fallback=|| view! { <div></div> } >
                {move || {
                    if let Some(ref rc) = children_rc.get() {
                        rc.borrow().clone()
                    } else {
                        Fragment::new(vec![])
                    }
                }}
            </Show>
        </Suspense>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let auth_state = AuthState::new();
    provide_context(auth_state);

    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="login" view=LoginPage />
                    <Route path="register" view=|| view! { <div>"Register Page"</div> } />
                    <Route path="password/restore" view=|| view! { <div>"Password Restore Page"</div> } />
                    <Route path="" view=|| {
                        view! {
                            <AuthGuard>
                                <HomePage />
                            </AuthGuard>
                        }
                    } />
                </Routes>
            </main>
        </Router>
    }
}
