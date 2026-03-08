# Task: Registration Verify and Complete Backend Integration Tests

## User Story
[02-registration.md](../../user-stories/02-registration.md)

## Related Implementation
- Task: [04-task-registration-verify-complete-backend.md](./04-task-registration-verify-complete-backend.md)
- Report: [04-task-registration-verify-complete-backend.md](../reports/04-task-registration-verify-complete-backend.md)

## Specs
- [BACKEND.md](../../specs/BACKEND.md)
- [GENERIC-BACKEND.md](../../specs/GENERIC-BACKEND.md)
- [RUST-COMMON-SPEC.md](../../specs/RUST-COMMON-SPEC.md)
- [PROJECT-STRUCTURE.md](../../specs/PROJECT-STRUCTURE.md)

## Description
Create integration tests for the registration verify and complete backend endpoints. These tests must verify external service interactions using curl requests to the running backend server.

## Acceptance Criteria

### 1. Test Environment Setup
- Create test helper functions:
  - `temp_sqlite_path()` - creates a temporary SQLite database file
  - `unique_session_id()` - generates unique session IDs for test isolation
  - `start_backend_with_config(sqlite_path)` - starts backend with custom SQLite config
  - `wait_for_backend_ready()` - waits for backend to be ready to accept requests

### 2. Registration Verify Endpoint Tests
Test `POST /api/auth/registration/verify`:
- **Valid verification code**: Send valid session_id and correct 6-digit code, expect success response with session_id and expires_at
- **Invalid verification code**: Send valid session_id with wrong code, expect error response
- **Expired session**: Create session, wait for expiry (or manipulate timestamp), send valid code, expect expired session error
- **Unknown session**: Send non-existent session_id, expect unknown session error
- **Empty code**: Send empty or non-6-digit code, expect validation error

### 3. Registration Complete Endpoint Tests
Test `POST /api/auth/registration/complete`:
- **Successful completion**: Send valid verified session_id with matching strong passwords, expect user object, access_token, and refresh_token
- **Password mismatch**: Send session_id with non-matching passwords, expect password mismatch error
- **Weak password**: Send session_id with weak password (no uppercase, no digit, too short), expect weak password error
- **Expired session**: Send expired session_id, expect expired session error
- **Unknown session**: Send non-existent session_id, expect unknown session error
- **Session already used**: Complete registration, then attempt again with same session_id, expect session used error

### 4. End-to-End Registration Flow Test
Test complete registration flow from init to complete:
- Call `POST /api/auth/registration/init` with email
- Extract session_id from response
- Call `POST /api/auth/registration/verify` with session_id and code (retrieve code from MailHog)
- Call `POST /api/auth/registration/complete` with session_id and passwords
- Verify user was created in SQLite database
- Verify tokens are valid JWT tokens
- Verify registration session is marked as used or deleted

### 5. Database Persistence Tests
- Verify users table has correct schema (id, email, created_at)
- Verify passwords table has correct schema (user_id, hash, algorithm)
- Verify sessions table has tokens stored (user_id, token, expires_at, token_type)
- Verify password is hashed using argon2 algorithm

### 6. JWT Token Validation
- Verify access_token and refresh_token are valid JWT format
- Verify tokens contain expected claims (user_id, exp, token_type)
- Verify tokens have correct expiration times

## Test Implementation Details

### Test Location
- Create tests in `backend/tests/registration_verify_complete.rs`

### Test Prerequisites
- Backend must be configured to use SQLite for persistence
- MailHog must be available for email verification code retrieval
- Tests marked with `#[ignore]` require running backend server and external services

### Test Structure
```rust
#[test]
#[ignore = "requires running backend server"]
fn test_registration_verify_valid_code() {
    // Implementation using curl requests via reqwest or std::process::Command
}

#[test]
#[ignore = "requires running backend server"]
fn test_registration_complete_success() {
    // Implementation using curl requests
}

// Add more tests for each scenario above
```

### Curl Request Examples
```bash
# Verify endpoint
curl -X POST http://localhost:8080/api/auth/registration/verify \
  -H "Content-Type: application/json" \
  -d '{"session_id": "uuid", "code": "123456"}'

# Complete endpoint
curl -X POST http://localhost:8080/api/auth/registration/complete \
  -H "Content-Type: application/json" \
  -d '{"session_id": "uuid", "password": "SecurePass123", "password_confirm": "SecurePass123"}'
```

### MailHog Integration
- Retrieve verification codes from MailHog API: `http://localhost:8025/api/v2/messages`
- Parse email body to extract 6-digit verification code
- Use code in verify endpoint test

## Verification
- Run `cargo test -p backend --test registration_verify_complete` - all non-ignored tests pass
- Run `cargo test -p backend --test registration_verify_complete -- --ignored` with backend running - all integration tests pass
- All tests follow the pattern from `01-test-login-backend.md` and `03-test-registration-init-backend.md`

## Files to Create
- `backend/tests/registration_verify_complete.rs`