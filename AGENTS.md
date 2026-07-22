# AGENTS.md — USTC Campus Agent

## Scope

This file governs all work in this repository. Read this file, then `README.md`, then the nearest domain README/contract before editing.

## Product boundary

- Project: **USTC Campus Agent**.
- Status: student competition project; not an official USTC service.
- Core product spine: Plugins Market + bounded campus Agent.
- First flagship Plugin: `ustc.opportunity-graph`.
- First vertical slice: Course Planning.
- Chinese product name: TBD; do not invent a new Chinese brand inside code/docs unless Develata decides it.

## Source of truth

- Current plans/contracts live under `docs/plan/`, `docs/architecture/`, `docs/contracts/`, and `docs/acceptance/`.
- Historical planning material is preserved under `docs/legacy/`; it is **not** canonical when it conflicts with current docs.
- `market/` is a logical catalog authority boundary even while it remains in this monorepo.
- Runtime generated state, credentials, local snapshots, and `.codegraph/` are not source.

## Engineering rules

- Correctness and safety precede UX, compatibility, maintainability, and performance.
- Define contracts before implementation for public APIs, CLI commands, schemas, permissions, source import, and Agent run state.
- Keep Platform authority in Rust domain code. Framework checkpoints, model transcripts, adapters, and browser/UI state cannot overwrite grants, approvals, receipts, audit, or source revisions.
- No direct writes to `main`; use PRs and exact-file staging.
- Do not push, tag, publish, or change GitHub visibility without explicit Develata approval.
- Do not use `git add -A`; stage exact files only.
- No secrets or real personal data in commits, fixtures, logs, screenshots, or Pages.
- For docs-only changes, run `python3 scripts/check_repo_contracts.py` and `git diff --check`.
- For Rust changes, run fmt, clippy, tests, and contract checks.

## Collaboration model

Multi-human, multi-agent, multi-device work is expected. Before a nontrivial slice:

1. Pick one owner and one reviewer.
2. Bind the slice to an issue/contract/case ID.
3. State touched directories and non-goals.
4. Prefer small PRs that change one semantic boundary.
5. Include real validation output in the PR.

## Public transition guard

This repo may become public later, but public visibility is a release/security decision. Before that: choose license, scrub secrets, replace private fixtures, verify iCourse/USTC source permissions, add non-official disclaimers, and pass the public-readiness checklist.
