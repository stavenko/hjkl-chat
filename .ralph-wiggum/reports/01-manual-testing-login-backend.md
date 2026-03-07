# Manual Testing Report: 01-login-backend

**Date:** 2026-03-07
**Tester:** Automated verification script

## Build Output

### cargo check --workspace
```
warning: variants `MissingEmail` and `MissingPassword` are never constructed
  --> backend/src/models/auth.rs:37:5
   |
25 | pub enum AuthError {
   |          --------- variants in this enum
...
37 |     MissingEmail,
   |     ^^^^^^^^^^^^
38 |     #[error("Missing password")]
39 |     MissingPassword,
   |     ^^^^^^^^^^^^^^^
   |
   = note: `AuthError` has a derived impl of the trait `Debug`, but this is intentionally ignored during dead code analysis
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: associated function `from_row` is never used
  --> backend/src/models/session.rs:16:12
   |
15 | impl Session {
   | ------------ associated function in this implementation
16 |     pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
   |            ^^^^^^^^

warning: variant `AwsConfig` is never constructed
  --> backend/src/providers/s3.rs:10:5
   |
 6 | pub enum S3ProviderError {
   |          --------------- variant in this enum
...
10 |     AwsConfig(String),
   |     ^^^^^^^^^
   |
   = note: `S3ProviderError` has a derived impl of the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: method `delete_object` is never used
  --> backend/src/providers/s3.rs:88:18
   |
22 | impl S3Provider {
   | --------------- method in this implementation
...
88 |     pub async fn delete_object(&self, key: &str) -> S3ProviderResult<()> {
   |                  ^^^^^^^^^^^^^

warning: method `delete` is never used
  --> backend/src/providers/local_filesystem.rs:36:12
   |
19 | impl LocalFileSystemProvider {
   | ---------------------------- method in this implementation
...
36 |     pub fn delete(&self, path: &Path) -> LocalFileSystemProviderResult<()> {
   |            ^^^^^^

warning: methods `execute` and `query_all` are never used
   --> backend/src/providers/sqlite.rs:74:12
    |
 41 | impl SQLiteProvider {
    | ------------------- methods in this implementation
...
 74 |     pub fn execute(&self, sql: &str, params: &[rusqlite::types::ValueRef<'_>]) -> SQLiteProviderResult<usize> {
    |            ^^^^^^^
...
107 |     pub fn query_all<T, F>(&self, sql: &str, params: &[rusqlite::types::ValueRef<'_>], mut f: F) -> SQLiteProviderResult<Vec<T>>
    |            ^^^^^^^^^

warning: fields `transporter` and `from_address` are never read
  --> backend/src/providers/smtp.rs:18:5
   |
17 | pub struct SMTPProvider {
   |            ------------ fields in this struct
18 |     transporter: AsyncSmtpTransport<lettre::Tokio1Executor>,
   |     ^^^^^^^^^^^
19 |     from_address: Mailbox,
   |     ^^^^^^^^^^^^

warning: method `send_email` is never used
  --> backend/src/providers/smtp.rs:53:18
   |
22 | impl SMTPProvider {
   | ----------------- method in this implementation
...
53 |     pub async fn send_email(
   |                  ^^^^^^^^^^

warning: `backend` (bin "backend") generated 8 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.48s
```

**Status:** PASS (warnings are acceptable - unused code for future features)

### cargo build -p backend
```
warning: `backend` (bin "backend") generated 8 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s
```

**Status:** PASS

### cargo build -p frontend
```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
```

**Status:** PASS

## Test Results

### cargo test -p backend
```
running 38 tests
test tests::integration::concurrent_tests::test_concurrent_mailhog_emails ... ignored, Requires MailHog service running
test tests::integration::isolation_tests::test_random_bucket_prefix_format ... ok
test tests::integration::isolation_tests::test_random_bucket_prefix_uniqueness ... ok
test tests::integration::isolation_tests::test_isolation_utils_combined_uniqueness ... ok
test tests::integration::isolation_tests::test_temp_sqlite_path_format ... ok
test tests::integration::isolation_tests::test_temp_sqlite_path_uniqueness ... ok
test tests::integration::isolation_tests::test_unique_email_format ... ok
test tests::integration::isolation_tests::test_unique_email_uniqueness ... ok
test tests::integration::concurrent_tests::test_concurrent_isolation_resources ... ok
test tests::integration::concurrent_tests::test_concurrent_path_generation ... ok
test tests::integration::concurrent_tests::test_concurrent_bucket_prefix_generation ... ok
test tests::integration::concurrent_tests::test_concurrent_email_generation ... ok
test tests::integration::auth_tests::test_login_nonexistent_email ... ok
test tests::integration::auth_tests::test_login_missing_fields_returns_error ... ok
test tests::integration::auth_tests::test_login_wrong_password ... ok
test tests::integration::login_service_tests::test_bcrypt_verification_failure_handling ... ok
test tests::integration::login_service_tests::test_login_successful_returns_valid_tokens ... ok
test tests::integration::mailhog_tests::test_mailhog_health_check ... ignored, Requires MailHog service running
test tests::integration::mailhog_tests::test_mailhog_multiple_emails ... ignored, Requires MailHog service running
test tests::integration::mailhog_tests::test_mailhog_retrieve_emails ... ignored, Requires MailHog service running
test tests::integration::mailhog_tests::test_mailhog_send_email ... ignored, Requires MailHog service running
test tests::integration::mailhog_tests::test_mailhog_verify_email_content ... ignored, Requires MailHog service running
test tests::integration::minio_tests::test_minio_bucket_cleanup ... ignored, Requires MinIO service running
test tests::integration::minio_tests::test_minio_create_bucket ... ignored, Requires MinIO service running
test tests::integration::minio_tests::test_minio_delete_bucket ... ignored, Requires MinIO service running
test tests::integration::minio_tests::test_minio_health_check ... ignored, Requires MinIO service running
test tests::integration::minio_tests::test_minio_upload_download_object ... ignored, Requires MinIO service running
test tests::integration::test_test_utils_generate_unique_values ... ok
test tests::integration::test_test_utils_generate_valid_values ... ok
test tests::integration::auth_tests::test_login_successful ... ok
test tests::integration::login_service_tests::test_session_timestamps_are_set_correctly ... ok
test tests::integration::login_service_tests::test_jwt_signing_error_handling ... ok
test tests::integration::login_service_tests::test_multiple_users_isolated ... ok
test tests::integration::login_service_tests::test_tokens_are_valid_jwt_format ... ok
test tests::integration::login_service_tests::test_database_query_failure_handling ... ok
test tests::integration::login_service_tests::test_tokens_can_be_decoded_with_expected_claims ... ok
test tests::integration::login_service_tests::test_session_user_id_matches_logged_in_user ... ok
test tests::integration::login_service_tests::test_session_tokens_match_response ... ok

test result: ok. 27 passed; 0 failed; 11 ignored; 0 measured; 0 filtered out; finished in 7.32s
```

**Status:** PASS - 27 tests passed, 0 failed, 11 ignored (external services not available)

## Backend Acceptance Criteria

### Criterion 1: POST /api/auth/login returns user, access_token, refresh_token
**Status:** CANNOT VERIFY - Backend requires MinIO service to start

**Evidence:**
The backend fails to start without MinIO because SQLiteProvider downloads the database from S3:
```
thread 'main' panicked at backend/src/main.rs:72:6:
Failed to initialize SQLite provider: S3(AwsSdk("dispatch failure"))
```

**Note:** Integration tests verify this functionality without requiring the full server to start. The test `test_login_successful` passes and confirms the endpoint logic works correctly.

### Criterion 2: Wrong credentials return error
**Status:** CANNOT VERIFY - Backend requires MinIO service to start

**Evidence:**
Integration test `test_login_wrong_password` passes, confirming the error handling logic is correct.

### Criterion 3: Integration tests cover all scenarios
**Status:** PASS

**Evidence:**
- `test_login_successful` - PASS
- `test_login_wrong_password` - PASS  
- `test_login_nonexistent_email` - PASS
- `test_login_missing_fields_returns_error` - PASS

### Criterion 4: cargo test - all tests pass
**Status:** PASS

**Evidence:**
27 tests passed, 0 failed, 11 ignored (external services)

### Criterion 5: Backend starts with config file, serves HTTP
**Status:** CANNOT VERIFY - Requires MinIO service

**Evidence:**
Backend startup fails without MinIO:
```
Failed to initialize SQLite provider: S3(AwsSdk("dispatch failure"))
```

### Criterion 6: docker-compose.yml includes all services
**Status:** PASS

**Evidence:**
docker/local/docker-compose.yml includes:
- backend service (builds from Dockerfile.backend, ports 5000:5000)
- frontend service (nginx:alpine, ports 3000:80)
- minio service (minio/minio:latest, ports 9000:9000, 9001:9001)
- mailhog service (mailhog/mailhog:latest, ports 1025:1025, 8025:8025)

## Frontend Acceptance Criteria

### Criterion 1: LoginPage exists at /login
**Status:** FAIL - Not implemented

**Evidence:**
Frontend source code only contains:
- `/project/frontend/src/main.rs` - Basic App component with "Hello from Leptos!"
- `/project/frontend/src/services.rs` - API base URL initialization

No LoginPage component exists.

### Criterion 2: LoginForm has email and password fields
**Status:** FAIL - Not implemented

**Evidence:**
No LoginForm component exists in the frontend.

### Criterion 3: Button disabled until both fields filled
**Status:** FAIL - Not implemented

**Evidence:**
No LoginForm component exists.

### Criterion 4: Server error displayed inline on password field
**Status:** FAIL - Not implemented

**Evidence:**
No LoginForm component exists.

### Criterion 5: "Forgot password?" link to /password/restore
**Status:** FAIL - Not implemented

**Evidence:**
No navigation or routing implemented.

### Criterion 6: "Don't have an account? Register" link to /register
**Status:** FAIL - Not implemented

**Evidence:**
No navigation or routing implemented.

### Criterion 7: auth_service module implements login async function
**Status:** FAIL - Not implemented

**Evidence:**
No auth_service module exists. Only services.rs with init_api_base_url().

### Criterion 8: Authentication check method exists
**Status:** FAIL - Not implemented

**Evidence:**
No authentication service exists.

### Criterion 9: Tokens stored in AuthState and localStorage
**Status:** FAIL - Not implemented

**Evidence:**
No AuthState or token storage implemented.

### Criterion 10: Frontend unit tests pass
**Status:** CANNOT VERIFY - No tests exist

**Evidence:**
No test files found in frontend directory.

### Criterion 11: "/" path redirects to "/login"
**Status:** FAIL - Not implemented

**Evidence:**
No routing implemented. App component is a simple "Hello from Leptos!" view.

## Issues Found

1. **Backend requires MinIO to start** - The SQLiteProvider implementation downloads the database from S3, which requires MinIO to be running. This prevents manual testing of the HTTP endpoints.

2. **chromium not installed** - Frontend DOM verification cannot be performed without a headless browser.

3. **docker-compose.yml not checked** - Need to verify the file exists and includes all required services.

## Summary

- Backend criteria: 3/6 PASS (1 CANNOT VERIFY without MinIO, 2 verified via integration tests)
- Frontend criteria: 0/11 PASS (11 FAIL - not implemented)
- Overall: FAIL

## Issues Found

1. **Backend requires MinIO to start** - The SQLiteProvider implementation downloads the database from S3, which requires MinIO to be running. This prevents manual testing of the HTTP endpoints via curl.

2. **Frontend not implemented** - The login frontend is completely missing:
   - No LoginPage component
   - No LoginForm component
   - No auth_service module
   - No routing setup
   - No AuthState for token storage
   - No unit tests

3. **chromium not installed** - Frontend DOM verification cannot be performed without a headless browser.

## Recommendations

1. **Backend is functional** - Integration tests verify the login logic works correctly. To test via HTTP:
   - Start MinIO: `docker-compose -f docker/local/docker-compose.yml up -d minio`
   - Start backend: `JWT_SECRET=testsecret123 cargo run -p backend -- run --config docker/test/config.toml`
   - Test with curl commands as specified in the task

2. **Frontend needs implementation** - Create a new task to implement:
   - LoginPage component at /login route
   - LoginForm with email/password fields
   - auth_service module with login() function
   - AuthState for token storage
   - Routing with authentication guard
   - Unit tests for form validation and error display