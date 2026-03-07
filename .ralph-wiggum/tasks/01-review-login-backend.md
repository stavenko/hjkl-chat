# Task: Review Login Backend Implementation

## Overview
Comprehensive review of the login backend implementation to verify code quality, spec compliance, and completeness before proceeding to frontend implementation.

## User Story
- @user-stories/01-login.md

## Spec Files to Read
1. @specs/BACKEND.md — Provider pattern, SQLiteProvider, application wiring, error handling
2. @specs/GENERIC-BACKEND.md — Provider pattern, use-case structure, endpoint conventions
3. @specs/RUST-COMMON-SPEC.md — Module conventions, error handling patterns
4. @specs/PROJECT-STRUCTURE.md — Project layout
5. @specs/GENERIC-FRONTEND.md — Frontend patterns (for future frontend implementation reference)

## Related Implementation
- Task: @.ralph-wiggum/tasks/01-task-login-backend.md
- Report: @.ralph-wiggum/reports/01-task-login-backend.md
- Test Task: @.ralph-wiggum/tasks/01-test-login-backend.md
- Test Report: @.ralph-wiggum/reports/01-test-login-backend.md
- Manual Testing Report: @.ralph-wiggum/reports/01-manual-testing-login-backend.md

## Review Checklist

### A. Test Suite Verification
**Command:** `cargo test -p backend`
**Check:**
- All tests pass (zero failed)
- Zero skipped tests (ignored tests must have proper documentation explaining why)
- Zero tests that panic or timeout
- Verify test coverage for: successful login, wrong password, non-existent email, missing fields

### B. No Hardcoded Values
**Command:** `grep -r "testsecret\|password123\|example.com" backend/src/ --include="*.rs" | grep -v test`
**Check:**
- No default settings or default methods used in production code
- No magic numbers or hardcoded strings in business logic
- All configuration must be read from config files or environment variables
- JWT_SECRET must come from environment variable
- If a required config value is missing, the application MUST fail with a clear error message
- Verify that `#[derive(Default)]` is only used where appropriate (not for Config)

### C. No TODOs or Unimplemented Code
**Commands:**
- `grep -r "TODO:" backend/src/ --include="*.rs"`
- `grep -r "unimplemented!()" backend/src/ --include="*.rs"`
- `grep -r "todo!()" backend/src/ --include="*.rs"`
- `grep -r 'panic!("not implemented")' backend/src/ --include="*.rs"`
**Check:**
- Zero occurrences of TODO comments
- Zero occurrences of unimplemented!(), todo!(), or panic!("not implemented")
- Code must be fully implemented or not present at all

### D. Build and Run Verification
**Commands:**
- `cargo check --workspace`
- `cargo build -p backend`
- `cargo clippy -p backend -- -D warnings`
**Check:**
- Zero compilation errors
- Zero clippy warnings (all warnings must be FIXED, not suppressed)
- If a warning suppression is absolutely necessary, verify `#[allow(...)]` has a comment explaining WHY
- Backend compiles cleanly

### E. Spec Compliance — Module Structure
**Check against @specs/GENERIC-BACKEND.md:**
- Module structure follows: cli, api/endpoints, api/configurator, usecases, config, providers
- Each provider is in its own file under `backend/src/providers/`
- Each use-case is in its own file under `backend/src/use_cases/`
- Each endpoint is in its own file under `backend/src/endpoints/`
- No mod.rs files (use Rust 2018 edition module system)

**Check against @specs/BACKEND.md:**
- SQLiteProvider downloads database from S3, caches locally, opens rusqlite connection
- SQLiteProvider has `dump_to_s3()` method called after INSERT/UPDATE
- All providers initialized in correct order: S3 -> LocalFileSystem -> SQLite -> SMTP
- Providers registered as Actix app data
- CLI implemented in cli.rs with clap
- Commands: `run --config`, `download-sqlite --config`

### F. Spec Compliance — Provider Pattern
**Check against @specs/GENERIC-BACKEND.md and @specs/BACKEND.md:**
- Each provider contains NO business logic
- Each provider exposes low-level async methods
- Each provider has its own error type (e.g., S3ProviderError, SQLiteProviderError)
- Providers may depend on other providers for lifecycle (SQLite depends on S3 and LocalFileSystem)
- Use-cases receive provider instances as dependencies

### G. Spec Compliance — Endpoint and Use-Case Pattern
**Check against @specs/BACKEND.md:**
- Endpoints are thin wrappers (parse request, call use-case, return response)
- All business logic is in use-cases, not endpoints
- ApiResponse<T> enum with Ok(T) and Err(Error) variants
- ApiResponse implements Responder trait
- Error struct has code and message fields
- Wrong credentials return HTTP 401 with {"status": "error", "message": "Invalid email or password"}

### H. Spec Compliance — Config
**Check against @specs/BACKEND.md:**
- Config is ONLY read from file, never constructed in code (not even for tests)
- Config does NOT implement Default trait
- Config has sections: main (addr, port), sqlite, s3, local_fs, smtp
- If config file is missing or invalid, application MUST fail with clear error

### I. Manual Testing Report Review
**Read:** @.ralph-wiggum/reports/01-manual-testing-login-backend.md
**Answer:**
- Is the backend starting and working? (Note: requires MinIO)
- Are curl requests executed properly and returning expected responses? (Note: cannot verify without MinIO)
- Do integration tests verify the functionality correctly? (Should be YES)
- Are there any issues that require fix tasks?

## Issues Found
**For each issue found, create a fix task and mark as pending in progress.md:**

1. **Issue Description:** [Describe the issue]
   **Fix Task:** [Create XX-task-fix-<description>.md]

2. **Issue Description:** [Describe the issue]
   **Fix Task:** [Create XX-task-fix-<description>.md]

## Deliverables
- Complete all review checks above
- Document all issues found with specific file paths and line numbers
- Create fix tasks for all issues
- Update @.ralph-wiggum/progress.md with review results
- Write report to @.ralph-wiggum/reports/01-review-login-backend.md

## Notes
- This review is critical before proceeding to frontend implementation
- Any issues found must be fixed before the login user story can be considered complete
- Frontend implementation will follow the same review process