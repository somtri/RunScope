#!/usr/bin/env bash
set -euo pipefail

API_URL="${RUN_SCOPE_API_URL:-http://localhost:8080}"

echo "Backend health:"
curl --fail-with-body "$API_URL/health"
echo

echo "Starting demo run:"
curl --fail-with-body \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"recipe_id":"lpbf_layer_demo"}' \
  "$API_URL/api/runs/start"
echo

echo "Open http://localhost:5173 to watch live telemetry."
