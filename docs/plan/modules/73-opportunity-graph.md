# M72 — Campus Opportunity Graph

## Metadata

- `Module ID`: `M72`
- `Package ID`: `ustc.opportunity-graph`
- `Status`: Accepted blueprint; offline Course Planning spike exists
- `Implementation State`: `bounded-spike`
- `Version`: `m72-opportunity-graph/v0`
- `Last Review`: `2026-07-25`
- `Primary code areas`: `plugins/first-party/opportunity-graph/`, current `crates/course-planning/`, future cohesive opportunity/profile modules

## 1. Purpose

`M72` answers: “What fits me, and what should I choose next?” It owns typed campus opportunities and tenant-private profile projections used for qualification, dependency, conflict, temporal availability, matching and planning.

Course Planning is one independently demonstrable journey inside this module, not the module's entire identity.

## 2. Non-goals

- owning public source/revision authority;
- building a universal property graph or generic graph database abstraction;
- storing raw USTC passwords/CAS sessions;
- cross-user profile data or public leakage of private preferences;
- automatic course enrollment/application submission;
- allowing model explanations to add facts/courses past deterministic validation.

## 3. Owned objects and state

```text
OpportunityId / OpportunityKind
EligibilityCondition / Dependency / Coverage / Conflict
TemporalWindow
OpportunityFactRef with M60 provenance
TenantProfileFact / ConsentRecord
ProfileProjection / DerivedMatch
PlanCandidate / Explanation / Stale state
```

Public opportunity facts and tenant-private profile facts are separate state classes. Derived matches never enter the public graph.

## 4. Public inputs and outputs

Inputs:

```text
reviewed M60 fact/revision references
create/view/update/delete consented profile fact commands
opportunity query/filter request
qualification/match/plan request
current source/profile snapshot IDs
```

Outputs:

```text
reviewed opportunity view with provenance/freshness
qualification/dependency/conflict result
ranked bounded candidate plans/matches
explanation containing evidence and uncertainty
profile deletion/revocation receipt
```

## 5. Dependency direction

Allowed dependencies:

- `M60` typed fact/revision/provenance/freshness interfaces;
- `M00` tenant/user/request context;
- `M90` tenant-scoped repository, consent/audit, artifact and clock ports;
- optional `M30` explanation assistance after deterministic results are fixed.

Forbidden dependencies:

- source fetch/parser internals;
- `M70`/`M71` private state;
- Dioxus/client types;
- model output as hard eligibility/planning truth;
- shared non-tenant-keyed profile cache.

## 6. Lifecycle

Public fact path:

```text
accepted M60 fact/revision
→ typed opportunity validation
→ current reviewed opportunity projection
→ stale/superseded/archive under source/time policy
```

Private profile path:

```text
explicit consent/input
→ tenant-owned profile fact
→ bounded derived qualification/match/plan
→ view/update/revoke/delete
→ delete durable payload and policy-covered recoverable copies
```

A candidate becomes stale when its pinned source/profile revision changes.

## 7. Failure and recovery

- Missing/stale/conflicting public fact: uncertainty/refusal, not invented match.
- Profile/consent missing or wrong tenant: deny before read/derivation.
- Unresolved alias/identity: exclude and warn, never guess.
- Hard constraint failure: candidate rejected even if soft/model score is high.
- Repository/cache deletion failure: do not claim deletion complete.
- Source/profile revision drift: mark prior result stale; recompute under explicit request/policy.
- Model explanation inconsistency: reject explanation or fall back to deterministic rationale.
- Projection loss: rebuild public views from M60 and private views from tenant-owned facts.

## 8. Configuration and secrets

Typed policies cover opportunity kinds, required fields, source classes, consent purpose/retention, hard constraint sets, ranking limits and explanation bounds. Profile state contains user-provided facts, not raw institutional credentials. iCourse remains link-out-only unless explicit permission changes.

## 9. Observability

Record tenant-safe request IDs, public source revisions, profile snapshot/consent IDs, candidate IDs, hard/soft decision classes, stale/conflict/uncertainty and deletion receipts. Logs avoid raw academic/profile payload by default.

## 10. Extension and replacement

Course, research, competition, lecture and scholarship packs are peer typed domain packs when they reuse stable opportunity/profile semantics. A materially different object/relationship model requires separate review; do not stretch one generic graph abstraction to force reuse. Deterministic planners/rankers and optional explainers are replaceable.

## 11. Performance path

Use typed indexed facts and tenant-scoped profile projections. Candidate generation is bounded by policy (for example beam width/result count), then independently recomputes hard constraints. Do not expose unbounded graph traversal or model ranking over raw campus/profile data.

## 12. Scope boundary

**MVP**

- one reviewed opportunity family, initially Course Planning integration if source/profile contracts are honest;
- tenant-isolated consented profile snapshot;
- deterministic eligibility/dependency/conflict/planning;
- provenance, freshness and uncertainty;
- view/delete profile behavior;
- bounded optional explanation that cannot change result.

**Later**

- research/competition/lecture/scholarship packs;
- richer cross-pack typed relations where real reuse exists;
- additional deterministic planners/rankers;
- consented notifications.

**Explicit non-goals**

- universal graph platform;
- automatic enrollment/application;
- community signal authoring hard facts;
- cross-user profile inference;
- live institutional password storage.

## 13. Small-module decomposition

1. `opportunity-types` — stable typed facts/relations/windows.
2. `opportunity-validation` — required fields/source/time/conflict.
3. `profile-domain` — tenant facts, consent, view/update/delete.
4. `qualification` — deterministic eligibility/dependency/coverage.
5. `conflict-engine` — temporal/resource/identity conflicts.
6. `candidate-engine` — bounded planning/matching.
7. `candidate-validation` — independent hard-constraint recomputation.
8. `ranking` — soft preferences/community signals below hard facts.
9. `explanation` — deterministic evidence and bounded optional model prose.
10. `course-pack` — current Course Planning adapter/domain pack.
11. future peer domain packs with separate contracts.
12. `opportunity-ports` — public/profile repositories, clock and audit fakes.

Existing `course-planning` is reviewed as items 6–8 and 10. It does not prove profile consent, Market lifecycle or live source integration.

## 14. Exit gate

`M72` is standalone-ready when public/private separation, tenant denial, consent/deletion, stale/conflict, deterministic planning and explanation consistency pass against fakes. It is accepted when one installed-plugin journey uses reviewed source facts and a tenant-owned profile to produce a zero-hard-violation result with provenance, then marks it stale on revision change and deletes private payload under the contract.
