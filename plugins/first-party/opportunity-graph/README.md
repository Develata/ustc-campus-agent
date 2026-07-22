# Campus Opportunity Graph

The Campus Opportunity Graph plugin represents campus opportunities as typed nodes and edges with eligibility, dependency, temporal windows, evidence, and user-profile facts.

## First vertical slice

Course Planning is the first slice:

- `OpportunityNode`: course offering;
- `RequirementNode`: curriculum requirement or elective group;
- `DependencyEdge`: prerequisite or recommended-before relation;
- `CoverageEdge`: course satisfies requirement;
- `ConflictEdge`: time or rule conflict;
- `TemporalWindow`: term/effective time;
- `EvidenceSignal`: official fact, mirror fact, community link-out, uncertainty;
- `ProfileFact`: user-provided academic snapshot or preference.

Future research/competition/lecture/scholarship packs must reuse the same core semantics. If they require a different ontology, they need a new ADR before entering the flagship plugin.
