# Dev Setup

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) + Docker Compose
- [Node.js](https://nodejs.org/) (for proto generation only)
- [Rust](https://rustup.rs/) (for proto generation only)

---

## First-time setup

### 1. Generate RSA keys

The auth service signs JWTs with an RSA private key. The other services verify tokens with the public key.

```bash
cd scripts
sh keys.sh
```

This generates `private.key` / `public.key` and copies them to the right places:
- `private.key` → `client/src/lib/server/private.key`
- `public.key` → `service-users/`, `service-notes/`, `service-utils/`

You only need to do this once (unless you rotate keys intentionally).

### 2. Get your OAuth credentials

You need at least one OAuth provider. Create an app at:
- **Google**: [console.cloud.google.com](https://console.cloud.google.com) → APIs & Services → Credentials
- **GitHub**: [github.com/settings/developers](https://github.com/settings/developers) → OAuth Apps

Set the callback URL to `http://localhost:8090/oauth-callback/<provider>` (e.g. `http://localhost:8090/oauth-callback/google`).

### 3. (Optional) External services

| Service | Used for | Env var(s) |
|---------|----------|------------|
| [Stripe](https://stripe.com) | Subscription payments | `STRIPE_API_KEY`, `STRIPE_PRICE_ID` |
| [SendGrid](https://sendgrid.com) | Email sending | `SENDGRID_API_KEY` |
| S3-compatible storage | File uploads | `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `S3_ENDPOINT` |

These are optional — the app starts without them, those features just won't work.

---

## Running the app

### Step 1 — Start the databases

```bash
docker compose -f docker-compose.db.yml up -d
```

This starts three PostgreSQL instances:
- `db-users` on port `5432`
- `db-notes` on port `5433`
- `db-utils` on port `5434`

### Step 2 — Start the services

```bash
JWT_SECRET=any_random_string \
GOOGLE_CLIENT_ID=your_google_client_id \
GOOGLE_CLIENT_SECRET=your_google_client_secret \
GITHUB_CLIENT_ID=your_github_client_id \
GITHUB_CLIENT_SECRET=your_github_client_secret \
SENDGRID_API_KEY=optional \
STRIPE_API_KEY=optional \
STRIPE_PRICE_ID=optional \
S3_ACCESS_KEY=optional \
S3_SECRET_KEY=optional \
S3_ENDPOINT=optional \
docker compose -f docker-compose.app.yml up
```

Or put your secrets in a `.env` file at the project root and run:

```bash
docker compose -f docker-compose.app.yml up
```

Example `.env`:
```
JWT_SECRET=supersecret
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
GITHUB_CLIENT_ID=
GITHUB_CLIENT_SECRET=
SENDGRID_API_KEY=
STRIPE_API_KEY=
STRIPE_PRICE_ID=
S3_ACCESS_KEY=
S3_SECRET_KEY=
S3_ENDPOINT=
```

### Step 3 — Open the app

| URL | What |
|-----|------|
| `http://localhost:3000` | SvelteKit frontend |
| `http://localhost:8090` | Auth service (OAuth callbacks) |

---

## Hot reload

All services mount their `src/` directory as a Docker volume:

- **SvelteKit** (`client/src/`) — Vite HMR, changes reflect instantly in the browser
- **Rust services** — use `cargo-watch`, recompile on file save (takes a few seconds)

---

## Regenerating protobuf types

After editing any `.proto` file in `proto/`:

```bash
sh proto.sh
```

This regenerates TypeScript types in `client/src/lib/proto/` and Rust types in each service's `src/proto/`. Both Rust and TypeScript will fail to compile if a field is missing, so do this before starting the services after any proto change.

---

## Useful commands

```bash
# View logs for a specific service
docker compose -f docker-compose.app.yml logs -f client
docker compose -f docker-compose.app.yml logs -f service-auth

# Restart a single service
docker compose -f docker-compose.app.yml restart service-notes

# Stop everything
docker compose -f docker-compose.app.yml down
docker compose -f docker-compose.db.yml down

# Stop and wipe databases
docker compose -f docker-compose.db.yml down -v
```
