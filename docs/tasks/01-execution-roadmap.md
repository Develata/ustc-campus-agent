# Execution roadmap

- `Status`: Current delivery order
- `Owning product plan`: [`../plan/02-product-positioning.md`](../plan/02-product-positioning.md)
- `Architecture decision`: [`ADR-0006`](../adr/0006-three-default-first-party-plugins.md)
- `Acceptance registry`: [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv)

This task document schedules implementation; it does not override the owning plan. The three-Plugin topology and implementation order remain fixed.

## Dependency spine

```text
R0 platform-owned Agent runtime kernel
  └→ P0 Market identity/read and invocation-resolution skeleton
       └→ P1 ChangeRadar source/revision/diff foundation
            ├→ P2 Affairs Navigator procedure entry
            └→ P3 ChangeRadar board feed
                 └→ P4 Opportunity Graph consent/profile integration
                      └→ P5 productization and adversarial verification
                           └→ P6 freeze/submission
```

The Course Planning spike was completed out of order. It remains reusable evidence inside Opportunity Graph but does not move the mainline past P1/P2/P3.

## Completed foundation

- Rust monorepo, logical Market boundary, CI and repository contract checker;
- GitHub collaboration baseline and protected `main`;
- exactly three default first-party identities and typed manifest skeletons;
- Course Planning bounded spike: strict synthetic fixture, deterministic planner, CLI smoke, provenance and fail-closed tests;
- R0 framework-neutral Agent runtime kernel: immutable run spec, legal transitions, replay, effect identity/order and budget accounting;
- plan/feature/contract/acceptance documentation layering.

## R0 — Platform-owned Agent runtime kernel

**Inputs**

- accepted platform authority and runtime boundary;
- exact package/component/grant/schema identities already defined by current contracts;
- ADR-0004 framework-neutral authority decision.

**Deliverables**

- immutable validated `RunSpec`;
- legal phase/command/event transitions and deterministic replay;
- effect intent/receipt identity and ordering;
- replay-stable budget accounting;
- typed fail-closed errors with no silent provider/tool/runtime fallback;
- adapter boundary retained without prematurely freezing a framework API.

**Exit gate**

`AGENT-001` and `AGENT-002` pass against the Rust kernel. Durable orchestration, provider/tool adapters, HTTP/SSE and external effects remain explicitly planned.

## P0 — Market identity, read and invocation-resolution skeleton

**Inputs**

- accepted package/capability schema;
- exact three first-party manifests and Rust identities.

**Deliverables**

- deterministic parse/validation of visible packages;
- anonymous package browse/detail projection;
- exact version/component/capability/source-policy display;
- typed installation/enable/grant state design before persistence;
- a minimal resolver boundary that can supply an exact installation/component/grant snapshot to the runtime kernel;
- no runtime claim for planned manifests.

**Exit gate**

All three identities bootstrap at exact versions in validation; UI/HTTP evidence, if added, distinguishes catalog availability from installation/runtime state.

## P1 — ChangeRadar source/revision/diff foundation

**Prerequisite**: one concrete public USTC source pair receives owner, URL/retrieval, permission/rate and parser-fixture review.

**Deliverables**

- reviewed SourceDefinition;
- stable source and revision identities;
- conditional retrieval with SSRF/content/time/size boundaries;
- immutable raw and normalized snapshots;
- deterministic parser/normalizer and semantic diff;
- durable candidate evidence and accepted-baseline state;
- idempotent retry/restart.

**Failure gate**

Fetch, snapshot, parse, normalize, diff, candidate or evidence failure cannot advance the accepted baseline.

**Exit gate**

One historical source change replays into one exact semantic-diff candidate with provenance; arbitrary URL fetch is rejected.

## P2 — Affairs Navigator structured procedure entry

**Depends on**: P1 source/revision semantics; may begin tree/policy/schema work in parallel once those identities are stable.

**Deliverables**

- stable tree/node/procedure/artifact IDs;
- versioned board policy;
- reviewed Git Markdown/YAML canonical artifacts;
- typed `ProcedureDraft` and direct supersession edges;
- Rust schema/cross-field/policy/citation validation;
- deterministic Markdown rendering;
- administrator plan/apply publication and projection refresh;
- exact lookup + structured search path.

**Exit gate**

One administrator-maintained board answers one real procedure with conditions, steps, time, sources and explicit uncertainty. Full-corpus RAG is not required.

## P3 — ChangeRadar per-board feed

**Depends on**: P1 accepted revisions and P2 stable board/node/procedure identities.

**Deliverables**

- shared source/change ledger reused by Affairs and Radar;
- node/source/policy-scoped maintainer candidates;
- leases, deterministic IDs and idempotent publish receipts;
- approved semantic `ChangeEvent` lifecycle;
- per-board RSS/Atom with stable GUID, affected scope, before/after and provenance.

**Exit gate**

One approved semantic change publishes exactly once. Layout/hash noise, parser failure and unreviewed inference never enter the feed.

## P4 — Opportunity Graph consent/profile integration

**Depends on**: shared source/temporal/provenance foundation and explicit profile consent/deletion contract.

**Deliverables**

- reviewed opportunity ontology and source projection;
- tenant-isolated, viewable/deletable profile facts;
- qualification, dependency, temporal-window and conflict explanation;
- existing Course Planning core behind honest install/grant/discovery boundaries;
- iCourse link-out-only behavior unless explicit permission changes.

**Exit gate**

The offline planner becomes one installed-plugin journey without weakening hard constraints, source authority or profile isolation.

## P5 — Productization and adversarial verification

- package detail and independent disable/re-enable surfaces;
- browser desktop/mobile, keyboard, focus, console and network checks;
- tenant isolation, redaction, revoke, stale/conflict and recovery tests;
- exact config/doctor/acceptance evidence;
- deployment/restore and resource-bound verification;
- compact user trial with no fabricated metrics.

## P6 — Freeze and submission

- freeze new scope; fix blockers only;
- record the three-Plugin narrative and failure/recovery demo;
- complete architecture/framework influence/source/license evidence;
- clean-host restore/read-back where applicable;
- verify delivery-surface checksum/version/smoke;
- submit only after required gates and independent blocker review pass.

## Deferred until a contract consumer exists

- separate Market repository;
- arbitrary third-party hosted execution;
- generic workflow/graph engine;
- personalized private ChangeRadar feeds;
- broad RAG or vector infrastructure;
- Android-native full experience before Web/PWA lifecycle proof.
