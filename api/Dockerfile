# Multi-stage build for Actix-web API
# 1) Build stage
FROM rust:1.80 as builder
WORKDIR /app

# Cache deps first
COPY Cargo.toml Cargo.lock ./
COPY ../Cargo.toml /app-workspace-Cargo.toml
# Create minimal workspace to cache dependencies
RUN mkdir -p api frontend docs locales && \
    echo "[workspace]\nmembers=[\"api\",\"frontend\"]" > Cargo.toml && \
    mkdir -p api && \
    printf "[package]\nname=\"api\"\nversion=\"0.1.0\"\nedition=\"2024\"\n\n[dependencies]\n" > api/Cargo.toml || true

# Now copy real sources
COPY . /app

# Build release binary
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release

# 2) Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Create non-root user
RUN useradd -m -u 10001 appuser

# Copy binary
COPY --from=builder /app/target/release/api /app/api

# Copy locales and migrations
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/locales /app/locales

# Storage dir for documents (bind-mount recommended)
ENV STORAGE_DIR=/app/storage
RUN mkdir -p ${STORAGE_DIR} && chown -R appuser:appuser /app

# Runtime env defaults
ENV API_PORT=8080
EXPOSE 8080

USER appuser

# Note: set DATABASE_URL and JWT_SECRET via environment variables
CMD ["/app/api"]
