SHELL := /bin/bash

# ============================================================
# Rusve — local development (DBs in Docker, services on host)
# ============================================================
#
# Prerequisites:
#   - Docker + Docker Compose  (databases)
#   - Rust + cargo             (services)
#   - cargo-watch              (watch mode: cargo install cargo-watch)
#   - Node.js + npm            (client)
#
# Setup:
#   1. cp .env.example .env    fill in secrets
#   2. make db                 start databases
#   3. make dev / watch / release
#
# Targets:
#   make db          Start databases in Docker
#   make stop-db     Stop databases
#   make otel        Start observability stack (OTel Collector, Zipkin, Prometheus, Grafana)
#   make stop-otel   Stop observability stack
#   make client      Run only the SvelteKit client
#   make dev         Run all services in debug mode
#   make watch       Run all services in watch mode (recompile on save)
#   make release     Build and run all services in release mode
# ============================================================

-include .env
export

# ── Ports ───────────────────────────────────────────────────
AUTH_PORT    ?= 8090
USERS_PORT   ?= 50051
NOTES_PORT   ?= 50052
UTILS_PORT   ?= 50053
SCRAPER_PORT ?= 50054

# ── Database URLs (quoted so & is not treated as shell bg op)
DB_USERS_URL    = 'postgresql://?host=localhost&port=5438&user=postgres&password=12345&dbname=users'
DB_NOTES_URL    = 'postgresql://?host=localhost&port=5439&user=postgres&password=12345&dbname=notes'
DB_UTILS_URL    = 'postgresql://?host=localhost&port=5440&user=postgres&password=12345&dbname=users'
DB_SCRAPING_URL = 'postgresql://?host=localhost&port=5441&user=postgres&password=12345&dbname=scraping'
DB_DATA_URL     = 'postgresql://?host=localhost&port=5442&user=postgres&password=12345&dbname=data'

# ── OTel collector endpoint (local collector Docker container) ──
OTEL_ENDPOINT ?= http://localhost:4317

# ── Env per service (single-line for safe shell expansion) ──
ENV_AUTH  = PORT=$(AUTH_PORT) RUST_LOG=info DATABASE_URL=$(DB_USERS_URL) CLIENT_URL=http://localhost:8080 AUTH_URL=http://localhost:8080 USERS_URL=http://localhost:$(USERS_PORT) UTILS_URL=http://localhost:$(UTILS_PORT) GOOGLE_CLIENT_ID=$(GOOGLE_CLIENT_ID) GOOGLE_CLIENT_SECRET=$(GOOGLE_CLIENT_SECRET) GITHUB_CLIENT_ID=$(GITHUB_CLIENT_ID) GITHUB_CLIENT_SECRET=$(GITHUB_CLIENT_SECRET) JWT_SECRET=$(JWT_SECRET) OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_ENDPOINT) OTEL_SERVICE_NAME=service-auth
ENV_USERS = PORT=$(USERS_PORT) RUST_LOG=info DATABASE_URL=$(DB_USERS_URL) CLIENT_URL=http://localhost:8080 JWT_SECRET=$(JWT_SECRET) STRIPE_API_KEY=$(STRIPE_API_KEY) STRIPE_PRICE_ID=$(STRIPE_PRICE_ID) OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_ENDPOINT) OTEL_SERVICE_NAME=service-users
ENV_NOTES = PORT=$(NOTES_PORT) RUST_LOG=info DATABASE_URL=$(DB_NOTES_URL) USERS_URL=http://localhost:$(USERS_PORT) JWT_SECRET=$(JWT_SECRET) OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_ENDPOINT) OTEL_SERVICE_NAME=service-notes
ENV_UTILS    = PORT=$(UTILS_PORT) RUST_LOG=info DATABASE_URL=$(DB_UTILS_URL) SENDGRID_API_KEY=$(SENDGRID_API_KEY) SMTP_HOST=$(SMTP_HOST) SMTP_PORT=$(SMTP_PORT) SMTP_USERNAME=$(SMTP_USERNAME) SMTP_PASSWORD=$(SMTP_PASSWORD) SMTP_FROM_EMAIL=$(SMTP_FROM_EMAIL) SMTP_FROM_NAME=$(SMTP_FROM_NAME) S3_ACCESS_KEY=$(S3_ACCESS_KEY) S3_SECRET_KEY=$(S3_SECRET_KEY) S3_ENDPOINT=$(S3_ENDPOINT) S3_BUCKET_NAME=rusve JWT_SECRET=$(JWT_SECRET) OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_ENDPOINT) OTEL_SERVICE_NAME=service-utils
ENV_SCRAPER  = PORT=$(SCRAPER_PORT) RUST_LOG=info DATABASE_URL=$(DB_SCRAPING_URL) DATA_DATABASE_URL=$(DB_DATA_URL) JWT_SECRET=$(JWT_SECRET) OTEL_EXPORTER_OTLP_ENDPOINT=$(OTEL_ENDPOINT) OTEL_SERVICE_NAME=service-scraper

.PHONY: db stop-db wait-db otel stop-otel dev watch release client-env client

# ── Databases ────────────────────────────────────────────────
db:
	docker compose -f docker-compose.db.yml up -d

stop-db:
	docker compose -f docker-compose.db.yml down

# ── Observability stack ──────────────────────────────────────
otel:
	docker compose -f docker-compose.otel.yml up -d

stop-otel:
	docker compose -f docker-compose.otel.yml down

wait-db:
	@echo "Waiting for databases to be ready..."
	@for svc in rusve-db-users rusve-db-notes rusve-db-utils rusve-db-scraping rusve-db-data; do \
		until [ "$$(docker inspect --format='{{.State.Health.Status}}' $$svc 2>/dev/null)" = "healthy" ]; do \
			echo "  $$svc not ready, retrying..."; sleep 2; \
		done; \
		echo "  $$svc healthy"; \
	done

# ── Write clients/webapp/.env for local URIs ────────────────────────
client-env:
	@printf 'USERS_URI=localhost:%s\nNOTES_URI=localhost:%s\nUTILS_URI=localhost:%s\nSCRAPER_URI=localhost:%s\nGRPC_SSL=false\nENV=development\nCOOKIE_DOMAIN=localhost\nPUBLIC_AUTH_URL=http://localhost:8080\nJWT_SECRET=%s\nUPSEND_KEY=%s\n' \
		$(USERS_PORT) $(NOTES_PORT) $(UTILS_PORT) $(SCRAPER_PORT) "$(JWT_SECRET)" "$(UPSEND_KEY)" \
		> clients/webapp/.env
	@echo "clients/webapp/.env written"

# ── Client only ─────────────────────────────────────────────
client: client-env
	cd clients/webapp && pnpm run dev

# ── Debug ────────────────────────────────────────────────────
dev: wait-db client-env
	@trap 'kill 0' INT TERM; \
	(cd services/service-auth    && $(ENV_AUTH)    cargo run) & \
	(cd services/service-users   && $(ENV_USERS)   cargo run) & \
	(cd services/service-notes   && $(ENV_NOTES)   cargo run) & \
	(cd services/service-utils   && $(ENV_UTILS)   cargo run) & \
	(cd services/service-scraper && $(ENV_SCRAPER) cargo run) & \
	(cd clients/webapp           && pnpm run dev) & \
	wait

# ── Watch (recompile on save) ────────────────────────────────
watch: wait-db client-env
	@trap 'kill 0' INT TERM; \
	(cd services/service-auth    && $(ENV_AUTH)    cargo watch -x run) & \
	(cd services/service-users   && $(ENV_USERS)   cargo watch -x run) & \
	(cd services/service-notes   && $(ENV_NOTES)   cargo watch -x run) & \
	(cd services/service-utils   && $(ENV_UTILS)   cargo watch -x run) & \
	(cd services/service-scraper && $(ENV_SCRAPER) cargo watch -x run) & \
	(cd clients/webapp           && pnpm run dev) & \
	wait

# ── Release ──────────────────────────────────────────────────
release: wait-db client-env
	cargo build --release --manifest-path services/service-auth/Cargo.toml
	cargo build --release --manifest-path services/service-users/Cargo.toml
	cargo build --release --manifest-path services/service-notes/Cargo.toml
	cargo build --release --manifest-path services/service-utils/Cargo.toml
	cargo build --release --manifest-path services/service-scraper/Cargo.toml
	cd clients/webapp && pnpm run build
	@trap 'kill 0' INT TERM; \
	($(ENV_AUTH)    ./services/service-auth/target/release/service-auth)       & \
	($(ENV_USERS)   ./services/service-users/target/release/service-users)     & \
	($(ENV_NOTES)   ./services/service-notes/target/release/service-notes)     & \
	($(ENV_UTILS)   ./services/service-utils/target/release/service-utils)     & \
	($(ENV_SCRAPER) ./services/service-scraper/target/release/service-scraper) & \
	(cd clients/webapp && node build) & \
	wait

