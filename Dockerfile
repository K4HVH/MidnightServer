FROM rust:1.89-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends protobuf-compiler libprotobuf-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir -p src/proto/generated && echo "fn main() {}" > src/main.rs
COPY build.rs ./
COPY proto/ proto/
COPY migrations/ migrations/
COPY .sqlx/ .sqlx/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    SQLX_OFFLINE=true cargo build --release && rm -rf src target/release/deps/MidnightServer*

COPY src/ src/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    SQLX_OFFLINE=true cargo build --release && \
    cp target/release/MidnightServer /usr/local/bin/midnight-server

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && addgroup --gid 1001 --system appgroup \
    && adduser --system --uid 1001 appuser

COPY --from=builder --chown=appuser:appgroup /usr/local/bin/midnight-server /usr/local/bin/midnight-server

USER appuser

EXPOSE 50051
ENV LISTEN_ADDR=0.0.0.0:50051

CMD ["midnight-server"]
