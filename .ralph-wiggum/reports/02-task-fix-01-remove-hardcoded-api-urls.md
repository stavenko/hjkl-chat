# Task Report: Remove Hardcoded API URLs from Frontend Services

**Task File:** 02-task-fix-01-remove-hardcoded-api-urls.md  
**Date:** 2026-03-08

## Objective

Remove hardcoded fallback API URLs from `frontend/src/services.rs` and make the application fail gracefully with clear error messages if configuration cannot be loaded.

## Changes Made

### Modified File: `frontend/src/services.rs`

#### 1. Updated `get_api_base_url()` Function

**Before:** Returned hardcoded `"http://localhost:8080"` if config not loaded  
**After:** Panics with clear error message: `"API base URL not initialized. Ensure init_api_base_url() was called before app mount."`

#### 2. Updated `fetch_config()` Function

**Before:** Returned hardcoded `"http://localhost:8080"` on any error  
**After:** Panics with specific error messages for each failure case:
- `"Failed to get window object: Running outside browser context"` — when web_sys::window() returns None
- `"Failed to fetch config.json: {error}"` — when fetch promise fails
- `"Failed to convert fetch response: {error}"` — when response conversion fails
- `"Failed to load config.json: HTTP {status}"` — when HTTP response is not OK
- `"Failed to read config.json body: {error}"` — when reading response body fails
- `"Failed to await config.json text: {error}"` — when awaiting text promise fails
- `"Failed to convert config.json to string"` — when text conversion fails
- `"Failed to parse config.json as JSON: {error}"` — when JSON parsing fails
- `"config.json missing required field 'api_base_url'"` — when config is missing required field

## Verification Results

### 1. Build Verification
```
cargo check -p frontend
✓ Finished successfully
```

### 2. Clippy Verification
```
cargo clippy -p frontend -- -D warnings
✓ Finished successfully (no warnings)
```

### 3. Test Verification
```
cargo test -p frontend
✓ 5 tests passed, 0 failed, 14 ignored (wasm tests)
```

## Acceptance Criteria Status

- [x] No hardcoded URLs remain in `frontend/src/services.rs`
- [x] `get_api_base_url()` panics with clear error if config not initialized
- [x] `fetch_config()` panics with specific error messages for each failure case
- [x] `cargo clippy -p frontend -- -D warnings` passes
- [x] `cargo test -p frontend` passes
- [x] Error messages clearly indicate what configuration is missing and how to fix it

## Compliance

The implementation now complies with:
1. **FRONTEND.md** (lines 58-78): Config is runtime configuration, must be loaded before app mount
2. **GENERIC-FRONTEND.md** (lines 367-394): API Base URL Configuration pattern
3. **RUST-COMMON-SPEC.md**: Error handling conventions with clear panic messages