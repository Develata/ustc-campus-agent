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
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
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

This validates only the retained offline spike inside Opportunity Graph. It does not prove Market installation/runtime integration. The platform now has an R0 framework-neutral runtime kernel; the next first-party product mainline remains ChangeRadar source/revision/diff.

## Operator smokes

```bash
cargo run --locked -p ustc-agentctl -- doctor
cargo run --locked -p ustc-agentctl -- market validate
cargo run --locked -p ustc-agentd -- --version
```

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
