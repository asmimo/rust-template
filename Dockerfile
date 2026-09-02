FROM lukemathwalker/cargo-chef:latest-rust-1.98.0-slim-bookworm AS chef
WORKDIR /app
RUN apt-get update && apt-get install lld clang pkg-config make -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG APP
ARG FEATURES
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    if [ -n "$FEATURES" ]; then \
      cargo chef cook --release --recipe-path recipe.json -p $APP --features $FEATURES; \
    else \
      cargo chef cook --release --recipe-path recipe.json -p $APP; \
    fi

COPY . .
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    if [ -n "$FEATURES" ]; then \
      cargo build --release --bin $APP --features $FEATURES; \
    else \
      cargo build --release --bin $APP; \
    fi \
    && cp /app/target/release/$APP /app/entrypoint

FROM oven/bun:slim AS rest
WORKDIR /app
COPY package.json bun.lock ./
RUN bun install

COPY . .
ARG TAILWIND_CONFIG
ENV TAILWIND_CONFIG=$TAILWIND_CONFIG
RUN bun run build.script

# FROM chef AS wget-bundle
# RUN mkdir -p /bundle/usr/bin \
#     && cp /usr/bin/wget /bundle/usr/bin/ \
#     && ldd /usr/bin/wget | awk '/=> \// { print $3 }' \
#        | while read lib; do \
#            mkdir -p "/bundle$(dirname $lib)"; \
#            cp -L "$lib" "/bundle$(dirname $lib)/"; \
#          done

# FROM gcr.io/distroless/cc-debian12 AS runtime
# WORKDIR /app
# COPY --from=wget-bundle /bundle/ /
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates wget \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/entrypoint entrypoint

COPY --from=rest /app/dist dist
COPY --from=rest /app/logo logo

ENV APP_ENVIRONMENT=production

EXPOSE 8080

ENTRYPOINT ["/app/entrypoint"]
