# M72 — Campus Opportunity Graph

## Metadata

- `Module ID`: `M72`
- `Package ID`: `ustc.opportunity-graph`
- `Status`: Accepted blueprint; offline Course Planning spike exists
- `Implementation State`: `bounded-spike`
- `Version`: `m72-opportunity-graph/v1`
- `Last Review`: `2026-08-02`
- `Primary code areas`: `plugins/first-party/opportunity-graph/`, current `crates/course-planning/`, future cohesive opportunity/profile modules

## 1. Purpose

`M72` answers: “What fits me, and what should I choose next?” It owns typed campus opportunities, product-specific opportunity preferences and deterministic qualification/dependency/conflict/matching/planning. It consumes a minimum purpose-bound M00 user-context projection; it does not own the platform's general user profile.

Course Planning is one independently demonstrable journey inside this module, not the module's entire identity.

## 2. Non-goals

- owning public source/revision authority;
- building a universal property graph or generic graph database abstraction;
- storing raw USTC passwords/CAS sessions;
- general name/person-number/identity/contact/residence profile facts, account links or tenant memberships;
- cross-user context data or public leakage of private opportunity preferences;
- automatic course enrollment/application submission;
- allowing model explanations to add facts/courses past deterministic validation.

## 3. Owned objects and state

```text
OpportunityId / OpportunityKind
EligibilityCondition / Dependency / Coverage / Conflict
TemporalWindow
OpportunityFactRef with M60 provenance
OpportunityPreference / PlanningConstraint / PreferenceConsent
PurposeBoundUserProfileRef with M00 projection revision
DerivedMatch / PlanningProfileSnapshot
PlanCandidate / Explanation / Stale state
```

Public opportunity facts, M00-owned general profile projections and M72-owned product preferences are separate state classes. Derived matches never enter the public graph or write back into the general profile.

## 4. Public inputs and outputs

Inputs:

```text
reviewed M60 fact/revision references
purpose-bound M00 `CurrentProfileProjection`
create/view/update/delete opportunity preference commands
opportunity query/filter request
qualification/match/plan request
current source/profile/preference snapshot IDs
```

Outputs:

```text
reviewed opportunity view with provenance/freshness
qualification/dependency/conflict result
ranked bounded candidate plans/matches
explanation containing evidence and uncertainty
preference deletion/revocation receipt
```

## 5. Dependency direction

Allowed dependencies:

- `M60` typed fact/revision/provenance/freshness interfaces;
- `M00` tenant/user/request context plus [`user-context-profile/v0`](../../contracts/user-context-profile.md) purpose-bound read projection;
- `M90` tenant-scoped repository, consent/audit, artifact and clock ports;
- optional `M30` explanation assistance after deterministic results are fixed.

Forbidden dependencies:

- source fetch/parser internals;
- `M70`/`M71` private state;
- Dioxus/client types;
- model output as hard eligibility/planning truth;
- shared non-tenant-keyed profile/preference cache;
- direct mutation of M00 profile facts or acceptance of an AI profile proposal.

## 6. Lifecycle

Public fact path:

```text
accepted M60 fact/revision
→ typed opportunity validation
→ current reviewed opportunity projection
→ stale/superseded/archive under source/time policy
```

Private planning-context path:

```text
M00 purpose grant + current purpose-bound profile projection
and explicit M72 opportunity preference input
→ pinned planning-profile/preference snapshot
→ bounded derived qualification/match/plan
→ view/update/revoke/delete M72 preferences
→ M00 profile edits remain separate M00 commands
```

A candidate becomes stale when its pinned source/profile revision changes.

## 7. Failure and recovery

- Missing/stale/conflicting public fact: uncertainty/refusal, not invented match.
- M00 profile projection/purpose grant or M72 preference consent missing/wrong tenant: deny before read/derivation.
- Unresolved alias/identity: exclude and warn, never guess.
- Hard constraint failure: candidate rejected even if soft/model score is high.
- Repository/cache deletion failure: do not claim deletion complete.
- Source/profile revision drift: mark prior result stale; recompute under explicit request/policy.
- Model explanation inconsistency: reject explanation or fall back to deterministic rationale.
- Projection loss: rebuild public views from M60, reload purpose-bound general context from M00 and rebuild M72 preferences from M72-owned state.

## 8. Configuration and secrets

Typed policies cover opportunity kinds, required M00 profile field keys, source classes, purpose/consent/retention, M72 preference schemas, hard constraint sets, ranking limits and explanation bounds. Neither M00 profile projections nor M72 preferences contain raw institutional credentials. iCourse remains link-out-only unless explicit permission changes.

## 9. Observability

Record tenant-safe request IDs, public source revisions, M00 profile projection revision/purpose grant, M72 preference snapshot/consent IDs, candidate IDs, hard/soft decision classes, stale/conflict/uncertainty and deletion receipts. Logs avoid raw academic/profile/preference payload by default.

## 10. Extension and replacement

Course, research, competition, lecture and scholarship packs are peer typed domain packs when they reuse stable opportunity/preference semantics and M00 profile-consumer boundary. A materially different object/relationship model requires separate review; do not stretch one generic graph abstraction to force reuse. Deterministic planners/rankers and optional explainers are replaceable.

## 11. Performance path

Use typed indexed facts, an M00 purpose-bound current profile projection and tenant-scoped M72 preference projections. Candidate generation is bounded by policy (for example beam width/result count), then independently recomputes hard constraints. Do not expose unbounded graph traversal or model ranking over raw campus/profile/preference data.

## 12. Scope boundary

**MVP**

- one reviewed opportunity family, initially Course Planning integration if source/profile contracts are honest;
- tenant-isolated purpose-bound M00 profile snapshot plus independently consented M72 preference snapshot;
- deterministic eligibility/dependency/conflict/planning;
- provenance, freshness and uncertainty;
- view/delete M72 preference behavior and link-out to M00 profile management;
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
3. `profile-consumer` — validates the exact M00 purpose-bound projection and pins its revision; owns no general fact.
4. `opportunity-preference` — M72-specific preference/constraint consent, view/update/delete.
5. `qualification` — deterministic eligibility/dependency/coverage.
6. `conflict-engine` — temporal/resource/identity conflicts.
7. `candidate-engine` — bounded planning/matching.
8. `candidate-validation` — independent hard-constraint recomputation.
9. `ranking` — soft preferences/community signals below hard facts.
10. `explanation` — deterministic evidence and bounded optional model prose.
11. `course-pack` — current Course Planning adapter/domain pack.
12. future peer domain packs with separate contracts.
13. `opportunity-ports` — public/preference repositories, M00 profile-consumer fake, clock and audit fakes.

Existing `course-planning` is bounded evidence for parts of candidate/ranking/course-pack behavior only. It does not prove M00 profile consumption, M72 preference consent, Market lifecycle or live source integration.

## 14. Exit gate

`M72` is standalone-ready when public/M00-profile/M72-preference separation, tenant/purpose denial, preference consent/deletion, stale/conflict, deterministic planning and explanation consistency pass against fakes. It is accepted when one installed-plugin journey uses reviewed source facts, a purpose-bound M00 profile projection and M72 preference snapshot to produce a zero-hard-violation result with provenance, then marks it stale on either revision change and deletes M72-owned private payload under the contract.
