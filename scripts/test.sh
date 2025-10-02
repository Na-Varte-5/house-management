#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
ROOT_DIR=$(dirname "$SCRIPT_DIR")

pushd "$ROOT_DIR" >/dev/null

# Backend checks
cd api
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cd ..

# Frontend build
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cd frontend
trunk build

popd >/dev/null
