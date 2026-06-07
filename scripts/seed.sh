#!/usr/bin/env bash
set -euo pipefail

API_URL="${RUN_SCOPE_API_URL:-http://localhost:8080}"

echo "Starting the LPBF layer-monitoring demo through $API_URL"
curl --fail-with-body \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"recipe_id":"lpbf_layer_demo"}' \
  "$API_URL/api/runs/start"
echo
