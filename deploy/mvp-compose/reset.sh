#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$ROOT"
answer=${UCA_RESET_CONFIRMATION:-}
if [ -z "$answer" ]; then
  printf 'This deletes all local test state for this MVP. Type RESET to continue: '
  IFS= read -r answer
fi
if [ "$answer" != RESET ]; then
  printf 'Cancelled.\n'
  exit 0
fi

docker compose down --volumes
rm -f -- secrets/admin-password.phc
printf 'MVP reset complete. Local test state and the local access password were deleted.\n'
