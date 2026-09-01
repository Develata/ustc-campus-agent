# Source import and revision contract

## Metadata

- `Status`: Accepted contract as `source-import/v1` under the R11 M60-B2 two-layer transport architecture and implemented as the bounded pure `M60-B1 source-registry` lifecycle prerequisite; supersedes the accepted V10 `DEC-M60-B2-ACCEPTANCE`; `source-import/v0` retained as explicitly historical (see §15)
- `Version`: `source-import/v1`
- `Last Review`: `2026-09-01`
- `Predecessor`: [`source-import/v0`](#15-source-importv0-historical-evidence-retained) — accepted for the historical P1-1 bounded `M60-B1 source-registry`; retained as immutable evidence and superseded by the current v1 B1 implementation
- `Accepted Per`: `ACCEPT_EXACT_M60_B2_R11_PACKET` — Develata accepted the exact `33046`-byte semantic packet (`sha256:34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e`) on 2026-08-13; prior V10 `DEC-M60-B2-ACCEPTANCE` is explicitly superseded historical evidence
- `Owning Blueprint`: [`M60 Campus Trust and Source Pipeline`](../plan/modules/70-campus-trust-source-pipeline.md)
- `Depends On`: [`module-boundaries.md`](module-boundaries.md), [`source-retrieval.md`](source-retrieval.md), and the existing crate-root `SourceAuthority` comparison policy
- `Acceptance`: `SRC-001` is `implemented`; `SRC-010`, `SRC-011`, `SRC-012` remain `planned`; catalog-only `SRC-002`–`SRC-009` and `SRC-013` remain non-admitted; `SRC-014` remains catalog-only/non-admitted
- `Primary Code`: `crates/platform-core/src/source_registry.rs` and `crates/platform-core/tests/source_registry.rs` implement bounded pure B1 under `source-import/v1`; `crates/platform-core/src/source_retrieval.rs` and `crates/platform-core/tests/source_retrieval.rs` implement the bounded offline pure-policy B2 projection under `source-retrieval/v0`; every transport/effect/B3+ path remains separately gated

## 1. Scope and authority

`source-import/v1` replaces the bounded B1 review-only state (`Proposed`/`Approved`) with one complete operational `SourceStatus` and adds the retrieval-policy inputs required by `source-retrieval/v0`. It owns the typed boundary between a reviewed source catalog and later retrieval, parsing, revision and baseline adapters. It defines:

- stable source identity, owner, authority class and exact canonical URL;
- one operational `SourceStatus`: `Proposed | Approved | Suspended | Revoked`;
- one non-zero monotone `SourceAuthorityRevision` initialized to `1` by initial proposal and checked by compare-and-swap on every post-proposal retrievability-affecting transition;
- retrieval-policy metadata with six fields whose limits later adapters must enforce;
- `RetrievalSubject` — a sealed snapshot of an approved source available only from current `Approved` state;
- immutable revision identity and provenance requirements for later M60 slices;
- fail-closed baseline advancement and publication boundaries.

It does **not** fetch a URL, resolve DNS, follow a redirect, read a clock, parse HTML/PDF, persist raw bytes, normalize records, compute a semantic diff, advance a baseline or publish a product event. Those effects belong to M60-B3 through M60-B8 and their ports/adapters. `source-retrieval/v0` owns DNS, redirect, response framing and transport-policy decisions.

A syntactically valid source definition or review receipt proves shape only. It does not prove that the URL is safe now, that permission exists, that an operator actually reviewed evidence or that retrieval may occur. An application boundary may admit an approved definition only after it authenticates and authorizes the reviewer and binds real evidence. Model-proposed URLs always enter `Proposed`; no model/tool call can construct immediate fetch authority.

## 2. Lifecycle and ordering

The complete M60 order is:

```text
Proposed SourceDefinition (authority_revision=1)
→ operator review evidence
→ Approved SourceDefinition (revision+1)
→ safe retrieval under exact policy (source-retrieval/v0)
→ immutable RawSnapshot
→ deterministic parser/normalizer
→ immutable SourceRevision
→ semantic candidate/change
→ durable evidence
→ atomic accepted-baseline advance
→ typed publication candidate
```

Initial proposal followed by source-status transitions; every post-proposal transition uses `expected_authority_revision` CAS:

```text
propose(full definition)                          -> Proposed(authority_revision=1)
Proposed  + approve(full receipt)                 -> Approved(revision+1)
Proposed  + revise(full replacement, evidence)    -> Proposed(revision+1)
Proposed  + revoke(evidence)                      -> Revoked(revision+1, prior_approval=None)
Approved  + revise(full replacement, evidence)    -> Proposed(revision+1)
Approved  + suspend(evidence)                     -> Suspended(revision+1)
Approved  + revoke(evidence)                      -> Revoked(revision+1, prior_approval=Some)
Suspended + revise(full replacement, evidence)    -> Proposed(revision+1)
Suspended + reinstate(full new receipt)           -> Approved(revision+1)
Suspended + revoke(evidence)                      -> Revoked(revision+1, prior_approval=Some)
Revoked                                           -> terminal
```

Normative consequences:

1. an unapproved definition is not retrievable;
2. `Suspended` blocks new retrieval while preserving historical evidence; `Revoked` is terminal;
3. `retrieval_subject` is available only from current `Approved` state;
4. `revise` preserves `SourceId`, replaces owner/URL/authority/policy as one atomic body, and always returns to `Proposed`;
5. reinstate requires complete new review receipt, not a boolean resume;
6. redirect targets are new URL decisions, never implicit approval;
7. retrieval failure creates no snapshot or revision;
8. parse/normalize failure creates no accepted revision;
9. diff/publication failure does not advance the accepted baseline;
10. every baseline update is compare-and-swap over the expected prior accepted revision;
11. the source domain never asks a model to repair or reinterpret source bytes.

## 3. Stable identity values

Five nominal string values exist in `crates/platform-core/src/source_registry.rs` as part of the current B1 v1 implementation:

```text
SourceId
SourceOwner
SourceUrl
SourceReviewerId
SourceReviewEvidenceId
```

Each is a named-field struct with a private backing string. It has one checked constructor, exact `as_str`, exact `Display`, `TryFrom<String>`, `TryFrom<&str>`, `FromStr`, validating Serde decode and exact Serde string encode. It derives `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`. It has no `Default`, mutable backing access, unchecked constructor, cross-kind conversion, normalization or semantic segment accessor.

`SourceStatusEvidenceId` is the implemented v1 nominal identifier using the same grammar/bound as `SourceId`. It has checked `new(String) -> Result<SourceStatusEvidenceId, SourceValueError>`, `as_str(&self) -> &str`, `into_inner(self) -> String`; `Clone + Debug + Eq + Ord + Hash`; no `Default`, Serde, `Display`, `TryFrom`, `FromStr` or unchecked constructor. It represents an opaque reference to transition evidence retained by an owning operator/governance surface. It is never interpreted as a credential, does not contain the evidence and is not self-proving authorization.

`SourceMediaType` is the media-type value with checked `parse(&str) -> Result<SourceMediaType, SourceValueError>`, no `Display`, `TryFrom`, `FromStr` or Serde. It is a private-field struct implementing `Clone + Debug + Eq`. Grammar: lowercase ASCII `type/subtype`, each side `1..=64`, RFC token bytes only, no whitespace, parameter, wildcard or structured fallback; total `3..=129` bytes.

### 3.1 `SourceAuthorityRevision`

`SourceAuthorityRevision(u64)` is non-zero (initial value `1`). It is the single current-authority generation for a stable `SourceId`. Initial `propose` is the constructor/admission exception: it takes no caller-supplied expected revision and initializes revision `1`. Every post-proposal mutation that can affect retrievability — `approve`, `suspend`, `reinstate`, `revoke`, `revise` — requires exact-revision CAS and increments it with checked arithmetic (u64 overflow is `RevisionExhausted`). There is no peer definition/status revision and no reset while the `SourceId` exists. `Copy + Clone + Debug + Eq + Ord + Hash`; private field, `get()` accessor; no `Default`, Serde or unchecked constructor.

### 3.2 `SourceId`

Uses this exact ASCII grammar:

```regex
^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$
```

- encoded length is `1..=128` bytes;
- uppercase, whitespace, non-ASCII and every unlisted byte are rejected;
- case folding, trimming and delimiter rewriting are forbidden;
- prefixes and delimiters carry no authority meaning.

### 3.3 `SourceOwner`, `SourceReviewerId`, `SourceReviewEvidenceId`

Same semantics as `source-import/v0` §§3.2–3.3.

### 3.4 `SourceUrl`

Same exact constrained public-HTTPS grammar as `source-import/v0` §3.4, with the
canonical lowercase DNS host additionally bounded to `3..=253` presentation bytes
while every label remains `1..=63` bytes. This closes representability with
`source-retrieval/v0`: every admitted `SourceUrl` host can construct the exact
`RetrievalDnsName`; a `254+` byte host is `InvalidHost`, not a later protocol mismatch.

## 4. Source definition v1

The exact public v1 values are:

```text
SourceId
SourceOwner
SourceUrl
SourceReviewerId
SourceReviewEvidenceId
SourceStatusEvidenceId
SourceAuthorityRevision
SourceMediaType
SourceRetrievalProtocolVersion
PublicIpPolicyVersion
SourceRetrievalPolicy
SourceReviewReceipt
SourceStatus
SourceStatusKind
SourceTransitionCommand
SourceDefinitionBody
SourceDefinition
RetrievalSubject
SourceValueErrorKind
SourceValueError
SourceRegistryError
SourceRegistry
```

`SourceDefinition` contains exactly:

```text
source_id:          SourceId
owner:              SourceOwner
url:                SourceUrl
authority:          SourceAuthority
retrieval_policy:   SourceRetrievalPolicy
authority_revision: SourceAuthorityRevision
status:             SourceStatus
```

`SourceStatus` is one operational state:

```text
Proposed {
    revision_evidence: Option<SourceStatusEvidenceId>
}
| Approved {
    receipt: SourceReviewReceipt
}
| Suspended {
    approval: SourceReviewReceipt,
    evidence: SourceStatusEvidenceId
}
| Revoked {
    prior_approval: Option<SourceReviewReceipt>,
    evidence: SourceStatusEvidenceId
}
```

`SourceStatusKind` is the evidence-free closed enum `Proposed | Approved | Suspended | Revoked`. `SourceTransitionCommand` is exactly `Revise | Approve | Suspend | Reinstate | Revoke`.

`SourceRetrievalPolicy` has exactly six fields (expanded from v0's two):

```text
minimum_interval_seconds: u32
maximum_response_bytes:   u32
maximum_elapsed_seconds:  u32
expected_media_type:      SourceMediaType
protocol_version:         SourceRetrievalProtocolVersion
public_ip_policy_version: PublicIpPolicyVersion
```

Bounds and grammar:
- `minimum_interval_seconds`: `1..=604800`;
- `maximum_response_bytes`: `1..=1048576`;
- `maximum_elapsed_seconds`: `1..=60`;
- `SourceMediaType`: lowercase ASCII `type/subtype`, each side `1..=64`, RFC token bytes only, no whitespace, parameter, wildcard or structured fallback; total `3..=129` bytes;
- `SourceRetrievalProtocolVersion`: exactly one closed enum variant (`V0StrictHttpsIpv4Http11_20260809`);
- `PublicIpPolicyVersion`: exactly one closed enum variant (`V0Ipv4Only20260809`).

These fields are operator ceilings consumed by `source-retrieval/v0`, not evidence that an adapter enforced them. `revise` is the only way to change any of these for a stable `SourceId`; it atomically increments `SourceAuthorityRevision`, replaces the full body and returns status to `Proposed`.

`SourceReviewReceipt` retains the v0 shape:

```text
reviewer:        SourceReviewerId
review:          SourceReviewEvidenceId
permission:      SourceReviewEvidenceId
rate:            SourceReviewEvidenceId
parser_fixture:  SourceReviewEvidenceId
```

There is no boolean shortcut and no optional field.

### 4.1 Constructors and traits

```text
SourceStatusEvidenceId::new(String) -> Result<SourceStatusEvidenceId, SourceValueError>
SourceMediaType::parse(&str) -> Result<SourceMediaType, SourceValueError>

SourceRetrievalPolicy::new(
    minimum_interval_seconds: u32,
    maximum_response_bytes: u32,
    maximum_elapsed_seconds: u32,
    expected_media_type: SourceMediaType,
    protocol_version: SourceRetrievalProtocolVersion,
    public_ip_policy_version: PublicIpPolicyVersion,
) -> Result<Self, SourceValueError>

SourceReviewReceipt::new(
    reviewer: SourceReviewerId,
    review: SourceReviewEvidenceId,
    permission: SourceReviewEvidenceId,
    rate: SourceReviewEvidenceId,
    parser_fixture: SourceReviewEvidenceId,
) -> Self

SourceDefinitionBody::new(
    owner: SourceOwner,
    url: SourceUrl,
    authority: SourceAuthority,
    retrieval_policy: SourceRetrievalPolicy,
) -> Result<Self, SourceValueError>

SourceDefinition::proposed(
    source_id: SourceId,
    owner: SourceOwner,
    url: SourceUrl,
    authority: SourceAuthority,
    retrieval_policy: SourceRetrievalPolicy,
) -> Result<Self, SourceValueError>

SourceRegistry::new() -> Self
```

`SourceDefinition::proposed` is the only definition constructor. It is fallible only because `SourceAuthority::ModelInference` is rejected as `NonSourceAuthority` and `SourceDefinitionBody::new` validates the retrieval policy. `SourceReviewReceipt::new` is total. No constructor takes `SourceStatus`; no `approved`, `from_parts`, builder, `TryFrom` or Serde path may bypass the registry approval transition.

Read-only accessors:
- `SourceRetrievalPolicy::{minimum_interval_seconds, maximum_response_bytes, maximum_elapsed_seconds} -> u32`
- `SourceRetrievalPolicy::expected_media_type -> &SourceMediaType`
- `SourceRetrievalPolicy::{protocol_version, public_ip_policy_version} -> copied enum`
- `SourceDefinitionBody::{owner, url, retrieval_policy} -> reference`, `authority -> SourceAuthority`
- `SourceDefinition::{source_id, owner, url, retrieval_policy} -> reference`, `{authority, authority_revision} -> copied value`, `{status, prior_approval} -> reference`
- `SourceStatus::kind -> SourceStatusKind`
- `RetrievalSubject::{source_id, source_url, source_retrieval_policy} -> reference`, `source_authority_revision -> SourceAuthorityRevision`

Traits: `SourceMediaType`, `SourceRetrievalPolicy`, `SourceDefinitionBody`, `SourceDefinition`, `SourceStatus`, review/status evidence and `RetrievalSubject` implement `Clone + Debug + Eq`. `SourceStatus` is a closed public enum; no caller-built status can be injected. `SourceRegistry` drops `Clone` from v0; it is one mutable aggregate/index owner. No aggregate, state, receipt, definition, registry or error implements Serde.

## 5. Registry behavior v1

`SourceRegistry` operations:

```text
new() -> SourceRegistry
len(&self) -> usize
is_empty(&self) -> bool
get(&self, id: &SourceId) -> Option<&SourceDefinition>
propose(&mut self, definition: SourceDefinition) -> Result<(), SourceRegistryError>
revise(&mut self, id: &SourceId, expected: SourceAuthorityRevision, replacement: SourceDefinitionBody, evidence: SourceStatusEvidenceId) -> Result<&SourceDefinition, SourceRegistryError>
approve(&mut self, id: &SourceId, expected: SourceAuthorityRevision, receipt: SourceReviewReceipt) -> Result<&SourceDefinition, SourceRegistryError>
suspend(&mut self, id: &SourceId, expected: SourceAuthorityRevision, evidence: SourceStatusEvidenceId) -> Result<&SourceDefinition, SourceRegistryError>
reinstate(&mut self, id: &SourceId, expected: SourceAuthorityRevision, receipt: SourceReviewReceipt) -> Result<&SourceDefinition, SourceRegistryError>
revoke(&mut self, id: &SourceId, expected: SourceAuthorityRevision, evidence: SourceStatusEvidenceId) -> Result<&SourceDefinition, SourceRegistryError>
retrieval_subject(&self, id: &SourceId) -> Result<RetrievalSubject, SourceRegistryError>
approved(&self, id: &SourceId) -> Result<&SourceDefinition, SourceRegistryError>
```

Required behavior:
- duplicate `SourceId` or duplicate canonical `SourceUrl` is rejected without replacing the first definition;
- after duplicate checks, `propose` canonicalizes the consumed definition to fresh revision `1` plus `Proposed { revision_evidence: None }`; a clone obtained from another registry cannot transplant `Approved`, `Suspended` or `Revoked` authority, evidence, receipt or revision into the receiving registry;
- every post-proposal lifecycle mutation requires an exact `expected_authority_revision` CAS; initial `propose(definition)` takes no expected revision and creates revision `1`; stale post-proposal revision rejects as `StaleAuthorityRevision`;
- `propose` precedence: `DuplicateSource` then `DuplicateUrl`;
- revision overflow on any mutation is `RevisionExhausted`;
- illegal transition is `IllegalTransition { status, command }`;
- `retrieval_subject` returns a sealed owned `RetrievalSubject` only for current `Approved` state; rejects missing, proposed, suspended and revoked as `SourceNotRetrievable`;
- `approved` rejects both missing and proposed/suspended/revoked entries;
- failed operations leave the registry structurally unchanged;
- iteration, deletion, URL lookup, authority fallback and mutable entry access are not public APIs;
- no operation performs I/O, reads time, computes a digest or infers review from source text.

`SourceRegistryError`:

```text
DuplicateSource { source_id: SourceId }
DuplicateUrl { url: SourceUrl }
SourceNotFound { source_id: SourceId }
SourceNotRetrievable { source_id: SourceId, status: SourceStatusKind }
SourceAlreadyApproved { source_id: SourceId }
StaleAuthorityRevision { expected: SourceAuthorityRevision, actual: SourceAuthorityRevision }
IllegalTransition { status: SourceStatusKind, command: SourceTransitionCommand }
RevisionExhausted { source_id: SourceId }
```

## 6. RetrivalSubject and authority gating

`RetrievalSubject` is a sealed owned snapshot available only from current `Approved` state. It binds:

```text
source_id
source_url
source_authority_revision
source_retrieval_policy // includes public_ip_policy_version and protocol_version
```

Fields are private, accessors are read-only, no Serde, no public unchecked constructor, no authority-bearing conversion from `SourceDefinition`. A subject is a policy input to `source-retrieval/v0`, not final effect authority. M60-B3 must later re-check the same source ID + authority revision atomically with idempotency before any network effect. Every carrier (`RetrievalPlanCandidate`, `AdmittedRetrievalPlan`, phase carriers, attempt receipt, `BoundedFetch`) carries the same authority revision; mismatch rejects without network I/O.

## 7. Construction and public-surface closure

Every public v1 struct is a named-field struct with private fields. Every constructible struct has one checked or total constructor named by this contract; `SourceRegistry` alone has `new()`. Public enums have exactly the variants listed here. There is no public alias, `pub use`, public constant, extra public module, `Default`, `Deref`, mutable accessor, public field, unchecked constructor or framework/database/network trait.

All fields have one read-only accessor named exactly as the field. Copy scalars/enums return by value; owned values return shared references. `SourceRegistry` exposes only the operations in §5.

The v1 implementation may import the existing crate-root `SourceAuthority` without modifying, re-exporting or implementing a trait for it. The source module neither duplicates nor changes the current authority comparison policy. `ModelInference` is rejected by `SourceDefinition::proposed`.

## 8. Deterministic value errors v1

`SourceValueErrorKind` retains every `source-import/v0` variant and adds:

```text
ZeroMaximumElapsedSeconds
MaximumElapsedSecondsTooLarge { max_seconds: u32 }
InvalidMediaType
InvalidOverrideWindow
```

The existing `Empty`, `TooLong`, `InvalidStart`, `InvalidCharacter`, `InvalidEnd`, `InvalidScheme`, `InvalidHost`, `InvalidPath`, `ZeroMinimumInterval`, `MinimumIntervalTooLarge`, `ZeroMaximumResponseBytes`, `MaximumResponseBytesTooLarge`, `OwnerBoundaryWhitespace`, `OwnerControlCharacter` and `NonSourceAuthority` variants remain authoritative.

`SourceValueError` carries only a static Rust value-kind name and one `SourceValueErrorKind`. It implements `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Display` and `Error`; its accessors are `value_kind()` and `kind()`. It does not retain or display rejected text, host, path, owner fragment or offending byte.

Constructor precedence is nominal input grammar first; policy fields in declaration order; then `NonSourceAuthority`.

## 9. Later source-revision contract

M60-B3 through B5 later introduce immutable evidence with these distinct identities:

```text
RawSnapshotId
ParserIdentity
NormalizedSnapshotId
SourceRevisionId
```

A `SourceRevision` binds all of:

```text
source_id
source_url
raw_snapshot_id
raw_sha256
normalized_snapshot_id
normalized_sha256
parser_identity
observed_at
published_at: Option<_>
effective_from: Option<_>
effective_to: Option<_>
provenance
```

Raw and normalized digests are separate nominal lowercase `sha256:` values. A revision ID never substitutes for either digest. `observed_at` is adapter-observed retrieval time; `published_at` is source-asserted publication time; `effective_from`/`effective_to` are the optional typed bounds of the source-asserted semantic validity interval. Missing source assertions or interval bounds remain `None`; the system never copies `observed_at` into another field merely to avoid nullability.

A revision is accepted only after raw evidence, deterministic parse/normalize output and provenance are durably committed. Re-processing identical raw bytes with a new parser creates a new normalized identity/revision; it does not rewrite history. `observed_at` remains source-revision evidence and never supplies a downstream fact's `known_at`; each product owns the earliest durable materialization time for its exact fact revision/parser output.

## 10. Retrieval and SSRF boundary

M60-B2 through `source-retrieval/v0` independently enforces:

- exact approved scheme/host/port/path (no URL supplied by model or untrusted content);
- DNS resolution and public-address policy at connection time;
- every redirect as a fresh allowlist decision;
- response content type, byte limit and timeout;
- minimum interval and explicit operator override evidence;
- no credential/cookie forwarding outside an exact approved adapter contract.

B1/v1 policy values are inputs to these checks, not substitutes for them. `SRC-010` remains `planned` after B1 and through B2 contract acceptance; pure policy/fake evidence is supporting evidence, not full SSRF acceptance.

## 11. Baseline and publication boundary

An accepted baseline is a `(source_id, source_revision_id)` pair plus an expected baseline revision. Advancement is atomic and occurs only after the candidate revision, normalized facts, semantic diff and evidence are all durable. Any failure preserves the old baseline. Replay rebuilds the same accepted pair without network access.

Publication consumes a typed candidate with old/new accepted revision references. It never receives arbitrary source text or a bare model claim. M70 ChangeRadar owns semantic-change interpretation and feed policy; M60 owns evidence identity and baseline truth.

## 12. Concrete source candidate: proposed only

P1-0 records one reviewed **candidate family**, not an approved registry row:

- `Proposed source family label` (not a B1 `SourceId`): `ustc-teach-calendar-fall`
- `Proposed 2025 SourceId`: `ustc-teach-calendar-fall-2025`
- `Proposed 2026 SourceId`: `ustc-teach-calendar-fall-2026`
- `Owner`: 中国科学技术大学教务处 / `www.teach.ustc.edu.cn`
- `2025 URL`: `https://www.teach.ustc.edu.cn/calendar/19081.html`
- `2026 URL`: `https://www.teach.ustc.edu.cn/calendar/20135.html`

This candidate family stays `Proposed`. No concrete USTC source is approved. The family label is research/product metadata, not a registry ID; each exact URL would require its own stable SourceId. Approval requires a separate operator review receipt per definition and is not inferred from this document.

## 13. M60-B1 implementation slices

P1-1 is retained as the historical bounded `source-import/v0` predecessor. It established the module declaration, source/test carriers and the exact `SRC-001` binding without approving a source or adding effects.

The current M60-B1 lifecycle successor implements `source-import/v1` in the same two Rust carriers:

- operational `SourceStatus = Proposed | Approved | Suspended | Revoked`;
- initial proposal at `SourceAuthorityRevision(1)` and exact CAS on every post-proposal mutation;
- six-field retrieval policy, including the one-variant `PublicIpPolicyVersion` inventory;
- evidence-bearing revise/approve/suspend/reinstate/revoke transitions;
- approved-only sealed `RetrievalSubject` projection;
- fail-closed duplicate, stale-revision, illegal-transition, revision-exhaustion and non-mutation-on-error behavior.

Migration from v0 to v1 is compile-time only because no production durable source rows exist. The historical P1-1 record and immutable §15 remain evidence, not current implementation authority.

## 14. Acceptance projection

The bounded B1 evidence — stable identity, owner, exact canonical URL, six-field retrieval policy, operational lifecycle, monotone authority revision and pure registry transitions — is implemented under `source-import/v1` and remains proven by the same active binding: `cargo test --locked -p ustc-campus-agent-core --test source_registry`.

```text
SRC-001 implemented (bounded B1 source-import/v1 lifecycle evidence)
SRC-010 planned
SRC-011 planned
SRC-012 planned
SRC-014 catalog-only / non-admitted
M60 planned (B1 lifecycle prerequisite and bounded offline B2 pure policy implemented; no transport/effect path)
```

`SRC-010` does not become `pass` from B1 lifecycle implementation or contract acceptance. `SRC-014` (`suspended/revoked source blocks new fetch`) remains catalog-only/non-admitted per the existing `platform-baseline.md` long-horizon catalog: B1 proves lifecycle and retrievability gating inside the pure registry, not the separately gated fetch integration.

## 15. `source-import/v0` — historical evidence retained

`source-import/v0` is explicitly historical evidence retained for the P1-1 B1 implementation record. It was:

- `Status`: accepted for bounded `M60-B1 source-registry`, implemented as a P1-1 review candidate;
- `Last Review`: `2026-08-08`;
- defined `SourceReviewState` as `Proposed | Approved { receipt: SourceReviewReceipt }`;
- defined `SourceRetrievalPolicy` with two fields (`minimum_interval_seconds`, `maximum_response_bytes`);
- defined `SourceRegistry` with six operations (`propose`, `approve`, `get`, `approved`, `len`, `is_empty`);
- defined `SourceRegistryError` with five variants (`DuplicateSource`, `DuplicateUrl`, `SourceNotFound`, `SourceNotApproved`, `SourceAlreadyApproved`).

`source-import/v1` replaces `SourceReviewState` with operational `SourceStatus` (adding `Suspended` and `Revoked`), adds `SourceAuthorityRevision` to every definition and mutation, expands `SourceRetrievalPolicy` from two to six fields, adds `revise`/`suspend`/`reinstate`/`revoke` registry operations, adds `retrieval_subject` and `RetrievalSubject`, adds `SourceStatusEvidenceId`, and replaces `SourceNotApproved` with `SourceNotRetrievable` plus `StaleAuthorityRevision`, `IllegalTransition` and `RevisionExhausted`.

Because v1 removes/renames `SourceReviewState` and changes `SourceDefinition`/`SourceRetrievalPolicy` shape, the change is versioned rather than described as a compatible amendment. No compatibility alias, dual state authority or implicit migration exists. The P1-1 B1 code in `crates/platform-core/src/source_registry.rs` remains the correct v0 implementation; a future v1 implementation is a separate packet.

## 16. Change rule

Changing the public v1 value set, grammar/bounds, status semantics, registry transition rules, error taxonomy or URL posture changes `source-import/v1` and requires owning-contract, checker, mutation-test, acceptance and downstream review on the same revision.

Adding one source row under the unchanged contract is registry data, but it still requires an operator review receipt and source-specific permission/rate/parser-fixture evidence. Changing the authority comparison policy remains owned by the crate-root/platform plan, not by an incidental source-registry edit.
