# MidnightServer

Rust gRPC backend template using [tonic](https://github.com/hyperium/tonic). Server counterpart to [MidnightUI](https://github.com/k4hvh/MidnightUI).

gRPC + gRPC-Web, PostgreSQL (SQLx, compile-time checked queries, auto-migrations), active health probes, structured logging, CORS, graceful shutdown.

## Setup

### System dependencies

```sh
# Fedora
sudo dnf install protobuf-compiler protobuf-devel

# Ubuntu / Debian
sudo apt install protobuf-compiler libprotobuf-dev
```

[Rust](https://rustup.rs/) (edition 2024) and [Docker](https://docs.docker.com/get-docker/) or [Podman](https://podman.io/) for the database.

Optional:

```sh
cargo install sqlx-cli --no-default-features --features postgres  # migration management
cargo install grpcurl                                              # gRPC testing
```

### Database

```sh
docker run -d --name midnight-postgres \
  -e POSTGRES_USER=midnight \
  -e POSTGRES_PASSWORD=midnight \
  -e POSTGRES_DB=midnight \
  -p 5432:5432 \
  postgres:17-alpine
```

### Run

```sh
cp .env.example .env
cargo run
```

Connects to Postgres, runs migrations, listens on `0.0.0.0:50051`.

### Test

```sh
SQLX_OFFLINE=true cargo test

# gRPC (requires grpcurl)
grpcurl -plaintext localhost:50051 list
grpcurl -plaintext localhost:50051 midnight.HealthService/ListHealthServices
```

### Docker build

```sh
docker build -t midnight-server .
docker run -e DATABASE_URL=postgres://midnight:midnight@host.docker.internal:5432/midnight midnight-server
```

## Configuration

All via environment variables (see [.env.example](.env.example)):

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | *(required)* | PostgreSQL connection string |
| `LISTEN_ADDR` | `0.0.0.0:50051` | Server bind address |
| `LOG_LEVEL` | `info` | Tracing filter directive |
| `LOG_STYLE` | `auto` | `plain`, `compact`, `pretty`, `json`, or `auto` |
| `CORS_ORIGINS` | `*` | Comma-separated origins, or `*` |
| `REQUEST_TIMEOUT_SECS` | `30` | Per-request gRPC timeout |
| `DB_MAX_CONNECTIONS` | `20` | Max database pool connections |

## Project layout

```
proto/midnight/          Protobuf definitions
migrations/              SQL migrations (auto-run on startup)
src/
  main.rs                Server entrypoint
  core/
    config.rs            Env-based config
    db.rs                Pool + migrations
    error.rs             AppError → gRPC Status
    health.rs            Probe-based HealthRegistry
    logging.rs           Tracing setup (4 styles)
    state.rs             AppState (config, db, health, uptime)
  grpc/
    health.rs            Health service RPCs
  proto/                 Generated protobuf code
tests/                   Unit tests
```

## Adding a service

1. Add a `.proto` file in `proto/midnight/`
2. `cargo build` (codegen runs automatically)
3. Implement the generated trait in `src/grpc/`
4. Register it in `Server::builder()` in `main.rs`

## License

GNU GPL V3