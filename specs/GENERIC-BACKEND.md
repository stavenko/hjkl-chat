# Generic Backend Specification

## Module Structure
- **cli** – command line settings, entrypoint called from `main`.
- **api**
  - **endpoints** – one endpoint per file (e.g., `auth_registration.rs`, `auth_login.rs`).
  - **configurator** – function that registers all endpoints with the chosen HTTP framework (e.g., Express, FastAPI, etc.).
- **usecases** – business‑logic functions invoked by endpoints.
- **config** – server configuration.
- **providers** – one module per external dependency (see below).

### What Is a Provider

A provider is a thin wrapper around a single external resource — a database, an object store, a mail server, a message queue, an HTTP API, a local filesystem, etc. Its only job is to expose low‑level async methods and translate low‑level errors into a provider‑specific error type.

**Rules:**
- A provider contains **no business logic**. No key naming, no (de)serialization, no domain concepts.
- All business logic belongs in **use‑cases**, which receive one or more provider instances as dependencies.
- Each provider defines its own error type (e.g., `SomeProviderError`) or maps to a common `ProviderError`.
- A provider may depend on other providers when it needs them for its lifecycle (e.g., a provider that caches data from another provider).
- A provider is initialized with a config struct and registered as application data so endpoints can access it.

### Provider Examples

```text
// SomeProvider wraps a single external client (pseudocode)
class SomeProvider {
    constructor(config) {
        this.client = new ExternalClient(config.endpoint, config.credentials);
    }

    async do_something(input) -> result { /* ... */ }
    async do_something_else(input) -> result { /* ... */ }
}
```

A provider that depends on another provider for its lifecycle:

```text
// SomeOtherProvider depends on SomeProvider at init time (pseudocode)
class SomeOtherProvider {
    constructor(someProvider, config) {
        // Use someProvider to bootstrap internal state
        let data = someProvider.do_something(config.key);
        this.internalState = process(data);
    }

    async query(input) -> result { /* ... */ }
    async flush(someProvider) { /* ... */ }
}
```

`SomeOtherProvider` owns only its internal state and raw operations. Higher‑level logic (what to query, when to flush) belongs in use‑cases.

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
    // 2️⃣ Retrieve any required provider(s) from the request context.
    const providerA = request.context.someProvider;
    const providerB = request.context.someOtherProvider;
    // 3️⃣ Invoke the appropriate use‑case, passing the input and provider(s). No business logic lives here.
    const result = await useCaseCommand(input, providerA, providerB);
    // 4️⃣ Convert the use‑case result (or error) into a generic HTTP response object.
    return httpResponseFromResult(result);
}
```

## Use‑case Example

```text
// Generic use‑case example (pseudocode)
function useCase(input, providerA, providerB) {
    // 1. Build queries or commands based on input
    const rawData = await providerA.do_something(input.key);
    // 2. Apply business logic and transformations
    const processed = transformData(rawData);
    // 3. Persist via another provider if needed
    await providerB.do_something_else(processed);
    // 4. Return output or throw an error object
    return processed;
}
```

## Notes (language‑agnostic)
- All modules are independent libraries; the entry point (e.g., a `main` function or script) invokes the CLI runner which wires together configuration, providers, and services.
- Each endpoint is a thin wrapper: it only parses the request payload, obtains required provider(s) from the request context, calls the appropriate use‑case, and returns a generic HTTP response object. No business logic resides in the endpoint.
- Use‑cases encapsulate business logic only and depend on providers; they do not depend on any specific web framework types.
