# Testing Considerations

## Overview
Each backend must have comprehensive tests. This is **critical** for reliability.

## 1. Mocking External Interactions
1. Whenever the code interacts with an external system (HTTP API calls, sending email, sending messages over TCP, etc.) **mocking is mandatory** – there are no exceptions.
2. All mock‑based tests are executed via a `docker‑compose` environment that provides the mock services and a test‑specific configuration.
3. S3-compatible storage is provided by MinIO in the test environment.

## 2. Required Mock Services for This Project
- **MailHog** – to capture and verify outgoing email messages.
- **MinIO** – S3-compatible object storage used for all persistent data.

## 3. Test Environment Setup
Create a `docker-compose.yml` that brings up the following containers:
- `minio` – an S3-compatible object store.
- `mailhog` – a MailHog instance for email capture.

The test suite should start this compose file before running and shut it down afterward.

## 4. Test Isolation Strategies
- **MinIO (S3)**: each test must use a **random bucket prefix** (e.g., `test-<uuid>-`) so that objects from different tests never collide. The test teardown should delete the prefixed objects.
- **SQLite**: each test must use a **temporary file** (e.g., `/tmp/test-<uuid>.db`) for its SQLite database. The file is deleted after the test finishes.
- **MailHog**: generate a **unique email address** for each test (e.g., `test+<uuid>@example.com`). If multiple emails are needed, add numeric or random suffixes to keep them distinct.

## 5. Example Docker‑Compose Snippet
```yaml
version: "3.8"
services:
  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    ports:
      - "9000:9000"   # S3 API
      - "9001:9001"   # Console UI
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:9000/minio/health/live"]
      interval: 5s
      timeout: 5s
      retries: 5

  mailhog:
    image: mailhog/mailhog:latest
    ports:
      - "1025:1025"   # SMTP
      - "8025:8025"   # HTTP UI
```

## 6. Example Test (concise)

Below is a short, clear test pattern broken into four steps:

1. **Init** – start the app with test fixtures.
2. **Fixtures** – provide deterministic data (e.g., a test user).
3. **Test‑scenario** – perform the request against the endpoint.
4. **Assertions** – verify status, response body and business rules.

```rust
#[actix_web::test]
async fn test_me_endpoint_success() {
    // 1. Init + 2. Fixtures
    let (det, app, ()) = create_app_with_fixtures(fixtures_me_endpoint)
        .await
        .unwrap();

    // 3. Test‑scenario: call /me with auth token
    let token = det.auth_token;
    let req = actix_test::TestRequest::get()
        .uri("/me")
        .append_header(("authorization", token))
        .to_request();
    let response = call_service(&app, req).await;
    let status = response.status();

    // 4. Assertions
    let response_body = read_body(response).await;
    if status != 200 {
        let body_str = std::str::from_utf8(&response_body).unwrap_or("Invalid UTF-8");
        panic!("Status is not ok: {status}, body: {body_str}");
    }
    let user_info: UserInfo =
        serde_json::from_slice(&response_body).expect("Response should be valid UserInfo JSON");
    // Verify user fields match fixtures
    assert_eq!(user_info.name, None, "Name should be None as test user is created without name");
    assert_eq!(
        user_info.email.as_deref(),
        Some(TEST_USER_EMAIL),
        "Email should match test user"
    );
}
```

This compact form follows the **init → fixtures → scenario → assertions** flow and can be reused for other endpoints.


## 6. Tests location.

Backend must own all tests.
It could be this:
```
backend/
  src
  tests
```

Or this:
```
backend/
  src/
    cli.rs
    tests.rs
    main.rs
      mod cli;
      #[cfg(test)]
      mod tests
    
```
Test must be running by `cargo test` or `cargo test -p backend`.



## 7. Test Configuration

For tests must be utilized actix test service.
IMPORTANT:
 - Manual implementations of testing server is prohibited.
 - Mocks PROHIBITED.

It is very IMPORTANT to reduce amount of written code. Use actix testings



- Provide a **test‑specific config file** (e.g., `tests/config.toml`) that points the providers to the Docker‑Compose host/ports and injects the random identifiers described above.
- S3 config should point to `http://localhost:9000` with MinIO credentials.
- SQLite config should specify a temporary file path pattern.
- The test harness should read this config and initialise the providers accordingly.



