# Task: Implement Frontend Login Page and Services

## Overview
Implement the frontend login page with LoginForm component, auth_service module, routing, and authentication state management as specified in user story 01-login.md.

## User Story
- [01-login.md](../user-stories/01-login.md)

## Spec Files to Read
- [FRONTEND.md](../specs/FRONTEND.md) — Frontend architecture and patterns
- [GENERIC-FRONTEND.md](../specs/GENERIC-FRONTEND.md) — Component, Page, Service, Form, Routing, State Management patterns
- [DESIGN.md](../specs/DESIGN.md) — Design system and component specifications

## Acceptance Criteria

### Routing
- Create `frontend/src/app.rs` with Leptos routing setup
- Define routes: `/login`, `/register`, `/password/restore`, `/` (protected)
- Protected route at `/` redirects to `/login` if not authenticated
- Follow Routing Pattern from GENERIC-FRONTEND.md

### AuthState
- Create `frontend/src/auth_state.rs` implementing authentication state management
- Store `access_token`, `refresh_token`, `user` (id, email)
- Integrate with `localStorage` for persistence
- Implement `is_authenticated()` method to check authentication status
- Follow State Management Pattern from GENERIC-FRONTEND.md

### Auth Service
- Create `frontend/src/auth_service.rs` module
- Implement `login(email: &str, password: &str) -> Result<LoginResponse, Error>` async function
- Call `POST /api/auth/login` endpoint with JSON body
- Return `LoginResponse` with `user`, `access_token`, `refresh_token` on success
- Handle error responses with `message` field
- Follow Service Pattern from GENERIC-FRONTEND.md

### Reusable Components
- Create `frontend/src/components/authentication_input.rs`
  - Props: `label`, `value`, `on_change`, `error` (Option), `input_type` (for password)
  - Render label, input container, and error text when present
  - Apply error styling when error is Some
  - Follow Component Pattern from GENERIC-FRONTEND.md

- Create `frontend/src/components/authentication_button.rs`
  - Props: `disabled`, `label`, `on_click`
  - Render submit button with disabled state styling
  - Follow Component Pattern from GENERIC-FRONTEND.md

### LoginForm Component
- Create `frontend/src/components/login_form.rs`
- Props: `on_success` callback
- State: `email` signal, `password` signal, `error` signal (Option)
- Button disabled until both email and password are filled
- On submit: call `auth_service::login`, handle success/error
- On success: store tokens in `AuthState`, navigate to home
- On error: display error message on password field
- Follow Form Pattern from GENERIC-FRONTEND.md

### LoginPage Component
- Create `frontend/src/pages/login_page.rs`
- Include `LoginForm` component
- "Forgot password?" link to `/password/restore`
- "Don't have an account? Register" link to `/register`
- Follow Page Pattern from GENERIC-FRONTEND.md

### Main Entry Point
- Update `frontend/src/main.rs` to use `App` with routing
- Mount `LoginPage` at `/login` route
- Set up protected route at `/` with authentication check

## Implementation Steps
1. Read all spec files and understand the patterns
2. Create `auth_state.rs` with token storage and `is_authenticated()` method
3. Create `auth_service.rs` with `login()` function
4. Create `components/authentication_input.rs`
5. Create `components/authentication_button.rs`
6. Create `components/login_form.rs`
7. Create `pages/login_page.rs`
8. Create `app.rs` with routing setup
9. Update `main.rs` to use routing
10. Add any required dependencies to `frontend/Cargo.toml`

## Verification
- `cargo check -p frontend` — compiles without errors
- `cargo clippy -p frontend -- -D warnings` — no warnings (except dead_code which is acceptable)
- `cargo build -p frontend` — builds successfully
- Frontend serves HTTP and `/login` route is accessible
- Login form renders with email, password fields, and submit button
- Button is disabled when fields are empty
- Navigation links to `/register` and `/password/restore` exist

## Report
Write implementation report to `/project/.ralph-wiggum/reports/02-task-login-frontend.md` listing:
- Files created
- Files modified
- Dependencies added
- Verification results