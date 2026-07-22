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
- Do not push, tag, publish, create a release or change visibility without explicit Develata approval.
- Repository docs describe product and collaboration contracts, not personal infrastructure.
