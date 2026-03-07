# User Story: Password Change

## Prerequisites
- S3Provider, LocalFileSystemProvider, SQLiteProvider, SMTPProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data
- User model, password hashing, and session/token handling implemented (from registration)
- Login endpoint implemented (user must be authenticated)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

## Frontend Prerequisites
- Frontend project bootstrapped with Leptos CSR and trunk (see @specs/FRONTEND.md)
- Frontend routing implemented in app.rs (see @specs/GENERIC-FRONTEND.md)
- Reusable form components implemented: TextInput, Button, PasswordStrength (see @specs/GENERIC-FRONTEND.md)
- AuthState implemented — user must be logged in to access this page (see @specs/GENERIC-FRONTEND.md)
- auth_service module created with API base URL configuration (see @specs/GENERIC-FRONTEND.md)

## Flow
```
PasswordChangeForm
```

## Design

[View all frames in Penpot](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d28e2d5ba)

**Components used:**
- [Input / Text](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5938ac9c2bc) — empty input with placeholder
- [Input / Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5939900ce33) — input with entered value
- [Input / Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a593a1713d45) — input with validation error
- [Button / Primary](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a58f0ea4eb3e) — active submit button
- [Button / Disabled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592241c003a) — inactive button
- [Password / Strength](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5923f3b9e1f) — password strength indicator

---

**Form:** `PasswordChangeForm`
**API:** `password-change`
**Endpoint:** `POST /api/auth/password/change`
**Auth:** Required (Bearer token)

**Frames:**
- [8a - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d28e2d5ba) — all fields empty, button disabled
- [8b - Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d291d4393) — server returns error on current password, inline error shown, button disabled
- [8c - Valid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d296241be) — all fields filled, passwords match, strong password, button active

**Validation:**
- Password strength is shown in real-time using `Password / Strength` component (weak/medium/strong)
- Confirm password is validated on input; mismatch shows inline error via `Input / Error`
- Button remains disabled until password is strong enough AND both new password fields match
- Current password is validated server-side; error shown inline via `Input / Error` on the current password field

**Error Display:**
- Server errors (wrong current password) are shown inline on the current password field using the `Input / Error` component
- The error message comes directly from the server response `message` field

**Request:**
```json
{
  "current_password": "OldSecurePass123",
  "new_password": "NewSecurePass456",
  "new_password_confirm": "NewSecurePass456"
}
```

**Response (Success):**
```json
{
  "status": "ok",
  "message": "Password changed successfully"
}
```

**Response (Error):**
```json
{
  "status": "error",
  "message": "Current password is incorrect"
}
```

---

## Acceptance Criteria

### Backend
- [ ] `POST /api/auth/password/change` — accepts `current_password`, `new_password`, `new_password_confirm`, validates current password, updates hash in SQLite, returns success
- [ ] Request requires valid `Authorization: Bearer <access_token>` header (uses `AuthenticatedUser` extractor)
- [ ] Wrong current password returns `{"status": "error", "message": "Current password is incorrect"}`
- [ ] Integration tests cover: successful password change, wrong current password, new password mismatch, unauthenticated request rejected
- [ ] `cargo test` — all tests pass, zero failures
- [ ] Backend starts with config file, serves HTTP on configured port
- [ ] `docker/local/docker-compose.yml` includes backend, frontend, MinIO, and MailHog services

### Frontend
- [ ] `PasswordChangePage` exists at route `/password/change`, requires authenticated user (redirects to `/login` if not authenticated)
- [ ] `PasswordChangeForm` — current password, new password, and confirm fields with `TextInput`, `PasswordStrength` indicator on new password, `Button` disabled until new passwords match and strength is sufficient
- [ ] Server error (wrong current password) displayed inline on the current password field via `Input / Error` component
- [ ] `auth_service` module implements `password_change` async function that sends `Authorization: Bearer` header
- [ ] On successful password change, user sees success confirmation
- [ ] Client-side validation: new password mismatch shows inline error via `Input / Error`
- [ ] Frontend unit tests pass — form validation, error display on wrong current password, authenticated request header, service function mocking
