<!-- Parent: ../AGENTS.md -->

# Documentation governance

## Purpose

`docs/` is split by semantic role. The repository is intentionally smaller than Deve-Notebook, so it copies the layering discipline without creating empty registries or speculative chapters.

- `plan/`: current engineering blueprint—authority, state, boundaries, failure and verification.
- `features/`: user-visible behavior and honest journeys.
- `contracts/`: typed CLI, schema, permission, interface and data contracts.
- `acceptance/`: active proof bindings plus the retained long-horizon case catalog. `matrix.tsv` is the current gate registry; catalog-only cases are non-pass and non-active.
- `overview/`: cross-layer maps; never a second source of authority.
- `tasks/`: implementation order and scoped delivery slices; tasks do not override plans.
- `guides/`: contributor, local-development and publication handoffs.
- `adr/`: time-ordered architecture decisions; ADRs explain why, while current plans define how.
- `coverage-matrix.md`: explicit mapping across blueprint, feature, contract and acceptance layers.

Raw discovery notes, rejected drafts, personal infrastructure, private backup procedures and copied chat/workspace archives do not belong in repository documentation.

## Reading order

Before changing product or runtime behavior:

1. read `plan/00-engineering-constitution.md`;
2. read `plan/01-terminology.md`;
3. read the matching plan chapter;
4. read its feature and contract projections through `coverage-matrix.md`;
5. read the matching acceptance cases and current task slice.

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
- Every plan chapter MUST state scope, authority, failure/recovery and verification entrypoints.
- Move dated evidence to a future `report/` directory only when real evidence exists; do not create the directory pre-emptively.
- Run `python3 scripts/check_repo_contracts.py` and `git diff --check` after docs-only changes.
