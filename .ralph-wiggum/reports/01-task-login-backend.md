# Report: 01-task-login-backend.md

## Status: COMPLETED

## Summary

The login backend implementation was already present from the bootstrap phase. This report documents the verification and fix required to make all tests pass.

## Verification Completed

All required files and implementations were verified to exist:

### 1. Providers (backend/src/providers/)
- **sqlite.rs** - SQLiteProvider with connection pooling (r2d2 + rusqlite)
  - Methods: query_one, execute_with_params, execute, query_all
  - Test helper: new_for_test() method for integration tests
- **s3.rs** - S3Provider for file storage using aws-sdk-s3
- **local_filesystem.rs** - LocalFileSystemProvider for local file storage
- **smtp.rs** - SMTPProvider for email sending using lettre

### 2. Models (backend/src/models/)
- **auth.rs** - Auth request/response types (LoginRequest, LoginResponse, UserInfo, AuthError)
- **user.rs** - User model with from_row implementation
- **session.rs** - Session model with from_row implementation

### 3. Use Cases (backend/src/use_cases/)
- **auth.rs** - login() function with:
  - User lookup by email
  - Password verification using bcrypt
  - JWT token generation (access + refresh)
  - Session persistence to SQLite

### 4. API Endpoints (backend/src/api/endpoints/)
- **auth.rs** - POST /api/auth/login endpoint with proper error handling

### 5. Database Migrations (backend/migrations/)
- **001_create_users.sql** - Users table with UUID primary key
- **002_create_sessions.sql** - Sessions table with foreign key to users

### 6. Integration Tests (backend/src/tests/integration/auth_tests.rs)
- test_login_successful - Login with valid credentials
- test_login_wrong_password - Login with wrong password returns error
- test_login_nonexistent_email - Login with non-existent email returns error
- test_login_missing_fields_returns_error - Login with empty email returns error

## Issue Fixed

### bcrypt Password Verification

**Problem:** The test `test_login_wrong_password` was failing because `bcrypt::verify()` returns `Ok(false)` for wrong passwords, not an error. The original code used `?` operator which only propagates errors, not false results.

**Solution:** Updated `backend/src/use_cases/auth.rs:32-36` to check the boolean result:

```rust
let password_valid = bcrypt::verify(password, &user.password_hash)
    .map_err(|e| AuthError::PasswordHashError(Box::new(e)))?;

if !password_valid {
    return Err(AuthError::InvalidCredentials);
}
```

## Test Results

```
running 28 tests
test tests::integration::auth_tests::test_login_missing_fields_returns_error ... ok
test tests::integration::auth_tests::test_login_nonexistent_email ... ok
test tests::integration::auth_tests::test_login_wrong_password ... ok
test tests::integration::auth_tests::test_login_successful ... ok

test result: ok. 17 passed; 0 failed; 11 ignored; 0 measured; 0 filtered out
```

## Build Verification

- `cargo check --workspace` - PASS (8 warnings for unused code, acceptable)
- `cargo build -p backend` - PASS
- `cargo test -p backend` - PASS (17 passed, 11 ignored - external services)

## Warnings

The following warnings exist but are acceptable (unused code for future features):
- AuthError variants: MissingEmail, MissingPassword (will be used for input validation)
- Session::from_row (will be used for session lookup)
- S3Provider methods and error variants (full implementation pending)
- LocalFileSystemProvider::delete (will be used for file cleanup)
- SQLiteProvider methods: execute, query_all (alternative query interfaces)
- SMTPProvider fields and methods (email sending not yet used)

## Acceptance Criteria Status

| Criteria | Status |
|----------|--------|
| SQLiteProvider with user/session queries | ✅ Implemented |
| S3Provider stub | ✅ Implemented |
| LocalFileSystemProvider stub | ✅ Implemented |
| SMTPProvider stub | ✅ Implemented |
| User model | ✅ Implemented |
| Session model | ✅ Implemented |
| Auth use-case with login() | ✅ Implemented |
| POST /api/auth/login endpoint | ✅ Implemented |
| Auth request/response types | ✅ Implemented |
| main.rs wiring | ✅ Implemented |
| Cargo.toml dependencies | ✅ Implemented |
| Database migrations | ✅ Implemented |
| Integration tests | ✅ Implemented (4 tests pass) |

## Files Modified

- `backend/src/use_cases/auth.rs` - Fixed bcrypt verification to check boolean result

## Files Verified (No Changes Needed)

- `backend/src/providers/sqlite.rs`
- `backend/src/providers/s3.rs`
- `backend/src/providers/local_filesystem.rs`
- `backend/src/providers/smtp.rs`
- `backend/src/models/auth.rs`
- `backend/src/models/user.rs`
- `backend/src/models/session.rs`
- `backend/src/api/endpoints/auth.rs`
- `backend/src/main.rs`
- `backend/migrations/001_create_users.sql`
- `backend/migrations/002_create_sessions.sql`
- `backend/src/tests/integration/auth_tests.rs`