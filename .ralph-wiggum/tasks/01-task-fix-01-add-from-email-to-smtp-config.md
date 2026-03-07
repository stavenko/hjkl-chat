# Task: 01-task-fix-01-add-from-email-to-smtp-config.md

## Overview
Add `from_email` field to SMTP config and update main.rs to read it from config instead of using hardcoded value.

## User Story
- @user-stories/01-login.md

## Related Review
- Review: @.ralph-wiggum/tasks/01-review-login-backend.md
- Report: @.ralph-wiggum/reports/01-review-login-backend.md

## Issue
**File:** `backend/src/main.rs:80`
**Problem:** The SMTP from_email is hardcoded as "noreply@example.com" instead of being read from the config file.

```rust
// Current code (line 80):
let _smtp_provider = SMTPProvider::new(
    &config.smtp.host,
    config.smtp.port,
    config.smtp.use_tls,
    &config.smtp.username,
    &config.smtp.password,
    "noreply@example.com",  // <-- HARDCODED
)
```

## Required Changes

### 1. Update `backend/src/config.rs`

Add `from_email` field to `SmtpConfig` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub username: String,
    pub password: String,
    pub from_email: String,  // <-- ADD THIS
}
```

### 2. Update `backend/src/main.rs`

Replace hardcoded value with config value:

```rust
let _smtp_provider = SMTPProvider::new(
    &config.smtp.host,
    config.smtp.port,
    config.smtp.use_tls,
    &config.smtp.username,
    &config.smtp.password,
    &config.smtp.from_email,  // <-- USE CONFIG
)
```

### 3. Update test config file `docker/test/config.toml`

Add `from_email` to the smtp section:

```toml
[smtp]
host = "mailhog"
port = 1025
use_tls = false
username = ""
password = ""
from_email = "noreply@example.com"  # <-- ADD THIS
```

## Verification

Run these commands to verify the fix:

```bash
# Verify no hardcoded email addresses in production code
grep -r "noreply@example.com" backend/src/ --include="*.rs" | grep -v test

# Should return no results (only config files may contain it)

# Build and test
cargo build -p backend
cargo test -p backend
cargo clippy -p backend -- -D warnings
```

## Deliverables

- Updated `backend/src/config.rs` with `from_email` field in `SmtpConfig`
- Updated `backend/src/main.rs` to read from config
- Updated `docker/test/config.toml` with `from_email` value
- All tests pass
- No clippy warnings introduced
- No hardcoded email addresses in production code

## Report

Write report to @.ralph-wiggum/reports/01-task-fix-01-add-from-email-to-smtp-config.md