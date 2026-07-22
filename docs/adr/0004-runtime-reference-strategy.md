# ADR-0004: Runtime reference strategy

- `Status`: Accepted
- `Date`: `2026-07-22`

## Decision

Keep the Rust platform authority core. Use Rig, goose, Pi and LangGraph as references or bounded adapters/baselines rather than adopting any as canonical platform authority.

## Consequences

- Grants, approvals, receipts, audit and source revisions are platform-domain facts.
- Framework checkpoints are adapter state only.
- Mature runtime adoption requires an equal-contract spike and a new/amending ADR.
- Current owned runtime and adoption gates live in [`plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md).
