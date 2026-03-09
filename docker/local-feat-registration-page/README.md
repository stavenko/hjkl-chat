# docker/local-feat-registration-page

Infrastructure services for the `feat-registration-page` branch. Ports are offset by +3 from `docker/local` to avoid conflicts.

## Services

| Service | Port | Purpose |
|---------|------|---------|
| MinIO | 9004 (API), 9005 (Console) | S3-compatible object storage |
| MailHog | 1028 (SMTP), 8028 (Web UI) | SMTP mock for email capture |
| config-server | 8093 | Serves `config.json` to the frontend via Trunk proxy |

## Usage

```bash
# 1. Start services
docker compose -f docker/local-feat-registration-page/docker-compose.yml up -d

# 2. Run backend
cargo run -p backend -- --config docker/local-feat-registration-page/config.toml

# 3. Run frontend (in another terminal, uses branch-specific Trunk.toml)
cd frontend && trunk serve --config ../docker/local-feat-registration-page/Trunk.toml
```

Frontend is available at `http://localhost:8083`. Trunk proxies `/api/*` to the backend on port 5003 and `/config.json` to the config-server on port 8093.

## Config files

- `config.toml` — backend configuration pointing to localhost services (ports +3)
- `config.json` — served by nginx to the frontend via Trunk proxy
- `nginx.conf` — nginx config for the config-server
- `Trunk.toml` — frontend dev server config (port 8083, proxies to backend 5003 and config-server 8093)
