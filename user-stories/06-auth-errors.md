# User Story: Authentication Error Handling

## Flow
```
Any Form → Error Display
```

## Design

**Error display components:**
- [Input / Error](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a593a1713d45) — input with validation error (red border, error message below)
- [Button / Disabled](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592241c003a) — inactive button shown when form is invalid

**Error state frames:**
- [1b - Create Account (Invalid)](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a5927d756d96) — invalid email, client-side validation error
- [3b - Set Password (Mismatch)](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a592d4156e1e) — password confirm mismatch, client-side validation error
- [4b - Login (Invalid)](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59cf461d765) — wrong credentials, server-side error
- [7b - New Password (Mismatch)](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d1aae4178) — password confirm mismatch, client-side validation error
- [8b - Change Password (Error)](https://penpot.hjkl.pro/view/742d722a-06ca-817e-8007-a42f6283e7ed?page-id=988fdbaf-c8f8-808f-8007-a55ba615f576&frame-id=988fdbaf-c8f8-808f-8007-a59d291d4393) — wrong current password, server-side error

---

## Error Display Pattern

All errors are displayed **inline** on the relevant input field using the `Input / Error` component:
- The input border changes to red (`border/error`)
- An error message appears below the input in red (`text/error`)
- The submit button remains disabled (`Button / Disabled`) while errors are present

There are **no toast notifications or modal alerts** — all feedback is inline within the form.

---

## Error Types

### 1. Client-Side Validation Errors

Shown immediately as the user types, before any API call.

| Form | Field | Condition | Error message |
|------|-------|-----------|---------------|
| `RegistrationForm` | email | Invalid format | "Invalid email format" |
| `CompleteRegistrationForm` | password_confirm | Doesn't match password | "Passwords don't match" |
| `PasswordRestoreCompleteForm` | password_confirm | Doesn't match password | "Passwords don't match" |
| `PasswordChangeForm` | new_password_confirm | Doesn't match new password | "Passwords don't match" |

**Response format:**
```json
{
  "status": "error",
  "errors": [
    { "field": "email", "message": "Invalid email format" },
    { "field": "password", "message": "Password must contain at least 8 characters" }
  ]
}
```

---

### 2. Wrong Credentials

**Form:** `LoginForm`
**API:** `login`
**Display:** Error shown inline on the **password** field using `Input / Error`

**Response:**
```json
{
  "status": "error",
  "message": "Invalid email or password"
}
```

---

### 3. Wrong Current Password

**Form:** `PasswordChangeForm`
**API:** `password-change`
**Display:** Error shown inline on the **current password** field using `Input / Error`

**Response:**
```json
{
  "status": "error",
  "message": "Current password is incorrect"
}
```

---

### 4. Expired/Invalid Session

**API:** `registration-verify`, `registration-complete`, `password-restore-verify`, `password-restore-complete`
**Response:**
```json
{
  "status": "error",
  "message": "Session expired or invalid"
}
```

---

### 5. Rate Limiting

**API:** Any auth endpoint
**Response:**
```json
{
  "status": "error",
  "message": "Too many requests. Please try again later",
  "retry_after": 60
}
```
