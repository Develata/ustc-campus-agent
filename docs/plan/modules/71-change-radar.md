# M70 — USTC ChangeRadar

## Metadata

- `Module ID`: `M70`
- `Package ID`: `ustc.change-radar`
- `Status`: Accepted blueprint; bounded source-revision/semantic-diff evidence
- `Implementation State`: `partial-evidence`
- `Version`: `m70-change-radar/v0`
- `Last Review`: `2026-08-27`
- `Primary code area`: `crates/change-radar/`, with M60-owned immutable revision values in `crates/platform-core/src/source_revision.rs`; package resources remain under `plugins/first-party/change-radar/`

Current retained evidence covers exact-source-pinned board-policy validation, deterministic typed field comparison, M60-owned source-health denial outcomes, complete-policy-bound candidate identity, service-minted repository commits, and explicitly bounded atomic in-memory candidate/baseline publication. It deliberately does not claim administrator approval/publication, durable storage, a feed, an approved live source, Market/Agent/ToolGateway invocation, M10/M80 composition, or the module exit gate.

## 1. Purpose

`M70` answers: “What changed, and does it affect me?” It turns accepted `M60` source revisions into reviewed semantic changes, board-scoped affected information and stable RSS/Atom feed items.

It reports meaningful changes, not page-layout/hash noise.

## 2. Non-goals

- owning source retrieval, snapshots or accepted baselines;
- publishing model guesses or parser noise;
- mutating Affairs procedures directly;
- personalized private feeds in the MVP;
- generic website monitoring across arbitrary URLs;
- automatic administrator publication.

## 3. Owned objects and state

```text
BoardId / BoardPolicyRef
SemanticDiffCandidate
AffectedScope
ChangeEvent:
  Proposed | Approved | Published | Rejected | Archived
FeedProjection / StableGuid
MaintainerLease/Cursor and candidate evidence
```

`M60` source revisions are immutable inputs. `M70` owns semantic change review/publication only.

## 4. Public inputs and outputs

Inputs:

```text
old/new accepted SourceRevision references
normalized typed facts or parser output
board policy and node/procedure references
administrator review command
```

Outputs:

```text
SemanticDiffCandidate with before/after/effective time/scope/provenance
Approved ChangeEvent
Per-board RSS/Atom item
Change query/feed projection and stable errors
```

## 5. Dependency direction

Allowed dependencies:

- `M60` revision/fact/provenance/publication contracts;
- shared opaque typed affected-object references supplied by composition when a change points at another product's object;
- `M00` actor/request context for approval;
- `M90` event/repository/feed-render/storage ports.

Forbidden dependencies:

- source fetch/parser internals;
- Dioxus/client types;
- direct Agent/model authority;
- `M71` or `M72` implementation/public Rust types; later product links are resolved through stable opaque references and composition mappings;
- concrete feed/server framework types in domain rules.

## 6. Lifecycle

```text
accepted old/new revisions
→ deterministic semantic comparison
→ Proposed candidate + evidence
→ Reviewed
→ Approved | Rejected
→ Published exactly once
→ Archived when superseded/withdrawn under policy
```

A maintainer Agent may propose a candidate under board/source/policy scope. It cannot approve or publish.

## 7. Failure and recovery

- Missing/incompatible revisions: no candidate.
- Parser/layout/hash-only difference: no semantic event.
- Candidate/evidence write failure: no baseline/publication acknowledgement.
- Concurrent duplicate candidate/publish: deterministic event/GUID and one winner.
- Conflicting authority/effective time: explicit conflict; no publication.
- Feed render/delivery failure: approved event remains canonical; projection retries idempotently.
- Revoked source/policy: block new candidates/publication and preserve policy-compliant history.

## 8. Configuration and secrets

A board policy pins allowed node/source IDs, semantic fields, required evidence, reviewer/administrator roles, publication rules and feed settings. No source credential, model key or private profile value enters product config.

## 9. Observability

Record old/new revision IDs, semantic fields changed, scope, policy version, proposer/reviewer/publisher IDs, candidate/event/GUID, publication receipt and rejection reason. Metrics distinguish no-change, noise-filtered, proposed, approved, rejected and published exactly once.

## 10. Extension and replacement

Semantic comparators and deterministic feed renderers are peer implementations behind typed contracts. An Agent maintainer is replaceable and never authoritative. RSS/Atom/HTML/API projections read the same approved events.

## 11. Performance path

Compare typed normalized facts, not raw pages. Bound diff size, candidate count and board concurrency. Feed generation is incremental and idempotent by stable event/GUID; it does not rescan all source history per request.

## 12. Scope boundary

**MVP**

- one reviewed source and one board;
- deterministic old/new semantic candidate;
- explicit administrator approve/reject;
- one stable RSS/Atom feed;
- before/after, effective time, affected scope and provenance;
- duplicate/noise/failure protection.

**Later**

- multiple boards/sources;
- richer affected-user matching under separate profile consent;
- subscriptions and notification adapters;
- bounded maintainer Agent assistance.

**Explicit non-goals**

- arbitrary website watcher;
- raw DOM/hash changes as user events;
- private personalized feed in MVP;
- autonomous publication.

## 13. Small-module decomposition

1. `board-policy` — board/source/scope/publication rules.
2. `semantic-diff` — typed deterministic before/after comparison.
3. `change-candidate` — candidate/evidence lifecycle.
4. `affected-scope` — node/audience/time scope.
5. `change-review` — approve/reject with actor/policy identity.
6. `change-event` — canonical event lifecycle and stable ID.
7. `feed-render` — deterministic RSS/Atom and stable GUID.
8. `change-query` — read projection.
9. `maintainer-port` — bounded candidate-only Agent interface.
10. `change-ports` — repository/feed/event fakes.

## 14. Exit gate

`M70` is standalone-ready when one historical change fixture proves semantic output, noise rejection, deterministic duplicate handling and no publication on every failure. It is accepted when one real approved source revision pair publishes one reviewed feed item exactly once with complete provenance.
