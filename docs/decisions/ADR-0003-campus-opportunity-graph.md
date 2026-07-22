# ADR-0003: Course Planning belongs to Campus Opportunity Graph

- Status: Amended by [`ADR-0006`](ADR-0006-three-default-first-party-plugins.md)
- Date: 2026-07-22

## Decision

Course Planning is a vertical slice inside `ustc.opportunity-graph`, not a separate top-level `PluginPackage`.

`ADR-0006` rejects the former single-flagship interpretation: Opportunity Graph is one of three formal default first-party Plugins, and Course Planning is retained as an out-of-order bounded spike rather than implementation-order authority.

## Rationale

Course selection is naturally an opportunity-planning problem with eligibility, dependency, temporal windows, conflicts, evidence, and user profile facts. This lets the MVP prove a concrete journey while preserving a coherent extension path to research, competitions, talks, and scholarships.
