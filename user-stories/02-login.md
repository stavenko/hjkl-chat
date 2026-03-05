# User Story: Login

## Prerequisites
- S3Provider implemented (see @specs/BACKEND.md)
- LocalFileSystemProvider implemented (see @specs/BACKEND.md)
- SQLiteProvider implemented (see @specs/BACKEND.md)
- SMTPProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data
- User model and password hashing implemented (from registration)
- Session model implemented (from registration)

## Frontend Prerequisites
- Frontend project bootstrapped with Leptos CSR and trunk (see @specs/FRONTEND.md)
- Frontend routing implemented in app.rs (see @specs/GENERIC-FRONTEND.md)
- Reusable form components implemented: TextInput, Button (see @specs/GENERIC-FRONTEND.md)
- AuthState implemented for token storage (see @specs/GENERIC-FRONTEND.md)
- auth_service module created with API base URL configuration (see @specs/GENERIC-FRONTEND.md)

## Flow
```
LoginForm
```

## Design

[View all frames in Penpot](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cf42ea713)

**Components used:**
- [Input / Text](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5938ac9c2bc) — empty input with placeholder
- [Input / Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5939900ce33) — input with entered value
- [Input / Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a593a1713d45) — input with validation error
- [Button / Primary](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a58f0ea4eb3e) — active submit button
- [Button / Disabled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592241c003a) — inactive button

---

**Form:** `LoginForm`
**API:** `login`
**Endpoint:** `POST /api/auth/login`

**Frames:**
- [4a - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cf42ea713) — both fields empty, button disabled
- [4b - Invalid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cf461d765) — server returns error, password field shows inline error "Invalid email or password", button disabled
- [4c - Valid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cf495e000) — both fields filled, button active

**Validation:**
- Button remains disabled (`Button / Disabled`) until both email and password are filled
- Button becomes active (`Button / Primary`) when both fields have values
- On server error (wrong credentials), the password field switches to `Input / Error` with the server error message
- "Forgot password?" link navigates to Password Restore flow
- "Don't have an account? Register" link navigates to Registration flow

**Error Display:**
- Server errors (invalid credentials) are shown inline on the password field using the `Input / Error` component
- The error message comes directly from the server response `message` field

**Request:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123"
}
```

**Response (Success):**
```json
{
  "status": "ok",
  "user": { "id": "uuid", "email": "user@example.com" },
  "access_token": "jwt-token",
  "refresh_token": "jwt-token"
}
```

**Response (Error):**
```json
{
  "status": "error",
  "message": "Invalid email or password"
}
```

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/auth/login` — accepts `email` and `password`, validates credentials against SQLite, returns `user`, `access_token`, `refresh_token` on success
- [ ] Wrong credentials return `{"status": "error", "message": "Invalid email or password"}` with appropriate HTTP status
- [ ] Integration tests cover: successful login, wrong password, non-existent email, missing fields
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker-compose.yml` includes backend, frontend, and required dependencies

### Frontend
- [ ] `LoginPage` exists at route `/login`
- [ ] `LoginForm` — email and password fields with `TextInput`, `Button` disabled until both fields are filled, calls `auth_service::login`
- [ ] Server error (wrong credentials) displayed inline on the password field via `Input / Error` component
- [ ] "Forgot password?" link navigates to `/password/restore`
- [ ] "Don't have an account? Register" link navigates to `/register`
- [ ] `auth_service` module implements `login` async function
- [ ] On successful login, tokens are stored in `AuthState` and `localStorage`, user is navigated to home
- [ ] Frontend unit tests pass — form validation, error display on failed login, service function mocking
