FROM lukemathwalker/cargo-chef:latest-rust-1.98.0-slim-bookworm AS chef
WORKDIR /app
RUN apt update && apt install lld clang pkg-config libssl-dev make -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json -p $APP

COPY . .
ENV SQLX_OFFLINE=true
ARG FEATURES
# RUN --mount=type=cache,target=/usr/local/cargo/registry \
#    --mount=type=cache,target=/app/target \
RUN if [ -n "$FEATURES" ]; then \
    cargo build --release --bin $APP --features $FEATURES; \
    else \
    cargo build --release --bin $APP; \
    fi

FROM oven/bun:slim AS rest
WORKDIR /app
RUN apt update && apt install curl -y
COPY . .
RUN bun install && bun run build.script

ARG TAILWIND_CONFIG
RUN if [ -n "$TAILWIND_CONFIG" ]; then \
    bun run ./scripts/generate-css --tailwind-config $TAILWIND_CONFIG; \
    fi

ARG MAXMINDDB_DOWNLOAD_URL
RUN if [ -n "$MAXMINDDB_DOWNLOAD_URL" ]; then \
    # hide the download URL from the build output
    bash -c 'curl -L ${MAXMINDDB_DOWNLOAD_URL} | tar zxv'; \
    fi

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates curl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

ARG APP
COPY --from=builder /app/target/release/${APP} entrypoint

COPY --from=rest /app/dist dist
COPY --from=rest /app/logo logo
COPY --from=rest /app/GeoLite2-City_*/GeoLite2-City.mmdb GeoLite2-City.mmdb

ENV APP_ENVIRONMENT=production

EXPOSE 8080

ENTRYPOINT ["./entrypoint"]
