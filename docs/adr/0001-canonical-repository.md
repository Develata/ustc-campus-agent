# ADR-0001: Canonical repository

- `Status`: Accepted
- `Date`: `2026-07-22`

## Decision

Use `Develata/ustc-campus-agent` as the canonical private GitHub repository for source, pull requests, issues, Actions and future releases.

## Rationale

GitHub provides the required CI, review, branch-protection, release and coding-agent integration for the competition timeline. A single canonical collaboration surface avoids split ownership and divergent review state.

## Consequences

- Do not create a GitHub organization initially.
- Do not create a second Market repository until the split conditions in [`plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md) are real.
- Do not perform remote operations without Develata authorization; the current operation-specific/campaign mechanism lives in [`../tasks/00-module-work-policy.md`](../tasks/00-module-work-policy.md) §3, while tags, releases, publication and visibility remain operation-specific unless a grant explicitly names them.
- Repository docs describe product and collaboration contracts, not personal infrastructure.
