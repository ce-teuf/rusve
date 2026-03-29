# Rusve = Rust + Svelte + PostgreSQL + gRPC

## What is Rusve? 

It is a attempt to find the best way to build **fast** and **scalable** web applications, while not beeing afraid of new technologies.

Feel free to ask questions, throw new ideas and propose changes. Any contribution is also welcome, especially some ux/ui redesigns.

Also, a little bit of self-promotion, i am building an [application](https://www.upsend.app) using this stack. Its goal is to take care of **files**, **images**, and **emails** for you. Feel free to give it a try, as it's free :)

## Alternative
If you need something a little more simple (Go, SQLite, server deployment), feel free to check out the second project I am running:
**[SGSG](https://github.com/mpiorowski/sgsg)**

## Currently working on...
- Telemetry
- Comment whole codebase
- TESTS!
- **Any other feature You will request :)**

## Architecture
- **[Rust](https://www.rust-lang.org/)** - Amazing language, not easy to pick up, but one that can give one of the best performance and safety on the market.
- **Modules** - Some people might call them microservices. Splitted into smaller parts, very easy to scale, and allows using any combination of languages and databases.
- **[SvelteKit](https://kit.svelte.dev/)** - Svelte currently is what I believe the best frontend framework. If you've never worked with it, don't worry; it's super easy to pick up.
As an example, developers from my team who were familiar with NextJS were able to grasp it in just one week and start coding web apps. Trust me, once you try it, it's hard to go back to anything else.
- **[gRPC](https://grpc.io/)** - Connection between services using gRPC, which is very fast and gives an option for bi-directional streaming:
    - **[Typesafety](https://protobuf.dev/)** - Thanks to protobuf, there is amazing type safety across the whole project, regardless of the language (not only for TypeScript, hi tRPC). Trust me; this is phenomenal.
  If you add one "field" to your User object, both JavaScript and Rust will lint, pointing out exactly where you need to take care of it. Adding a new language like Java or Go? Type safety for them as well.
    - **[Streaming](https://grpc.io/docs/what-is-grpc/core-concepts/#server-streaming-rpc)** - gRPC allows streaming data, which, for larger datasets, offers incredible performance.
- **[Caddy](https://caddyserver.com/)** - Reverse proxy handling HTTPS automatically via Let's Encrypt. Single entry point routing traffic to SvelteKit and the auth service, with zero TLS configuration needed.

![image](https://github.com/mpiorowski/rusve/assets/26543876/aa648032-8bf5-4039-ad88-15780ac36fea)
 
## Additional features
- **Stripe Subscription** - Fully working subscription flow.
- **S3 file storage** - Functionality for storing, deleting, and downloading files from any S3-compatible API.
- **SendGrid email sending** - Email sending with just one SendGrid API key.
- **No TypeScript Build, Fully Typed with JSDocs** - Despite the absence of a TypeScript build, the code remains fully typed using JSDocs. While this approach may be somewhat controversial due to requiring more lines of code, the more I work with pure JSDocs, the more I appreciate its versatility.
It supports features like Enums, as const, and even Zod's z.infer<typeof User>, eliminating the need for the entire TypeScript build step.
- **Very Secure OAuth Implementation** - Utilizes the Phantom Token Approach with additional client-to-server authorization using an RSA key, ensuring robust security.
- **Minimal External Libraries** - Emphasizes a minimalistic approach to external libraries. From my experience, relying less on external dependencies contributes to code maintainability. This approach makes it easier to make changes even after years. It's simply the way to go.
- **Single Source of Truth Validation** - Centralizing validation on the backend simplifies logic, streamlining error checks, and ensuring a single, authoritative source for error management. Displaying these errors on the frontend remains efficient, delivering a seamless user experience.
- **Performance and Error Logging with Grafana Integration** - Efficiently log performance metrics and errors within the application, consolidating data for streamlined analysis. Utilize Grafana integration to visualize and monitor performance calls and errors, facilitating proactive management and optimization.
- **Docker for Seamless Deployment** - Leverage Docker for consistent deployment across both development and production environments. Streamline server deployment by encapsulating the application and its dependencies in containers, ensuring easy setup and scalability while maintaining environment consistency.
- **GitHub Actions for Automated Workflow** - GitHub Actions automate linting and code checks on every pull request, ensuring code quality across all services.
- **Client side streaming** - Thanks to SvelteKit's newest feature, we can load and render crucial data first. Subsequently, all remaining data is returned as promises and rendered when they resolve.
- **Files, Images and Emails** - A little bit of self promotion, this application is using my another dead simple service (free) for managing files, images and emails - [UpSend](https://www.upsend.app)

## Aria and PWA with offline service workers
![image](https://user-images.githubusercontent.com/26543876/236647026-0db54439-b841-4e69-8a2f-6976e423b453.png)

## Proto

Whenever You change proto definitions, always remember to generate new types:
```
sh proto.sh
```

## Deployment

The only prerequisites are `Docker` and `Docker Compose`.

### Development

1. Start databases:
```
docker compose -f docker-compose.db.yml up
```

2. Start client + services:
```
JWT_SECRET=JWT_SECRET \
GOOGLE_CLIENT_ID=GOOGLE_CLIENT_ID \
GOOGLE_CLIENT_SECRET=GOOGLE_CLIENT_SECRET \
GITHUB_CLIENT_ID=GITHUB_CLIENT_ID \
GITHUB_CLIENT_SECRET=GITHUB_CLIENT_SECRET \
SENDGRID_API_KEY=SENDGRID_API_KEY \
UPSEND_KEY=UPSEND_KEY \
STRIPE_API_KEY=STRIPE_API_KEY \
STRIPE_PRICE_ID=STRIPE_PRICE_ID \
S3_ACCESS_KEY=S3_ACCESS_KEY \
S3_SECRET_KEY=S3_SECRET_KEY \
S3_ENDPOINT=S3_ENDPOINT \
docker compose -f docker-compose.app.yml up
```

### Production (VPS)

**Prerequisites**: A VPS with Docker + Docker Compose, and a domain pointing to it.

1. Edit `Caddyfile` and `docker-compose.prod.yml` — replace `yourdomain.com` with your actual domain.

2. On the VPS, create a `.env` file with your secrets:
```
JWT_SECRET=
DB_PASSWORD=
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
GITHUB_CLIENT_ID=
GITHUB_CLIENT_SECRET=
SENDGRID_API_KEY=
UPSEND_KEY=
STRIPE_API_KEY=
STRIPE_PRICE_ID=
S3_ACCESS_KEY=
S3_SECRET_KEY=
S3_ENDPOINT=
```

3. Start databases:
```
docker compose -f docker-compose.db.yml up -d
```

4. Build and start all services + Caddy:
```
docker compose -f docker-compose.prod.yml up -d --build
```

Caddy automatically obtains a TLS certificate from Let's Encrypt on the first request. HTTP is redirected to HTTPS automatically.
