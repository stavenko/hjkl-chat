# Task 0: Bootstrap Cargo Workspace

## Summary

Set up the Cargo workspace with three member crates (backend, frontend, common/keyword-extractor) and the basic directory structure required for the project.

## User Story

@user-stories/00-bootstrap-and-testing.md

## Spec Files

- @specs/PROJECT-STRUCTURE.md — workspace layout
- @specs/BACKEND.md — backend provider overview
- @specs/GENERIC-BACKEND.md — module patterns
- @specs/RUST-COMMON-SPEC.md — Rust module conventions
- @specs/FRONTEND.md — Leptos CSR frontend
- @specs/TESTING.md — docker-compose test environment

## What to Do

### 1. Root Cargo.toml

Create `/project/Cargo.toml` as a workspace with three member crates:
- `backend/`
- `frontend/`
- `common/keyword-extractor/`

### 2. Backend Crate

Create `/project/backend/Cargo.toml` with:
- `actix-web` as a dependency
- `serde` with `derive` feature
- `serde_json`
- `tokio` with `full` feature
- `tracing`
- `thiserror`

Create minimal backend structure:
- `/project/backend/src/main.rs` — entry point that starts an empty actix-web server
- `/project/backend/src/config.rs` — empty config struct

### 3. Frontend Crate

Create `/project/frontend/Cargo.toml` with:
- `leptos` with `csr` feature
- `leptos_meta`
- `wasm-bindgen`
- `web-sys`
- `gloo-net`
- `serde` with `derive` feature
- `serde_json`

Create minimal frontend structure:
- `/project/frontend/src/main.rs` — minimal Leptos CSR app that compiles
- `/project/frontend/index.html` — HTML entry point for trunk

### 4. Common Crate

Create `/project/common/keyword-extractor/Cargo.toml` with minimal dependencies.

Create `/project/common/keyword-extractor/src/lib.rs` that compiles (empty lib).

### 5. Directory Structure

Create docker directories:
- `/project/docker/test/` — for test docker-compose
- `/project/docker/local/` — for local dev docker-compose

### 6. Gitignore

Create `/project/.gitignore` covering:
- `target/`
- IDE files (`.idea/`, `.vscode/`)
- OS files (`*.swp`, `*.swo`, `.DS_Store`)
- Test artifacts (temp SQLite databases)

## Acceptance Criteria

- `cargo check --workspace` succeeds
- Project structure matches PROJECT-STRUCTURE.md
- No stub modules, only what is needed to compile