# User Story: Bootstrap and Testing Enabler

## Summary

Set up the Cargo workspace, all crate scaffolding, and the test infrastructure so that subsequent user stories can build on a working project that compiles and has a test harness ready.

## Specs

- @specs/PROJECT-STRUCTURE.md — workspace layout with backend, frontend, common crates
- @specs/BACKEND.md — backend provider overview and startup sequence
- @specs/GENERIC-BACKEND.md — provider pattern, endpoint pattern, use-case pattern
- @specs/FRONTEND.md — Leptos CSR frontend
- @specs/RUST-COMMON-SPEC.md — Rust module conventions (no inline modules, no mod.rs, no stubs)
- @specs/TESTING.md — docker-compose test environment, test isolation, test patterns

---

## What must be done

### 1. Cargo Workspace

Create the root `Cargo.toml` workspace with three member crates:
- `backend/` — binary crate, actix-web server
- `frontend/` — binary crate, Leptos CSR app
- `common/keyword-extractor/` — library crate

Each crate must have its own `Cargo.toml` with minimal dependencies to compile. The workspace must `cargo check` successfully.

### 2. Backend Crate Skeleton

Set up module structure per GENERIC-BACKEND.md and RUST-COMMON-SPEC.md:
- `src/main.rs` — entry point
- `src/config.rs` — empty config struct (will be filled by later stories)
- `src/api/` — configurator module (empty route registration)
- `src/providers/` — module declarations only (no provider implementations yet)
- `src/usecases/` — module declarations only

No stubs. Only declare modules that have real code. An empty `main` that starts actix-web with no routes is fine.

### 3. Frontend Crate Skeleton

Set up a minimal Leptos CSR app that compiles and renders a placeholder page. Just enough to verify the toolchain works.

### 4. Common Crate Skeleton

`common/keyword-extractor/` with a `lib.rs` that compiles. No implementation needed yet.

### 5. `.gitignore`

Add a `.gitignore` at the project root covering Rust/Cargo build artifacts (`target/`), IDE files, OS files, and test artifacts (temp SQLite databases, etc.).

### 6. Test Infrastructure

- `docker-compose.yml` at project root with MinIO and MailHog (see TESTING.md)
- `tests/config.toml` with test-specific configuration pointing to docker-compose services
- Test utility module with isolation helpers (random bucket prefix, temp SQLite path, unique email)
- At least one trivial integration test that proves the harness works (e.g. connect to MinIO, create a bucket, delete it)

---

## Acceptance Criteria

- [ ] `cargo check --workspace` succeeds
- [ ] `cargo build -p backend` succeeds
- [ ] `cargo build -p frontend` succeeds (or `cargo build -p keyword-extractor` if frontend needs wasm target)
- [ ] `docker-compose up -d` starts MinIO and MailHog
- [ ] `cargo test` runs at least one integration test against docker-compose services
- [ ] Project structure matches PROJECT-STRUCTURE.md
- [ ] Module conventions follow RUST-COMMON-SPEC.md (no inline modules, no mod.rs, no stubs)
- [ ] `.gitignore` exists and covers `target/`, IDE files, OS files, temp test artifacts
