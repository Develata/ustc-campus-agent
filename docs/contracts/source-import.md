# Source import and revision contract

## Metadata

- `Status`: Accepted for bounded `M60-B1 source-registry`; later M60 slices remain planned
- `Version`: `source-import/v0`
- `Last Review`: `2026-08-08`
- `Owning Blueprint`: [`M60 Campus Trust and Source Pipeline`](../plan/modules/70-campus-trust-source-pipeline.md)
- `Depends On`: [`module-boundaries.md`](module-boundaries.md) and the existing crate-root `SourceAuthority` order
- `Acceptance`: current matrix rows `SRC-001`, `SRC-010`, `SRC-011`, `SRC-012` remain `planned`; catalog-only `SRC-002`–`SRC-009` and `SRC-013` remain non-admitted
- `Primary Code`: none in P1-0; a later accepted P1-1 may add `crates/platform-core/src/source_registry.rs`

## 1. Scope and authority

`source-import/v0` owns the typed boundary between a reviewed source catalog and later retrieval, parsing, revision and baseline adapters. It defines:

- stable source identity, owner, authority class and exact canonical URL;
- proposed versus approved review state;
- retrieval-budget metadata whose limits later adapters must enforce;
- immutable revision identity and provenance requirements for later M60 slices;
- fail-closed baseline advancement and publication boundaries.

It does **not** fetch a URL, resolve DNS, follow a redirect, read a clock, parse HTML/PDF, persist raw bytes, normalize records, compute a semantic diff, advance a baseline or publish a product event. Those effects belong to M60-B2 through M60-B8 and their ports/adapters.

A syntactically valid source definition or review receipt proves shape only. It does not prove that the URL is safe now, that permission exists, that an operator actually reviewed evidence or that retrieval may occur. An application boundary may admit an approved definition only after it authenticates and authorizes the reviewer and binds real evidence. Model-proposed URLs always enter `Proposed`; no model/tool call can construct immediate fetch authority.

## 2. Lifecycle and ordering

The complete M60 order is:

```text
Proposed SourceDefinition
→ operator review evidence
→ Approved SourceDefinition
→ safe retrieval under exact policy
→ immutable RawSnapshot
→ deterministic parser/normalizer
→ immutable SourceRevision
→ semantic candidate/change
→ durable evidence
→ atomic accepted-baseline advance
→ typed publication candidate
```

Normative consequences:

1. an unapproved definition is not retrievable;
2. redirect targets are new URL decisions, never implicit approval;
3. retrieval failure creates no snapshot or revision;
4. parse/normalize failure creates no accepted revision;
5. diff/publication failure does not advance the accepted baseline;
6. every baseline update is compare-and-swap over the expected prior accepted revision;
7. the source domain never asks a model to repair or reinterpret source bytes.

## 3. M60-B1 stable identity values

P1-1 introduces five nominal string values in `crates/platform-core/src/source_registry.rs`:

```text
SourceId
SourceOwner
SourceUrl
SourceReviewerId
SourceReviewEvidenceId
```

Each is a named-field struct with a private backing string. It has one checked `parse`, exact `as_str`, exact `Display`, `TryFrom<String>`, `TryFrom<&str>`, `FromStr`, validating Serde decode and exact Serde string encode. It derives `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash`. It has no `Default`, mutable backing access, unchecked constructor, cross-kind conversion, normalization or semantic segment accessor.

### 3.1 `SourceId`

`SourceId` uses this exact ASCII grammar:

```regex
^[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?$
```

- encoded length is `1..=128` bytes;
- uppercase, whitespace, non-ASCII and every unlisted byte are rejected;
- case folding, trimming and delimiter rewriting are forbidden;
- prefixes and delimiters carry no authority meaning.

### 3.2 `SourceOwner`

`SourceOwner` is an exact human/governance owner label. It accepts `1..=128` UTF-8 bytes, rejects leading/trailing whitespace and every control character, and preserves accepted text exactly. It is never interpreted as an account, role or permission.

### 3.3 `SourceReviewerId` and `SourceReviewEvidenceId`

`SourceReviewerId` and `SourceReviewEvidenceId` each use the same byte grammar and bound as `SourceId`, but all three are nominally distinct. A reviewer ID names the authenticated reviewer identity supplied by an application boundary; an evidence ID is an opaque reference to evidence retained by an owning operator/governance surface. Neither string proves reviewer authorization, contains the evidence or carries a credential.

### 3.4 `SourceUrl`

`SourceUrl` is intentionally narrower than a general URL library. P1-1 admits exactly one canonical public-HTTPS shape:

- encoded length is `1..=2048` ASCII bytes;
- scheme is exactly lowercase `https://`;
- userinfo, password, explicit port, query and fragment are forbidden;
- the host is lowercase DNS text with at least two labels;
- each label is `1..=63` ASCII bytes, begins and ends alphanumeric and has only interior alphanumeric or `-`;
- IP literals, `localhost`, empty labels and a trailing dot are forbidden;
- the path begins with `/`, contains no empty, `.` or `..` segment and is otherwise limited to ASCII alphanumeric plus `-._~` and uppercase percent triplets;
- no decoding, Unicode/IDNA normalization, slash folding, dot-segment removal or percent-case rewriting occurs.

This constrained value is an exact reviewed endpoint identity, not permission to fetch arbitrary URLs. M60-B2 still owns DNS/IP, redirect, content-type, timeout, size and transport enforcement.

## 4. M60-B1 source definition

The exact public B1 values are:

```text
SourceId
SourceOwner
SourceUrl
SourceReviewerId
SourceReviewEvidenceId
SourceRetrievalPolicy
SourceReviewReceipt
SourceReviewState
SourceDefinition
SourceRegistry
SourceValueErrorKind
SourceValueError
SourceRegistryError
```

`SourceDefinition` contains exactly:

```text
source_id:        SourceId
owner:            SourceOwner
url:              SourceUrl
authority:        SourceAuthority
retrieval_policy: SourceRetrievalPolicy
review_state:     SourceReviewState
```

`SourceOwner` is a human/governance owner label, not an account identity; its exact validation is in §3.2.

`SourceRetrievalPolicy` contains exactly:

```text
minimum_interval_seconds: u32
maximum_response_bytes:   u32
```

Both are non-zero. `minimum_interval_seconds <= 604800`; `maximum_response_bytes <= 1048576`. These are operator ceilings consumed by M60-B2, not evidence that an adapter enforced them.

`SourceReviewReceipt` contains one reviewer identity and exactly four evidence references:

```text
reviewer:        SourceReviewerId
review:          SourceReviewEvidenceId
permission:      SourceReviewEvidenceId
rate:            SourceReviewEvidenceId
parser_fixture:  SourceReviewEvidenceId
```

The receipt has no boolean shortcuts and no optional field. A structurally complete receipt can still be false if a caller forged the references; reviewer authentication/authorization is an application-boundary obligation.

`SourceReviewState` is the bounded B1 review-admission state only. It is not the blueprint's complete operational `SourceStatus`: `Suspended` and `Revoked`, with their evidence-bearing transitions, must be accepted before any live M60-B2 retrieval adapter may consume an approved definition. B1 exposes no retrieval, so this deferral cannot leave a fetch path active.

`SourceReviewState` is exactly:

```text
Proposed
Approved { receipt: SourceReviewReceipt }
```

Every new definition starts as `Proposed`. Approval is an explicit registry transition that consumes a complete receipt. There is no constructor for an already-approved definition and no implicit approval based on host suffix, owner text, authority rank or fixture presence.

### 4.1 Exact B1 constructors and traits

The only public constructors outside the three string values' `parse` families are:

```text
SourceRetrievalPolicy::new(
    minimum_interval_seconds: u32,
    maximum_response_bytes: u32,
) -> Result<Self, SourceValueError>

SourceReviewReceipt::new(
    reviewer: SourceReviewerId,
    review: SourceReviewEvidenceId,
    permission: SourceReviewEvidenceId,
    rate: SourceReviewEvidenceId,
    parser_fixture: SourceReviewEvidenceId,
) -> Self

SourceDefinition::proposed(
    source_id: SourceId,
    owner: SourceOwner,
    url: SourceUrl,
    authority: SourceAuthority,
    retrieval_policy: SourceRetrievalPolicy,
) -> Result<Self, SourceValueError>

SourceRegistry::new() -> Self
```

`SourceDefinition::proposed` is the only definition constructor. It is fallible only because `SourceAuthority::ModelInference` is an explanation class, not a source, and must be rejected as `NonSourceAuthority`; every other field has already passed its owning validator. `SourceReviewReceipt::new` is total because reviewer/evidence references have already passed their nominal validators. No constructor takes `SourceReviewState`; no `approved`, `from_parts`, builder, `TryFrom` or Serde path may bypass the registry approval transition.

The complete aggregate/error trait surface is:

| Type | Traits |
|---|---|
| `SourceRetrievalPolicy` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` |
| `SourceReviewReceipt` | `Debug`, `Clone`, `PartialEq`, `Eq` |
| `SourceReviewState` | `Debug`, `Clone`, `PartialEq`, `Eq` |
| `SourceDefinition` | `Debug`, `Clone`, `PartialEq`, `Eq` |
| `SourceRegistry` | `Debug`, `Clone`, `PartialEq`, `Eq` |
| `SourceValueErrorKind` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` |
| `SourceValueError` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Display`, `Error` |
| `SourceRegistryError` | `Debug`, `Clone`, `PartialEq`, `Eq`, `Display`, `Error` |

No aggregate, state, receipt, definition, registry or registry error implements Serde in B1. In particular, no field-wise decode can instantiate `Approved` outside `SourceRegistry::approve`. The five nominal strings alone have the validating string Serde paths stated in §3.

Every public B1 struct except `SourceRegistry` exposes exactly one read-only accessor per field, named exactly as the field. `SourceDefinition::owner()` returns `&SourceOwner`; nominal/owned values return shared references; `Copy` scalars/enums return by value. `SourceRegistry` exposes only §5's operations.

## 5. M60-B1 registry behavior

`SourceRegistry` is a pure in-memory `BTreeMap<SourceId, SourceDefinition>` with no `Default` and one `new()` constructor. It owns these operations only:

```text
propose(definition)                   -> Result<(), SourceRegistryError>
approve(source_id, review_receipt)    -> Result<(), SourceRegistryError>
get(source_id)                        -> Option<&SourceDefinition>
approved(source_id)                   -> Result<&SourceDefinition, SourceRegistryError>
len()                                 -> usize
is_empty()                            -> bool
```

Required behavior:

- duplicate `SourceId` or duplicate canonical `SourceUrl` is rejected without replacing the first definition;
- approval of a missing ID is rejected;
- approval of an already approved ID is rejected and preserves the first receipt;
- `approved` rejects both missing and proposed entries;
- failed operations leave the whole registry byte-for-byte/structurally unchanged;
- iteration, deletion, URL lookup, authority fallback and mutable entry access are not public B1 APIs;
- no operation performs I/O, reads time, computes a digest or infers review from source text.

`SourceRegistryError` is exactly:

```text
DuplicateSource { source_id: SourceId }
DuplicateUrl { url: SourceUrl }
SourceNotFound { source_id: SourceId }
SourceNotApproved { source_id: SourceId }
SourceAlreadyApproved { source_id: SourceId }
```

The ID may be rendered because source IDs are public catalog references, not secrets. Rejected owner/URL/evidence input is never retained or echoed by `SourceValueError`.

## 6. B1 construction and public-surface closure

Every public B1 struct is a named-field struct with private fields. Every constructible struct has one checked or total constructor named by this contract; `SourceRegistry` alone has `new()`. Public enums have exactly the variants listed here. There is no public alias, `pub use`, public constant, extra public module, `Default`, `Deref`, mutable accessor, public field, unchecked constructor or framework/database/network trait.

All fields have one read-only accessor named exactly as the field. Copy scalars/enums return by value; owned values return shared references. `SourceRegistry` exposes only the six operations in §5.

The B1 implementation may import the existing crate-root `SourceAuthority` without modifying, re-exporting or implementing a trait for it. The source module neither duplicates nor changes the current authority order. `ModelInference` is rejected by `SourceDefinition::proposed`: an explanation class cannot become a source definition or approval candidate.

## 7. B1 deterministic value errors

`SourceValueErrorKind` is exactly:

```text
Empty
TooLong { max_bytes: usize }
InvalidStart
InvalidCharacter { byte_index: usize }
InvalidEnd
InvalidScheme
InvalidHost
InvalidPath
ZeroMinimumInterval
MinimumIntervalTooLarge { max_seconds: u32 }
ZeroMaximumResponseBytes
MaximumResponseBytesTooLarge { max_bytes: u32 }
OwnerBoundaryWhitespace
OwnerControlCharacter { byte_index: usize }
NonSourceAuthority
```

`SourceValueError` carries only a static Rust value-kind name and one `SourceValueErrorKind`. It implements `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Display` and `Error`; its accessors are `value_kind()` and `kind()`. It does not retain or display rejected text, host, path, owner fragment or offending byte.

Each parser has deterministic precedence: empty; length; then the value-specific left-to-right grammar. Policy validation checks minimum interval before maximum response size. `SourceDefinition::proposed` receives already validated values and then rejects `SourceAuthority::ModelInference` as `NonSourceAuthority`; it has no other failure.

## 8. Later source-revision contract

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
effective_at: Option<_>
provenance
```

Raw and normalized digests are separate nominal lowercase `sha256:` values. A revision ID never substitutes for either digest. `observed_at` is adapter-observed retrieval time; `published_at` is source-asserted publication time; `effective_at` is typed semantic time. Missing source assertions remain `None`; the system never copies `observed_at` into another field merely to avoid nullability.

A revision is accepted only after raw evidence, deterministic parse/normalize output and provenance are durably committed. Re-processing identical raw bytes with a new parser creates a new normalized identity/revision; it does not rewrite history.

## 9. Retrieval and SSRF boundary

Only M60-B2 may turn an approved definition into a retrieval request. Its later contract must independently enforce:

- exact approved scheme/host/port/path;
- no URL supplied by model or untrusted content;
- DNS resolution and public-address policy at connection time;
- every redirect as a fresh allowlist decision;
- response content type, byte limit and timeout;
- minimum interval and explicit operator override evidence;
- no credential/cookie forwarding outside an exact approved adapter contract.

B1 policy values are inputs to these checks, not substitutes for them. `SRC-010` remains `planned` after B1.

## 10. Baseline and publication boundary

An accepted baseline is a `(source_id, source_revision_id)` pair plus an expected baseline revision. Advancement is atomic and occurs only after the candidate revision, normalized facts, semantic diff and evidence are all durable. Any failure preserves the old baseline. Replay rebuilds the same accepted pair without network access.

Publication consumes a typed candidate with old/new accepted revision references. It never receives arbitrary source text or a bare model claim. M70 ChangeRadar owns semantic-change interpretation and feed policy; M60 owns evidence identity and baseline truth.

## 11. Concrete source candidate: proposed only

P1-0 records one reviewed **candidate family**, not an approved registry row:

- `Proposed source family label` (not a B1 `SourceId`): `ustc-teach-calendar-fall`
- `Proposed 2025 SourceId`: `ustc-teach-calendar-fall-2025`
- `Proposed 2026 SourceId`: `ustc-teach-calendar-fall-2026`
- `Owner`: 中国科学技术大学教务处 / `www.teach.ustc.edu.cn`
- `2025 URL`: `https://www.teach.ustc.edu.cn/calendar/19081.html`
- `2026 URL`: `https://www.teach.ustc.edu.cn/calendar/20135.html`
- `Discovery index`: `https://www.teach.ustc.edu.cn/category/calendar`
- `Robots`: `https://www.teach.ustc.edu.cn/robots.txt`
- `Candidate minimum interval`: `21600` seconds
- `Candidate maximum response`: `131072` bytes
- `Candidate content kind`: public HTML
- `Retention posture`: internal evidence; product output emits normalized facts and source links, not wholesale republished HTML

Observed 2026-08-08 reconnaissance:

- both calendar pages returned HTTP `200` and `Content-Type: text/html; charset=UTF-8`;
- no authentication or cookie was required;
- `robots.txt` did not disallow `/calendar/` or `/category/calendar`;
- no `ETag` or `Last-Modified` validator was observed;
- public accessibility and robots posture do **not** establish a copyright/republication license;
- the page pair is suitable for parser-fixture review, but raw HTML remains local evidence and is not committed to Git.

This candidate family stays `Proposed` throughout P1-0 and P1-1. The family label is research/product metadata, not a B1 registry ID; each exact URL would require its own stable SourceId, as listed above. Approval requires a separate operator review receipt per definition and is not inferred from this document.

## 12. P1-1 implementation slice

After independent GO on the exact P1-0 packet, P1-1 may implement only M60-B1:

- add `crates/platform-core/src/source_registry.rs`;
- add one `pub mod source_registry;` declaration in `crates/platform-core/src/lib.rs`;
- add `crates/platform-core/tests/source_registry.rs`;
- update this contract, M60 blueprint, execution roadmap, P1 proposal, acceptance projections and exact checker/test carriers;
- promote only `SRC-001` if its exact registered binding passes;
- keep `SRC-010`, `SRC-011`, `SRC-012` and every catalog-only SRC row unchanged;
- add no dependency, adapter, network call, persistence, clock, parser or concrete approved source.

The required positive/negative evidence covers every grammar edge, no-echo errors, duplicate rejection, missing/proposed/already-approved states, first-receipt preservation, failed-transition atomicity, exact API closure and zero I/O/dependency widening.

## 13. Acceptance projection

P1-0 changes no acceptance status.

P1-1 may change only `SRC-001` from `planned` to `implemented` after all of these exist on one exact revision:

```text
python3 scripts/check_repo_contracts.py --ci
python3 -m unittest scripts.tests.test_check_repo_contracts.SourceRegistryContractTests
cargo test --locked -p ustc-campus-agent-core --test source_registry
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
cargo test --locked --workspace --doc
git diff --check
```

`SRC-001` then means bounded B1 stable identity/owner/policy/review-state evidence only. It does not prove live permission, safe retrieval, raw snapshot, parser, source revision, semantic diff, baseline, publication or product readiness.

## 14. Change rule

Changing the public B1 value set, grammar/bounds, review-state semantics, registry transition rules, error taxonomy or URL posture changes `source-import/v0` and requires owning-contract, checker, mutation-test, acceptance and downstream review on the same revision.

Adding one source row under the unchanged contract is registry data, but it still requires an operator review receipt and source-specific permission/rate/parser-fixture evidence. Changing the authority order remains owned by the crate-root/platform plan, not by an incidental source-registry edit.
