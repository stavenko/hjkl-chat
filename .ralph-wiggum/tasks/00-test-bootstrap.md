# Task 0-Test: Bootstrap Integration Tests

## Summary

Write integration tests to verify that the docker-compose test environment works correctly with external services (MinIO and MailHog), and that the test isolation utilities function properly.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Spec Files

- @specs/TESTING.md — docker-compose test environment, test isolation, test patterns
- @specs/GENERIC-BACKEND.md — provider pattern for external services
- @specs/RUST-COMMON-SPEC.md — Rust module conventions

## What to Do

### 1. MinIO Integration Tests

Write integration tests that verify:
- MinIO service starts and responds to health checks
- Can create a bucket with a unique prefix (using test isolation utilities)
- Can upload an object to the bucket
- Can download the object and verify content
- Can delete the bucket
- Test cleanup runs properly (no leftover buckets between tests)

### 2. MailHog Integration Tests

Write integration tests that verify:
- MailHog service starts and responds to health checks
- Can send an email via SMTP
- Can retrieve sent emails via MailHog API
- Can verify email content (sender, recipient, subject, body)

### 3. Test Isolation Utilities

Verify the test isolation utilities work correctly:
- `random_bucket_prefix()` generates unique prefixes
- `temp_sqlite_path()` generates valid temporary file paths
- `unique_email()` generates valid unique email addresses

### 4. Concurrent Test Execution

Add tests that verify:
- Multiple tests can run concurrently without conflicts
- Each test uses isolated resources (buckets, emails, temp files)
- Test cleanup doesn't interfere with other running tests

## Acceptance Criteria

- `cargo test -p backend -- --test-threads=1` passes with all tests
- `cargo test -p backend -- --test-threads=4` passes (concurrent execution)
- Tests connect to actual MinIO service via docker-compose
- Tests connect to actual MailHog service via docker-compose
- Each test uses unique, isolated resources
- No test pollution between tests (each test is independent)
- Tests fail gracefully when services are not running
- All test cleanup runs successfully