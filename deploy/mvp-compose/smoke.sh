#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT"
port=${UCA_MVP_PORT:-8787}
base="http://127.0.0.1:${port}"
work=$(mktemp -d)
cleanup() {
  docker compose down --volumes --remove-orphans >/dev/null 2>&1 || true
  rm -rf "$work"
}
trap cleanup EXIT

docker compose config --quiet
docker compose build --pull
docker compose up -d

healthy=0
for _ in $(seq 1 150); do
  if curl --fail --silent "$base/healthz" > "$work/health.json"; then
    healthy=1
    break
  fi
  sleep 2
done
if [ "$healthy" -ne 1 ]; then
  docker compose ps
  docker compose logs --no-color --tail 200
  printf 'health timeout\n' >&2
  exit 1
fi

python3 - "$work/health.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value == {'schema': 'ustc-agentd-health/v1', 'status': 'ok'}, value
PY
curl --fail --silent "$base/" > "$work/index.html"
for marker in 'AFFAIRS NAVIGATOR' 'CHANGE RADAR' 'OPPORTUNITY GRAPH'; do
  grep -Fq "$marker" "$work/index.html"
done

curl --fail --silent \
  -H 'x-ustc-client-protocol-major: 1' \
  "$base/api/v1/affairs/proc%3Austc%3Aundergraduate%3Atranscript-certificate" \
  > "$work/affairs.json"
python3 - "$work/affairs.json" <<'PY'
import json, pathlib, sys
value = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert value['kind'] == 'available', value
assert value['terminal']['outcome']['kind'] == 'found', value
PY

curl --fail --silent \
  -H 'content-type: application/json' \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  -d '{"confirm_publish":true}' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/published.json"
curl --fail --silent \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/status-before.json"
python3 - "$work/published.json" "$work/status-before.json" <<'PY'
import json, pathlib, sys
published = json.loads(pathlib.Path(sys.argv[1]).read_text())
status = json.loads(pathlib.Path(sys.argv[2]).read_text())
assert published['outcome']['kind'] == 'published', published
assert status['publication_revision'] is not None, status
assert status['control_evidence_event_count'] > 0, status
PY

docker compose restart
healthy=0
for _ in $(seq 1 60); do
  if curl --fail --silent "$base/healthz" >/dev/null; then
    healthy=1
    break
  fi
  sleep 2
done
[ "$healthy" -eq 1 ] || { printf 'restart health timeout\n' >&2; exit 1; }
curl --fail --silent \
  -H 'x-ustc-agent-administrator-demo: confirm-v1' \
  "$base/api/v1/demo/administrator/affairs/publication" \
  > "$work/status-after.json"
cmp "$work/status-before.json" "$work/status-after.json"

docker compose ps
printf 'MVP_COMPOSE_SMOKE=PASS\n'
