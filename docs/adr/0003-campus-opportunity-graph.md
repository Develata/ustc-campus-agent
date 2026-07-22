# ADR-0003: Course Planning belongs to Campus Opportunity Graph

- `Status`: Amended by [`ADR-0006`](0006-three-default-first-party-plugins.md)
- `Date`: `2026-07-22`

## Decision

Course Planning is a vertical slice inside `ustc.opportunity-graph`, not a separate top-level `PluginPackage`.

ADR-0006 rejects the former single-flagship interpretation: Opportunity Graph is one of three formal default first-party Plugins, and Course Planning is retained as an out-of-order bounded spike rather than implementation-order authority.

## Rationale

Course selection is an opportunity-planning problem with eligibility, dependency, temporal windows, conflicts, evidence and user-profile facts. The slice proves a concrete deterministic journey while preserving extension to research, competitions, talks and scholarships.

## Consequences

The current status and limits are owned by [`features/03-campus-opportunity-graph.md`](../features/03-campus-opportunity-graph.md) and [`contracts/data-models.md`](../contracts/data-models.md).
