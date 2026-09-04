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
printf 'MVP reset complete. All local test state was deleted.\n'
