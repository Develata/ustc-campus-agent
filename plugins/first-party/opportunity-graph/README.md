# Campus Opportunity Graph

- Package ID: `ustc.opportunity-graph`
- Status: development; bounded offline Course Planning spike implemented
- Product question: What fits me, and what should I choose next?

Canonical documentation:

- product behavior and honest status: [`docs/features/03-campus-opportunity-graph.md`](../../../docs/features/03-campus-opportunity-graph.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- implemented data model: [`docs/contracts/data-models.md`](../../../docs/contracts/data-models.md)

`crates/course-planning` is currently an offline read-only proof. It is not declared as an installable component until Market installation, grant, enable/disable and Agent-discovery boundaries exist.
