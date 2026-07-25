# AGENTS.md — USTC Campus Agent

## Scope

This file governs all work in this repository. Read this file, then `README.md`, then the nearest domain README/contract before editing.

## Product boundary

- Project: **USTC Campus Agent**.
- Status: student competition project; not an official USTC service.
- Core product spine: Plugins Market + bounded campus Agent.
- Default first-party Plugins: `ustc.affairs-navigator`, `ustc.change-radar`, and `ustc.opportunity-graph`.
- Frozen first-party product implementation order: ChangeRadar source/revision/diff foundation → Affairs Navigator structured procedure entry → ChangeRadar board feed → Opportunity Graph consent/profile integration. Shared platform foundations such as the framework-neutral Agent runtime kernel and minimal Market resolver may precede that product sequence when the dependency roadmap requires them.
- Course Planning is an out-of-order bounded spike inside Opportunity Graph; it does not change product topology, first-party product implementation order, or Market/runtime readiness.
- Chinese product name: TBD; do not invent a new Chinese brand inside code/docs unless Develata decides it.

## Source of truth

- Documentation roles and reading order are governed by `docs/AGENTS.md` and `docs/coverage-matrix.md`.
- Current engineering authority lives under `docs/plan/` and `docs/contracts/`; features, acceptance, tasks, guides, overview and ADRs have the distinct roles defined by `docs/AGENTS.md`.
- `market/` is a logical catalog authority boundary even while it remains in this monorepo.
- Raw discovery archives, personal infrastructure/backup procedures, runtime generated state, credentials, local snapshots, and `.codegraph/` are not repository source.

## Engineering rules

- Correctness and safety precede UX, compatibility, maintainability, and performance.
- Define contracts before implementation for public APIs, CLI commands, schemas, permissions, source import, and Agent run state.
- Keep Platform authority in Rust domain code. Framework checkpoints, model transcripts, adapters, and browser/UI state cannot overwrite grants, approvals, receipts, audit, or source revisions.
- Keep the Dioxus Fullstack boundary thin. Web/PWA, Docker Compose server and Android are required targets; iOS/desktop are later. Client code renders server-owned state and submits typed intent. An admitted Dioxus server function MAY call one public application command/query port, but neither client nor server-function adapters may own domain calculation/mutation or reach concrete repositories, executors, providers or journals. Product authority remains in explicit backend/application modules.
- Remote `main` remains protected. Large modules normally use a dedicated branch and PR. During the current solo skeleton phase, Develata MAY explicitly allow local `main` work for root interfaces and composition scaffolding, but this does not bypass remote branch protection, review, or current-operation push approval.
- Do not push, tag, publish, or change GitHub visibility without explicit Develata approval.
- Do not use `git add -A`; stage exact files only.
- No secrets or real personal data in commits, fixtures, logs, screenshots, or Pages.
- For docs-only changes, run `python3 scripts/check_repo_contracts.py` and `git diff --check`.
- For Rust changes, run fmt, clippy, tests, and contract checks.

## Mandatory work loop

Every work item MUST follow this order:

1. Read `docs/plan/00-engineering-constitution.md` carefully.
2. Read `docs/plan/01-terminology.md` carefully.
3. Read the relevant `docs/plan/` chapters and decide whether the authoritative plan must change.
4. Read the corresponding contracts, features, acceptance rows, registries, overview and task documents, and decide which projections must change.
5. Implement the smallest cohesive code or documentation slice only after the governing documents are clear.
6. Run a quick gate over every changed **and untracked** file in the slice; plain `git diff` is insufficient for new files.
7. Use at most three independent review subagents; the main agent independently verifies every finding, fixes all accepted blockers and closes every review lane.
8. Run final baseline checks, contract checks and the bound acceptance-matrix commands.
9. Exercise the real feature path when the slice has a runnable client, CLI, API, runtime or integration surface; otherwise record the smoke as not applicable rather than inventing evidence.
10. Loop to the next planned small module or make an exact-scope commit.

Code is a projection of `docs/plan/` and contracts. Tests and real smoke are projections of acceptance rows. A task or implementation convenience cannot override the plan.

## Collaboration model

Multi-human, multi-agent, multi-device work is expected. Before a nontrivial slice:

1. Pick one owner and one reviewer.
2. Bind the slice to an issue/contract/case ID.
3. State touched directories and non-goals.
4. Place the work inside one independently owned large module with an explicit public boundary.
5. Split that large module into small high-cohesion, low-coupling implementation modules that can be reviewed and committed in batches.
6. Include real validation output in each commit/PR review packet.

A "large module" is defined by independence, not line count. It owns one coherent responsibility, hides its internal state, exposes narrow versioned inputs/outputs, can be developed against fake counterparts, and can join the product through the composition root without requiring unrelated large modules to finish. Large modules combine like independently tested robot parts; they do not reach into one another's internals.

Small modules MAY be committed incrementally on the large-module branch. Push/PR/merge happens only after the large module's own contracts and exit gate pass and Develata authorizes the current remote operation. Cross-module integration code belongs in `ustc-agentd` or another declared composition surface, never as a hidden dependency inside one module.

## Public transition guard

This repo may become public later, but public visibility is a release/security decision. Before that: choose license, scrub secrets, replace private fixtures, verify iCourse/USTC source permissions, add non-official disclaimers, and pass the public-readiness checklist.
