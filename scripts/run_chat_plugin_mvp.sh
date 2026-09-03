#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-/tmp/ustc-campus-agent-chat-plugin-mvp-target}
export CARGO_TARGET_DIR

cd "$ROOT"

printf '%s\n' '[1/6] repository contract checks'
python3 scripts/check_repo_contracts.py

printf '%s\n' '[2/6] formatting'
cargo fmt --all -- --check

printf '%s\n' '[3/6] provider-neutral chat engine unit tests'
cargo test --locked -p ustc-campus-agent-runtime --lib

printf '%s\n' '[4/6] OpenAI-compatible Responses adapter unit tests'
cargo test --locked -p ustc-campus-agent-adapters --lib

printf '%s\n' '[5/6] loopback chat + plugin integration tests'
cargo test --locked -p ustc-agentd --test chat_plugin_mvp -- --test-threads=1

printf '%s\n' '[6/6] optional credentialed provider smoke'
python3 scripts/smoke_chat_provider.py

printf '%s\n' 'chat-plugin-mvp-tests: PASS'
