# Task: 01-task-fix-02-fix-clippy-warnings.md

## Overview
Fix all clippy warnings (excluding dead_code which is handled in a separate task).

## User Story
- @user-stories/01-login.md

## Related Review
- Review: @.ralph-wiggum/tasks/01-review-login-backend.md
- Report: @.ralph-wiggum/reports/01-review-login-backend.md

## Issues

The following clippy warnings must be fixed:

### Issue 1: needless_borrow in auth.rs endpoint

**File:** `backend/src/api/endpoints/auth.rs:33`
**Current:**
```rust
match login_use_case(sqlite.get_ref().clone(), email, password, &jwt_secret.get_ref()).await {
```
**Fix:**
```rust
match login_use_case(sqlite.get_ref().clone(), email, password, jwt_secret.get_ref()).await {
```

### Issue 2: let_unit_value in sqlite.rs

**File:** `backend/src/providers/sqlite.rs:49`
**Current:**
```rust
let path = fs_provider.save(data)?;
path
```
**Fix:**
```rust
fs_provider.save(data)?
```

### Issue 3: needless_borrow in sqlite.rs

**File:** `backend/src/providers/sqlite.rs:116`
**Current:**
```rust
results.push(f(&row)?);
```
**Fix:**
```rust
results.push(f(row)?);
```

### Issue 4: if_same_then_else in smtp.rs

**File:** `backend/src/providers/smtp.rs:33-43`
**Current:**
```rust
let transporter = if use_tls {
    AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(host)
        .port(port)
        .credentials(creds)
        .build()
} else {
    AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(host)
        .port(port)
        .credentials(creds)
        .build()
};
```
**Fix:** Remove the if/else since both branches are identical:
```rust
let transporter = AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(host)
    .port(port)
    .credentials(creds)
    .build();
```

### Issue 5: redundant_closure in smtp.rs

**File:** `backend/src/providers/smtp.rs:67`
**Current:**
```rust
self.transporter.send(email).await.map_err(|e| SMTPProviderError::SmtpTransport(e))?;
```
**Fix:**
```rust
self.transporter.send(email).await.map_err(SMTPProviderError::SmtpTransport)?;
```

### Issue 6: redundant_closure in auth.rs use-case

**File:** `backend/src/use_cases/auth.rs:28`
**Current:**
```rust
|row| User::from_row(row),
```
**Fix:**
```rust
User::from_row,
```

## Verification

Run these commands to verify the fixes:

```bash
cargo clippy -p backend -- -D warnings
```

Should complete with zero errors.

## Deliverables

- Fixed `backend/src/api/endpoints/auth.rs` (needless_borrow)
- Fixed `backend/src/providers/sqlite.rs` (let_unit_value, needless_borrow)
- Fixed `backend/src/providers/smtp.rs` (if_same_then_else, redundant_closure)
- Fixed `backend/src/use_cases/auth.rs` (redundant_closure)
- `cargo clippy -p backend -- -D warnings` passes with zero errors
- All tests still pass

## Report

Write report to @.ralph-wiggum/reports/01-task-fix-02-fix-clippy-warnings.md