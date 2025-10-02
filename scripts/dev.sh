#!/usr/bin/env bash
set -euo pipefail

# Run backend API and frontend concurrently.
# Requires: trunk installed, cargo, and MySQL configured via api/.env

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
ROOT_DIR=$(dirname "$SCRIPT_DIR")

pushd "$ROOT_DIR" >/dev/null

# Ensure wasm target
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

# Start backend
(
  cd api
  echo "[dev] Starting API on :${API_PORT:-8080}"
  cargo run
) &
API_PID=$!

# Start frontend
(
  cd frontend
  echo "[dev] Starting Frontend (Trunk serve)"
  trunk serve --port ${FRONTEND_PORT:-8081} --proxy-backend http://127.0.0.1:${API_PORT:-8080} --proxy-rewrite /api=/api
) &
FE_PID=$!

cleanup() {
  echo "Shutting down..."
  kill $API_PID $FE_PID 2>/dev/null || true
}
trap cleanup INT TERM

wait $API_PID $FE_PID
