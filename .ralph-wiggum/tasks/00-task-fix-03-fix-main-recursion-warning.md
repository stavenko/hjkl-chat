# Task 0-Fix-03: Fix Main Recursion Warning in Frontend

## Summary

Fix the clippy warning about `main()` function name in `frontend/src/main.rs` by either renaming the function or adding an allow attribute with explanation.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Issue Details

**File:** `frontend/src/main.rs:11`
**Warning:**
```
error: recursing into entrypoint `main`
  --> frontend/src/main.rs:11:8
   |
11 | pub fn main() {
   |        ^^^^
   |
   = help: consider using another function for this recursion
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.94.0/index.html#main_recursion
```

**Root Cause:** The `#[wasm_bindgen]` exported function is named `main()`, which conflicts with Rust's conventional `main` entry point. Clippy warns about this potential confusion.

## Required Changes

### frontend/src/main.rs

Rename the function to `run()` and update the index.html to call `run()` instead of `main()`:

```rust
use leptos::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <div>"Hello from Leptos!"</div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
```

### frontend/index.html

Update the JavaScript to call `run()` instead of `main()`:

```html
<script>
  window.addEventListener('TrunkApplicationStarted', () => {
    if (typeof window.wasmBindings?.run === 'function') {
      window.wasmBindings.run();
    }
  });
</script>
```

## Acceptance Criteria

- [ ] `main()` function renamed to `run()` in `frontend/src/main.rs`
- [ ] JavaScript in `frontend/index.html` updated to call `run()`
- [ ] `cargo build -p frontend` produces no warnings
- [ ] `cargo clippy -p frontend -- -D warnings` passes
- [ ] Frontend still compiles and runs correctly

## Verification Commands

```bash
cargo build -p frontend 2>&1 | grep -i warning || echo "No warnings"
cargo clippy -p frontend -- -D warnings
```

## Related Files

- @.ralph-wiggum/reports/00-review-bootstrap.md