# Campus Opportunity Graph

- Package ID: `ustc.opportunity-graph`
- Status: `partial-evidence`
- Product question: What fits me, and what should I choose next?

Canonical documentation:

- product behavior and honest status: [`docs/features/03-campus-opportunity-graph.md`](../../../docs/features/03-campus-opportunity-graph.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- implemented data model: [`docs/contracts/data-models.md`](../../../docs/contracts/data-models.md)

`crates/course-planning` remains the deterministic offline course pack.
`crates/opportunity-graph` now adds bounded exact-consent, tenant-private profile,
qualification/planning, source/profile staleness and revoke/delete evidence.
Neither crate is yet a complete installable component: M10 ingress, Market
installation/grants, shared Harness/ToolGateway composition, durable profile
state, M00 consent UI/command and live M60 retrieval remain unimplemented.
