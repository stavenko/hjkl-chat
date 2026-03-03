# Generic Backend Specification

## Module Structure
- **cli** – command line settings, entrypoint called from `main`.
- **api**
  - **endpoints** – one endpoint per file (e.g., `auth_registration.rs`, `auth_login.rs`).
  - **configurator** – function that registers all endpoints with the chosen HTTP framework (e.g., Express, FastAPI, etc.).
- **usecases** – business‑logic functions invoked by endpoints.
- **config** – server configuration.
- **providers**
  - **s3**
  - **cache**
  - **sqlite**
  - **smtp**
  - … other providers.

### Providers Overview
Providers expose external functionality such as object storage, caches, or any third‑party service.
They contain **no business logic**; they only wrap the external client, expose low‑level async methods, and translate low‑level errors into a provider‑specific error type.
All business logic, key naming, and (de)serialization belong in **use‑cases**, which receive a provider instance as a dependency.

#### S3Provider
Wraps an S3-compatible client. Initialized with endpoint URL, bucket name, and credentials.
```text
// S3Provider (generic pseudocode)
class S3Provider {
    constructor(endpoint, bucket, accessKey, secretKey) {
        this.client = new S3Client(endpoint, accessKey, secretKey);
        this.bucket = bucket;
    }

    // Retrieve an object by key
    async get_object(key) -> bytes { /* ... */ }

    // Store an object
    async put_object(key, data) { /* ... */ }

    // List objects under a prefix
    async list_objects(prefix) -> [key] { /* ... */ }

    // Delete an object
    async delete_object(key) { /* ... */ }

    // Check if an object exists and get metadata
    async head_object(key) -> metadata { /* ... */ }
}
```
The provider returns an `S3Error` (or maps to a common `ProviderError`) and does **not** perform any domain‑specific processing.

#### CacheProvider
Local filesystem cache for S3 objects. Initialized with a cache directory path.
```text
// CacheProvider (generic pseudocode)
class CacheProvider {
    constructor(cacheDir) {
        this.cacheDir = cacheDir;
    }

    // Get cached file path for a key, or None if not cached
    get(key) -> Option<PathBuf> { /* ... */ }

    // Write data to cache, return the local file path
    put(key, data) -> PathBuf { /* ... */ }

    // Remove a cached entry
    evict(key) { /* ... */ }
}
```
The provider manages only local file I/O and path mapping. No business logic, no network calls.

#### SQLiteProvider
Wraps a `rusqlite` connection. Initialized with an S3Provider, a CacheProvider, and the S3 object path of the database file. On construction it fetches the `.db` file from S3 through the cache and opens a connection.
```text
// SQLiteProvider (generic pseudocode)
class SQLiteProvider {
    constructor(s3Provider, cacheProvider, dbObjectPath) {
        // 1. Check cache for the DB file
        let localPath = cacheProvider.get(dbObjectPath);
        if (localPath == None) {
            // 2. Fetch from S3 and cache it
            let data = s3Provider.get_object(dbObjectPath);
            localPath = cacheProvider.put(dbObjectPath, data);
        }
        // 3. Open rusqlite connection
        this.conn = sqlite::open(localPath);
    }

    // Execute a query, return rows
    async query(sql, params) -> rows { /* ... */ }

    // Execute a statement (INSERT, UPDATE, DELETE)
    async execute(sql, params) -> affected_rows { /* ... */ }

    // Flush: write the current DB file back to S3
    async flush(s3Provider, cacheProvider) { /* ... */ }
}
```
The provider owns only the connection lifecycle and raw SQL execution. Schema design, migrations, and query composition belong in use‑cases.

## Services Overview

Services represent internal stateful components of the backend that can perform long‑running or scheduled work without blocking request handling. Typical responsibilities include:

- **Task scheduling** – run a job at a specific time or interval (e.g., cron‑style tasks).
- **Background processing** – accept work items via a queue, process them asynchronously while the server continues to serve new HTTP requests.

A service should expose a minimal public API (e.g., `schedule(task, when)`, `enqueue(job)`, `status()`) and hide its internal implementation details. Business logic stays in **use‑cases**; services only orchestrate execution and manage lifecycle concerns.

## Endpoints Configurator Example

The configurator registers each endpoint with the HTTP framework. In a generic form:

```
function configureRoutes(router) {
    router.scope("/api")
        .addRoute("POST", "/new-message", newMessageHandler)
        .addRoute("POST", "/delete-session", deleteSessionHandler)
        .addRoute("GET", "/ws", websocketHandler);
}
```

Each `addRoute` call binds an HTTP method and path to the corresponding handler function.

## Endpoint Example

```text
// Generic endpoint handler example (pseudocode)
// Endpoints are thin wrappers that delegate all business logic to use‑cases.
async function newMessageHandler(request) {
    // 1️⃣ Parse the incoming request payload (e.g., JSON body) into the input shape expected by the use‑case.
    const input = request.body;
    // 2️⃣ Retrieve any required provider(s) from the request context (e.g., S3, cache, SQLite).
    const s3 = request.context.s3Provider;
    const sqlite = request.context.sqliteProvider;
    // 3️⃣ Invoke the appropriate use‑case, passing the input and provider(s). No business logic lives here.
    const result = await useCaseCommand(input, s3, sqlite);
    // 4️⃣ Convert the use‑case result (or error) into a generic HTTP response object.
    return httpResponseFromResult(result);
}
```

## Use‑case Example

```text
// Generic use‑case example (pseudocode)
function useCase(input, s3Provider, sqliteProvider) {
    // 1. Build queries or commands based on input
    const query = "SELECT * FROM users WHERE id = ?";
    // 2. Interact with the provider
    const result = await sqliteProvider.query(query, [input.userId]);
    // 3. Process the raw result into the desired output shape
    const output = transformResult(result);
    // 4. Return output or throw an error object
    return output;
}
```

## Notes (language‑agnostic)
- All modules are independent libraries; the entry point (e.g., a `main` function or script) invokes the CLI runner which wires together configuration, providers, and services.
- Each endpoint is a thin wrapper: it only parses the request payload, obtains required provider(s) from the request context, calls the appropriate use‑case, and returns a generic HTTP response object. No business logic resides in the endpoint.
- Use‑cases encapsulate business logic only and depend on providers; they do not depend on any specific web framework types.
