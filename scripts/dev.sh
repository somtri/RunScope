#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cleanup() {
  jobs -p | xargs -r kill
}
trap cleanup EXIT INT TERM

echo "Starting run_scope backend at http://localhost:8080"
(cd "$ROOT_DIR/backend" && cargo run) &

echo "Starting run_scope frontend at http://localhost:5173"
(cd "$ROOT_DIR/frontend" && npm run dev) &

wait
