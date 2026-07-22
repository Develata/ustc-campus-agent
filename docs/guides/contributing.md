# Contributing and collaboration guide

## Scope

USTC Campus Agent expects multi-human, multi-agent and multi-device work. This guide governs work slicing and review; repository authority remains in `AGENTS.md`, `docs/plan/` and typed contracts.

## Before editing

1. Read root `AGENTS.md`, `README.md` and `docs/AGENTS.md`.
2. Follow the reading order in `docs/coverage-matrix.md` for the touched domain.
3. Check `git status --short --branch` and preserve foreign work.
4. Name one owner and one reviewer.
5. Bind the slice to a task/ADR/contract/acceptance case.
6. State touched paths and explicit non-goals.

## Slice shape

Good:

```text
owning plan/contract
→ minimal implementation
→ feature projection
→ acceptance evidence
→ independent review
```

Bad:

```text
unrelated UI polish + schema change + runtime refactor + source parser + CI cleanup
```

Prefer one semantic boundary per PR.

## Ownership lanes

Formal team usernames remain TBD; do not invent aliases. Current CODEOWNERS fallback remains `@Develata`.

| Lane | Owns |
|---|---|
| Product / Source | three Plugin journeys, source/revision authority, source permissions and fixture oracles |
| Backend / Runtime / Security | Rust authority, Market/install/grants, Agent gateway, privacy/security |
| Frontend / Demo | Market + Agent Web/PWA, Chinese UX, browser evidence and Pages handoff |
| Evaluation / Release | acceptance registry, CI/release gates, restore and public-readiness evidence |
| Market steward | `market/`, package schema/policy and `plugins/first-party/` boundaries |

When actual team usernames are confirmed, update `.github/CODEOWNERS` in a narrow PR.

## Branch and PR contract

Branch patterns:

- `feat/<short-topic>`
- `fix/<short-topic>`
- `docs/<short-topic>`
- `chore/<short-topic>`
- `spike/<short-topic>` for disposable evidence

Every nontrivial PR includes:

- owner/reviewer and linked task/case;
- touched directories and non-goals;
- real validation output and explicit not-run gates;
- screenshots/browser evidence for visible behavior;
- rollback/recovery notes for stateful changes;
- explicit list of new files.

## Multi-agent rules

- One agent owns one slice at a time.
- Avoid broad formatting and unrelated cleanup.
- Do not edit outside the declared slice.
- Do not trust another agent's success claim without real output/read-back.
- Review current staged/worktree state, not a stale pre-fix snapshot.
- Use exact-path staging; never `git add -A`.
- Do not push, tag, publish or change visibility without explicit Develata authorization.

## Handoff

Before handoff:

1. run relevant gates;
2. update plan/feature/contract/acceptance mappings;
3. record failures/blocked/not-run honestly;
4. ensure no secrets, personal infrastructure or private data entered tracked docs/artifacts;
5. report worktree/commit/push state precisely.
