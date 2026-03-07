# Task 0-Fix-05: Add API Base URL Initialization to Frontend

## Summary

Implement the API base URL initialization in the frontend as required by `specs/FRONTEND.md`. The frontend must fetch `/config.json` before mounting the app.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Issue Details

**File:** `frontend/src/main.rs`
**Problem:** Frontend does not call `init_api_base_url()` before mounting the app.

**Reference from FRONTEND.md (lines 117-124):**
```rust
#[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    spawn_local(async {
        crate::services::init_api_base_url().await;
        mount_to_body(App);
    });
}
```

**Requirements from FRONTEND.md (lines 61-68):**
- On app init, the frontend fetches `/config.json` before mounting the app
- `/config.json` contains: `{"api_base_url": "<string>"}`
- All service functions prepend the loaded `api_base_url` to endpoint paths
- `init_api_base_url()` must complete before `mount_to_body()`

## Required Changes

### 1. Create frontend/src/services.rs

```rust
use once_cell::sync::Lazy;
use std::sync::RwLock;

static API_BASE_URL: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));

pub async fn init_api_base_url() {
    let resp = reqwest::get("/config.json")
        .await
        .expect("Failed to fetch config.json");
    let config: serde_json::Value = resp.json().await.expect("Failed to parse config.json");
    let api_base_url = config["api_base_url"]
        .as_str()
        .expect("config.json must contain api_base_url")
        .to_string();
    *API_BASE_URL.write().expect("Failed to write API_BASE_URL") = api_base_url;
}

pub fn get_api_base_url() -> String {
    API_BASE_URL.read().expect("Failed to read API_BASE_URL").clone()
}
```

### 2. Update frontend/src/main.rs

```rust
use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div>"Hello from Leptos!"</div>
    }
}

mod services;

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    leptos::spawn_local(async {
        services::init_api_base_url().await;
        mount_to_body(App);
    });
}
```

### 3. Update frontend/Cargo.toml

Add required dependencies:
- `reqwest` with `webassembly` and `json` features
- `serde_json`
- `once_cell`

## Acceptance Criteria

- [ ] `frontend/src/services.rs` created with `init_api_base_url()` and `get_api_base_url()`
- [ ] `frontend/src/main.rs` updated to call `services::init_api_base_url().await` before mounting
- [ ] Dependencies added to `frontend/Cargo.toml`
- [ ] `cargo build -p frontend` succeeds
- [ ] `cargo clippy -p frontend -- -D warnings` passes
- [ ] API base URL is loaded before app mounts

## Verification Commands

```bash
cargo build -p frontend
cargo clippy -p frontend -- -D warnings
```

## Related Files

- @.ralph-wiggum/reports/00-review-bootstrap.md
- @specs/FRONTEND.md