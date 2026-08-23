# syntax=docker/dockerfile:1

# Container image for the `example` binary (bins/example).
#
# Two targets:
#   - dev:     toolchain + cargo-watch, used by dev.compose.yml. No source is
#              baked in — compose bind-mounts the repository at /app.
#   - release: production image (the default target). Multi-stage build with
#              cargo-chef for dependency layer caching, non-root runtime on
#              debian:bookworm-slim.
#
# Template convention: one folder per binary under docker/, named after the
# binary in bins/. To add another binary, copy this folder to docker/<name>/
# and replace `example` everywhere (package name, binary path, compose
# service/credentials).

# MSRV is 1.98 (clippy.toml). The build image is pinned to match; it also
# keeps cargo-chef/cargo-watch installable, whose dependencies require a
# newer rustc than older MSRVs did.
ARG RUST_VERSION=1.98

# ── dev ──────────────────────────────────────────────────────────────────────

# Development toolchain. Source comes from the bind mount, so this stage only
# provides rustc and cargo-watch for the rebuild-on-save loop.
FROM rust:${RUST_VERSION} AS dev
RUN cargo install cargo-watch --locked
WORKDIR /app
# Conventional port of the incoming adapter (http-axum); keep in sync with
# the port mapping in dev.compose.yml.
EXPOSE 8080
CMD ["cargo", "watch", "-x", "run -p example"]

# ── release ──────────────────────────────────────────────────────────────────

# cargo-chef plans the dependency graph so dependencies compile in their own
# layer, cached until Cargo.lock changes.
FROM rust:${RUST_VERSION} AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# --locked keeps the build reproducible against the committed Cargo.lock.
RUN cargo build --release --locked --package example

# Minimal runtime: glibc matches the rust build image (bookworm); TLS root
# certificates for outgoing HTTPS via rustls-native-certs.
FROM debian:bookworm-slim AS release
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home app
COPY --from=builder /app/target/release/example /usr/local/bin/example
USER app
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/example"]
