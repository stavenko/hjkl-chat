# Manual Testing Task: Backend Registration Init

**User Story:** 02-registration.md  
**Implementation Task:** 03-task-registration-init-backend.md  
**Test Task:** 03-test-registration-init-backend.md  
**Spec Files:**
- [BACKEND.md](../specs/BACKEND.md) — Provider pattern, API structure
- [GENERIC-BACKEND.md](../specs/GENERIC-BACKEND.md) — Backend architecture patterns
- [RUST-COMMON-SPEC.md](../specs/RUST-COMMON-SPEC.md) — Error handling, module conventions

## Objective

Execute manual verification of all acceptance criteria from user story 02-registration.md for the registration init backend implementation. This task is a VERIFICATION SCRIPT that does NOT fix anything — it only collects evidence.

## Manual Testing Script

### Step 1: Build the Project

Execute the following commands and log the full output into the report:

```bash
cargo check -p backend
cargo build -p backend
```

**Report:** Include full output, note any warnings or errors.

---

### Step 2: Run Backend Tests

Execute:

```bash
cargo test -p backend
```

**Report:** Include full output, count passed/failed/ignored tests.

---

### Step 3: Start Docker Services

Start the test environment with external services:

```bash
docker-compose -f docker/test/docker-compose.yml up -d
```

Wait 10 seconds for services to be ready.

**Report:** Include docker-compose output, verify MinIO and MailHog are running.

---

### Step 4: Start Backend Application

Start the backend server in the background:

```bash
DATABASE_URL="sqlite:///tmp/test.db" SMTP_HOST="localhost" SMTP_PORT=1025 SMTP_FROM_EMAIL="test@example.com" cargo run -p backend &
sleep 5
```

**Report:** Include startup output, confirm server is listening on configured port.

---

### Step 5: Test Backend Acceptance Criteria via curl

For each backend acceptance criterion, call the relevant endpoint and log request and response:

#### 5.1 POST /api/auth/registration/init — Successful Registration Init

```bash
curl -X POST http://localhost:8080/api/auth/registration/init \
  -H "Content-Type: application/json" \
  -d '{"email": "test_user_$(date +%s@example.com)"}'
```

**Verify:**
- Response contains `status: "ok"`
- Response contains `message` field
- Response contains `session_id` (UUID format)
- Response contains `resend_available_at` (ISO8601 timestamp)

**Report:** Log request, response, and PASS/FAIL status.

---

#### 5.2 POST /api/auth/registration/init — Invalid Email Format

```bash
curl -X POST http://localhost:8080/api/auth/registration/init \
  -H "Content-Type: application/json" \
  -d '{"email": "invalid-email"}'
```

**Verify:**
- Response returns error status (HTTP 4xx)
- Response contains error message

**Report:** Log request, response, and PASS/FAIL status.

---

#### 5.3 POST /api/auth/registration/init — Empty Email

```bash
curl -X POST http://localhost:8080/api/auth/registration/init \
  -H "Content-Type: application/json" \
  -d '{"email": ""}'
```

**Verify:**
- Response returns error status (HTTP 4xx)

**Report:** Log request, response, and PASS/FAIL status.

---

#### 5.4 Verify Email Sent via MailHog API

```bash
curl http://localhost:8025/api/v2/messages
```

**Verify:**
- At least one email message exists
- Email contains 6-digit verification code
- Email recipient matches the test email

**Report:** Log MailHog API response, extracted email content, and PASS/FAIL status.

---

### Step 6: Stop Backend Application

```bash
pkill -f "cargo run -p backend" || true
```

**Report:** Confirm process terminated.

---

### Step 7: Stop Docker Services

```bash
docker-compose -f docker/test/docker-compose.yml down
```

**Report:** Include docker-compose output.

---

## Report Format

Write the full report to `/project/.ralph-wiggum/reports/03-manual-testing-registration-init-backend.md` with the following structure:

```markdown
# Manual Testing Report: Backend Registration Init

## Build Results
- cargo check: PASS/FAIL
- cargo build: PASS/FAIL
- [Full output]

## Test Results
- Tests passed: X
- Tests failed: X
- Tests ignored: X
- [Full output]

## Docker Services
- MinIO: STARTED/FAILED
- MailHog: STARTED/FAILED
- [Full output]

## Backend Server
- Startup: SUCCESS/FAILED
- Listening on port 8080: YES/NO
- [Full output]

## Acceptance Criteria Verification

### Backend Criteria

1. POST /api/auth/registration/init — accepts email, sends verification email via SMTP, returns session_id and resend_available_at
   - Request: [curl command]
   - Response: [JSON response]
   - Status: PASS/FAIL

2. Integration tests cover successful registration flow
   - Tests executed: [list]
   - Status: PASS/FAIL

3. cargo test — all tests pass, zero failures
   - Passed: X, Failed: X
   - Status: PASS/FAIL

4. Backend starts with config file, serves HTTP on configured port
   - Server started: YES/NO
   - HTTP endpoint accessible: YES/NO
   - Status: PASS/FAIL

5. docker/local/docker-compose.yml includes backend, frontend, MinIO, and MailHog services
   - Services verified: [list]
   - Status: PASS/FAIL

## Summary
- Overall: PASS/FAIL
- Issues found: [list any issues]
```

## Deliverables

1. Report at `/project/.ralph-wiggum/reports/03-manual-testing-registration-init-backend.md` with all verification results
2. Clear PASS/FAIL status for each acceptance criterion based on actual output