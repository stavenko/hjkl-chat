# Generic Frontend Specification

## Component Pattern

### Structure
All Leptos components follow this structure:
```rust
use leptos::*;

#[component]
pub fn ComponentName(
    // Props with explicit types
    prop_name: RwSignal<String>,
) -> impl IntoView {
    // Component logic
    
    view! {
        <div>
            {/* Component content */}
        </div>
    }
}
```

### Props Pattern
- Use `RwSignal<T>` for mutable props that child components can modify
- Use `ReadSignal<T>` for read-only props
- Use `Option<T>` for optional props with defaults where appropriate
- All props must have explicit types (no `impl IntoView` for props)

### File Organization
- Components live in `src/components/`
- File naming: lowercase with hyphens (e.g., `user-avatar.rs`)
- Page-specific components in `src/pages/<page-name>/components/`
- Reusable components in `src/components/` at the root level

### Reusable vs Page-Specific
- **Reusable components:** Generic UI elements (buttons, inputs, modals, cards)
- **Page-specific components:** Components only used within one page context
- Reusable components should have no business logic, only presentation

## Page Pattern

### Structure
```rust
use leptos::*;

#[component]
pub fn PageName() -> impl IntoView {
    // Page-specific signals and resources
    
    view! {
        <div class="page-name">
            <h1>"Page Title"</h1>
            {/* Page content */}
        </div>
    }
}
```

### Layout Composition
- Pages are composed of reusable components
- Layout components (header, footer, sidebar) are separate components
- Page should not import layout components directly; use a Layout wrapper component

### Routing Integration
- Each page maps to a route in `src/app.rs`
- Pages should be self-contained; dependencies injected via Context API
- Page-specific state should be scoped to the page component

### Page-Specific State
- Use `RwSignal` and `ReadSignal` for local page state
- Use `use_context` and `provide_context` for shared state
- Use `create_resource` for async data that persists across renders

## Service Pattern

### Structure
Services live in `src/services/` and handle API communication:
```rust
use leptos::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::RequestInit;

pub async fn api_endpoint_name(input: EndpointInput) -> Result<EndpointOutput, ApiError> {
    let api_base_url = get_api_base_url();
    let url = format!("{}/api/endpoint", api_base_url);
    
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.body(Some(&serde_json::to_string(&input)?));
    
    let response = window()
        .fetch_with_str_and_init(&url, &opts)
        .await?
        .unwrap();
    
    let text = wasm_bindgen_futures::JsFuture::from(response.text()?)
        .await?
        .as_string()
        .unwrap_or_default();
    
    let result: EndpointOutput = serde_json::from_str(&text)?;
    Ok(result)
}
```

### Error Handling
- Services return `Result<T, ApiError>`
- `ApiError` wraps network errors, parse errors, and server errors
- Components should handle errors gracefully with user-friendly messages

### Request/Response Typing
- Define strong types for all request input and response output
- Use `serde` for serialization/deserialization
- Types should match backend API contract exactly

### File Organization
- One file per service domain (e.g., `auth.rs`, `users.rs`, `messages.rs`)
- All files in `src/services/`
- Moduled in `src/services/mod.rs`

## Form Pattern

### Form State Management
```rust
#[component]
pub fn FormName() -> impl IntoView {
    let email = create_rw_signal(String::new());
    let password = create_rw_signal(String::new());
    let error = create_rw_signal<Option<String>>(None);
    let is_loading = create_rw_signal(false);
    
    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let error = error.clone();
        let is_loading = is_loading.clone();
        
        move |_| {
            // Validation
            if email.get().is_empty() {
                error.set(Some("Email is required".to_string()));
                return;
            }
            
            is_loading.set(true);
            error.set(None);
            
            // Submit
            let email_val = email.get();
            let password_val = password.get();
            
            leptos::spawn_local(async move {
                match services::api_submit(email_val, password_val).await {
                    Ok(_) => {
                        // Success handling
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                    }
                }
                is_loading.set(false);
            });
        }
    };
    
    view! {
        <form on:submit=on_submit>
            <input type="email" bind:value=email />
            <input type="password" bind:value=password />
            <Show when=error.get().is_some() fallback=|| view! {}>
                <div class="error">{error.get()}</div>
            </Show>
            <button type="submit" disabled=is_loading.get()>
                "Submit"
            </button>
        </form>
    }
}
```

### Validation Patterns
- Validate on input blur and form submit
- Show inline errors for each field
- Show form-level errors for submission errors
- Disable submit button during loading state

### Submission Handling
- Set loading state before async call
- Clear previous errors before submission
- Handle success and error cases explicitly
- Reset form or redirect on success

## Routing Pattern

### Setup
```rust
use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage />
                    <Route path="login" view=LoginPage />
                    <Route path="register" view=RegisterPage />
                    <Route path="dashboard" view=DashboardPage />
                </Routes>
            </main>
        </Router>
    }
}
```

### Route Configuration
- Define routes in a single `Routes` component
- Use `path=""` for the root route
- Named routes use `path="<name>"`

### Nested Routes
```rust
<Routes>
    <Route path="admin" view=AdminLayout>
        <Route path="" view=AdminDashboard />
        <Route path="users" view=AdminUsers />
        <Route path="settings" view=AdminSettings />
    </Route>
</Routes>
```

### Route Guards
```rust
#[component]
pub fn AuthGuard(child: Children) -> impl IntoView {
    let auth_state = expect_context::<AuthState>();
    let is_authenticated = move || auth_state.is_logged_in();
    
    view! {
        <Suspense fallback=|| view! { <div>"Loading..."</div> } >
            <Show when=is_authenticated fallback=|| {
                navigate("/login");
                view! {}
            }>
                {child()}
            </Show>
        </Suspense>
    }
}
```

## State Management Pattern

### Signal Usage Guidelines
- Use `create_rw_signal` for local component state
- Use `create_signal` to create separate read/write signals when needed
- Pass `ReadSignal` to child components that only need to read
- Pass `RwSignal` to child components that need to modify
- Use `create_memo` for derived state that should be cached

### Resource Usage for Async Data
```rust
#[component]
pub fn ComponentWithResource() -> impl IntoView {
    let data = create_resource(
        || (),
        async move { services::fetch_data().await },
    );
    
    view! {
        <Suspense fallback=|| view! { <div>"Loading..."</div> } >
            <Show when=move || data.get().is_some() fallback=|| view! {}>
                <div>{move || data.get().unwrap()}</div>
            </Show>
        </Suspense>
    }
}
```

### Context API for Shared State
```rust
// Define the state type
pub struct SharedState {
    pub value: RwSignal<String>,
}

// Provide context in parent
#[component]
pub fn Parent() -> impl IntoView {
    let state = SharedState {
        value: create_rw_signal("initial".to_string()),
    };
    provide_context(state);
    
    view! { <Child /> }
}

// Consume context in child
#[component]
pub fn Child() -> impl IntoView {
    let state = expect_context::<SharedState>();
    
    view! { <div>{move || state.value.get()}</div> }
}
```

### localStorage Integration
```rust
use wasm_bindgen::JsCast;
use web_sys::Storage;

fn get_from_storage(key: &str) -> Option<String> {
    window()
        .local_storage()
        .ok()
        .and_then(|storage| storage.get(key).ok())
}

fn set_in_storage(key: &str, value: &str) -> Result<(), wasm_bindgen::JsValue> {
    window()
        .local_storage()?
        .set(key, value)
}

fn remove_from_storage(key: &str) -> Result<(), wasm_bindgen::JsValue> {
    window()
        .local_storage()?
        .remove(key)
}

// Auth token storage example
pub struct AuthState {
    access_token: RwSignal<Option<String>>,
    refresh_token: RwSignal<Option<String>>,
}

impl AuthState {
    pub fn new() -> Self {
        let access_token = create_rw_signal(get_from_storage("access_token"));
        let refresh_token = create_rw_signal(get_from_storage("refresh_token"));
        
        Self { access_token, refresh_token }
    }
    
    pub fn save_tokens(&self, access: &str, refresh: &str) {
        set_in_storage("access_token", access).ok();
        set_in_storage("refresh_token", refresh).ok();
        self.access_token.set(Some(access.to_string()));
        self.refresh_token.set(Some(refresh.to_string()));
    }
    
    pub fn clear_tokens(&self) {
        remove_from_storage("access_token").ok();
        remove_from_storage("refresh_token").ok();
        self.access_token.set(None);
        self.refresh_token.set(None);
    }
    
    pub fn is_logged_in(&self) -> bool {
        self.access_token.get().is_some()
    }
}
```

## API Base URL Configuration

### Initialization
```rust
// src/services/mod.rs
use leptos::*;

struct ApiBaseUrl(String);

pub fn get_api_base_url() -> String {
    expect_context::<ApiBaseUrl>().0.clone()
}

pub async fn init_api_base_url() {
    let response = window().fetch_with_str("/config.json").await.unwrap().unwrap();
    let text = JsFuture::from(response.text().unwrap()).await.unwrap();
    let config: serde_json::Value = serde_json::from_str(text.as_string().unwrap().as_str()).unwrap();
    let api_base_url = config["api_base_url"].as_str().unwrap().to_string();
    
    provide_context::<ApiBaseUrl>(ApiBaseUrl(api_base_url));
}
```

### Usage
- Call `init_api_base_url()` before mounting the app
- Access via `get_api_base_url()` in service functions
- Prepend to all API endpoint paths
