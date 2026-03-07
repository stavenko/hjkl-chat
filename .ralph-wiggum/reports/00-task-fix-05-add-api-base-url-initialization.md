# Report: Task 00-Fix-05 - Add API Base URL Initialization

## Summary

Successfully implemented API base URL initialization in the frontend as required by `specs/FRONTEND.md`.

## Files Created

1. **frontend/src/services.rs**
   - Created new module with `API_BASE_URL` static
   - Implemented `init_api_base_url()` async function that fetches `/config.json`
   - Implemented `get_api_base_url()` function to retrieve the stored URL
   - Added `#[allow(dead_code)]` to `get_api_base_url()` since it's not yet used

## Files Modified

1. **frontend/src/main.rs**
   - Added `mod services;` declaration
   - Renamed `run()` function to `main()` (exported via `#[wasm_bindgen]`)
   - Wrapped app mounting in `leptos::spawn_local(async { ... })`
   - Added call to `services::init_api_base_url().await` before `mount_to_body(App)`
   - Added `#[allow(clippy::main_recursion)]` attribute
   - Removed standalone `main()` function that called `run()`

2. **frontend/Cargo.toml**
   - Added `reqwest = { version = "0.11", features = ["wasm-streams", "json"] }`
   - Added `once_cell = "1.19"`

## Verification

```bash
cargo check -p frontend
cargo clippy -p frontend -- -D warnings
```

Both commands pass successfully with no warnings.

## Implementation Details

The implementation follows the exact pattern specified in `specs/FRONTEND.md` lines 117-124:

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

The `init_api_base_url()` function:
1. Fetches `/config.json` from the server
2. Parses the JSON response
3. Extracts `api_base_url` string
4. Stores it in a thread-safe `RwLock<String>` wrapped in `Lazy`

All API service calls will use `get_api_base_url()` to prepend the base URL to endpoint paths (to be implemented in future tasks).