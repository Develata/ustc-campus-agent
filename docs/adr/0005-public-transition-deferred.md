# ADR-0005: Public transition deferred (superseded)

- `Status`: Superseded by Develata's 2026-09-04 public/MIT decision
- `Date`: `2026-07-22`
- `Superseded`: `2026-09-04`

## Historical decision

The repository remains private during initial competition development but may become public later only after explicit approval and public-readiness gates pass.

## Current state

The repository is now public and the project source is licensed under the MIT License. This supersedes only the earlier private/no-license posture. GitHub Pages, download links, tags, Releases, broader source ingestion and production deployment claims remain separately gated publication decisions.

## Consequences

- Preserve the MIT license and distinguish source licensing from third-party data/source permissions.
- Do not publish GitHub Pages downloads until verified releases exist.
- Public-facing claims must match current manifests/features/acceptance evidence.
- The current gate is [`acceptance/public-readiness.md`](../acceptance/public-readiness.md); delivery/security constraints live in [`plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md).
