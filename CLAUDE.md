# Project: hjkl-chat

## Git Workflow

- **Fast-forward merges only.** Never use `--no-ff`.
- Every branch must be rebased onto master before merging: `git rebase master`
- Merge with: `git merge --ff-only <branch>`
- If the branch cannot fast-forward, rebase it first — do not create merge commits.

## Architecture

- **Frontend**: Leptos 0.6 CSR, built with Trunk (`trunk build`, `trunk serve` on port 8080)
- **Backend**: Rust (Actix/Axum) with S3 storage
- **Docker**: Development services in `docker/<branch-name>/docker-compose.yml`

## Frontend

### Config

Runtime config is served by a Docker nginx container, NOT from `frontend/config.json`.
The actual config lives at `docker/<branch-name>/config.json` and is proxied via `Trunk.toml`.
After editing it, restart the container: `docker compose restart config-server`.

Config structure:
```json
{
  "api_base_url": "...",
  "features": {
    "debug-light-dark-switch": true
  }
}
```

Features are stored in a static and accessed via `services::get_features().is_enabled("feature-name")`.

### CSS Theme System

Uses `data-theme` attribute on `<html>` with tokens defined in `styles/tokens.css`.
Light theme is default (`:root`), dark theme via `[data-theme="dark"]`.

### Testing

Run frontend unit tests:
```sh
wasm-pack test --headless --chrome
```

**Chromedriver version must match installed Chrome.** wasm-pack auto-downloads the latest,
which may not match. If tests fail with `http status: 404`, check versions and replace
the cached binary at `~/Library/Caches/.wasm-pack/chromedriver-*/chromedriver`.

**Leptos effects run as browser microtasks**, not synchronously. Tests that depend on
effect execution must be `async` and yield with:
```rust
async fn tick() {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::NULL))
        .await.unwrap();
}
```

The `#[wasm_bindgen(start)]` entry point in `lib.rs` is gated with `#[cfg(not(test))]`
to avoid conflicts with the test harness.
