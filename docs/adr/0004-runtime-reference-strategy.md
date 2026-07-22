# ADR-0004: Runtime reference strategy

- `Status`: Accepted
- `Date`: `2026-07-22`
- `Amended`: `2026-07-23`

## Decision

Keep the Rust platform authority core. Treat Rig, LangGraph, Pi Agent, goose and Hermes Agent as mandatory capability-level design references whenever their relevant runtime surface is being designed. They remain references, benchmarks or bounded adapters rather than canonical platform authority.

Reference does not mean combining five frameworks. For each new runtime capability, the design must inspect current official documentation/source, identify the exact invariant or mechanism worth borrowing, record what is deliberately rejected, and map the accepted pattern into platform-owned types and acceptance evidence.

## Consequences

- Grants, approvals, receipts, audit and source revisions are platform-domain facts.
- Framework checkpoints are adapter state only.
- Provider loops, event streams, interruption, context shaping, tool registries, skills, memory and delegation are compared by capability; no framework receives authority merely because its implementation is mature.
- Dated source/release evidence must be revalidated before adoption or upgrade. A remembered feature list is not sufficient architecture evidence.
- Mature runtime adoption requires an equal-contract spike and a new/amending ADR.
- Current owned runtime and adoption gates live in [`plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md).
