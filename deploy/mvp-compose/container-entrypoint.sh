#!/usr/bin/env bash
set -euo pipefail

STATE_DIR=/data
KEY_SOURCE=/run/secrets/uca_agent_api_key
KEY_PRIVATE_DIR=/run/uca-agent-private
KEY_PRIVATE=$KEY_PRIVATE_DIR/uca_agent_api_key
APP_BIND=127.0.0.1:8788
PROXY_BIND=0.0.0.0:8787

trim_rust_whitespace() {
  local value=$1
  local matched ws
  local unicode_ws=(
    $'\xC2\x85' $'\xC2\xA0' $'\xE1\x9A\x80'
    $'\xE2\x80\x80' $'\xE2\x80\x81' $'\xE2\x80\x82' $'\xE2\x80\x83'
    $'\xE2\x80\x84' $'\xE2\x80\x85' $'\xE2\x80\x86' $'\xE2\x80\x87'
    $'\xE2\x80\x88' $'\xE2\x80\x89' $'\xE2\x80\x8A' $'\xE2\x80\xA8'
    $'\xE2\x80\xA9' $'\xE2\x80\xAF' $'\xE2\x81\x9F' $'\xE3\x80\x80'
  )
  while :; do
    case "$value" in
      [[:space:]]*) value=${value#?}; continue ;;
    esac
    matched=0
    for ws in "${unicode_ws[@]}"; do
      case "$value" in "$ws"*) value=${value#"$ws"}; matched=1; break ;; esac
    done
    [ "$matched" -eq 1 ] || break
  done
  while :; do
    case "$value" in
      *[[:space:]]) value=${value%?}; continue ;;
    esac
    matched=0
    for ws in "${unicode_ws[@]}"; do
      case "$value" in *"$ws") value=${value%"$ws"}; matched=1; break ;; esac
    done
    [ "$matched" -eq 1 ] || break
  done
  printf '%s' "$value"
}

current_uid=$(id -u)
if [ "$current_uid" -eq 0 ]; then
  if [ -L "$STATE_DIR" ]; then
    printf 'refusing symlink state directory: %s\n' "$STATE_DIR" >&2
    exit 73
  fi
  install -d -o 65532 -g 65532 -m 0700 "$STATE_DIR"
  umask 077

  if [ "${UCA_AGENT_PROVIDER:-mock}" = openai-compatible ]; then
    if [ ! -f "$KEY_SOURCE" ] || [ ! -r "$KEY_SOURCE" ] || [ -L "$KEY_SOURCE" ]; then
      printf 'provider key source is missing, unreadable, or a symlink\n' >&2
      exit 66
    fi
    key_value=$(trim_rust_whitespace "$(cat "$KEY_SOURCE")")
    if [ "$key_value" = unused-placeholder-for-deterministic-mock-mode ]; then
      printf 'the bundled mock provider placeholder is forbidden in openai-compatible mode\n' >&2
      exit 66
    fi
    install -d -o 65532 -g 65532 -m 0700 "$KEY_PRIVATE_DIR"
    install -o 65532 -g 65532 -m 0600 "$KEY_SOURCE" "$KEY_PRIVATE"
  fi

  export UCA_ENTRYPOINT_PRIVILEGES_DROPPED=1
  exec /usr/bin/setpriv \
    --reuid=65532 \
    --regid=65532 \
    --clear-groups \
    --no-new-privs \
    --inh-caps=-all \
    --ambient-caps=-all \
    --bounding-set=-all \
    "$0" "$@"
fi

if [ "$current_uid" -ne 65532 ] || [ "${UCA_ENTRYPOINT_PRIVILEGES_DROPPED:-}" != 1 ]; then
  printf 'entrypoint privilege transition was not completed\n' >&2
  exit 77
fi
if [ -L "$STATE_DIR" ] || [ ! -d "$STATE_DIR" ] || [ ! -w "$STATE_DIR" ]; then
  printf 'state directory is not a private writable directory: %s\n' "$STATE_DIR" >&2
  exit 73
fi
umask 077

if [ "${UCA_AGENT_PROVIDER:-mock}" = openai-compatible ]; then
  if [ ! -f "$KEY_PRIVATE" ] || [ ! -r "$KEY_PRIVATE" ] || [ -L "$KEY_PRIVATE" ]; then
    printf 'private provider key is missing, unreadable, or a symlink\n' >&2
    exit 66
  fi
fi

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

if [ "${UCA_ENTRYPOINT_SECRET_PROBE:-}" = 1 ]; then
  if [ "${UCA_AGENT_PROVIDER:-mock}" != openai-compatible ]; then
    printf 'entrypoint secret probe requires openai-compatible mode\n' >&2
    exit 64
  fi
  if [ "$(stat -c '%u:%g:%a' "$KEY_PRIVATE")" != 65532:65532:600 ]; then
    printf 'private provider key ownership or mode is invalid\n' >&2
    exit 66
  fi
  if ! grep -Eq '^CapEff:[[:space:]]+0+$' /proc/self/status \
    || ! grep -Eq '^CapBnd:[[:space:]]+0+$' /proc/self/status \
    || ! grep -Eq '^NoNewPrivs:[[:space:]]+1$' /proc/self/status; then
    printf 'entrypoint did not drop capabilities or preserve no-new-privileges\n' >&2
    exit 66
  fi
  printf 'ENTRYPOINT_SECRET_PROBE=PASS uid=%s key=%s caps=%s\n' \
    "$current_uid" '65532:65532:600' 'none'
  exit 0
fi

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
