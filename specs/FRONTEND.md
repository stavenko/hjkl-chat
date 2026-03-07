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

`config.json` is a **runtime** configuration file. It is **never** part of the build output. The serving infrastructure provides it per environment.

- On app init, the frontend fetches `/config.json` before mounting the app or making any API calls.
- `/config.json` contains:
  ```json
  {
    "api_base_url": "<string>"
  }
  ```
- All service functions prepend the loaded `api_base_url` to endpoint paths.
- The backend port must not be in Chrome's restricted port range (e.g., 6000-6063).

### Serving config.json per environment

**Docker Compose (dev/staging):** The reverse proxy (e.g., nginx) serves both the static frontend files from `dist/` and a `config.json` mounted or generated per environment. Example: `config.json` is a volume mount pointing to the backend's internal address.

**CDN (production):** The build output (`dist/`) is deployed to CDN without `config.json`. The CDN or edge server serves `config.json` separately, configured with the production API URL.

**Local dev with `trunk serve`:** Trunk proxies `/api/` requests to the local backend. A `tests/config.json` file exists for local testing and must match the port in `tests/config.toml`.


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

## App Mounting

Trunk compiles the Rust crate to WASM and generates a JS module that initializes it. The app is mounted via a two-step process in `index.html`:

1. Trunk's generated `<script type="module">` loads the WASM, sets `window.wasmBindings`, and dispatches a `TrunkApplicationStarted` event on `window`.
2. An inline script listens for this event and calls `window.wasmBindings.main()`.

**Important:** The event listener must be on `window`, not `document` — Trunk dispatches the event via the global `dispatchEvent()`, which targets `window`.

```html
<script>
  window.addEventListener('TrunkApplicationStarted', () => {
    if (typeof window.wasmBindings?.main === 'function') {
      window.wasmBindings.main();
    }
  });
</script>
```

The `main()` function (exported via `#[wasm_bindgen]`) initializes the panic hook, loads runtime config, and mounts the Leptos app:

```rust
#[wasm_bindgen]
pub fn main() {
    console_error_panic_hook::set_once();
    spawn_local(async {
        crate::services::init_api_base_url().await;
        mount_to_body(App);
    });
}
```

`init_api_base_url()` must complete before `mount_to_body()` so that all components have access to the API base URL from the first render.
