# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rusve is a monorepo full-stack template demonstrating a production-grade architecture: five Rust microservices behind a Caddy reverse proxy, communicating via gRPC (Protobuf), with a SvelteKit frontend. Real-world usage: [UpSend.app](https://www.upsend.app).

## Repository Layout

```
services/           # Rust microservices
  service-auth/     # OAuth2 + JWT + local email/password (HTTP/Axum on :8090)
  service-users/    # User profiles + Stripe (gRPC on :50051)
  service-notes/    # Notes CRUD (gRPC on :50052)
  service-utils/    # Files (S3) + Email (gRPC on :50053)
  service-scraper/  # Scraping pipeline admin API (gRPC on :50054)
clients/webapp/     # SvelteKit frontend (:3000 dev, :8080 via Caddy)
proto/              # Protobuf definitions (shared source of truth)
scrapers/           # Python scraper CLI (httpx + BeautifulSoup, no Chromium)
telemetry/          # OTel Collector, Prometheus, Grafana configs
scripts/            # RSA key generation, Capacitor mobile setup
documentation/      # Architecture guides (scraping-pipeline.md, etc.)
```

## Common Commands

All orchestration goes through `make` at the project root:

```bash
make db           # Start all PostgreSQL containers (run first)
make stop-db      # Stop databases
make dev          # Run all 5 services in debug mode
make watch        # Hot-reload all services (requires cargo-watch)
make client       # Run SvelteKit frontend only
make release      # Build & run release binaries
make otel         # Start observability stack (Zipkin/Prometheus/Grafana)
make stop-otel    # Stop telemetry
```

### Frontend (SvelteKit)
```bash
cd clients/webapp
pnpm run dev              # Dev server
pnpm run build            # Production build
pnpm run check            # Svelte type-check
pnpm run lint             # ESLint (max-warnings=0)
pnpm run test:unit        # Vitest
pnpm run test:integration # Playwright
```

### Rust Services
```bash
cd services/service-<name>
cargo run                 # Debug run
cargo watch -x run        # Hot reload
cargo test                # Unit tests
cargo clippy -- -D warnings  # Lint
cargo fmt --check         # Format check
```

### Proto Regeneration
```bash
sh proto.sh   # Regenerates Protobuf bindings for all services and the client
```

The `proto.sh` script fixes were applied: it now removes `proto.rs` files (not directories), the npm script uses `npx`, and all output paths point to `services/service-X/src/`.

### Python Scraper
```bash
cd scrapers
pip install -r requirements.txt
SCRAPING_DB_URL=postgresql://postgres:12345@localhost:5441/scraping \
  python run.py --url https://example.com --type article
# API endpoint mode:
  python run.py --url https://api.example.com/items --type product --api
```

## Architecture

### Request Flow
```
Browser/Mobile → Caddy (:8080/:443)
  /oauth-*    → service-auth  (HTTP/Axum)
  everything  → clients/webapp (SvelteKit SSR)
                  ↓ gRPC calls directly from SvelteKit server
              service-users / service-notes / service-utils / service-scraper
```

### Auth Flow
1. `service-auth` generates PKCE challenge + CSRF token (stored in PostgreSQL), redirects to OAuth provider (Google/GitHub)
2. Provider redirects back; `service-auth` verifies PKCE, exchanges code, creates/updates user, issues a JWT
3. JWT sent to client as URL param (web) or deep link (mobile via `mobile:` prefix in state)
4. Client stores JWT as HttpOnly cookie; all subsequent gRPC calls include it as metadata
5. `service-users` validates the JWT on every gRPC request

Local email/password auth: `service-auth` handles `/local-register` and `/local-login` (Argon2id hashing). Credentials stored in `local_credentials` table in `db-users`. A dummy hash is pre-computed at startup to prevent email enumeration via timing.

### gRPC / Protobuf
- `proto/main.proto` is the single source of truth for all service contracts
- Additional message types split into separate `.proto` files (`notes.proto`, `scraper.proto`, etc.) — all use `package proto`
- Rust services use `tonic` + `prost`; the frontend uses `@grpc/grpc-js` + `@grpc/proto-loader`
- Run `sh proto.sh` after any `.proto` change

### Databases
Five separate PostgreSQL 18 instances (custom image `ceteuf/postgres-extended:pg18-alpine-v3`):
- `users` on port **5438** — auth + profiles + stripe
- `notes` on port **5439** — notes
- `utils` on port **5440** — files + emails
- `scraping` on port **5441** — scrape_sources, scrape_jobs, scrape_items (staging)
- `data` on port **5442** — data_items (validated + pushed data)

### Scraping Pipeline
See `documentation/scraping-pipeline.md` for full architecture. Summary:

```
scrapers/run.py  →  db_scraping (5441)  →  service-scraper  →  db_data (5442)
                     (staging, raw)         (gRPC admin API)     (approved data)
                                                  ↑
                                        SvelteKit /admin/scraper
                                        (role = ADMIN only)
```

- **No Chromium** — scraper uses `httpx` + `BeautifulSoup` for HTML or direct JSON API calls
- **Two integration modes per source:** `MANUAL` (admin approval required) or `AUTO` (cron-scheduled push if `field_rules` pass)
- **`field_rules` JSONB** on each source defines validation constraints (required, format, type, min_length, min/max)
- **Scheduler:** `tokio-cron-scheduler` — AUTO sources are registered as cron jobs at `service-scraper` startup; reloaded on `UpdateSource`
- **Validation** is triggered by `ApproveAllValid` RPC — PENDING items are validated against field_rules before being set to VALID/INVALID
- **`service-scraper` Env:** `PORT`, `RUST_LOG`, `DATABASE_URL` (db_scraping), `DATA_DATABASE_URL` (db_data), `JWT_SECRET`

### Admin UI
Routes under `src/routes/(admin)/` are protected by `role = ADMIN (2)` check in the layout. Structure:
```
/admin/scraper                          # jobs dashboard + sources overview
/admin/scraper/sources                  # list + create sources
/admin/scraper/sources/[sourceId]       # edit mode/schedule/field_rules
/admin/scraper/[jobId]                  # items preview + approve/reject/push
```

### Docker Compose Files
| File | Purpose |
|------|---------|
| `docker-compose.db.yml` | All databases (5438–5442) |
| `docker-compose.app.yml` | All services + Caddy (dev, with hot-reload mounts) |
| `docker-compose.otel.yml` | Telemetry stack |
| `docker-compose.prod.yml` | Full production stack |

## Key Environment Variables

```
# OAuth
GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET
GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET

# Core
JWT_SECRET, DATABASE_URL, CLIENT_URL, AUTH_URL, PORT, RUST_LOG

# service-scraper specific
DATA_DATABASE_URL   # connection to db_data (port 5442)
SCRAPER_URI         # consumed by SvelteKit (e.g. localhost:50054)

# Python scraper
SCRAPING_DB_URL     # e.g. postgresql://postgres:12345@localhost:5441/scraping

# Optional features
STRIPE_API_KEY, STRIPE_PRICE_ID
SENDGRID_API_KEY
S3_ACCESS_KEY, S3_SECRET_KEY, S3_ENDPOINT
OTEL_EXPORTER_OTLP_ENDPOINT  # defaults to localhost:4317
```

RSA keys for JWT signing are generated by `scripts/keys.sh` and expected at `scripts/private.key` / `scripts/public.key`.

## Notable Patterns

- **Protobuf-first:** All cross-service types flow from `proto/main.proto`; never duplicate types manually
- **Per-service `Env` struct:** Each service has a `lib.rs` with an `Env` struct loaded from env vars at startup — add new config there
- **SvelteKit calls gRPC directly:** No REST/BFF layer; server-side SvelteKit routes (`+page.server.ts`, `+server.ts`) call gRPC services using `@grpc/grpc-js`
- **Streaming:** gRPC server-streaming used for list operations; bidirectional streaming for file upload/download in `service-utils`
- **`grpcSafe` pattern:** Unary calls use `new Promise((r) => service.Method(req, meta, grpcSafe(r)))`; streaming uses `.on("data" / "error" / "end")`
- **Two-pool services:** `service-scraper` holds two `deadpool_postgres::Pool` — one for `db_scraping`, one for `db_data`
- **Mobile:** Capacitor config lives at `clients/webapp/capacitor.config.cjs` (the `.ts` file is a stale placeholder — ignore it)
- **CI:** GitHub Actions in `.github/workflows/` runs `cargo clippy` + `cargo check` per service and ESLint + `svelte-check` for the client on every PR
