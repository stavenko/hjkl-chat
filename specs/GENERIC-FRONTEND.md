# Generic Frontend Specification

## Module Structure
- **main.rs** — mount the app, initialize the Leptos router.
- **app.rs** — root `App` component, top-level `<Router>` with `<Routes>`.
- **pages/** — one file per page or form (e.g., `registration_form.rs`, `login_form.rs`).
- **components/** — reusable UI components (e.g., `text_input.rs`, `button.rs`, `password_strength.rs`).
- **services/** — API call functions, one module per backend domain (e.g., `auth_service.rs`).
- **state/** — global reactive state (e.g., `auth_state.rs` — tokens, current user).

## What Is a Component

A component is a reusable UI building block. It receives data through props, manages local state with signals, and returns a view. Components contain no business logic — they only render UI and emit events.

**Rules:**
- A component is defined with `#[component]` and returns `impl IntoView`.
- Props are defined with `#[prop]` attributes on function parameters.
- A component contains **no API calls**. API interactions belong in **pages** or **services**.
- A component may use local signals for UI state (hover, focus, toggle).
- A component exposes callbacks via prop functions (e.g., `on_change`, `on_submit`) for parent coordination.

### Component Example

```text
// TextInput component with error display (pseudocode)
#[component]
fn TextInput(
    #[prop] label: String,
    #[prop] placeholder: String,
    #[prop] value: ReadSignal<String>,
    #[prop] on_change: Callback<String>,
    #[prop(optional)] error: Option<ReadSignal<String>>,
    #[prop(optional)] input_type: Option<String>,
) -> impl IntoView {
    view! {
        <div class="input-group">
            <label>{label}</label>
            <input
                type={input_type.unwrap_or("text")}
                placeholder={placeholder}
                prop:value={value}
                on:input=move |ev| on_change(event_target_value(&ev))
            />
            {move || error.and_then(|e| {
                let msg = e.get();
                if msg.is_empty() { None }
                else { Some(view! { <span class="error">{msg}</span> }) }
            })}
        </div>
    }
}
```

A component that composes other components:

```text
// PasswordStrength indicator (pseudocode)
#[component]
fn PasswordStrength(
    #[prop] password: ReadSignal<String>,
) -> impl IntoView {
    let strength = move || compute_strength(password.get());
    view! {
        <div class="password-strength">
            <div class={move || format!("bar {}", strength())} />
            <span>{move || strength()}</span>
        </div>
    }
}
```

`PasswordStrength` owns only its derived computation. The password value comes from the parent; what to do with the strength level is the parent's concern.

## What Is a Page

A page is a component that represents a full route. It receives route parameters, calls services to fetch or submit data, and composes child components into a complete view.

**Rules:**
- A page is defined with `#[component]` like any component.
- A page may call service functions to interact with the backend.
- A page manages form state (signals for fields, validation, errors).
- A page is registered as a `<Route>` in `app.rs`.
- Navigation between pages uses `leptos_router` primitives (`<A>`, `use_navigate`).

### Page Example

```text
// LoginForm page with validation + API call (pseudocode)
#[component]
fn LoginPage() -> impl IntoView {
    // 1. Create signals for form fields
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal(String::new());

    // 2. Derived validation signal
    let is_valid = move || !email.get().is_empty() && !password.get().is_empty();

    // 3. Submit handler calls service
    let on_submit = move |_| {
        let email_val = email.get();
        let password_val = password.get();
        spawn_local(async move {
            match auth_service::login(email_val, password_val).await {
                Ok(response) => {
                    // Store tokens, navigate to home
                    auth_state::set_tokens(response.access_token, response.refresh_token);
                    use_navigate()("/", NavigateOptions::default());
                }
                Err(e) => error.set(e.message),
            }
        });
    };

    // 4. Render using child components
    view! {
        <form on:submit=on_submit>
            <TextInput label="Email" value=email.read_only() on_change=move |v| email.set(v) />
            <TextInput label="Password" value=password.read_only() on_change=move |v| password.set(v)
                       error=Some(error.read_only()) input_type=Some("password") />
            <Button label="Sign In" disabled=move || !is_valid() on_click=on_submit />
        </form>
    }
}
```

## What Is a Service

A service is a module of async functions that call backend API endpoints. Services parse JSON responses and return typed results. They contain no UI logic.

**Rules:**
- A service is a plain module with async functions, not a component.
- Each function makes one HTTP request using `gloo-net` (or equivalent WASM HTTP client).
- Functions return `Result<T, ApiError>` where `ApiError` contains the server's `message` field.
- A service reads the API base URL from configuration.
- A service attaches auth tokens from global state when required.

### Service Example

```text
// auth_service module (pseudocode)

struct LoginResponse {
    status: String,
    user: User,
    access_token: String,
    refresh_token: String,
}

struct ApiError {
    status: String,
    message: String,
}

async fn login(email: String, password: String) -> Result<LoginResponse, ApiError> {
    let body = json!({ "email": email, "password": password });
    let response = Request::post(&format!("{}/api/auth/login", api_base_url()))
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) if resp.ok() => {
            let data = resp.json::<LoginResponse>().await;
            Ok(data)
        }
        Ok(resp) => {
            let err = resp.json::<ApiError>().await;
            Err(err)
        }
        Err(e) => Err(ApiError { status: "error".into(), message: e.to_string() }),
    }
}
```

## Form Pattern

Forms follow a consistent reactive pattern:

1. **One signal per field** — `create_rw_signal(String::new())` for each input.
2. **Derived validation signal** — a closure that reads all field signals and returns `bool`.
3. **Submit handler** — calls the appropriate service function, handles success/error.
4. **Error signal** — populated from the server response `message` field on failure.
5. **Button state** — `disabled` prop bound to the negation of the validation signal.

## Routing

All routes are declared in `app.rs` using `leptos_router`:

```text
#[component]
fn App() -> impl IntoView {
    provide_context(AuthState::new());

    view! {
        <Router>
            <Routes>
                <Route path="/register" view=RegistrationPage />
                <Route path="/login" view=LoginPage />
                <Route path="/password/restore" view=PasswordRestorePage />
                <Route path="/password/change" view=PasswordChangePage />
                <Route path="/files" view=FilesBrowserPage />
            </Routes>
        </Router>
    }
}
```

Each `<Route>` binds a URL path to a page component.

## State Management

- **Global state** — use `provide_context` / `use_context` for data shared across pages (auth tokens, current user). Defined in `state/` modules.
- **Local state** — use `create_signal` or `create_rw_signal` for data scoped to a single component or page (form fields, UI toggles).
- **Auth state** — stored in a context struct with signals for `access_token`, `refresh_token`, and `current_user`. Tokens are persisted to `localStorage` and restored on app init.

```text
// auth_state module (pseudocode)
struct AuthState {
    access_token: RwSignal<Option<String>>,
    refresh_token: RwSignal<Option<String>>,
    current_user: RwSignal<Option<User>>,
}

impl AuthState {
    fn new() -> Self {
        // Restore from localStorage if available
    }

    fn set_tokens(&self, access: String, refresh: String) {
        // Update signals and localStorage
    }

    fn clear(&self) {
        // Clear signals and localStorage
    }

    fn is_authenticated(&self) -> bool {
        self.access_token.get().is_some()
    }
}
```

## Error Display

- Server errors are shown **inline on form fields** using the component's error prop.
- The error message comes directly from the server response `message` field.
- No toasts, modals, or global error banners.
- Clear the error signal when the user modifies the field.

## Validation

- Client-side validation uses **derived signals** that read field values and return `bool`.
- The submit button's `disabled` prop is bound to the validation result.
- Validation runs reactively — as the user types, the button state updates immediately.
- Server-side validation errors are displayed after form submission via the error signal.

## Notes
- The frontend is a standalone WASM application; it does not share a process with the backend.
- Each page is a thin orchestrator: it manages form state, calls services, and renders components. No HTTP or business logic resides in components.
- Services encapsulate all HTTP interaction and depend on no UI types.
