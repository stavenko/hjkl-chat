# Fix Task: Remove Hardcoded API URLs from Frontend Services

**User Story:** 01-login.md  
**Related Implementation Task:** 02-task-login-frontend.md  
**Related Review Task:** 02-review-login-frontend.md  
**Related Review Report:** /project/.ralph-wiggum/reports/02-review-login-frontend.md

## Objective

Remove hardcoded fallback API URLs from `frontend/src/services.rs` and make the application fail gracefully with a clear error message if the configuration file cannot be loaded.

## Specification References

1. [FRONTEND.md](../specs/FRONTEND.md) — Lines 58-78: API Configuration section states config.json is runtime configuration and must be loaded before app mount
2. [GENERIC-FRONTEND.md](../specs/GENERIC-FRONTEND.md) — Lines 367-394: API Base URL Configuration pattern
3. [RUST-COMMON-SPEC.md](../specs/RUST-COMMON-SPEC.md) — Error handling conventions

## Current Issues

In `frontend/src/services.rs`:
- Line 19: `get_api_base_url()` returns hardcoded `"http://localhost:8080"` if config not loaded
- Line 34: `fetch_config()` uses hardcoded `"http://localhost:8080"` as default on any error

These hardcoded values violate:
1. The "no hardcoded values" rule from the review checklist
2. The spec requirement that the application MUST fail with a clear error if configuration is missing

## Required Changes

### 1. Modify `get_api_base_url()` Function

**Current behavior:** Returns hardcoded fallback URL if config not loaded
**Required behavior:** Panic with clear error message if config not loaded

```rust
pub fn get_api_base_url() -> String {
    // Attempt to get from context first (preferred method per GENERIC-FRONTEND.md)
    if let Ok(api_base_url) = leptos::use_context::<ApiBaseUrl>() {
        return api_base_url.0.clone();
    }
    
    // Fallback to static variable for edge cases
    if let Ok(url) = API_BASE_URL.lock() {
        if let Some(ref url) = *url {
            return url.clone();
        }
    }
    
    // CRITICAL: Fail with clear error if configuration is missing
    panic!("API base URL not initialized. Ensure init_api_base_url() was called before app mount.");
}
```

### 2. Modify `fetch_config()` Function

**Current behavior:** Returns hardcoded default URL on any error
**Required behavior:** Return a Result type that propagates errors

```rust
async fn fetch_config() -> String {
    let window = match web_sys::window() {
        Some(w) => w,
        None => panic!("Failed to get window object: Running outside browser context"),
    };
    
    let promise = window.fetch_with_str("/config.json");
    
    let response_value = match JsFuture::from(promise).await {
        Ok(v) => v,
        Err(e) => panic!("Failed to fetch config.json: {:?}", e),
    };
    
    let response: web_sys::Response = match response_value.dyn_into() {
        Ok(r) => r,
        Err(e) => panic!("Failed to convert fetch response: {:?}", e),
    };
    
    if !response.ok() {
        panic!("Failed to load config.json: HTTP {} {}", response.status(), response.statusText().unwrap_or_default());
    }
    
    let text_promise = match response.text() {
        Ok(p) => p,
        Err(e) => panic!("Failed to read config.json body: {:?}", e),
    };
    
    let text = match JsFuture::from(text_promise).await {
        Ok(t) => t,
        Err(e) => panic!("Failed to await config.json text: {:?}", e),
    };
    
    let text_str = match text.as_string() {
        Some(s) => s,
        None => panic!("Failed to convert config.json to string"),
    };
    
    let config: serde_json::Value = match serde_json::from_str(&text_str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse config.json as JSON: {}", e),
    };
    
    match config["api_base_url"].as_str() {
        Some(url) => url.to_string(),
        None => panic!("config.json missing required field 'api_base_url'"),
    }
}
```

### 3. Update `init_api_base_url()` Function

The current implementation already handles the config loading correctly. Ensure it's called before `mount_to_body()` as specified in FRONTEND.md.

## Verification Steps

1. **Build verification:**
   ```bash
   cargo check -p frontend
   cargo clippy -p frontend -- -D warnings
   cargo build -p frontend
   ```

2. **Test verification:**
   ```bash
   cargo test -p frontend
   ```

3. **Runtime verification (if environment allows):**
   - Start frontend with valid config.json — should load successfully
   - Start frontend without config.json — should panic with clear error message

## Acceptance Criteria

- [ ] No hardcoded URLs remain in `frontend/src/services.rs`
- [ ] `get_api_base_url()` panics with clear error if config not initialized
- [ ] `fetch_config()` panics with specific error messages for each failure case
- [ ] `cargo clippy -p frontend -- -D warnings` passes
- [ ] `cargo test -p frontend` passes
- [ ] Error messages clearly indicate what configuration is missing and how to fix it

## Deliverables

1. Modified `frontend/src/services.rs` with no hardcoded URLs
2. Report at `/project/.ralph-wiggum/reports/02-task-fix-01-remove-hardcoded-api-urls.md`
3. Updated `/project/.ralph-wiggum/progress.md`