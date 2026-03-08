# Task: Manual Testing for Frontend Login Implementation

## Overview
Execute manual verification of all acceptance criteria from user story 01-login.md for the frontend login implementation. This is a VERIFICATION SCRIPT that collects evidence - it does NOT fix anything.

## User Story
- [01-login.md](../user-stories/01-login.md)

## Related Tasks
- Implementation: [02-task-login-frontend.md](./02-task-login-frontend.md)
- Integration Tests: [02-test-login-frontend.md](./02-test-login-frontend.md)

## Related Reports
- Implementation Report: /project/.ralph-wiggum/reports/02-task-login-frontend.md
- Test Report: /project/.ralph-wiggum/reports/02-test-login-frontend.md

## Spec Files to Reference
- [FRONTEND.md](../specs/FRONTEND.md) — Frontend architecture
- [GENERIC-FRONTEND.md](../specs/GENERIC-FRONTEND.md) — Component patterns
- [DESIGN.md](../specs/DESIGN.md) — Design system

## Manual Testing Procedure

### Step 1: Build the Project
Execute the following commands and log the FULL output into the report:

```bash
# Backend build
cd /project
cargo check -p backend 2>&1
cargo build -p backend 2>&1

# Frontend build
cargo check -p frontend 2>&1
cargo build -p frontend 2>&1
```

**Log:** Full output of all 4 commands, including any warnings or errors.

### Step 2: Run Backend Tests
Execute and log results:

```bash
cargo test -p backend 2>&1
```

**Log:** Test results showing passed/failed/ignored counts.

### Step 3: Start Backend Service
Start the backend server in the background:

```bash
# Start backend with test config
cd /project
RUST_LOG=debug cargo run -p backend -- --config docker/test/config.toml &
BACKEND_PID=$!
echo "Backend PID: $BACKEND_PID"

# Wait for backend to be ready
sleep 5
```

**Log:** Startup output and PID.

### Step 4: Test Backend Login Endpoint with curl
Execute curl requests and log both the request and response:

```bash
# Test 1: Successful login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"TestPass123"}' \
  -v 2>&1

# Test 2: Wrong password
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"wrongpassword"}' \
  -v 2>&1

# Test 3: Non-existent email
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"nonexistent@example.com","password":"TestPass123"}' \
  -v 2>&1

# Test 4: Missing fields
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com"}' \
  -v 2>&1
```

**Log:** Full verbose output of each curl request including headers, body, and response.

### Step 5: Start Frontend Service
Build and serve the frontend:

```bash
cd /project/frontend
trunk serve --port 3000 --no-open &
FRONTEND_PID=$!
echo "Frontend PID: $FRONTEND_PID"

# Wait for frontend to be ready
sleep 10
```

**Log:** Startup output and PID.

### Step 6: Verify Frontend Login Page with Headless Chrome
Use headless chromium to render the login page and capture DOM:

```bash
# Render login page
chromium --headless --dump-dom http://localhost:3000/login > /tmp/login_page_dom.html 2>&1
cat /tmp/login_page_dom.html

# Check if chromium is available, if not note it in the report
which chromium || which google-chrome || echo "Chromium not available"
```

**Log:** Full DOM output or error message if chromium is not available.

### Step 7: Verify Protected Route Redirect
```bash
# Render home page (should redirect to /login if not authenticated)
chromium --headless --dump-dom http://localhost:3000/ > /tmp/home_page_dom.html 2>&1
cat /tmp/home_page_dom.html
```

**Log:** Full DOM output showing redirect behavior.

### Step 8: Stop Services
```bash
kill $BACKEND_PID 2>/dev/null
kill $FRONTEND_PID 2>/dev/null
echo "Services stopped"
```

**Log:** Confirmation of service termination.

## Acceptance Criteria Verification

### Backend Criteria
For each criterion, write PASS or FAIL based on actual output:

| Criterion | Expected | Actual Result | PASS/FAIL |
|-----------|----------|---------------|-----------|
| `cargo test -p backend` — all tests pass | Zero failures | [LOG OUTPUT] | |
| Backend starts with config file | Server running on configured port | [LOG OUTPUT] | |
| `POST /api/auth/login` accepts email/password | Returns user, access_token, refresh_token on success | [LOG OUTPUT] | |
| Wrong credentials return error | `{"status": "error", "message": "Invalid email or password"}` | [LOG OUTPUT] | |
| `docker/local/docker-compose.yml` includes all services | backend, frontend, MinIO, MailHog present | [LOG OUTPUT] | |

### Frontend Criteria
For each criterion, write PASS or FAIL based on actual output:

| Criterion | Expected | Actual Result | PASS/FAIL |
|-----------|----------|---------------|-----------|
| `LoginPage` exists at `/login` route | DOM contains login page elements | [LOG OUTPUT] | |
| `LoginForm` has email and password fields | DOM contains input fields with labels | [LOG OUTPUT] | |
| Button disabled until both fields filled | Check button disabled state in empty form | [LOG OUTPUT] | |
| Server error displayed inline on password field | Error message appears below password input | [LOG OUTPUT] | |
| "Forgot password?" link to `/password/restore` | DOM contains link with href="/password/restore" | [LOG OUTPUT] | |
| "Don't have an account? Register" link to `/register` | DOM contains link with href="/register" | [LOG OUTPUT] | |
| `auth_service` module implements `login` | Source code contains async login function | [SOURCE REVIEW] | |
| `auth_state` stores tokens in localStorage | Source code uses localStorage for tokens | [SOURCE REVIEW] | |
| "/" redirects to "/login" when not authenticated | DOM shows redirect or login page | [LOG OUTPUT] | |

## Penpot Design Verification

Compare the rendered DOM against Penpot design frames:

**Design frames to verify:**
- [Login - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=b950d130-c95a-8005-8007-ad3c0ebbb6de&frame-id=82852407-cb8b-809b-8007-ad6962f86950)
- [Login - Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=b950d130-c95a-8005-8007-ad3c0ebbb6de&frame-id=82852407-cb8b-809b-8007-ad6ed7fd51a7)
- [Login - Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=b950d130-c95a-8005-8007-ad3c0ebbb6de&frame-id=82852407-cb8b-809b-8007-ad6c1156d750)

**Check:**
- Component structure matches design (AuthenticationInput, AuthenticationButton, hjkl-chat logo)
- Labels and placeholders match
- Link text matches ("Forgot password?", "Don't have an account? Register")

## Report Format

Write the full manual testing report to `/project/.ralph-wiggum/reports/02-manual-testing-login-frontend.md` with:

1. **Build Output Section** — Full output from all build commands
2. **Test Results Section** — cargo test output
3. **Backend HTTP Testing Section** — All curl requests and responses
4. **Frontend DOM Section** — Chromium dump output or availability note
5. **Acceptance Criteria Table** — PASS/FAIL for each criterion with evidence
6. **Design Verification Section** — Comparison with Penpot frames
7. **Summary** — Overall result and any issues found

## Notes

- This task is a VERIFICATION SCRIPT only — do NOT fix any issues found
- If issues are found, document them clearly for a follow-up fix task
- Chromium may not be available in the container — note this in the report if so
- If backend requires MinIO to start, document this dependency and any workarounds used