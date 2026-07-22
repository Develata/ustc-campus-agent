# Market catalog boundary

`market/` is a logical source-of-truth boundary for PluginPackage metadata even while it lives inside the platform monorepo.

## Invariants

- Package manifests are declarative.
- First-party package ids are stable and reverse-DNS-like.
- Permission/capability expansion is never automatic.
- A malformed or secret-bearing manifest must be rejected before import.
- Runtime installation/grant state lives outside this catalog boundary.
- A future physical split to `ustc-campus-agent-market` must preserve this directory's contracts.

## Default first-party packages

- `packages/ustc.affairs-navigator/package.json` — USTC Affairs Navigator; planned structured-procedure product.
- `packages/ustc.change-radar/package.json` — USTC ChangeRadar; planned source/revision/diff and approved-feed product.
- `packages/ustc.opportunity-graph/package.json` — Campus Opportunity Graph; Course Planning exists as a bounded offline development spike.

The three package identities are equally formal. Their implementation sequence is ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph integration; implementation priority does not collapse the catalog to one flagship package.
