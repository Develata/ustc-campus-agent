# Campus Opportunity Graph

The Campus Opportunity Graph plugin represents campus opportunities as typed nodes and edges with eligibility, dependency, temporal windows, evidence, and user-profile facts.

## Course Planning bounded spike

Course Planning is currently the first implemented slice inside this Plugin:

- `OpportunityNode`: course offering;
- `RequirementNode`: curriculum requirement or elective group;
- `DependencyEdge`: prerequisite or recommended-before relation;
- `CoverageEdge`: course satisfies requirement;
- `ConflictEdge`: time or rule conflict;
- `TemporalWindow`: term/effective time;
- `EvidenceSignal`: official fact, mirror fact, community link-out, uncertainty;
- `ProfileFact`: user-provided academic snapshot or preference.

The development-time authority core is `crates/course-planning`. It validates the synthetic `course-planning/v0` fixture, applies source authority and hard constraints, and emits deterministic `course-plan-result/v0` JSON through the operator smoke command `ustc-agentctl course plan`.

This core is intentionally offline and read-only. The CLI smoke does **not** establish Market installation, enable/disable, grant, or Agent-discovery semantics. The crate is therefore not declared as an installable `NativeRustComponent` until the typed Market read/install path can enforce those lifecycle boundaries. Real catalog/iCourse adapters remain separate risk-first source spikes and must not weaken the source authority contract.

Future research/competition/lecture/scholarship packs must reuse the same core semantics. If they require a different ontology, they need a new ADR before entering this plugin. Opportunity Graph remains one of the three default first-party Plugins; Course Planning does not change the frozen cross-plugin implementation order.
