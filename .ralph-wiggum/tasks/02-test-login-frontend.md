# Task 02-test-login-frontend.md

**User Story:** 01-login.md  
**Related Implementation Task:** 02-task-login-frontend.md  
**Related Implementation Report:** /project/.ralph-wiggum/reports/02-task-login-frontend.md

## Objective

Create integration tests to verify the frontend login implementation with external service interactions. Focus on testing the API communication between the frontend and backend login endpoints.

## Scope

### Test Modules to Create

Create `frontend/tests/login_integration_tests.rs` with the following test coverage:

#### 1. API Service Tests
- Test `auth_service::login()` function calls the correct backend endpoint
- Test request body format matches backend expectations (email/password JSON)
- Test response parsing from backend login API
- Test error handling for failed login attempts (wrong credentials, network errors)

#### 2. Configuration Tests
- Test `services::init_api_base_url()` loads configuration correctly
- Test `services::get_api_base_url()` returns the configured base URL
- Test API calls use the correct base URL from configuration

#### 3. Auth State Tests
- Test `auth_state::AuthState` stores JWT tokens in localStorage
- Test `auth_state::AuthState` retrieves tokens from localStorage
- Test token persistence across simulated page reloads
- Test logout clears tokens from localStorage

#### 4. Component Interaction Tests
- Test `LoginForm` component calls `auth_service::login()` on submit
- Test `AuthenticationButton` displays correct state (logged in/out)
- Test navigation to `/login` route on unauthenticated access to `/`

## Dependencies

Add dev-dependencies to `frontend/Cargo.toml`:
- `wasm-bindgen-test` for wasm test runtime
- `tokio` for async test support (if not already present)
- `console_error_panic_hook` for panic handling in tests

## Test Requirements

1. **Integration Testing Approach**: Tests must verify external service interactions - specifically the HTTP calls to the backend login API endpoint (`POST /api/auth/login`)

2. **Test Isolation**: 
   - Use unique test user credentials
   - Mock or stub backend responses where actual backend is not available
   - Clean up any localStorage state between tests

3. **WASM Compatibility**: All tests must compile and run in WASM environment using `wasm-bindgen-test`

4. **External Service Verification**: 
   - Verify fetch API calls are made with correct method, URL, headers, and body
   - Verify response parsing handles both success (200 OK with JWT tokens) and error cases (401 Unauthorized, 400 Bad Request)

## Acceptance Criteria

- [ ] `cargo test -p frontend` runs successfully
- [ ] All tests in `login_integration_tests.rs` pass
- [ ] Tests verify at least 3 different scenarios: successful login, failed login, configuration loading
- [ ] No tests are ignored or skipped without documented reason
- [ ] Tests follow patterns from `backend/tests/` for consistency
- [ ] `cargo clippy -p frontend -- -D warnings` passes (excluding dead_code)
- [ ] `cargo check --workspace` passes

## Deliverables

1. `frontend/tests/login_integration_tests.rs` - Integration test module
2. Updated `frontend/Cargo.toml` with dev dependencies
3. Report at `/project/.ralph-wiggum/reports/02-test-login-frontend.md` documenting:
   - Tests created
   - Test results
   - Any issues encountered
   - Commands run and their output

## Notes

- Focus on integration tests that verify external service interactions (API calls to backend)
- Unit tests for individual components can be created later if needed
- The manual-testing task will verify the actual UI works end-to-end
- Tests should be designed to run in both isolated mode (mocked backend) and integrated mode (real backend)