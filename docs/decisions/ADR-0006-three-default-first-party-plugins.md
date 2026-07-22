# ADR-0006: Three default first-party Plugins and frozen implementation order

- Status: Accepted
- Date: 2026-07-22
- Amends: [`ADR-0003`](ADR-0003-campus-opportunity-graph.md)

## Context

The confirmed product architecture has three formal default first-party Plugins. A later single-flagship proposal incorrectly turned implementation prioritization into product identity, moved Affairs Navigator and ChangeRadar outside the competition product, and promoted Course Planning to the first platform slice without approval.

## Decision

USTC Campus Agent has three formal, independently installable and disableable default first-party `PluginPackage`s:

| Package ID | Product-facing name | User question |
|---|---|---|
| `ustc.affairs-navigator` | USTC Affairs Navigator | What should I do now? |
| `ustc.change-radar` | USTC ChangeRadar | What changed, and does it affect me? |
| `ustc.opportunity-graph` | Campus Opportunity Graph | What fits me, and what should I choose next? |

They are three projections over one Campus Trust Kernel, not three independent crawlers or Agent runtimes:

```text
approved official sources
→ Source Registry
→ immutable revisions and normalized facts
→ temporal / conflict / provenance authority
   ├── procedure projection → Affairs Navigator
   ├── change projection    → ChangeRadar
   └── opportunity graph + consent-aware profile → Opportunity Graph
```

The implementation order is frozen as:

```text
ChangeRadar source/revision/diff foundation
→ Affairs Navigator structured procedure entry
→ ChangeRadar per-board semantic feed and RSS/Atom
→ Opportunity Graph consent/profile integration
```

Course Planning remains a valid vertical slice **inside** `ustc.opportunity-graph`. The merged deterministic planner is retained as an out-of-order bounded spike: it proves fixture validation and hard-constraint planning, but it does not redefine the platform's product topology, implementation order, or Market lifecycle readiness.

No Chinese product brand or `Course Compass` display name is approved. Such labels remain absent until Develata explicitly chooses them.

## Consequences

- All three package identities must exist in the Market catalog and Rust authority constants.
- Planned package manifests must state their implementation status and must not claim executable components.
- The next implementation mainline returns to the ChangeRadar foundation and then Affairs Navigator.
- Affairs and ChangeRadar share one source/revision/change ledger; they must not build separate crawler authority.
- Opportunity Graph work may reuse the existing Course Planning spike later, after the shared source/temporal foundation and consent boundary are in place.
- The single-flagship proposal is preserved under `docs/legacy/` only as rejected history.
