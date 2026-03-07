# Task 0-Fix-01: Remove Dead Code Warning for Config Struct

## Summary

Add `#[allow(dead_code)]` attribute to the `Config` struct in `backend/src/config.rs` with a comment explaining it will be used in future stories.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Issue Details

**File:** `backend/src/config.rs:2`
**Warning:**
```
warning: struct `Config` is never constructed
 --> backend/src/config.rs:2:12
  |
2 | pub struct Config {
  |            ^^^^^^
  |
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
```

**Root Cause:** The `Config` struct is declared but not yet used. It will be populated with configuration fields in future user stories.

## Required Changes

### backend/src/config.rs

Add `#[allow(dead_code)]` attribute with explanatory comment:

```rust
#[allow(dead_code)]
/// Configuration for the backend server.
/// Will be populated with fields in future stories.
pub struct Config {
    // Configuration will be added in later stories
}
```

## Acceptance Criteria

- [ ] `cargo build -p backend` produces no warnings
- [ ] `cargo clippy -p backend -- -D warnings` passes
- [ ] Comment explains why `#[allow(dead_code)]` is necessary
- [ ] No other changes to the file

## Verification Commands

```bash
cargo build -p backend 2>&1 | grep -i warning || echo "No warnings"
cargo clippy -p backend -- -D warnings
```

## Related Files

- @.ralph-wiggum/reports/00-review-bootstrap.md