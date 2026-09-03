#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT"
docker info >/dev/null
docker compose up --build -d
port=${UCA_MVP_PORT:-8787}
url="http://127.0.0.1:${port}"
for _ in $(seq 1 150); do
  health=$(curl --fail --silent "${url}/healthz" || true)
  if printf '%s' "$health" | grep -Fq '"schema":"ustc-agentd-health/v1"' \
    && printf '%s' "$health" | grep -Fq '"status":"ok"'; then
    printf 'MVP is ready: %s\n' "$url"
    exit 0
  fi
  sleep 2
done
docker compose ps
docker compose logs --no-color --tail 120
printf 'MVP did not become healthy within 5 minutes\n' >&2
exit 1
