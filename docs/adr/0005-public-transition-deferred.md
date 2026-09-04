# ADR-0005: Public transition deferred (superseded for visibility and project license)

- `Status`: Superseded for repository visibility by Develata's 2026-09-04 public decision and for project licensing by Develata's 2026-09-05 MIT decision; release/download gates remain active
- `Date`: `2026-07-22`
- `Superseded`: `2026-09-04`
- `License decision`: `2026-09-05`

## Historical decision

The repository remains private during initial competition development but may become public later only after explicit approval and public-readiness gates pass.

## Current state

The GitHub repository is now intentionally public, and its project-authored software and documentation are licensed under MIT. This does not publish a tag, GitHub Release, Pages site, stable download or production deployment; those remain separate owner and evidence gates. Third-party content and campus data retain separate rights and source-permission requirements.

## Consequences

- Keep `LICENSE.md`, package metadata and competition-facing license wording consistent with MIT; any future license change requires another explicit owner decision.
- Do not publish GitHub Pages downloads until verified releases exist.
- Public-facing claims must match current manifests/features/acceptance evidence.
- The current gate is [`acceptance/public-readiness.md`](../acceptance/public-readiness.md); delivery/security constraints live in [`plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md).
