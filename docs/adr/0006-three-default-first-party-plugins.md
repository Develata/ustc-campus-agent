# ADR-0006: Three default first-party Plugins and frozen implementation order

- `Status`: Accepted
- `Date`: `2026-07-22`
- `Amends`: [`ADR-0003`](0003-campus-opportunity-graph.md)

## Context

The confirmed product architecture has three formal default first-party Plugins. A later single-flagship proposal incorrectly converted implementation prioritization into product identity, moved Affairs Navigator and ChangeRadar outside the competition product and promoted Course Planning without approval.

## Decision

USTC Campus Agent has exactly three independently versioned, installable and disableable default first-party `PluginPackage`s:

| Package ID | Product-facing name | User question |
|---|---|---|
| `ustc.affairs-navigator` | USTC Affairs Navigator | What should I do now? |
| `ustc.change-radar` | USTC ChangeRadar | What changed, and does it affect me? |
| `ustc.opportunity-graph` | Campus Opportunity Graph | What fits me, and what should I choose next? |

They are three projections over one Campus Trust Kernel, not three independent crawlers or Agent runtimes:

```text
approved sources
→ Source Registry
→ immutable revisions and normalized facts
→ temporal / conflict / provenance authority
   ├── procedure projection → Affairs Navigator
   ├── semantic change projection → ChangeRadar
   └── opportunity facts + consent-aware profile → Opportunity Graph
```

The implementation order is frozen:

```text
ChangeRadar source/revision/diff foundation
→ Affairs Navigator structured procedure entry
→ ChangeRadar per-board semantic feed and RSS/Atom
→ Opportunity Graph consent/profile integration
```

Course Planning remains a valid vertical slice inside `ustc.opportunity-graph`. The deterministic planner is an out-of-order bounded spike: it proves fixture validation and hard-constraint planning, but does not redefine product topology, implementation order or Market lifecycle readiness.

No Chinese product brand or `Course Compass` display name is approved.

## Consequences

- All three package identities exist in the Market catalog and Rust authority constants.
- Planned manifests state honest status and do not claim executable components.
- Within first-party product delivery, the next mainline returns to ChangeRadar and then Affairs Navigator; shared platform foundations may precede that product slice.
- Affairs and ChangeRadar share one source/revision/change ledger.
- Opportunity Graph may reuse Course Planning after shared source/temporal and consent boundaries exist.
- The rejected draft is not canonical documentation; this ADR preserves the decision and rationale without retaining a duplicate proposal dump.
- Current detailed contracts live in [`plan/02-product-positioning.md`](../plan/02-product-positioning.md) and [`plan/06-first-party-plugins.md`](../plan/06-first-party-plugins.md).
