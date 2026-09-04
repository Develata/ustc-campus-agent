# ADR-0005: Public transition deferred (superseded for visibility)

- `Status`: Superseded for repository visibility by Develata's 2026-09-04 public decision; licensing and release/download gates remain active
- `Date`: `2026-07-22`
- `Superseded`: `2026-09-04`

## Historical decision

The repository remains private during initial competition development but may become public later only after explicit approval and public-readiness gates pass.

## Current state

The GitHub repository is now intentionally public and source-visible. This does not grant an open-source license and does not publish a tag, GitHub Release, Pages site, stable download or production deployment; those remain separate owner and evidence gates.

## Consequences

- Do not add an open-source license without a separate explicit owner decision.
- Do not publish GitHub Pages downloads until verified releases exist.
- Public-facing claims must match current manifests/features/acceptance evidence.
- The current gate is [`acceptance/public-readiness.md`](../acceptance/public-readiness.md); delivery/security constraints live in [`plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md).
