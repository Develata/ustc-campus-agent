# ADR-0002: Market boundary inside monorepo

- Status: Accepted
- Date: 2026-07-22

## Decision

Keep `market/` inside the platform monorepo for MVP, but treat it as a strict logical authority boundary.

## Rationale

A physical split would add cost before the PluginPackage contract is proven. A logical boundary preserves future split compatibility without cross-repo overhead.
