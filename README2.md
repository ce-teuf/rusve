# What is Rusve?

Rusve is a **full-stack template** you can clone and build on. It ships with auth, user profiles, CRUD, file storage, email sending, and payments already wired together — so you start from something real instead of from scratch.

## What the app lets you do

- Sign in with Google or GitHub
- Edit your profile (name, bio, avatar, resume PDF)
- Create and manage notes (with a drawer-based detail view)
- Upload and download files (stored on S3)
- Send emails (via SendGrid)
- Subscribe and manage a Stripe subscription

## What each part does

| Directory | Responsibility |
|-----------|----------------|
| `client/` | SvelteKit frontend. Handles all pages, forms, and UI. Makes gRPC calls server-side only — the browser never talks directly to any Rust service. |
| `service-auth/` | OAuth 2.0 flow (Google, GitHub). Validates the OAuth callback, then issues a signed JWT. No database — stateless. |
| `service-users/` | Owns the user record, profile data, and Stripe subscription state. Has its own PostgreSQL database. |
| `service-notes/` | Notes CRUD. Stores notes scoped to the authenticated user. Has its own PostgreSQL database. |
| `service-utils/` | File uploads/downloads (S3) and email sending (SendGrid). Tracks metadata in its own PostgreSQL database. |
| `proto/` | Protobuf definitions shared by all services. This is the single source of truth for types — both Rust and TypeScript generate their types from here. After any change, run `sh proto.sh`. |

## How a request flows

```
Browser
  └── Caddy (HTTPS, public)
        ├── /oauth-*  →  service-auth
        └── /*        →  client (SvelteKit)
                            ├── gRPC → service-users  (Docker internal)
                            ├── gRPC → service-notes  (Docker internal)
                            └── gRPC → service-utils  (Docker internal)
```

The Rust services are never directly reachable from the internet. All external traffic goes through SvelteKit.
