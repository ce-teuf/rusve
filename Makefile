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
#   make client      Run only the SvelteKit client
#   make dev         Run all services in debug mode
#   make watch       Run all services in watch mode (recompile on save)
#   make release     Build and run all services in release mode
# ============================================================

-include .env
export

# ── Ports ───────────────────────────────────────────────────
AUTH_PORT  ?= 8090
USERS_PORT ?= 50051
NOTES_PORT ?= 50052
UTILS_PORT ?= 50053

# ── Database URLs (quoted so & is not treated as shell bg op)
DB_USERS_URL = 'postgresql://?host=localhost&port=5432&user=postgres&password=12345&dbname=users'
DB_NOTES_URL = 'postgresql://?host=localhost&port=5433&user=postgres&password=12345&dbname=notes'
DB_UTILS_URL = 'postgresql://?host=localhost&port=5434&user=postgres&password=12345&dbname=users'

# ── Env per service (single-line for safe shell expansion) ──
ENV_AUTH  = PORT=$(AUTH_PORT) RUST_LOG=info DATABASE_URL=$(DB_USERS_URL) CLIENT_URL=http://localhost:3000 AUTH_URL=http://localhost:$(AUTH_PORT) USERS_URL=http://localhost:$(USERS_PORT) GOOGLE_CLIENT_ID=$(GOOGLE_CLIENT_ID) GOOGLE_CLIENT_SECRET=$(GOOGLE_CLIENT_SECRET) GITHUB_CLIENT_ID=$(GITHUB_CLIENT_ID) GITHUB_CLIENT_SECRET=$(GITHUB_CLIENT_SECRET) JWT_SECRET=$(JWT_SECRET)
ENV_USERS = PORT=$(USERS_PORT) RUST_LOG=info DATABASE_URL=$(DB_USERS_URL) CLIENT_URL=http://localhost:3000 JWT_SECRET=$(JWT_SECRET) STRIPE_API_KEY=$(STRIPE_API_KEY) STRIPE_PRICE_ID=$(STRIPE_PRICE_ID)
ENV_NOTES = PORT=$(NOTES_PORT) RUST_LOG=info DATABASE_URL=$(DB_NOTES_URL) USERS_URL=http://localhost:$(USERS_PORT) JWT_SECRET=$(JWT_SECRET)
ENV_UTILS = PORT=$(UTILS_PORT) RUST_LOG=info DATABASE_URL=$(DB_UTILS_URL) SENDGRID_API_KEY=$(SENDGRID_API_KEY) S3_ACCESS_KEY=$(S3_ACCESS_KEY) S3_SECRET_KEY=$(S3_SECRET_KEY) S3_ENDPOINT=$(S3_ENDPOINT) S3_BUCKET_NAME=rusve JWT_SECRET=$(JWT_SECRET)

.PHONY: db stop-db dev watch release client-env client

# ── Databases ────────────────────────────────────────────────
db:
	docker compose -f docker-compose.db.yml up -d

stop-db:
	docker compose -f docker-compose.db.yml down

# ── Write client/.env for local URIs ────────────────────────
client-env:
	@printf 'USERS_URI=localhost:%s\nNOTES_URI=localhost:%s\nUTILS_URI=localhost:%s\nGRPC_SSL=false\nENV=development\nCOOKIE_DOMAIN=localhost\nPUBLIC_AUTH_URL=http://localhost:%s\nJWT_SECRET=%s\nUPSEND_KEY=%s\n' \
		$(USERS_PORT) $(NOTES_PORT) $(UTILS_PORT) $(AUTH_PORT) "$(JWT_SECRET)" "$(UPSEND_KEY)" \
		> client/.env
	@echo "client/.env written"

# ── Client only ─────────────────────────────────────────────
client: client-env
	cd client && pnpm run dev

# ── Debug ────────────────────────────────────────────────────
dev: client-env
	@trap 'kill 0' INT TERM; \
	(cd service-auth  && $(ENV_AUTH)  cargo run) & \
	(cd service-users && $(ENV_USERS) cargo run) & \
	(cd service-notes && $(ENV_NOTES) cargo run) & \
	(cd service-utils && $(ENV_UTILS) cargo run) & \
	(cd client        && pnpm run dev) & \
	wait

# ── Watch (recompile on save) ────────────────────────────────
watch: client-env
	@trap 'kill 0' INT TERM; \
	(cd service-auth  && $(ENV_AUTH)  cargo watch -x run) & \
	(cd service-users && $(ENV_USERS) cargo watch -x run) & \
	(cd service-notes && $(ENV_NOTES) cargo watch -x run) & \
	(cd service-utils && $(ENV_UTILS) cargo watch -x run) & \
	(cd client        && pnpm run dev) & \
	wait

# ── Release ──────────────────────────────────────────────────
release: client-env
	cargo build --release --manifest-path service-auth/Cargo.toml
	cargo build --release --manifest-path service-users/Cargo.toml
	cargo build --release --manifest-path service-notes/Cargo.toml
	cargo build --release --manifest-path service-utils/Cargo.toml
	cd client && pnpm run build
	@trap 'kill 0' INT TERM; \
	($(ENV_AUTH)  ./service-auth/target/release/service-auth)   & \
	($(ENV_USERS) ./service-users/target/release/service-users) & \
	($(ENV_NOTES) ./service-notes/target/release/service-notes) & \
	($(ENV_UTILS) ./service-utils/target/release/service-utils) & \
	(cd client && node build) & \
	wait
