# User Stories: Authentication

## 1. Registration Using Email

### Flow Overview
```
┌──────────────────┐      ┌──────────────────┐      ┌──────────────────────┐
│  Registration   │      │      Email        │      │ Continue Registration │
│     Form        │──────│     Verification  │──────│     with Password    │
└──────────────────┘      └──────────────────┘      └──────────────────────┘
```

### Step 1: Registration Form

**Form Name:** `RegistrationForm`

**Description:** Initial registration form where user enters email address only.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| email | string | yes | valid email format |

**API Name:** `registration-init`

**API Endpoint:** `POST /api/auth/registration/init`

**Request:**
```json
{
  "email": "user@example.com"
}
```

**Response:**
```json
{
  "status": "ok",
  "message": "Verification email sent",
  "session_id": "uuid"
}
```

---

### Step 2: Email Verification

**Form Name:** `EmailVerificationForm`

**Description:** User enters the verification code received via email.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| code | string | yes | 6-digit code |

**API Name:** `registration-verify`

**API Endpoint:** `POST /api/auth/registration/verify`

**Request:**
```json
{
  "session_id": "uuid",
  "code": "123456"
}
```

**Response:**
```json
{
  "status": "ok",
  "message": "Email verified",
  "session_id": "uuid",
  "expires_at": "ISO8601 timestamp"
}
```

---

### Step 3: Continue Registration with Password

**Form Name:** `CompleteRegistrationForm`

**Description:** User sets their password after email verification.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| password | string | yes | min 8 chars, mixed case, number |
| password_confirm | string | yes | must match password |

**API Name:** `registration-complete`

**API Endpoint:** `POST /api/auth/registration/complete`

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
  "message": "Registration completed",
  "user": {
    "id": "uuid",
    "email": "user@example.com"
  },
  "access_token": "jwt-token",
  "refresh_token": "jwt-token"
}
```

---

## 2. Login

### Flow Overview
```
┌──────────────────┐
│      Login       │
│      Form        │
└──────────────────┘
```

**Form Name:** `LoginForm`

**Description:** User authenticates with email and password.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| email | string | yes | valid email format |
| password | string | yes | min 1 char |

**API Name:** `login`

**API Endpoint:** `POST /api/auth/login`

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
  "message": "Login successful",
  "user": {
    "id": "uuid",
    "email": "user@example.com"
  },
  "access_token": "jwt-token",
  "refresh_token": "jwt-token"
}
```

**Response (Failure):**
```json
{
  "status": "error",
  "message": "Invalid credentials"
}
```

---

## 3. Logout

### Flow Overview
```
┌──────────────────┐
│      Logout      │
│      Form        │
└──────────────────┘
```

**Form Name:** `LogoutForm`

**Description:** User logs out of the session.

**Fields:** None (uses session from auth token)

**API Name:** `logout`

**API Endpoint:** `POST /api/auth/logout`

**Request:**
```json
{
  "refresh_token": "jwt-token"
}
```

**Response:**
```json
{
  "status": "ok",
  "message": "Logged out successfully"
}
```

---

## 4. Password Restore

### Flow Overview
```
┌──────────────────┐      ┌──────────────────┐      ┌──────────────────────┐
│   Request Password│      │      Email        │      │   Set New Password   │
│     Restore       │──────│     Verification  │──────│                      │
└──────────────────┘      └──────────────────┘      └──────────────────────┘
```

### Step 1: Request Password Restore

**Form Name:** `PasswordRestoreRequestForm`

**Description:** User requests password reset by entering their email.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| email | string | yes | valid email format |

**API Name:** `password-restore-init`

**API Endpoint:** `POST /api/auth/password/restore/init`

**Request:**
```json
{
  "email": "user@example.com"
}
```

**Response:**
```json
{
  "status": "ok",
  "message": "Password reset email sent"
}
```

---

### Step 2: Verify Reset Code

**Form Name:** `PasswordRestoreVerifyForm`

**Description:** User enters the verification code received via email.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| code | string | yes | 6-digit code |

**API Name:** `password-restore-verify`

**API Endpoint:** `POST /api/auth/password/restore/verify`

**Request:**
```json
{
  "email": "user@example.com",
  "code": "123456"
}
```

**Response:**
```json
{
  "status": "ok",
  "message": "Code verified",
  "session_id": "uuid"
}
```

---

### Step 3: Set New Password

**Form Name:** `PasswordRestoreCompleteForm`

**Description:** User sets a new password after verification.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| session_id | string | yes | valid session uuid |
| password | string | yes | min 8 chars, mixed case, number |
| password_confirm | string | yes | must match password |

**API Name:** `password-restore-complete`

**API Endpoint:** `POST /api/auth/password/restore/complete`

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

---

## 5. Password Change

### Flow Overview
```
┌──────────────────┐
│   Change Password│
│      Form        │
└──────────────────┘
```

**Form Name:** `PasswordChangeForm`

**Description:** Authenticated user changes their password.

**Fields:**
| Field | Type | Required | Validation |
|-------|------|----------|------------|
| current_password | string | yes | must match existing password |
| new_password | string | yes | min 8 chars, mixed case, number |
| new_password_confirm | string | yes | must match new_password |

**API Name:** `password-change`

**API Endpoint:** `POST /api/auth/password/change`

**Headers:**
```
Authorization: Bearer <access_token>
```

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

**Response (Failure):**
```json
{
  "status": "error",
  "message": "Current password is incorrect"
}
```

---

## API Endpoints Summary

| # | API Name | Endpoint | Method | Auth Required |
|---|----------|----------|--------|---------------|
| 1 | registration-init | `/api/auth/registration/init` | POST | No |
| 2 | registration-verify | `/api/auth/registration/verify` | POST | No |
| 3 | registration-complete | `/api/auth/registration/complete` | POST | No |
| 4 | login | `/api/auth/login` | POST | No |
| 5 | logout | `/api/auth/logout` | POST | Yes |
| 6 | password-restore-init | `/api/auth/password/restore/init` | POST | No |
| 7 | password-restore-verify | `/api/auth/password/restore/verify` | POST | No |
| 8 | password-restore-complete | `/api/auth/password/restore/complete` | POST | No |
| 9 | password-change | `/api/auth/password/change` | POST | Yes |
