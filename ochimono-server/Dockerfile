# Chef stage - install cargo-chef.
FROM rust:1.98.0 AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

# Planner stage - generate the dependency recipe.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Cache workspace dependencies separately from application source.
FROM chef AS build-base
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config \
    && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --locked --recipe-path recipe.json

FROM build-base AS builder
COPY . .
RUN cargo build --release --locked -p server -p migration

# Runtime stage for the HTTP server and one-shot migration tool.
FROM debian:stable-slim AS server-runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 app \
    && useradd --uid 10001 --gid app --no-create-home --shell /usr/sbin/nologin app
WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin/server
COPY --from=builder /app/target/release/migration /usr/local/bin/migration
USER app
ENV SERVER_BIND_ADDRESS=0.0.0.0:8000
EXPOSE 8000
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:8000/health/live || exit 1
CMD ["server"]
