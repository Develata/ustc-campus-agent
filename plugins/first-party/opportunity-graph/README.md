# Campus Opportunity Graph

- Package ID: `ustc.opportunity-graph`
- Status: `partial-evidence`
- Product question: What fits me, and what should I choose next?

Canonical documentation:

- product behavior and honest status: [`docs/features/03-campus-opportunity-graph.md`](../../../docs/features/03-campus-opportunity-graph.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- implemented data model: [`docs/contracts/data-models.md`](../../../docs/contracts/data-models.md)

`crates/course-planning` remains the deterministic offline course pack.
`crates/opportunity-graph` owns exact consent, tenant-private profile,
qualification/planning, source/profile staleness and revoke/delete semantics.
The bounded MVP composition now adds typed M10 profile/view/plan/delete operations,
transaction-current Market authority for the four static application use cases, a DemoReviewed M60
source adapter, an atomic file-backed private-profile store and a colocated Web
journey. The declared native/resource components live under
[`market/packages/ustc.opportunity-graph/components`](../../../market/packages/ustc.opportunity-graph/components/).

This remains a loopback demo boundary: structured demo facts are synthetic and visibly
`DemoReviewed`. Course Planning also uses an iCourse aggregate-rating snapshot with
unresolved data-use permission; see the [current data scope](../../../docs/features/06-mvp-core-capabilities.md).
Production SSO/TLS, live USTC M60 retrieval, enrollment/application
effects and backup-erasure guarantees are not claimed.
