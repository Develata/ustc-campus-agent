# CI workflow transition invariant ledger

## Metadata

- `Layer`: `Acceptance / CI transition`
- `Status`: `guard-active-not-required`
- `Version`: `ci-transition/v1`
- `Last Review`: 2026-08-25
- `Authority Owns`: legacy-to-v2 CI workflow invariants plus the trusted-base governance-controller activation state
- `Authority Defers To`: `.github/workflows/ci.yml` for full build/test CI; `.github/workflows/ci-governance.yml` for the active governance Check Run; `scripts/tests/fixtures/ci-v2.yml` for inert future fixture test data

## Scope

This ledger is the transition authority for the M90 CI evidence shard slice and the S1 trusted-base governance bootstrap. It enumerates the legacy invariants that remain unchanged while the inert future workflow fixture and process-isolated checker shard runner are introduced, and separately records the active governance controller. It does not activate selective acceptance, does not promote any acceptance status, and does not remove or weaken any existing CI gate.

### State declarations

- The active workflow `.github/workflows/ci.yml` remains the legacy/full CI authority. It is not modified by this slice.
- The trusted-base workflow `.github/workflows/ci-governance.yml` is active and publishes the head-scoped `ci-governance` Check Run; it executes no PR-controlled bytes.
- `scripts/tests/fixtures/ci-v2.yml` is inert test data only. It is not placed under `.github/workflows`. Its presence is not implementation or activation evidence.
- No selective omission, path filtering, or package-selective Rust is active.
- No acceptance matrix status is promoted by this slice.
- Making `ci-governance` a protected-main required check remains gated on the post-merge S2 negative-to-positive same-ID smoke and exact GitHub Actions app read-back; acceptance CLI authority choice and selective CI activation remain later prerequisites.
- This is the first activation of the controller, so no pre-repair PR-bound live record exists. S2 must still inventory the smoke head and stop on any foreign same-name/head/app record before policy mutation.

## Invariant ledger

| ID | Legacy invariant | Legacy carrier | Future v2 carrier | Mechanical test | Slice state |
|---|---|---|---|---|---|
| `CI-TR-001` | active workflow exact digest remains frozen | `.github/workflows/ci.yml` SHA-256 `919080325ade109dab32b556cbc97fb3fcd5844e45ad72e3b74ad231cb669146` | `scripts/tests/fixtures/ci-v2.yml` inert fixture is governed by explicit semantic invariants and is intentionally not whole-file fingerprinted | `scripts/check_repo_contracts.py` freezes the active workflow digest and enforces the fixture's semantic invariants; mutation tests remove/mutate each semantic invariant independently | `inert-not-active` |
| `CI-TR-002` | `pull_request` plus push-to-main trigger semantics remain | `.github/workflows/ci.yml` `on: pull_request` and `push: branches: [main]` | `scripts/tests/fixtures/ci-v2.yml` preserves `on: pull_request` and `push: branches: [main]` | `scripts/check_repo_contracts.py` validates trigger semantics in both active and fixture; mutation tests remove each trigger independently | `inert-not-active` |
| `CI-TR-003` | stable `rust` and `docs-and-contracts` job names remain | `.github/workflows/ci.yml` jobs `rust` and `docs-and-contracts` | `scripts/tests/fixtures/ci-v2.yml` preserves job names `rust` and `docs-and-contracts` | `scripts/check_repo_contracts.py` validates job names in both active and fixture; mutation tests rename/remove each independently | `inert-not-active` |
| `CI-TR-004` | full Python discovery is replaced only in the inert fixture by exact-inventory full sharding, with no omission | `.github/workflows/ci.yml` `docs-and-contracts` job runs `python3 -m unittest discover -s scripts/tests -p 'test_*.py'` | `scripts/tests/fixtures/ci-v2.yml` replaces discovery with `python3 scripts/run_checker_shards.py --jobs 4 --timeout-seconds 1800 --inventory scripts/checker_test_inventory.json --evidence-dir <RUNNER_TEMP> --require-clean --require-runner-image-identity` | `scripts/run_checker_shards.py` bidirectional inventory coverage and exact-union fan-in; `scripts/checker_test_inventory.json` sorted unique IDs with explicit count; mutation tests for missing/unexpected/duplicate/zero IDs | `inert-not-active` |
| `CI-TR-005` | full Rust fmt/clippy/workspace-test/doctest commands remain | `.github/workflows/ci.yml` `rust` job runs `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked --all-targets --all-features`, `cargo test --locked --all-features --doc` | `scripts/tests/fixtures/ci-v2.yml` preserves `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-targets --all-features --locked`, `cargo test --workspace --all-features --doc --locked` | `scripts/check_repo_contracts.py` validates all four Rust commands in fixture; mutation tests remove/mutate each command independently | `inert-not-active` |
| `CI-TR-006` | repository checker remains an exact executable command after the full checker suite | `.github/workflows/ci.yml` `docs-and-contracts` job runs `python3 scripts/check_repo_contracts.py` | `scripts/tests/fixtures/ci-v2.yml` runs `python3 scripts/check_repo_contracts.py` exactly once after the shard runner | `scripts/check_repo_contracts.py` validates checker command presence, count, and after-runner ordering in fixture; mutation tests remove/duplicate/reorder the command | `inert-not-active` |
| `CI-TR-007` | trusted-base governance is active without yet becoming a required check | `.github/workflows/ci-governance.yml` publishes one head-scoped `ci-governance` Check Run under global serialization and executes no PR-controlled bytes | protected-main requirement remains absent until the S2 failure-to-success same-ID smoke and exact app-bound activation read-back | `scripts/check_repo_contracts.py` structurally validates both active workflows, the head-scoped external identity, stable repeated head/base/updated-at file observations, permissions, sole effect endpoints and all-workflow authority; `CiGovernanceWorkflowTests` exercises the state machine and mutations | `guard-active-not-required` |
