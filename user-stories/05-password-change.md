# User Story: Password Change

## Prerequisites
- S3Provider, LocalFileSystemProvider, SQLiteProvider, SMTPProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data
- User model, password hashing, and session/token handling implemented (from registration)
- Login endpoint implemented (user must be authenticated)
- AuthenticatedUser extractor implemented (see @specs/AUTH-MIDDLEWARE.md)

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
