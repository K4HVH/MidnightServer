# MidnightServer

A Rust gRPC backend template built with [tonic](https://github.com/hyperium/tonic), designed as the server counterpart to [MidnightUI](https://github.com/k4hvh/MidnightUI) (SolidJS frontend). Includes everything needed to start building production gRPC services: database, health checks, structured logging, error handling, and gRPC-Web support.

## Features

- **gRPC + gRPC-Web** — tonic server with `tonic-web` for browser clients and `tonic-reflection` for service discovery
- **PostgreSQL** — connection pool via SQLx with compile-time query checking and auto-migrations on startup
- **Health Checks** — active probe-based registry that periodically polls services (database, etc.) with configurable intervals and timeouts
- **Structured Logging** — four output styles (Plain, Compact, Pretty, JSON) with per-request tracing via `tower-http::TraceLayer`
- **Error Handling** — centralized `AppError` enum with `thiserror`, auto-mapped to gRPC status codes
- **Runtime Config** — lock-free `ArcSwap<Config>` for hot-reloading configuration
- **CORS** — configurable origins, permissive or explicit
- **Graceful Shutdown** — handles `Ctrl+C` and `SIGTERM`

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [protoc](https://grpc.io/docs/protoc-installation/) (Protocol Buffers compiler)
- [Docker](https://docs.docker.com/get-docker/) or [Podman](https://podman.io/) (for the dev database)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) (optional, for managing migrations)
  ```sh
  cargo install sqlx-cli --no-default-features --features postgres
  ```
- [grpcurl](https://github.com/fullstorydev/grpcurl) (optional, for testing)

## Quick Start

### 1. Start the database

```sh
# Docker
docker run -d --name midnight-postgres \
  -e POSTGRES_USER=midnight \
  -e POSTGRES_PASSWORD=midnight \
  -e POSTGRES_DB=midnight \
  -p 5432:5432 \
  postgres:17-alpine

# Or Podman
podman run -d --name midnight-postgres \
  -e POSTGRES_USER=midnight \
  -e POSTGRES_PASSWORD=midnight \
  -e POSTGRES_DB=midnight \
  -p 5432:5432 \
  postgres:17-alpine
```

### 2. Configure environment

```sh
cp .env.example .env
# Edit .env if you changed the database credentials
```

### 3. Run the server

```sh
cargo run
```

The server will connect to PostgreSQL, run any pending migrations, and start listening on `0.0.0.0:50051`.

### 4. Test it

```sh
# Aggregate health check
grpcurl -plaintext localhost:50051 midnightui.HealthService/Check

# Database health check
grpcurl -plaintext -d '{"service": "database"}' localhost:50051 midnightui.HealthService/Check

# List available services (reflection)
grpcurl -plaintext localhost:50051 list
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | *(required)* | PostgreSQL connection string |
| `LISTEN_ADDR` | `0.0.0.0:50051` | gRPC server bind address |
| `LOG_LEVEL` | `info` | Tracing filter (`debug`, `info`, `warn`, `error`, or [directives](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)) |
| `LOG_STYLE` | `auto` | Log format: `plain`, `compact`, `pretty`, `json`, or `auto` (pretty in debug, plain in release) |
| `CORS_ORIGINS` | `*` | Comma-separated allowed origins, or `*` for permissive |

## Project Structure

```
├── proto/midnightui/         # Protobuf service definitions
│   └── health.proto
├── migrations/               # SQLx migrations (auto-run on startup)
│   └── 20250101000000_initial.sql
├── src/
│   ├── main.rs               # Server entrypoint, wiring, shutdown
│   ├── core/
│   │   ├── config.rs         # Config struct, loaded from env
│   │   ├── db.rs             # Pool creation + migration runner
│   │   ├── error.rs          # AppError enum → gRPC Status mapping
│   │   ├── health.rs         # Probe-based HealthRegistry
│   │   ├── logging.rs        # 4-style tracing initialization
│   │   └── state.rs          # AppState (config, db, health, uptime)
│   ├── grpc/
│   │   └── health.rs         # HealthService gRPC handler
│   └── proto/                # Generated protobuf code (build.rs)
├── build.rs                  # Protobuf codegen via tonic-prost-build
├── Cargo.toml
├── Dockerfile                # Multi-stage production build
└── .env.example
```

## Database

### Migrations

Migrations live in `migrations/` and run automatically when the server starts via `sqlx::migrate!()`. To create a new migration:

```sh
sqlx migrate add <name>
```

This creates a new `.sql` file in `migrations/`. Write your SQL, then restart the server — it will apply pending migrations on startup.

### Connection Pool

The pool is created with `PgPoolOptions::new().max_connections(5)`. The `PgPool` is available throughout the application via `state.db()`.

## Adding a New gRPC Service

1. **Define the proto** — add a `.proto` file in `proto/midnight/`
2. **Rebuild** — `cargo build` runs `build.rs` which generates Rust code into `src/proto/generated/`
3. **Implement the handler** — create a file in `src/grpc/`, implement the generated trait
4. **Register the service** — add it to the `Server::builder()` chain in `main.rs`
5. **Register a health probe** (if applicable) — call `state.health().register(...)` in `main.rs`

## Health Check System

The `HealthRegistry` runs active probes rather than relying on stored status. Each registered service provides a check function that runs on a configurable interval:

- **Immediate probe** on registration (no waiting for first interval)
- **5-second timeout** per probe to prevent hanging
- **Failure messages** propagated to the health response
- **Aggregate check** (empty service name) returns `NOT_SERVING` if any service is down

Built-in probes:
- `server` — always healthy (60s interval)
- `database` — runs `SELECT 1` against the pool (30s interval)

## Docker

```sh
docker build -t midnight-server .
docker run -e DATABASE_URL=postgres://midnight:midnight@host.docker.internal:5432/midnight midnight-server
```

## Frontend Integration

MidnightServer serves gRPC-Web via `tonic-web`, allowing direct browser connections from the SolidJS frontend:

```ts
import { createGrpcWebTransport } from "@connectrpc/connect-web";

const transport = createGrpcWebTransport({
  baseUrl: "http://localhost:50051",
});
```

## License

MIT