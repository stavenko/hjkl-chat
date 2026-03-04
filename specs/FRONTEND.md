# Frontend

## Technology
Frontend must be written in Rust, using the Leptos framework in **CSR (Client-Side Rendering)** mode. Leptos compiles to WASM and runs entirely in the browser. The backend is a separate actix-web API server.

## Architecture
See @specs/GENERIC-FRONTEND.md for component, page, service, form, routing, and state management patterns.

## Design Source
All pages and components must be taken and updated from corresponding elements in Penpot applications.

### Penpot API Access

The Penpot instance is at `https://penpot.hjkl.pro`. Authentication uses a token stored in the project `.env` file as `PENPOT_ACCESS_TOKEN`.

**Authentication:** All API requests require the header:
```
Authorization: Token <PENPOT_ACCESS_TOKEN>
```

**Design file ID:** `742d722a-06ca-817e-8007-a42f6283e7ed`
**Page ID:** `988fdbaf-c8f8-808f-8007-a55ba615f576`

**API endpoints:**

1. **Get file data** (full design tree with all objects):
   ```
   GET /api/rpc/command/get-file?id=<file-id>&components-v2=true
   ```
   Returns transit+json with all pages, frames, and shape objects.

2. **Get frame thumbnails** (map of frame IDs to thumbnail asset UUIDs):
   ```
   GET /api/rpc/command/get-file-object-thumbnails?file-id=<file-id>
   ```
   Returns a map where keys are `<file-id>/<page-id>/<frame-id>/frame` and values are asset UUIDs.

3. **Download thumbnail PNG** (actual rendered image of a frame):
   ```
   GET /assets/by-id/<thumbnail-uuid>
   ```
   Returns `image/png`.

**How to get a design frame image:**
1. Call `get-file-object-thumbnails` with the file ID
2. Find the entry matching your frame ID (from the user story's Penpot URL `frame-id` parameter)
3. Use the returned UUID to download from `/assets/by-id/<uuid>`

User story Penpot URLs follow this pattern:
```
https://penpot.hjkl.pro/view/<file-id>?page-id=<page-id>&frame-id=<frame-id>
```
Extract `frame-id` from the URL to look up the thumbnail.

## API Configuration
- API base URL is configured via environment variable at build time (e.g., `API_BASE_URL`).
- All service functions prepend this base URL to endpoint paths.
- Default for local development: `http://localhost:8080`.

## Auth Token Storage
- Access and refresh tokens are stored in `localStorage`.
- On app init, `AuthState` restores tokens from `localStorage`.
- On login/registration completion, tokens are written to `localStorage`.
- On logout, tokens are cleared from `localStorage`.

## Build Toolchain
- Build with `trunk` for WASM compilation and asset bundling.
- `trunk serve` for local development with hot-reload.
- `trunk build --release` for production builds.
- Output directory: `dist/`.
