# Project Structure

The project is organized as a **Cargo workspace** with three crates.

## Workspace Layout

```
Cargo.toml              # Workspace root
backend/
  Cargo.toml            # actix-web API server
  src/
    ...
frontend/
  Cargo.toml            # Leptos CSR application (compiles to WASM)
  src/
    ...
common/
  keyword-extractor/
    Cargo.toml           # Shared keyword extraction library
    src/
      ...
```

## Crates

### `backend`
Actix-web HTTP API server. See `BACKEND.md` and `GENERIC-BACKEND.md` for architecture.

### `frontend`
Leptos CSR application. Compiles to WASM, runs in the browser. Communicates with the backend via HTTP API. See `FRONTEND.md`.

### `common/keyword-extractor`
Shared TF-IDF keyword extraction library used by both backend and frontend. Backend uses it to compute keywords on file save; frontend uses it to rebuild the local search index. See `SEARCH-ENGINE.md`.
