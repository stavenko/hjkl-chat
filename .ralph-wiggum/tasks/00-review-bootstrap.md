# Task 0-Review: Bootstrap Implementation Review

## Summary

Review the bootstrap implementation for code quality, spec compliance, and completeness. This review must verify all aspects of the implementation and create fix tasks for any issues found.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Spec Files to Read

- @specs/PROJECT-STRUCTURE.md — workspace layout
- @specs/BACKEND.md — backend provider overview
- @specs/GENERIC-BACKEND.md — provider pattern, endpoint pattern, use-case pattern
- @specs/RUST-COMMON-SPEC.md — Rust module conventions
- @specs/FRONTEND.md — Leptos CSR frontend
- @specs/TESTING.md — docker-compose test environment, test patterns

## Implementation Task

@.ralph-wiggum/tasks/00-task-bootstrap.md

## Test Task

@.ralph-wiggum/tasks/00-test-bootstrap.md

## Reports

- @.ralph-wiggum/reports/00-task-bootstrap.md
- @.ralph-wiggum/reports/00-test-bootstrap.md
- @.ralph-wiggum/reports/00-manual-testing-bootstrap.md

## What to Check

### A. Test Suite Verification

Run these commands and verify results:

```bash
# 1. Run all tests
cargo test -p backend
```

**Verify:**
- All tests pass (0 failed)
- No tests are skipped (0 ignored is acceptable if documented why)
- No tests are ignored without a clear reason in comments

```bash
# 2. Run tests with ignored tests included
cargo test -p backend -- --ignored
```

**Verify:**
- Document which tests require external services
- Verify that ignored tests have proper `#[ignore]` attributes with reasons

### B. No Hardcoded Values

Search the entire codebase for hardcoded values:

```bash
# 1. Check for default() usage
grep -r "default()" backend/src/ frontend/src/

# 2. Check for magic numbers in non-test code
grep -rE "[0-9]{3,}" backend/src/ frontend/src/ | grep -v tests

# 3. Check for hardcoded strings that should be config
grep -rE '"[a-zA-Z0-9_-]{16,}"' backend/src/ frontend/src/ | grep -v tests | grep -v "http" | grep -v "localhost"
```

**Verify:**
- No hardcoded configuration values in main code
- All configuration is read from environment variables or config files
- No default() calls that silently provide fallback values
- If a required config value is missing, the application must fail with a clear error message

### C. No TODOs or Unimplemented Code

Search the entire codebase:

```bash
# 1. Check for TODO comments
grep -ri "TODO" backend/src/ frontend/src/ common/

# 2. Check for unimplemented!() macros
grep -r "unimplemented!" backend/src/ frontend/src/ common/

# 3. Check for todo!() macros
grep -r "todo!" backend/src/ frontend/src/ common/

# 4. Check for panic!("not implemented")
grep -r "panic!" backend/src/ frontend/src/ common/ | grep -i "not implement"
```

**Verify:**
- Zero results for all searches
- All code is fully implemented or not present at all

### D. Build and Run Verification

```bash
# 1. Run cargo check
cargo check --workspace

# 2. Run cargo build
cargo build -p backend
cargo build -p frontend

# 3. Run the backend and verify it serves HTTP
cargo run -p backend &
sleep 2
curl -s http://localhost:8080/ || echo "Backend not responding on port 8080"
pkill -f "cargo run" || true
```

**Verify:**
- cargo check succeeds with no errors
- cargo build succeeds
- Backend starts and serves HTTP on the configured port

### E. Linting and Warnings

```bash
# 1. Run clippy
cargo clippy -p backend -- -D warnings
cargo clippy -p frontend -- -D warnings

# 2. Run build and capture warnings
cargo build -p backend 2>&1 | grep -i warning
cargo build -p frontend 2>&1 | grep -i warning
```

**Verify:**
- No clippy warnings
- No compiler warnings
- If a warning is absolutely necessary, it must have `#[allow(...)]` with a comment explaining why

### F. Spec Compliance

Read the following spec files and verify the implementation matches:

#### PROJECT-STRUCTURE.md

**Verify:**
- Workspace has exactly three member crates: backend, frontend, common/keyword-extractor
- Directory structure matches the spec
- Docker directories exist: docker/test/, docker/local/

#### RUST-COMMON-SPEC.md

**Verify:**
- No inline modules (no `mod foo { ... }` syntax)
- No mod.rs files anywhere in the project
- All modules use the `<name>.rs` pattern
- No stub modules (no empty modules or modules with only TODO comments)

#### GENERIC-BACKEND.md

**Verify:**
- Backend module structure follows the provider pattern
- If providers exist, they are in `src/providers/`
- If endpoints exist, they are in `src/api/`
- If use-cases exist, they are in `src/usecases/`
- Module declarations match the spec's expected structure

#### FRONTEND.md

**Verify:**
- Frontend uses Leptos with CSR feature
- Frontend has proper HTML entry point
- Frontend compiles to wasm

#### TESTING.md

**Verify:**
- docker/test/docker-compose.yml has MinIO and MailHog
- docker/local/docker-compose.yml has backend, frontend, MinIO, MailHog
- Test isolation utilities exist and work
- Integration tests follow the patterns described in the spec

## Issues Found (to be filled during review)

Document all issues found here and create corresponding fix tasks.

## Fix Tasks to Create

For each issue found, create a task file:
- `00-task-fix-<description>.md`

Example:
- `00-task-fix-01-remove-dead-code-warning.md`
- `00-task-fix-02-add-dockerfile-backend.md`

## Acceptance Criteria

- [ ] All tests pass with `cargo test -p backend`
- [ ] No hardcoded values in main code (config only)
- [ ] Zero TODOs, zero unimplemented!(), zero todo!(), zero panic!("not implemented")
- [ ] Project builds with `cargo build --workspace`
- [ ] Backend serves HTTP when run
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings (or documented with #[allow] + comment)
- [ ] Project structure matches PROJECT-STRUCTURE.md
- [ ] Module conventions follow RUST-COMMON-SPEC.md
- [ ] Backend follows GENERIC-BACKEND.md patterns
- [ ] Frontend follows FRONTEND.md patterns
- [ ] Testing follows TESTING.md patterns
- [ ] All fix tasks created and marked as pending in progress.md