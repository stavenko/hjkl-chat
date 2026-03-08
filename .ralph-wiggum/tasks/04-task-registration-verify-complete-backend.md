# Task: Registration Verify and Complete Backend Implementation

## User Story
[02-registration.md](../../user-stories/02-registration.md)

## Specs
- [BACKEND.md](../../specs/BACKEND.md)
- [GENERIC-BACKEND.md](../../specs/GENERIC-BACKEND.md)
- [RUST-COMMON-SPEC.md](../../specs/RUST-COMMON-SPEC.md)
- [PROJECT-STRUCTURE.md](../../specs/PROJECT-STRUCTURE.md)

## Description
Implement the backend endpoints for registration verification and completion steps (Step 2 and Step 3 of the registration flow).

## Acceptance Criteria

### 1. Registration Verify Endpoint
- Create `api/endpoints/auth_registration_verify.rs` with `POST /api/auth/registration/verify` endpoint
- Create `use_cases/registration_verify.rs` with `RegistrationVerifyUseCase`
  - Accept `session_id` and `code` (6-digit verification code)
  - Validate the code against the session stored in SQLite
  - Check session expiry (15 minutes from creation)
  - Return `session_id` and `expires_at` on success
  - Return appropriate errors for: invalid code, expired session, unknown session
- Endpoint must follow the pattern from BACKEND.md (thin wrapper, ApiResponse<T>)

### 2. Registration Complete Endpoint
- Create `api/endpoints/auth_registration_complete.rs` with `POST /api/auth/registration/complete` endpoint
- Create `use_cases/registration_complete.rs` with `RegistrationCompleteUseCase`
  - Accept `session_id`, `password`, `password_confirm`
  - Validate passwords match
  - Validate password strength (must be strong - see password requirements in user story)
  - Create user record in SQLite (id, email, created_at)
  - Store password hash in SQLite (user_id, hash, algorithm)
  - Generate JWT access_token and refresh_token
  - Store tokens in SQLite sessions table
  - Return `user` object, `access_token`, `refresh_token` on success
  - Delete or mark the registration session as used
  - Return appropriate errors for: password mismatch, weak password, expired session, unknown session

### 3. Database Schema
- Add migration for `passwords` table if not exists:
  - `user_id` (UUID, foreign key to users)
  - `hash` (TEXT)
  - `algorithm` (TEXT)
- Ensure `users` table exists with: `id`, `email`, `created_at`
- Ensure `sessions` table exists with: `user_id`, `token`, `expires_at`, `token_type` (access/refresh)

### 4. Wire Endpoints
- Register both endpoints in `api/configurator.rs` under `/api/auth` scope
- Ensure endpoints have access to SQLiteProvider and SMTPProvider via Actix web data

### 5. Error Handling
- Define use-case specific error types following GENERIC-BACKEND.md pattern
- Convert use-case errors to `Error` struct with `code` and `message` fields
- Follow error handling pattern from BACKEND.md

## Technical Requirements
- Follow module structure from GENERIC-BACKEND.md
- All providers accessed via Actix web data (Arc wrapped)
- Use rusqlite for database queries
- Use argon2 for password hashing
- Use jsonwebtoken for JWT generation
- All config values read from config file, no hardcoded values
- Password strength validation: require uppercase, lowercase, digit, min 8 chars

## Files to Create
- `backend/src/api/endpoints/auth_registration_verify.rs`
- `backend/src/api/endpoints/auth_registration_complete.rs`
- `backend/src/use_cases/registration_verify.rs`
- `backend/src/use_cases/registration_complete.rs`
- `backend/src/models/password.rs` (if not exists)
- `backend/migrations/002_passwords.sql` (if needed)

## Verification
- Run `cargo check -p backend` - must compile without errors
- Run `cargo clippy -p backend -- -D warnings` - must pass without warnings
- Run `cargo test -p backend` - all tests must pass