# docker/test

Infrastructure services for running integration tests.

## Services

| Service | Port | Purpose |
|---------|------|---------|
| MinIO | 9000 (API), 9001 (Console) | S3-compatible object storage |
| MailHog | 1025 (SMTP), 8025 (Web UI) | SMTP mock for email testing |

## Usage

```bash
docker compose -f docker/test/docker-compose.yml up -d
cargo test
docker compose -f docker/test/docker-compose.yml down
```

## Config files

- `config.toml` — backend configuration pointing to localhost services
- `config.json` — frontend config (api_base_url)
