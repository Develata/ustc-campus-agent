# CI workflow transition invariant ledger

## Metadata

- `Layer`: `Acceptance / CI transition`
- `Status`: `active-full-sharded`
- `Version`: `ci-transition/v2`
- `Last Review`: 2026-08-27
- `Authority Owns`: active full-suite CI sharding, required-context continuity, PR supersession cancellation, and trusted-base governance-controller activation state
- `Authority Defers To`: `.github/workflows/ci.yml` for active full build/test CI; `.github/workflows/ci-governance.yml` for the required governance Check Run; `scripts/tests/fixtures/ci-v2.yml` for retained mutation/reference test data

## Scope

This ledger is the transition authority for the M90 CI evidence shard activation and trusted-base governance controller. It records that full exact-inventory checker sharding is active inside the stable `docs-and-contracts` required job while full Rust and repository checks remain unconditional. It does not activate selective acceptance, omit tests by path/package, promote an acceptance row, or change product authority.

### State declarations

- The active workflow `.github/workflows/ci.yml` is the full exact-inventory sharded CI authority.
- `scripts/tests/fixtures/ci-v2.yml` remains inert reference and mutation-test data; it is not a second active workflow.
- No selective omission, path filtering, or package-selective Rust is active.
- No acceptance matrix status is promoted by this activation.
- Pull-request supersession cancellation is active; push-to-main runs are never cancelled by this policy.
- The stable required contexts remain `rust`, `docs-and-contracts`, and `ci-governance`.
- The trusted-base workflow `.github/workflows/ci-governance.yml` publishes the head-scoped required `ci-governance` Check Run and executes no PR-controlled bytes.
- `Status`: `active-full-sharded`
- `Version`: `ci-transition/v2`

## Invariant ledger

| ID | Legacy invariant | Active v2 carrier | Retained reference carrier | Mechanical test | Slice state |
|---|---|---|---|---|---|
| `CI-TR-001` | active workflow must remain fail-closed without a mutable whole-file fingerprint | `.github/workflows/ci.yml` is governed by explicit semantic invariants for triggers, jobs, pinned actions, commands, evidence and cancellation | `scripts/tests/fixtures/ci-v2.yml` remains semantic mutation/reference data and is intentionally not whole-file fingerprinted | `scripts/check_repo_contracts.py` validates active and reference workflow semantics; mutation tests remove or relocate each invariant independently | `active-full-sharded` |
| `CI-TR-002` | `pull_request` plus push-to-main trigger semantics remain | `.github/workflows/ci.yml` preserves both triggers and cancels only superseded pull-request runs | `scripts/tests/fixtures/ci-v2.yml` preserves the trigger baseline without owning active cancellation | checker mutation tests remove each trigger, widen cancellation to push, or drift the concurrency group | `active-full-sharded` |
| `CI-TR-003` | stable required `rust` and `docs-and-contracts` context names remain | `.github/workflows/ci.yml` preserves the active job IDs and explicit display names `rust` and `docs-and-contracts` | `scripts/tests/fixtures/ci-v2.yml` preserves its reference job IDs | checker mutation tests rename/remove each job or display name independently | `active-full-sharded` |
| `CI-TR-004` | full Python discovery may be replaced only by an exact full-suite proof with no omission | `.github/workflows/ci.yml` runs `python3 scripts/run_checker_shards.py --jobs 4 --timeout-seconds 1800` over the exact inventory | `scripts/tests/fixtures/ci-v2.yml` retains the same runner shape | bidirectional inventory coverage, exact-union fan-in, process isolation, per-shard reports, and missing/unexpected/duplicate/zero-ID mutation tests | `active-full-sharded` |
| `CI-TR-005` | full Rust fmt/clippy/workspace-test/doctest commands remain unconditional | `.github/workflows/ci.yml` runs all four full Rust commands in required job `rust` | `scripts/tests/fixtures/ci-v2.yml` retains the same full Rust command set | checker mutation tests remove or conditionalize each command/job; exact-head Actions proves execution | `active-full-sharded` |
| `CI-TR-006` | repository checker remains an exact executable command after successful full checker fan-in | `.github/workflows/ci.yml` runs `python3 scripts/check_repo_contracts.py` exactly once after the shard runner and always uploads runner evidence | `scripts/tests/fixtures/ci-v2.yml` retains checker ordering and evidence-upload shape | executable-command parsing rejects display-only copies, duplicates, reordering, missing `always()`, and missing evidence paths | `active-full-sharded` |
| `CI-TR-007` | trusted-base governance remains required and PR-byte-independent | `.github/workflows/ci-governance.yml` publishes one head-scoped app-bound `ci-governance` Check Run; protected main requires it with strict head freshness | branch-protection read-back binds `rust`, `docs-and-contracts`, and `ci-governance` to GitHub Actions app ID `15368` | structural workflow tests plus negative-to-positive same-ID smoke, exact owner grant, Check Run read-back, and strict branch-protection read-back | `guard-required` |
