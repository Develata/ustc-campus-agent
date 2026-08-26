#!/usr/bin/env bash
set -euo pipefail

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bind=${USTC_AFFAIRS_BIND:-127.0.0.1:8787}
state_root=${USTC_AFFAIRS_STATE_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/ustc-campus-agent/affairs-demo}
fixture="$repo_root/fixtures/affairs/proc-011-reviewed.json"

if [[ ! -f "$fixture" ]]; then
  printf 'source-grounded Affairs fixture missing: %s\n' "$fixture" >&2
  exit 1
fi

install -d -m 700 "$state_root"
printf 'USTC Affairs Navigator: http://%s\n' "$bind"
printf 'Retained public source fixture: https://www.teach.ustc.edu.cn/service/svc-student/13824.html\n'
printf 'Local demo state: %s\n' "$state_root"

cd "$repo_root"
exec cargo run --locked -p ustc-agentd -- serve-web \
  --bind "$bind" \
  --fixture "$fixture" \
  --store "$state_root/records.json" \
  --idempotency "$state_root/idempotency.json"
