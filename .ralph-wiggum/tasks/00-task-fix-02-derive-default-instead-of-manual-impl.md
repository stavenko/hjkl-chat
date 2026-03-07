# Task 0-Fix-02: Use Derive Macro Instead of Manual Default Implementation

## Summary

Replace the manual `impl Default for Config` with `#[derive(Default)]` attribute in `backend/src/config.rs`.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Issue Details

**File:** `backend/src/config.rs:6`
**Warning:**
```
error: this `impl` can be derived
  --> backend/src/config.rs:6:1
   |
6 | / impl Default for Config {
7 | |     fn default() -> Self {
8 | |         Self {}
9 | |     }
10 | | }
   | |_^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.94.0/index.html#derivable_impls
```

**Root Cause:** The manual implementation of `Default` is trivial and can be replaced with the derive macro, which is the idiomatic Rust pattern.

## Required Changes

### backend/src/config.rs

Replace the manual implementation with derive:

```rust
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
/// Configuration for the backend server.
/// Will be populated with fields in future stories.
pub struct Config {
    // Configuration will be added in later stories
}
```

Note: This task assumes Task 0-Fix-01 has already added the `#[allow(dead_code)]` attribute.

## Acceptance Criteria

- [ ] Manual `impl Default for Config` is removed
- [ ] `#[derive(Default)]` is added to the struct
- [ ] `cargo build -p backend` produces no warnings
- [ ] `cargo clippy -p backend -- -D warnings` passes
- [ ] No other changes to the file

## Verification Commands

```bash
cargo build -p backend 2>&1 | grep -i warning || echo "No warnings"
cargo clippy -p backend -- -D warnings
```

## Related Files

- @.ralph-wiggum/reports/00-review-bootstrap.md
- @.ralph-wiggum/tasks/00-task-fix-01-remove-dead-code-warning.md