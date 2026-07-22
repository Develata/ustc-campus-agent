# ADR-0002: Market boundary inside monorepo

- `Status`: Accepted
- `Date`: `2026-07-22`

## Decision

Keep `market/` inside the platform monorepo for the MVP while treating it as a strict logical Catalog Authority boundary.

## Rationale

A physical split would add cross-repository versioning, review, CI and release cost before independent package lifecycles are proven. A logical boundary preserves future split compatibility without that overhead.

## Consequences

- Market schemas/manifests remain declarative and independently validated.
- Runtime installation/grant state never enters catalog manifests.
- A future physical split follows the conditions in [`plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md), not aesthetic preference.
