# ── Stage 1: Build backend ────────────────────────────────
FROM rust:1.84-bookworm AS backend-builder
WORKDIR /build
COPY api/ api/
WORKDIR /build/api
RUN cargo build --release

# ── Stage 2: Build frontend ──────────────────────────────
FROM rust:1.84-bookworm AS frontend-builder
RUN cargo install trunk && rustup target add wasm32-unknown-unknown
WORKDIR /build
COPY frontend/ frontend/
WORKDIR /build/frontend
RUN trunk build --release

# ── Stage 3: Runtime ─────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libmariadb3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /build/api/target/release/api ./api
COPY --from=frontend-builder /build/frontend/dist ./frontend/dist

ENV API_PORT=8080
EXPOSE 8080

CMD ["./api"]
