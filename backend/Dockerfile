# --- cargo-chef base (dependency caching) ---
FROM rust:1.92-slim-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Compile dependencies first — this layer is cached unless Cargo.toml changes.
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# sqlx::migrate! embeds ./migrations at compile time; queries are runtime-checked
# (no live DB needed to build).
RUN cargo build --release --bin revision-backend

# --- runtime (rustls => no OpenSSL needed) ---
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates wget \
    && rm -rf /var/lib/apt/lists/*
RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser
WORKDIR /app
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/target/release/revision-backend /usr/local/bin/revision-backend
USER appuser
EXPOSE 8080
CMD ["revision-backend"]
