# docker/dev

Runs the entire application stack in Docker. No host toolchain required beyond Docker itself.

## Services

| Service | Port | Purpose |
|---------|------|---------|
| backend | 5000 | Rust backend (built in Docker) |
| frontend | 3000 | nginx serving the frontend dist |
| MinIO | 9000 (API), 9001 (Console) | S3-compatible object storage |
| MailHog | 1025 (SMTP), 8025 (Web UI) | SMTP mock for email capture |

## Prerequisites

Build the frontend dist before starting:

```bash
cd frontend && trunk build --release
```

A `Dockerfile.backend` is required at `docker/dev/Dockerfile.backend` to build the backend image.

## Usage

```bash
docker compose -f docker/dev/docker-compose.yml up -d
```

Frontend is available at `http://localhost:3000`, backend API at `http://localhost:5000`.

## Config files

- `config.toml` — backend configuration using Docker network addresses
- `config.json` — frontend config (api_base_url)
