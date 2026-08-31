# Local development guide

## Prerequisites

- Rust stable 1.97.1 with `rustfmt` and `clippy`.
- Python 3 for repository contract checks.
- CodeGraph CLI for local code navigation.
- GitHub CLI only for authorized repository operations.

## Disk preflight and Rust gates

```bash
df -h . /tmp
export CARGO_TARGET_DIR=/tmp/hermes-cargo-target
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
```

Using `/tmp/hermes-cargo-target` keeps disposable build state outside the repository. Remove it only after no active review/debug step depends on it.

## Repository contract gate

```bash
checker_evidence="$(mktemp -d)"
PYTHONPYCACHEPREFIX="$(mktemp -d)" python3 scripts/run_checker_shards.py \
  --jobs 4 \
  --timeout-seconds 1800 \
  --inventory scripts/checker_test_inventory.json \
  --evidence-dir "$checker_evidence"
python3 scripts/check_repo_contracts.py
git diff --check
```

The checker validates:

- internal Markdown links;
- obvious secret patterns;
- exact three first-party IDs/versions/status/capabilities/install policies;
- Rust/catalog identity agreement;
- safe component paths and safe non-first-party package policy;
- acceptance matrix shape, gate/status vocabulary and duplicate IDs.

## Course Planning bounded-spike smoke

```bash
cargo run --locked -p ustc-agentctl -- course plan \
  --fixture market/fixtures/course-planning/minimal-v0.json \
  --format json
```

The command must emit `course-plan-result/v0`, at least two candidates for the canonical fixture and `hard_constraint_violations: 0`.

This validates only the retained offline spike inside Opportunity Graph. It does not prove Market installation/runtime integration. The bounded ChangeRadar mainline now also has durable fixed-administrator publication; approved live-source retrieval and production administration remain separate.

## Operator smokes

```bash
cargo run --locked -p ustc-agentctl -- doctor
cargo run --locked -p ustc-agentctl -- market validate
cargo run --locked -p ustc-agentd -- --version
```

For the loopback three-plugin demo, start `./scripts/run_three_plugin_mvp.sh` and exercise both fixed administrator callers from another process:

```bash
cargo run --locked -p ustc-agentctl -- affairs publication-status --server 127.0.0.1:8787
cargo run --locked -p ustc-agentctl -- affairs publish-demo --server 127.0.0.1:8787 --confirm
cargo run --locked -p ustc-agentctl -- changes publication-status --server 127.0.0.1:8787
cargo run --locked -p ustc-agentctl -- changes publish-demo --server 127.0.0.1:8787 --confirm
```

The ChangeRadar focused acceptance closure is:

```bash
cargo test --locked -p ustc-campus-agent-change-radar --all-features
cargo test --locked -p ustc-campus-agent-application-ingress --test change_publication
cargo test --locked -p ustc-agentd --test change_composition
cargo test --locked -p ustc-agentd --test affairs_web
cargo test --locked -p ustc-agentctl
```

Publication requires explicit confirmation and only numeric loopback servers are accepted. Stop the daemon, restart it with the same owner-only state directory, and verify `changes publication-status`, public JSON and Atom retain one identical receipt/GUID/item. This is bounded demo evidence, not production SSO, remote administration or live-source proof.

## CodeGraph

The local `.codegraph/` index is ignored by Git.

```bash
codegraph status .
codegraph sync .
codegraph explore --project . "market plugin authority"
```

For Agent tool calls, resolve and pass the current repository root as `projectPath`; do not hard-code a contributor-specific clone path in repository documentation.

## Cleanup

After all verification/review that needs build outputs is complete:

```bash
rm -rf /tmp/hermes-cargo-target
```

Do not commit `target/`, `.codegraph/`, local snapshots, evidence containing private payloads or credentials.
