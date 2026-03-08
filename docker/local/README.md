# docker/local

Infrastructure services for local development. Backend and frontend run on the host machine; only dependencies run in Docker.

## Services

| Service | Port | Purpose |
|---------|------|---------|
| MinIO | 9000 (API), 9001 (Console) | S3-compatible object storage |
| MailHog | 1025 (SMTP), 8025 (Web UI) | SMTP mock for email capture |
| config-server | 8090 | Serves `config.json` to the frontend via Trunk proxy |

## Usage

```bash
# 1. Start services
docker compose -f docker/local/docker-compose.yml up -d

# 2. Run backend
cargo run -p backend -- --config docker/local/config.toml

# 3. Run frontend (in another terminal)
cd frontend && trunk serve
```

Frontend is available at `http://localhost:8080`. Trunk proxies `/api/*` to the backend on port 5000 and `/config.json` to the config-server on port 8090.

## Config files

- `config.toml` — backend configuration pointing to localhost services
- `config.json` — served by nginx to the frontend via Trunk proxy
- `nginx.conf` — nginx config for the config-server
