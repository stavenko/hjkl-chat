# User Story: Password Restore

## Prerequisites
- S3Provider, LocalFileSystemProvider, SQLiteProvider, SMTPProvider implemented (see @specs/BACKEND.md)
- All providers wired into application startup and registered as Actix-web app data
- User model and password hashing implemented (from registration)

## Flow
```
PasswordRestoreRequestForm → PasswordRestoreVerifyForm → PasswordRestoreCompleteForm
```

## Design

[View all frames in Penpot](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cfca98234)

**Components used:**
- [Input / Text](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5938ac9c2bc) — empty input with placeholder
- [Input / Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5939900ce33) — input with entered value
- [Input / Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a593a1713d45) — input with validation error
- [Button / Primary](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a58f0ea4eb3e) — active submit button
- [Button / Disabled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592241c003a) — inactive button
- [Password / Strength](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5923f3b9e1f) — password strength indicator

---

## Step 1: Request Password Restore

**Form:** `PasswordRestoreRequestForm`
**API:** `password-restore-init`
**Endpoint:** `POST /api/auth/password/restore/init`

**Frames:**
- [5a - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cfca98234) — email field empty, button disabled, "Back to Sign In" link
- [5b - Valid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cfccd153f) — valid email entered, button active

**Request:**
```json
{ "email": "user@example.com" }
```

**Response:**
```json
{
  "status": "ok",
  "message": "Password reset email sent",
  "resend_available_at": "ISO8601"
}
```

> **Note:** The `resend_available_at` field is an ISO8601 timestamp indicating when the user can request a new code. The client calculates the countdown from this server timestamp, not from a local timer.

---

## Step 2: Verify Reset Code

**Form:** `PasswordRestoreVerifyForm`
**API:** `password-restore-verify`
**Endpoint:** `POST /api/auth/password/restore/verify`

**Frames:**
- [6a - Empty](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d089e3e93) — code field empty, button disabled, countdown timer "Resend code in 59s"
- [6b - Filled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d08c4e4f3) — code entered, button active, countdown still visible
- [6c - Resend](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d08e8c969) — countdown expired, "Resend" link becomes active

**Resend Code Timeout:**
- After sending a reset email, the server returns `resend_available_at` timestamp
- Client displays a countdown: "Resend code in Xs" (greyed out, not clickable)
- When countdown reaches 0, the text changes to "Didn't receive a code? **Resend**" (clickable link)
- Clicking "Resend" calls `POST /api/auth/password/restore/init` again and resets the timer

**Request:**
```json
{ "email": "user@example.com", "code": "123456" }
```

**Response:**
```json
{
  "status": "ok",
  "session_id": "uuid"
}
```

---

## Step 3: Set New Password

**Form:** `PasswordRestoreCompleteForm`
**API:** `password-restore-complete`
**Endpoint:** `POST /api/auth/password/restore/complete`

**Frames:**
- [7a - Filling](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d1a7a5644) — password entered, strength indicator shows "Medium", confirm empty, button disabled
- [7b - Mismatch](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d1aae4178) — strong password, confirm doesn't match, inline error, button disabled
- [7c - Valid](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d1ae0c24d) — both fields match, strong password, button active

**Validation:**
- Password strength is shown in real-time using `Password / Strength` component (weak/medium/strong)
- Confirm password is validated on input; mismatch shows inline error via `Input / Error`
- Button remains disabled until password is strong enough AND both fields match

**Request:**
```json
{
  "session_id": "uuid",
  "password": "NewSecurePass123",
  "password_confirm": "NewSecurePass123"
}
```

**Response:**
```json
{
  "status": "ok",
  "message": "Password restored successfully"
}
```
