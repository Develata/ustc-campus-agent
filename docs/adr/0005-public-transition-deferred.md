# ADR-0005: Public transition deferred

- `Status`: Accepted
- `Date`: `2026-07-22`

## Decision

The repository remains private during initial competition development but may become public later only after explicit approval and public-readiness gates pass.

## Consequences

- Do not add an open-source license accidentally.
- Do not publish GitHub Pages downloads until verified releases exist.
- Public-facing claims must match current manifests/features/acceptance evidence.
- The current gate is [`acceptance/public-readiness.md`](../acceptance/public-readiness.md); delivery/security constraints live in [`plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md).
