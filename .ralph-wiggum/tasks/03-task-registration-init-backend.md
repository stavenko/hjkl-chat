# Implementation Task: Backend Registration Init Endpoint

**User Story:** 02-registration.md  
**Spec Files:**
- [BACKEND.md](../specs/BACKEND.md) — Provider pattern, API structure
- [GENERIC-BACKEND.md](../specs/GENERIC-BACKEND.md) — Backend architecture patterns
- [RUST-COMMON-SPEC.md](../specs/RUST-COMMON-SPEC.md) — Error handling, module conventions

## Objective

Implement the backend registration initialization endpoint `POST /api/auth/registration/init` that:
1. Accepts an email address
2. Generates a verification code
3. Stores the registration session in SQLite
4. Sends a verification email via SMTP
5. Returns `session_id` and `resend_available_at` timestamp

## Implementation Requirements

### 1. Models

Create `backend/src/models/registration.rs`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationInitRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationInitResponse {
    pub status: String,
    pub message: String,
    pub session_id: Uuid,
    pub resend_available_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationSession {
    pub id: Uuid,
    pub email: String,
    pub verification_code: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub resend_available_at: DateTime<Utc>,
}
```

### 2. Use Cases

Create `backend/src/use_cases/registration.rs`:

```rust
pub struct RegistrationUseCase {
    sqlite: SQLiteProvider,
    smtp: SMTPProvider,
}

impl RegistrationUseCase {
    pub fn new(sqlite: SQLiteProvider, smtp: SMTPProvider) -> Self {
        Self { sqlite, smtp }
    }

    pub async fn init_registration(
        &self,
        email: &str,
    ) -> Result<RegistrationInitResponse, RegistrationError> {
        // 1. Validate email format
        // 2. Generate 6-digit verification code
        // 3. Create session in SQLite with timestamps:
        //    - created_at: now
        //    - expires_at: now + 15 minutes
        //    - resend_available_at: now + 60 seconds
        // 4. Send email via SMTPProvider::send_email()
        // 5. Return RegistrationInitResponse
    }
}
```

### 3. API Endpoint

Create `backend/src/api/endpoints/registration.rs`:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct RegistrationInitRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationInitResponse {
    pub status: String,
    pub message: String,
    pub session_id: String,
    pub resend_available_at: String,
}

pub async fn registration_init(
    req: web::Json<RegistrationInitRequest>,
    use_case: web::Data<RegistrationUseCase>,
) -> Result<web::Json<RegistrationInitResponse>, actix_web::Error> {
    // Call use_case.init_registration()
    // Return JSON response
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(resources::registration::registration_init);
}
```

### 4. Wire into Application

Update `backend/src/main.rs`:
- Create `RegistrationUseCase` with SQLite and SMTP providers
- Register as Actix web data
- Add route to API config

## Database Schema

Add to `backend/src/providers/sqlite.rs` migrations:

```sql
CREATE TABLE IF NOT EXISTS registration_sessions (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    verification_code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    resend_available_at TIMESTAMP NOT NULL
);
```

## Verification Steps

1. **Build verification:**
   ```bash
   cargo check -p backend
   cargo clippy -p backend -- -D warnings
   cargo build -p backend
   ```

2. **Test verification:**
   ```bash
   cargo test -p backend
   ```

## Acceptance Criteria

- [ ] `RegistrationInitRequest` model with email field
- [ ] `RegistrationInitResponse` model with status, message, session_id, resend_available_at
- [ ] `RegistrationSession` model for database storage
- [ ] `RegistrationUseCase` with `init_registration()` method
- [ ] `POST /api/auth/registration/init` endpoint implemented
- [ ] Verification code is 6 digits
- [ ] Session expires after 15 minutes
- [ ] Resend available after 60 seconds
- [ ] Email sent via SMTP with verification code
- [ ] Database migration for registration_sessions table
- [ ] `cargo clippy -p backend -- -D warnings` passes
- [ ] `cargo test -p backend` passes

## Deliverables

1. `backend/src/models/registration.rs`
2. `backend/src/use_cases/registration.rs`
3. `backend/src/api/endpoints/registration.rs`
4. Updated `backend/src/main.rs` with wiring
5. Updated `backend/src/providers/sqlite.rs` with migration
6. Report at `/project/.ralph-wiggum/reports/03-task-registration-init-backend.md`