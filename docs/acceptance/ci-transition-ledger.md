# CI workflow transition invariant ledger

## Metadata

- `Layer`: `Acceptance / CI transition`
- `Status`: `inert-not-active`
- `Version`: `ci-transition/v0`
- `Last Review`: 2026-08-25
- `Authority Owns`: legacy-to-v2 CI workflow invariant mapping for the M90 CI evidence shard slice
- `Authority Defers To`: `.github/workflows/ci.yml` for active CI authority; `scripts/tests/fixtures/ci-v2.yml` for inert future fixture test data

## Scope

This ledger is the transition authority for the M90 CI evidence shard slice. It enumerates the legacy invariants that must remain unchanged while the inert future workflow fixture and process-isolated checker shard runner are introduced. It does not activate selective acceptance, does not promote any acceptance status, and does not remove or weaken any existing CI gate.

### State declarations

- The active workflow `.github/workflows/ci.yml` remains the legacy/full CI authority. It is not modified by this slice.
- `scripts/tests/fixtures/ci-v2.yml` is inert test data only. It is not placed under `.github/workflows`. Its presence is not implementation or activation evidence.
- No selective omission, path filtering, or package-selective Rust is active.
- No acceptance matrix status is promoted by this slice.
- Trusted governance guard configuration, acceptance CLI authority choice, and selective activation remain later prerequisites outside this slice.

## Invariant ledger

| ID | Legacy invariant | Legacy carrier | Future v2 carrier | Mechanical test | Slice state |
|---|---|---|---|---|---|
| `CI-TR-001` | active workflow exact digest remains frozen | `.github/workflows/ci.yml` SHA-256 `919080325ade109dab32b556cbc97fb3fcd5844e45ad72e3b74ad231cb669146` | `scripts/tests/fixtures/ci-v2.yml` inert fixture is governed by explicit semantic invariants and is intentionally not whole-file fingerprinted | `scripts/check_repo_contracts.py` freezes the active workflow digest and enforces the fixture's semantic invariants; mutation tests remove/mutate each semantic invariant independently | `inert-not-active` |
| `CI-TR-002` | `pull_request` plus push-to-main trigger semantics remain | `.github/workflows/ci.yml` `on: pull_request` and `push: branches: [main]` | `scripts/tests/fixtures/ci-v2.yml` preserves `on: pull_request` and `push: branches: [main]` | `scripts/check_repo_contracts.py` validates trigger semantics in both active and fixture; mutation tests remove each trigger independently | `inert-not-active` |
| `CI-TR-003` | stable `rust` and `docs-and-contracts` job names remain | `.github/workflows/ci.yml` jobs `rust` and `docs-and-contracts` | `scripts/tests/fixtures/ci-v2.yml` preserves job names `rust` and `docs-and-contracts` | `scripts/check_repo_contracts.py` validates job names in both active and fixture; mutation tests rename/remove each independently | `inert-not-active` |
| `CI-TR-004` | full Python discovery is replaced only in the inert fixture by exact-inventory full sharding, with no omission | `.github/workflows/ci.yml` `docs-and-contracts` job runs `python3 -m unittest discover -s scripts/tests -p 'test_*.py'` | `scripts/tests/fixtures/ci-v2.yml` replaces discovery with `python3 scripts/run_checker_shards.py --jobs 4 --timeout-seconds 1800 --inventory scripts/checker_test_inventory.json --evidence-dir <RUNNER_TEMP> --require-clean --require-runner-image-identity` | `scripts/run_checker_shards.py` bidirectional inventory coverage and exact-union fan-in; `scripts/checker_test_inventory.json` sorted unique IDs with explicit count; mutation tests for missing/unexpected/duplicate/zero IDs | `inert-not-active` |
| `CI-TR-005` | full Rust fmt/clippy/workspace-test/doctest commands remain | `.github/workflows/ci.yml` `rust` job runs `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo test --locked --all-targets --all-features`, `cargo test --locked --all-features --doc` | `scripts/tests/fixtures/ci-v2.yml` preserves `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, `cargo test --workspace --all-targets --all-features --locked`, `cargo test --workspace --all-features --doc --locked` | `scripts/check_repo_contracts.py` validates all four Rust commands in fixture; mutation tests remove/mutate each command independently | `inert-not-active` |
| `CI-TR-006` | repository checker remains an exact executable command after the full checker suite | `.github/workflows/ci.yml` `docs-and-contracts` job runs `python3 scripts/check_repo_contracts.py` | `scripts/tests/fixtures/ci-v2.yml` runs `python3 scripts/check_repo_contracts.py` exactly once after the shard runner | `scripts/check_repo_contracts.py` validates checker command presence, count, and after-runner ordering in fixture; mutation tests remove/duplicate/reorder the command | `inert-not-active` |
