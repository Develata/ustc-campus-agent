#!/usr/bin/env bash
set -euo pipefail

STATE_DIR=/data
APP_BIND=127.0.0.1:8788
PROXY_BIND=0.0.0.0:8787

if [ -L "$STATE_DIR" ]; then
  printf 'refusing symlink state directory: %s\n' "$STATE_DIR" >&2
  exit 73
fi
install -d -m 0700 "$STATE_DIR"
umask 077

required=(
  /app/fixtures/affairs/proc-011-reviewed.json
  /app/fixtures/change-radar/academic-calendar-demo-reviewed.json
  /app/fixtures/change-radar/evidence/academic-calendar-r1.reviewed.txt
  /app/fixtures/change-radar/evidence/academic-calendar-r1.normalized.json
  /app/fixtures/change-radar/evidence/academic-calendar-r2.reviewed.txt
  /app/fixtures/change-radar/evidence/academic-calendar-r2.normalized.json
  /app/fixtures/opportunity-graph/course-planning-demo-reviewed.json
  /app/market/fixtures/course-planning/minimal-v0.json
)
for path in "${required[@]}"; do
  if [ ! -f "$path" ] || [ ! -r "$path" ] || [ -L "$path" ]; then
    printf 'required MVP input is missing, unreadable, or a symlink: %s\n' "$path" >&2
    exit 66
  fi
done

/app/ustc-agentd serve-web \
  --bind "$APP_BIND" \
  --fixture /app/fixtures/affairs/proc-011-reviewed.json \
  --change-fixture /app/fixtures/change-radar/academic-calendar-demo-reviewed.json \
  --opportunity-fixture /app/fixtures/opportunity-graph/course-planning-demo-reviewed.json \
  --opportunity-catalog /app/market/fixtures/course-planning/minimal-v0.json \
  --opportunity-profile-store "$STATE_DIR/opportunity-profiles.json" \
  --store "$STATE_DIR/affairs-records.json" \
  --idempotency "$STATE_DIR/affairs-idempotency.json" \
  --session-store "$STATE_DIR/m00-sessions.json" &
app_pid=$!

socat "TCP-LISTEN:8787,bind=0.0.0.0,reuseaddr,fork" "TCP:127.0.0.1:8788" &
proxy_pid=$!

terminate_children() {
  kill -TERM "$proxy_pid" "$app_pid" 2>/dev/null || true
}
trap terminate_children TERM INT HUP

set +e
wait -n "$app_pid" "$proxy_pid"
status=$?
set -e
terminate_children
wait "$proxy_pid" 2>/dev/null || true
wait "$app_pid" 2>/dev/null || true
exit "$status"
