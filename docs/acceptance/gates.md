# Acceptance gates

## PR gate

Repository CI runs:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo test --locked --all-features --doc
python3 scripts/run_checker_shards.py \
  --jobs 4 \
  --timeout-seconds 1800 \
  --inventory scripts/checker_test_inventory.json \
  --evidence-dir "$RUNNER_TEMP/uca-checker-evidence" \
  --require-clean \
  --require-runner-image-identity
python3 scripts/check_repo_contracts.py
git diff --check
```

For a local docs-only iteration, `python3 scripts/check_repo_contracts.py` and `git diff --check` are the minimum quick gate; final PR readiness still uses the complete CI command set. The doctest command remains an unconditional, blocking `Doc tests` step in the `rust` job on `pull_request`; moving, conditioning, making it non-blocking or changing it to a multiline carrier requires an explicit checker-contract update.

## Core demo gate

Every `matrix.tsv` row whose `gate` contains `core-demo` must pass for the claimed demo scope. A feature can be omitted from a smaller intermediate demonstration only if the claim and gate profile are explicitly narrowed; the full three-Plugin competition demo cannot reinterpret planned as pass.

## Release gate

Release requires all release-bound rows plus artifact/build/restore/read-back evidence appropriate to the delivery surface. Local test/build success is not remote release success.

## Public gate

Repository visibility is already public by explicit owner decision. Any new tag, GitHub Release, Pages, stable-download or public-runtime surface additionally requires `public-readiness.md`, the applicable license/notice state, full reachable-history audit, fixture/source permission, disclaimer, browser and remote delivery-surface verification.

## Evidence states

`matrix.tsv` is the active gate registry. `platform-baseline.md` retains long-horizon case IDs and planned assertions; catalog-only presence does not make a case current or required for the present competition slice. Activating one requires an owning plan/feature/contract projection and a row in `matrix.tsv`.

- `implemented` in the matrix means the named binding exists and currently passes the owning contract.
- `planned`, skipped, unavailable and not-run are non-pass.
- A manual binding needs an identified reviewer and evidence artifact before it can pass.
- Evidence from a stale pre-fix worktree does not certify the final staged/committed state.

## Fingerprint moratorium

New source lexical/body fingerprint guards are under moratorium. Existing guards stay until an authority-native replacement has:

1. a named replacement owner/evidence mechanism;
2. equivalent or stronger mutation tests;
3. proof that current accepted coverage does not decrease;
4. a reviewed deletion.

Prefer typed schema, Cargo/rustc metadata, compiler evidence, public behavior tests, database constraints and exact acceptance output over lexical body matching.

## Risk-tiered gates

| Tier | Scope | Gate |
|---|---|---|
| 1 — fast schema/registry/link/status projection | docs-only, schema/registry/link/status checks | `python3 scripts/check_repo_contracts.py` + `git diff --check` |
| 2 — Rust behavior/dependency | Rust source, `Cargo.toml`, dependency changes | tier 1 + `cargo fmt --all -- --check` + `cargo clippy --locked --workspace --all-targets -- -D warnings` + `cargo test --locked --workspace --all-targets` + `cargo test --locked --workspace --doc` |
| 3 — authority/security mutation | authority/security invariants, grant blocks, permission semantics, lifecycle state machines | tier 2 + exact-inventory checker shards (the PR-gate command above) + contract cross-check + independent review |
| 4 — release/full evidence | release, public visibility, deployment | tier 3 + all release-bound rows + artifact/build/restore/read-back evidence |

## Docs-only fast path

Non-authority docs (guides, overview, ADRs, design packets, README prose) use the docs-only fast path: `python3 scripts/check_repo_contracts.py` and `git diff --check`. Authority-bearing plans, contracts, acceptance rows, task grants and campaign blocks invoke the required stronger gate tier. A docs-only change to an authority-bearing file still invokes the stronger gate; the fast path is for files that do not own authority.

## State authority separation

Do not create one global state file. Preserve separate authority by fact type:

| Fact type | Owner |
|---|---|
| large-module implementation state | structured module registry / current owning table (`docs/plan/modules/00-module-map.md`) |
| acceptance status | `docs/acceptance/matrix.tsv` |
| active delivery lane | roadmap / taskbook (`docs/tasks/01-execution-roadmap.md`) |
| README / issues | generated or checked projections |

## Replacement-before-deletion ledger

A small replacement-before-deletion ledger may be added under the narrowest existing governance owner. A guard is not deleted until its replacement is reviewed and proven equivalent-or-stronger. Do not hash whole mutable CI workflows or campaign blocks as a permanent substitute for semantic validation when a narrower authority-native carrier can be designed.
