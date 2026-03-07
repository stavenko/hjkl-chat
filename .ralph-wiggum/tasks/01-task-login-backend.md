# Task: Implement Login Backend Endpoint

## Overview
Implement the `POST /api/auth/login` endpoint for user authentication.

## User Story
- @user-stories/01-login.md

## Spec Files
- @specs/BACKEND.md — Provider pattern, SQLiteProvider, application wiring
- @specs/GENERIC-BACKEND.md — Provider pattern, use-case structure, endpoint conventions
- @specs/RUST-COMMON-SPEC.md — Module conventions, error handling patterns
- @specs/PROJECT-STRUCTURE.md — Project layout

## Acceptance Criteria
1. Create `backend/src/providers/sqlite.rs` — SQLiteProvider with methods to:
   - Query users by email
   - Query sessions by token
   - Use connection pooling (r2d2 + rusqlite)

2. Create `backend/src/providers/s3.rs` — S3Provider stub for file storage (use aws-sdk-s3)

3. Create `backend/src/providers/local_filesystem.rs` — LocalFileSystemProvider stub for local file storage

4. Create `backend/src/providers/smtp.rs` — SMTPProvider stub for email sending (use lettre)

5. Create `backend/src/models/user.rs` — User model with:
   - Fields: id (Uuid), email (String), password_hash (String), created_at (DateTime)
   - Implement From/Into for database row conversion

6. Create `backend/src/models/session.rs` — Session model with:
   - Fields: id (Uuid), user_id (Uuid), access_token (String), refresh_token (String), expires_at (DateTime), created_at (DateTime)
   - Implement From/Into for database row conversion

7. Create `backend/src/use_cases/auth.rs` — Auth use-case module with:
   - `login(email: &str, password: &str) -> Result<LoginResponse, AuthError>`
   - Validate email exists in SQLite
   - Verify password using bcrypt
   - Generate JWT access_token and refresh_token
   - Create and persist session record
   - Return user info (id, email) and tokens

8. Create `backend/src/endpoints/auth.rs` — Auth endpoints module with:
   - `POST /api/auth/login` — accepts LoginRequest { email, password }, returns LoginResponse
   - Wrong credentials return `{"status": "error", "message": "Invalid email or password"}` with HTTP 401
   - Wire up use-case and providers

9. Create `backend/src/models/auth.rs` — Auth request/response types:
   - LoginRequest { email: String, password: String }
   - LoginResponse { status: String, user: UserInfo, access_token: String, refresh_token: String }
   - Error response { status: String, message: String }

10. Update `backend/src/main.rs`:
    - Initialize all providers (SQLite, S3, LocalFileSystem, SMTP)
    - Register providers as Actix-web app data
    - Configure auth routes
    - Start server on configured port

11. Update `backend/Cargo.toml`:
    - Add dependencies: rusqlite, r2d2, r2d2_sqlite, bcrypt, jsonwebtoken, uuid, chrono, aws-sdk-s3, lettre

12. Create database migration in `backend/migrations/`:
    - `001_create_users.sql` — CREATE TABLE users (id UUID PRIMARY KEY, email TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, created_at TIMESTAMP NOT NULL)
    - `002_create_sessions.sql` — CREATE TABLE sessions (id UUID PRIMARY KEY, user_id UUID REFERENCES users(id), access_token TEXT NOT NULL, refresh_token TEXT NOT NULL, expires_at TIMESTAMP NOT NULL, created_at TIMESTAMP NOT NULL)

13. Add integration tests in `backend/src/tests/auth_tests.rs`:
    - Test successful login with valid credentials
    - Test wrong password returns error
    - Test non-existent email returns error
    - Test missing fields returns error

## Deliverables
- All files created as specified above
- `cargo check --workspace` succeeds
- `cargo build -p backend` succeeds
- `cargo test -p backend` passes with all new auth tests

## Notes
- Follow the provider pattern from GENERIC-BACKEND.md
- Use connection pooling for SQLite
- Store password hashes using bcrypt
- Generate secure JWT tokens with appropriate expiration
- All configuration from config file or environment variables