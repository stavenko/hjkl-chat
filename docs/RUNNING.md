# Running hjkl-chat

Three Docker environments under `docker/`, each for a different workflow.

## Environments

| Environment | Docker runs | Host runs | Use case |
|-------------|-------------|-----------|----------|
| **test** | MinIO, MailHog | `cargo test` | Integration tests |
| **local** | MinIO, MailHog, config-server | `cargo run`, `trunk serve` | Day-to-day development |
| **dev** | Everything (backend, frontend, MinIO, MailHog) | Nothing | Full stack in Docker |

---

## Local development

Best for iterating on code with hot-reload.

```bash
# Start infrastructure
docker compose -f docker/local/docker-compose.yml up -d

# Terminal 1 — backend
cargo run -p backend -- --config docker/local/config.toml

# Terminal 2 — frontend
cd frontend && trunk serve
```

| URL | What |
|-----|------|
| http://localhost:8080 | Frontend (Trunk dev server) |
| http://localhost:5000 | Backend API (direct) |
| http://localhost:8025 | MailHog web UI |
| http://localhost:9001 | MinIO console |

Trunk proxies `/api/*` requests to the backend and `/config.json` to the config-server, configured in `frontend/Trunk.toml`.

---

## Running tests

```bash
# Start test infrastructure
docker compose -f docker/test/docker-compose.yml up -d

# Run tests
cargo test

# Tear down
docker compose -f docker/test/docker-compose.yml down
```

---

## Full Docker stack (dev)

Runs everything in containers. Requires building the frontend first.

```bash
# Build frontend
cd frontend && trunk build --release && cd ..

# Start all services
docker compose -f docker/dev/docker-compose.yml up -d
```

| URL | What |
|-----|------|
| http://localhost:3000 | Frontend (nginx) |
| http://localhost:5000 | Backend API |
| http://localhost:8025 | MailHog web UI |
| http://localhost:9001 | MinIO console |

---

## Stopping services

```bash
docker compose -f docker/<env>/docker-compose.yml down
```

Add `-v` to also remove volumes (resets MinIO data).
