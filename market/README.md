# Market catalog boundary

`market/` is a logical source-of-truth boundary for PluginPackage metadata even while it lives inside the platform monorepo.

## Invariants

- Package manifests are declarative.
- First-party package ids are stable and reverse-DNS-like.
- Permission/capability expansion is never automatic.
- A malformed or secret-bearing manifest must be rejected before import.
- Runtime installation/grant state lives outside this catalog boundary.
- A future physical split to `ustc-campus-agent-market` must preserve this directory's contracts.

## Current package

- `packages/ustc.opportunity-graph/package.json` — Campus Opportunity Graph with the Course Planning domain pack.
