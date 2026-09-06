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

- `packages/ustc.affairs-navigator/package.json` — USTC Affairs Navigator; bounded reviewed-procedure query/publication evidence; broader sources and production administration remain planned.
- `packages/ustc.change-radar/package.json` — USTC ChangeRadar; bounded source/revision/diff plus durable fixed-administrator JSON/Atom publication product; production source and administration remain planned.
- `packages/ustc.opportunity-graph/package.json` — Campus Opportunity Graph; declares public plus explicit-consent private capabilities and component/resource descriptors for the active bounded composition candidate.

The three package identities are equally formal. Their implementation sequence is ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph integration; implementation priority does not collapse the catalog to one flagship package.

## Read and use packages

The catalog declares packages; installation and grants belong to the runtime.
See the [current Market feature](../docs/features/00-market-browse-install.md) and
[MCP/Skill setup](../docs/guides/mcp-skills.md) for the implemented public-read profile.
