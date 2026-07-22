# Ownership map

This file records collaboration lanes before a GitHub Organization or formal team aliases exist.

## Current state

- GitHub repository owner: Develata personal account.
- CODEOWNERS fallback: `@Develata` for all paths.
- Formal lane owners are **TBD until team members accept roles**. Do not invent GitHub usernames or team aliases in CODEOWNERS.

## Required lanes before the first feature PR

| Lane | Paths / contracts | Initial responsibility |
|---|---|---|
| Product / Source | `docs/plan/`, `docs/contracts/source-import.md`, source fixtures | Course Planning journey, source authority, USTC/iCourse permission path |
| Backend / Runtime / Security | `crates/`, `apps/`, `docs/contracts/`, `SECURITY.md` | Rust authority core, install/grant/gateway/privacy |
| Frontend / Demo | future `apps/web/`, `docs/public/`, demo script | Market + Agent Web/PWA, Pages handoff, browser evidence |
| Evaluation / Release | `docs/acceptance/`, `.github/workflows/`, `scripts/` | CI, evidence, fixture oracle, release/public gates |
| Market steward | `market/`, `plugins/first-party/` | PluginPackage schema, review policy, first-party packages |

## Promotion rule

When team GitHub usernames are known, update `.github/CODEOWNERS` in one small PR. If a GitHub Organization is created later, replace personal usernames with team aliases and add an ADR for the ownership move.
