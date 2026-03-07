# User Story: Registration Using Email

## Prerequisites
- S3Provider implemented (see @specs/BACKEND.md)
- LocalFileSystemProvider implemented (see @specs/BACKEND.md)
- SQLiteProvider implemented (see @specs/BACKEND.md)
- SMTPProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data

## Frontend Prerequisites
- Frontend project bootstrapped with Leptos CSR and trunk (see @specs/FRONTEND.md)
- Frontend routing implemented in app.rs (see @specs/GENERIC-FRONTEND.md)
- Reusable form components implemented: TextInput, Button, PasswordStrength (see @specs/GENERIC-FRONTEND.md)
- AuthState implemented for token storage (see @specs/GENERIC-FRONTEND.md)
- auth_service module created with API base URL configuration (see @specs/GENERIC-FRONTEND.md)

## Flow
```
RegistrationForm → Email Verification → CompleteRegistrationForm
```

## Design

[View all frames in Penpot](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a56ee6c3bd00)

**Components used:**
- [Input / Text](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5938ac9c2bc) — empty input with placeholder
- [Input / Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5939900ce33) — input with entered value
- [Input / Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a593a1713d45) — input with validation error
- [Button / Primary](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a58f0ea4eb3e) — active submit button
- [Button / Disabled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592241c003a) — inactive button (form incomplete)
- [Password / Strength](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5923f3b9e1f) — password strength indicator

---

## Step 1: Registration Form

**Form:** `RegistrationForm`
**API:** `registration-init`
**Endpoint:** `POST /api/auth/registration/init`

**Frames:**
- [1a - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a56ee6c3bd00) — initial state, email field empty, button disabled
- [1b - Invalid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5927d756d96) — invalid email entered, inline error shown, button disabled
- [1c - Valid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592870c18b8) — valid email entered, button active

**Validation:**
- Email is validated on input (client-side) using `Input / Error` component
- Button remains disabled (`Button / Disabled`) until email passes validation
- Button becomes active (`Button / Primary`) only when email is valid

**Request:**
```json
{ "email": "user@example.com" }
```

**Response:**
```json
{
  "status": "ok",
  "message": "Verification email sent",
  "session_id": "uuid",
  "resend_available_at": "ISO8601"
}
```

> **Note:** The `resend_available_at` field is an ISO8601 timestamp indicating when the user can request a new code. The client calculates the countdown from this server timestamp, not from a local timer.

---

## Step 2: Email Verification

**Form:** `EmailVerificationForm`
**API:** `registration-verify`
**Endpoint:** `POST /api/auth/registration/verify`

**Frames:**
- [2a - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a56ef4d59447) — code field empty, button disabled, countdown timer shown ("Resend code in 59s")
- [2b - Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592a3e163b8) — code entered, button active, countdown still visible
- [2c - Resend](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592afa31e65) — countdown expired, "Resend" link becomes active

**Resend Code Timeout:**
- After sending a verification email, the server returns `resend_available_at` timestamp
- Client displays a countdown: "Resend code in Xs" (greyed out, not clickable)
- When countdown reaches 0, the text changes to "Didn't receive a code? **Resend**" (clickable link)
- Clicking "Resend" calls `POST /api/auth/registration/init` again and resets the timer

**Request:**
```json
{ "session_id": "uuid", "code": "123456" }
```

**Response:**
```json
{
  "status": "ok",
  "session_id": "uuid",
  "expires_at": "ISO8601"
}
```

---

## Step 3: Complete Registration

**Form:** `CompleteRegistrationForm`
**API:** `registration-complete`
**Endpoint:** `POST /api/auth/registration/complete`

**Frames:**
- [3a - Filling](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a56f0d32cefb) — password entered, strength indicator shows "Medium", confirm empty, button disabled
- [3b - Mismatch](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592d4156e1e) — strong password, confirm doesn't match, inline error, button disabled
- [3c - Valid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592e0f1a196) — both fields match, strong password, button active

**Validation:**
- Password strength is shown in real-time using `Password / Strength` component (weak/medium/strong)
- Confirm password is validated on input; mismatch shows inline error via `Input / Error`
- Button remains disabled until password is strong enough AND both fields match

**Request:**
```json
{
  "session_id": "uuid",
  "password": "SecurePass123",
  "password_confirm": "SecurePass123"
}
```

**Response:**
```json
{
  "status": "ok",
  "user": { "id": "uuid", "email": "user@example.com" },
  "access_token": "jwt-token",
  "refresh_token": "jwt-token"
}
```

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/auth/registration/init` — accepts email, sends verification email via SMTP, returns `session_id` and `resend_available_at`
- [ ] `POST /api/auth/registration/verify` — accepts `session_id` and `code`, validates the code, returns `session_id` and `expires_at`
- [ ] `POST /api/auth/registration/complete` — accepts `session_id`, `password`, `password_confirm`, creates user in SQLite, returns `user`, `access_token`, `refresh_token`
- [ ] Integration tests cover: successful registration flow end-to-end, invalid email format rejected, wrong verification code rejected, expired session rejected, password mismatch rejected
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend
- [ ] `RegistrationPage` exists at route `/register` with three-step flow (init → verify → complete)
- [ ] `RegistrationForm` component — email field with `TextInput`, submit `Button` disabled until email is valid, calls `auth_service::registration_init`
- [ ] `EmailVerificationForm` component — code field with `TextInput`, countdown timer from `resend_available_at`, resend link active after countdown, calls `auth_service::registration_verify`
- [ ] `CompleteRegistrationForm` component — password and confirm fields with `TextInput`, `PasswordStrength` indicator, `Button` disabled until passwords match and strength is sufficient, calls `auth_service::registration_complete`
- [ ] `auth_service` module implements `registration_init`, `registration_verify`, `registration_complete` async functions
- [ ] On successful completion, tokens are stored in `AuthState` and `localStorage`, user is navigated away from registration
- [ ] Client-side validation: invalid email shows inline error via `Input / Error`, password mismatch shows inline error
- [ ] Frontend unit tests pass — component rendering, validation logic, service function mocking
- [ ] Component design matches penpot. Appropriate components used, colors
match, layout matches.

