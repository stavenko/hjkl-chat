# Task: Manual Testing for Login Backend

## Overview
Execute manual verification of all acceptance criteria from user story 01-login.md. This is a VERIFICATION SCRIPT that collects evidence - it does NOT fix anything.

## User Story
- @user-stories/01-login.md

## Related Tasks
- Implementation: @.ralph-wiggum/tasks/01-task-login-backend.md
- Tests: @.ralph-wiggum/tasks/01-test-login-backend.md

## Spec Files
- @specs/BACKEND.md — Provider pattern, application wiring
- @specs/GENERIC-BACKEND.md — Provider pattern, use-case structure, endpoint conventions
- @specs/RUST-COMMON-SPEC.md — Module conventions, error handling patterns

## Acceptance Criteria to Verify

### Backend Criteria
1. `POST /api/auth/login` — accepts `email` and `password`, validates credentials against SQLite, returns `user`, `access_token`, `refresh_token` on success
2. Wrong credentials return `{"status": "error", "message": "Invalid email or password"}` with appropriate HTTP status
3. Integration tests cover: successful login, wrong password, non-existent email, missing fields
4. `cargo test` — all tests pass, zero failures
5. Backend starts with config file, serves HTTP on configured port
6. `docker/local/docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend Criteria
1. `LoginPage` exists at route `/login`
2. `LoginForm` — email and password fields with `TextInput`, `Button` disabled until both fields are filled, calls `auth_service::login`
3. Server error (wrong credentials) displayed inline on the password field via `Input / Error` component
4. "Forgot password?" link navigates to `/password/restore`
5. "Don't have an account? Register" link navigates to `/register`
6. `auth_service` module implements `login` async function
7. There is a method in authentication service which checks for authentication
8. On successful login, tokens are stored in `AuthState` and `localStorage`, user is navigated to home
9. Frontend unit tests pass — form validation, error display on failed login, service function mocking
10. "/" path requires authentication and automatically redirects to "/login"

## Execution Steps

### Step 1: Build the Project
```bash
cd /project
cargo check --workspace
cargo build -p backend
cargo build -p frontend
```
**Log the complete build output into the report.**

### Step 2: Run Backend Tests
```bash
cd /project
cargo test -p backend
```
**Log the complete test output into the report. Verify zero failures.**

### Step 3: Start Backend Server
```bash
cd /project
cargo run -p backend -- --config backend/config.toml
```
**Log the startup output. Wait for "Listening on" message.**

### Step 4: Test Backend Endpoints with curl

#### Test 4.1: Successful Login
```bash
# First create a test user (you may need to use a registration endpoint or direct SQL)
# Then test login:
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "SecurePass123"}'
```
**Log the complete curl request and response into the report. Mark PASS if returns user, access_token, refresh_token.**

#### Test 4.2: Wrong Password
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "WrongPassword"}'
```
**Log the complete curl request and response into the report. Mark PASS if returns {"status": "error", "message": "Invalid email or password"}.**

#### Test 4.3: Non-existent Email
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "nonexistent@example.com", "password": "SecurePass123"}'
```
**Log the complete curl request and response into the report.**

#### Test 4.4: Missing Fields
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": ""}'
```
**Log the complete curl request and response into the report.**

### Step 5: Start Frontend Server
```bash
cd /project/frontend
trunk serve --public-url /
```
**Log the startup output. Wait for "Trunk ready" message.**

### Step 6: Test Frontend with Headless Chrome

#### Test 6.1: Verify LoginPage Exists at /login
```bash
chromium --headless --dump-dom http://localhost:8080/login > /tmp/login_page_dom.html
head -100 /tmp/login_page_dom.html
```
**Log the first 100 lines of DOM output. Mark PASS if LoginPage components are present.**

#### Test 6.2: Verify LoginForm Has Email and Password Fields
```bash
chromium --headless --dump-dom http://localhost:8080/login | grep -i "email\|password" | head -20
```
**Log output. Mark PASS if email and password input fields are present.**

#### Test 6.3: Verify Button is Disabled Initially
```bash
chromium --headless --dump-dom http://localhost:8080/login | grep -i "button\|disabled" | head -20
```
**Log output. Mark PASS if submit button has disabled attribute when fields are empty.**

#### Test 6.4: Verify "Forgot Password" Link Exists
```bash
chromium --headless --dump-dom http://localhost:8080/login | grep -i "forgot\|password.*restore" | head -10
```
**Log output. Mark PASS if link to /password/restore is present.**

#### Test 6.5: Verify "Register" Link Exists
```bash
chromium --headless --dump-dom http://localhost:8080/login | grep -i "register\|account" | head -10
```
**Log output. Mark PASS if link to /register is present.**

### Step 7: Stop Servers
```bash
# Stop backend (Ctrl+C or pkill)
# Stop frontend (Ctrl+C or pkill)
```

## Report Format

Write the complete report to `/project/.ralph-wiggum/reports/01-manual-testing-login-backend.md` with the following structure:

```markdown
# Manual Testing Report: 01-login-backend

## Build Output
### cargo check --workspace
[full output]

### cargo build -p backend
[full output]

### cargo build -p frontend
[full output]

## Test Results
### cargo test -p backend
[full output]

## Backend Acceptance Criteria

### Criterion 1: POST /api/auth/login returns user, access_token, refresh_token
**Status:** PASS/FAIL
**Evidence:**
[full curl request and response]

### Criterion 2: Wrong credentials return error
**Status:** PASS/FAIL
**Evidence:**
[full curl request and response]

... [continue for all backend criteria]

## Frontend Acceptance Criteria

### Criterion 1: LoginPage exists at /login
**Status:** PASS/FAIL
**Evidence:**
[headless chrome output showing page exists]

### Criterion 2: LoginForm has email and password fields
**Status:** PASS/FAIL
**Evidence:**
[headless chrome output showing form fields]

... [continue for all frontend criteria]

## Summary
- Backend criteria: X/Y passed
- Frontend criteria: X/Y passed
- Overall: PASS/FAIL
```

## Notes
- If chromium is not available, note this in the report and skip frontend DOM tests
- If the backend requires a test user to be created first, document the steps taken
- Include timestamps for all tests
- Be thorough in logging actual output - this is the evidence for verification