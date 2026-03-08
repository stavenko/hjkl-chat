# Integration Test Task: Backend Registration Init

**User Story:** 02-registration.md  
**Implementation Task:** 03-task-registration-init-backend.md  
**Spec Files:**
- [BACKEND.md](../specs/BACKEND.md) — Provider pattern, API structure
- [GENERIC-BACKEND.md](../specs/GENERIC-BACKEND.md) — Backend architecture patterns
- [RUST-COMMON-SPEC.md](../specs/RUST-COMMON-SPEC.md) — Error handling, module conventions

## Objective

Create integration tests for the registration init backend implementation that verify external service interactions with SQLite and SMTP providers.

## Test File

Create `backend/tests/registration_service_tests.rs` with the following test modules:

### 1. Registration Init Tests

Test the `POST /api/auth/registration/init` endpoint:

- `test_registration_init_successful` — Verify successful registration init returns correct response format with session_id and resend_available_at
- `test_registration_init_generates_6_digit_code` — Verify verification code is exactly 6 digits
- `test_registration_init_creates_session_in_database` — Verify session is persisted in SQLite with correct timestamps
- `test_registration_init_sends_email_via_smtp` — Verify email is sent through SMTP provider (check MailHog API)
- `test_registration_init_session_expires_in_15_minutes` — Verify expires_at is 15 minutes from created_at
- `test_registration_init_resend_available_in_60_seconds` — Verify resend_available_at is 60 seconds from created_at
- `test_registration_init_empty_email_returns_error` — Verify empty email returns error response
- `test_registration_init_invalid_email_format_returns_error` — Verify invalid email format returns error
- `test_registration_init_duplicate_email_returns_error` — Verify duplicate email within active session returns error

### 2. Session Database Tests

Test SQLite session persistence:

- `test_session_stored_with_correct_schema` — Verify registration_sessions table has all required columns
- `test_session_uuid_is_valid_format` — Verify session id is valid UUID format
- `test_session_email_is_unique` — Verify UNIQUE constraint on email column
- `test_session_timestamps_are_rfc3339_compatible` — Verify all timestamp columns store valid datetime

### 3. Email Verification Tests

Test SMTP email sending:

- `test_email_contains_verification_code` — Verify sent email contains the 6-digit verification code
- `test_email_sent_to_correct_address` — Verify email recipient matches request email
- `test_email_has_subject_line` — Verify email has appropriate subject line

### 4. Error Handling Tests

Test error scenarios:

- `test_database_error_handling` — Verify database errors are handled gracefully
- `test_smtp_error_handling` — Verify SMTP errors are handled gracefully (connection refused)

## Test Requirements

1. **Test Isolation:**
   - Use `unique_email()` helper from `test_utils` for unique email addresses
   - Use `temp_sqlite_path()` for isolated database per test
   - Clean up test data after each test

2. **External Service Verification:**
   - SQLite: Verify data persisted correctly using direct database queries
   - SMTP: Verify emails sent by checking MailHog API at `http://localhost:8025/api/v2/messages`

3. **Timestamp Verification:**
   - Use `chrono` for timestamp comparison
   - Allow 1-second tolerance for timestamp comparisons
   - Verify all timestamps are in UTC

## Verification Steps

1. **Build verification:**
   ```bash
   cargo check -p backend
   cargo clippy -p backend -- -D warnings
   cargo build -p backend
   ```

2. **Test verification:**
   ```bash
   cargo test -p backend registration_service_tests
   ```

## Acceptance Criteria

- [ ] All registration init tests pass
- [ ] All session database tests pass
- [ ] All email verification tests pass (when MailHog available, otherwise ignored with reason)
- [ ] All error handling tests pass
- [ ] Tests use proper isolation (unique emails, temp databases)
- [ ] Tests have cleanup logic
- [ ] Tests ignored without running have clear `#[ignore = "reason"]` comments
- [ ] `cargo clippy -p backend -- -D warnings` passes
- [ ] `cargo test -p backend` passes with no failures

## Deliverables

1. `backend/tests/registration_service_tests.rs` with all test modules
2. Report at `/project/.ralph-wiggum/reports/03-test-registration-init-backend.md`