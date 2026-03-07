# Task: Integration Tests for Login Backend

## Overview
Create comprehensive integration tests to verify the login backend endpoint works correctly with external services (SQLite database, JWT token generation, bcrypt password verification).

## User Story
- @user-stories/01-login.md

## Spec Files
- @specs/BACKEND.md — Provider pattern, SQLiteProvider, application wiring
- @specs/GENERIC-BACKEND.md — Provider pattern, use-case structure, endpoint conventions
- @specs/RUST-COMMON-SPEC.md — Module conventions, error handling patterns
- @specs/PROJECT-STRUCTURE.md — Project layout

## Related Implementation
- Task: @.ralph-wiggum/tasks/01-task-login-backend.md
- Report: @.ralph-wiggum/reports/01-task-login-backend.md

## Acceptance Criteria

1. **Create `backend/src/tests/integration/login_service_tests.rs`** — Integration tests for login service with external service interactions:
   - Test successful login with valid credentials returns proper JWT tokens
   - Test that access_token and refresh_token are valid JWT format (3 parts separated by dots)
   - Test that tokens can be decoded and contain expected claims (user_id, exp, etc.)
   - Test wrong password returns HTTP 401 with error message
   - Test non-existent email returns HTTP 401 with error message
   - Test missing email field returns HTTP 400
   - Test missing password field returns HTTP 400
   - Test that session record is created in SQLite database on successful login
   - Test that session expires_at is set to a future timestamp

2. **Test Isolation Requirements:**
   - Each test must use unique email addresses (append UUID or timestamp)
   - Each test must clean up created users and sessions after completion
   - Tests must use `SQLiteProvider::new_for_test()` for isolated database
   - Tests must not depend on external MinIO/MailHog services (login doesn't require them)

3. **Token Validation Tests:**
   - Verify JWT tokens are properly signed with the configured secret key
   - Verify token expiration is set correctly (access_token: short-lived, refresh_token: long-lived)
   - Verify token payload contains user_id and email claims

4. **Session Persistence Tests:**
   - Verify session record exists in database after successful login
   - Verify session.user_id matches the logged-in user
   - Verify session.access_token and refresh_token match the response
   - Verify session.created_at and expires_at are set correctly

5. **Error Handling Tests:**
   - Verify bcrypt password verification failures return proper error
   - Verify database query failures are handled gracefully
   - Verify JWT signing errors are handled gracefully

6. **Build and Test Verification:**
   - `cargo check --workspace` succeeds
   - `cargo build -p backend` succeeds
   - `cargo test -p backend` passes with all new login service tests
   - Tests must run successfully without external MinIO/MailHog services

## Deliverables
- `backend/src/tests/integration/login_service_tests.rs` with comprehensive integration tests
- `cargo test -p backend` passes with all tests
- All tests properly isolated and cleaned up

## Notes
- Follow test patterns from existing `backend/src/tests/integration/auth_tests.rs`
- Use test isolation utilities: `temp_sqlite_path()`, `unique_email()`
- JWT secret should be from environment variable or test config (not hardcoded)
- Focus on external service interactions: SQLite, bcrypt, JWT library