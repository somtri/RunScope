#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Fetching Rust dependencies..."
cargo fetch --manifest-path "$ROOT_DIR/backend/Cargo.toml"

echo "Installing frontend dependencies..."
(cd "$ROOT_DIR/frontend" && npm install)

echo "run_scope dependencies are ready."
