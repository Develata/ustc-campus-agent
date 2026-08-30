#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIND=${USTC_AGENTD_BIND:-127.0.0.1:8787}
STATE_DIR=${USTC_AGENTD_STATE_DIR:-${XDG_STATE_HOME:-${HOME}/.local/state}/ustc-campus-agent/three-plugin-mvp}

# `ustc-agentd serve-web` parses `SocketAddr` and is the single loopback authority.
# Invalid or non-loopback values fail closed before the listener is created.

if ! command -v cargo >/dev/null 2>&1; then
  printf 'cargo is required to build and run the MVP\n' >&2
  exit 69
fi

if [ -L "$STATE_DIR" ]; then
  printf 'refusing symlink state directory: %s\n' "$STATE_DIR" >&2
  exit 73
fi
if [ -e "$STATE_DIR" ] && [ ! -d "$STATE_DIR" ]; then
  printf 'state path is not a directory: %s\n' "$STATE_DIR" >&2
  exit 73
fi

umask 077
if [ ! -e "$STATE_DIR" ]; then
  install -d -m 0700 "$STATE_DIR"
fi
state_mode=$(stat -c '%a' "$STATE_DIR")
state_owner=$(stat -c '%u' "$STATE_DIR")
if [ "$state_mode" != 700 ] || [ "$state_owner" != "$(id -u)" ]; then
  printf 'state directory must be owned by the current user with mode 0700: %s\n' "$STATE_DIR" >&2
  exit 73
fi

AFFAIRS_FIXTURE="$ROOT/fixtures/affairs/proc-011-reviewed.json"
CHANGE_FIXTURE="$ROOT/fixtures/change-radar/academic-calendar-demo-reviewed.json"
OPPORTUNITY_FIXTURE="$ROOT/fixtures/opportunity-graph/course-planning-demo-reviewed.json"
OPPORTUNITY_CATALOG="$ROOT/market/fixtures/course-planning/minimal-v0.json"

for required in \
  "$AFFAIRS_FIXTURE" \
  "$CHANGE_FIXTURE" \
  "$OPPORTUNITY_FIXTURE" \
  "$OPPORTUNITY_CATALOG"
do
  if [ ! -f "$required" ] || [ ! -r "$required" ]; then
    printf 'required MVP input is missing or unreadable: %s\n' "$required" >&2
    exit 66
  fi
done

printf 'USTC Campus Agent three-plugin MVP\n'
printf '  Web:   http://%s/\n' "$BIND"
printf '  State: %s\n' "$STATE_DIR"
printf '  Stop:  Ctrl-C\n'

cd "$ROOT"
exec cargo run --locked -p ustc-agentd --bin ustc-agentd -- \
  serve-web \
  --bind "$BIND" \
  --fixture "$AFFAIRS_FIXTURE" \
  --change-fixture "$CHANGE_FIXTURE" \
  --opportunity-fixture "$OPPORTUNITY_FIXTURE" \
  --opportunity-catalog "$OPPORTUNITY_CATALOG" \
  --opportunity-profile-store "$STATE_DIR/opportunity-profiles.json" \
  --store "$STATE_DIR/affairs-records.json" \
  --idempotency "$STATE_DIR/affairs-idempotency.json" \
  --session-store "$STATE_DIR/m00-sessions.json"
