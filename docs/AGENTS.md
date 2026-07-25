<!-- Parent: ../AGENTS.md -->

# Documentation governance

## Purpose

`docs/` is split by semantic role. The repository is intentionally smaller than Deve-Notebook, so it copies the layering discipline without creating empty registries or speculative chapters.

- `plan/`: current engineering blueprint—foundation/cross-system policy plus one independently owned blueprint per large module under `plan/modules/`.
- `features/`: user-visible behavior and honest journeys.
- `contracts/`: typed module boundaries, CLI, schema, permission, interface and data contracts.
- `acceptance/`: active proof bindings plus the retained long-horizon case catalog. `matrix.tsv` is the current gate registry; catalog-only cases are non-pass and non-active.
- `overview/`: cross-layer maps; never a second source of authority.
- `tasks/`: large-module work policy, small-module batches and assembly order; tasks do not override plans.
- `guides/`: contributor, local-development and publication handoffs.
- `adr/`: time-ordered architecture decisions; ADRs explain why, while current plans define how.
- `coverage-matrix.md`: explicit mapping across blueprint, feature, contract and acceptance layers.

Raw discovery notes, rejected drafts, personal infrastructure, private backup procedures and copied chat/workspace archives do not belong in repository documentation.

## Reading order

Before changing product or runtime behavior:

1. read `plan/00-engineering-constitution.md`;
2. read `plan/01-terminology.md`;
3. read `plan/modules/00-module-map.md` and the matching large-module blueprint;
4. read the matching cross-system plan chapter;
5. read `contracts/module-boundaries.md` and matching specific contracts through `coverage-matrix.md`;
6. read the matching feature, acceptance cases, `tasks/00-module-work-policy.md` and current roadmap batch.

## Authority rules

- Current plan and typed contracts MUST agree. If they do not, stop and resolve the contradiction explicitly.
- Features describe what users observe; they MUST NOT invent authority or runtime semantics.
- Acceptance rows prove claims; `planned` is not pass.
- Tasks schedule work but MUST NOT redefine product topology or invariants.
- Overview and guides summarize; they MUST link to, rather than duplicate, owning contracts.
- ADRs are decision history. An amended ADR is not current behavior authority.
- Code is a projection of approved plans/contracts, not an excuse to weaken them.

## Editing discipline

- Keep chapters cohesive and proportionate; do not create empty placeholder trees.
- Every new public behavior needs a feature projection and an acceptance binding.
- Every cross-system plan MUST state scope, authority, failure/recovery and verification entrypoints. Every large-module blueprint MUST additionally satisfy the complete blueprint contract in `plan/AGENTS.md`.
- Move dated evidence to a future `report/` directory only when real evidence exists; do not create the directory pre-emptively.
- Run `python3 scripts/check_repo_contracts.py` and `git diff --check` after docs-only changes.
