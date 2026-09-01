# M60-B2 accepted-contract record

## Mutable state

- `Stage`: `M60_B2_CONTRACT_ACCEPTED`
- `Packet status`: `ACCEPTED`
- `Mode`: accepted-contract plus bounded implementation projection; `source-import/v1` + `source-retrieval/v0` use the exact R11 two-layer M60/M90 semantic packet; bounded M60-B1 v1 lifecycle prerequisite and the first offline-only M60-B2 pure policy are implemented
- `Bound source commit`: `a1b0efe33239b33afeea40e7981cf15f8a65cd1e`
- `Bound source tree`: `bd3f096a26baf758b58bf80874e09a1604e885c0`
- `Superseded packet`: `sha256:ba36425adc164ca9b3ec75addd4be2e4b299b5f8a8cfb75cf6a710679acd32ab` over `77276` bytes — historical evidence only; this R4 replacement packet supersedes it
- `R4 replacement packet digest`: `sha256:34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e` over `33046` bytes beginning immediately after the `BEGIN` marker newline and ending immediately before the `END` marker token, including the final packet newline
- `Independent review`: `FINAL_PRODUCT_GO` for the exact R11 packet
- `Semantic acceptance`: `ACCEPTED` by Develata on 2026-08-13
- `Acceptance decision`: `ACCEPT_EXACT_M60_B2_R11_PACKET`
- `B1 lifecycle prerequisite`: implemented as bounded pure `source-import/v1` registry lifecycle; this is prerequisite truth, not M60-B2 implementation authority
- `Retained M60-B2 implementation`: bounded offline pure policy admitted by `M60_B2_REPRESENTABILITY_CLARIFICATION_20260901` and the independently reviewed implementation taskbook; no transport/effect/B3+ authority
- `Concrete source`: `ustc-teach-calendar-fall` remains `Proposed`
- `Remote shipping`: not granted by this document
- `Accepted-contract projection`: the exact R11 semantic packet is current contract authority; bounded M60-B1 lifecycle Rust and the first offline-only M60-B2 pure policy exist; no network path, source approval, B3 admission, push/PR/merge/tag/release

## Acceptance receipt (HISTORICAL — SUPERSEDED BY R4)

The following V10 acceptance and authority receipts are retained as explicitly superseded historical evidence. The R11 exact semantic packet supersedes them and is accepted only at the contract layer.

On 2026-08-11 Develata selected:

> 按建议有条件接受：认可 V10 contract structure，但 live B2 retrieval adapter 前必须补齐 operational Suspended/Revoked lifecycle；随后另立 implementation packet

### DEC-M60-B2-ACCEPTANCE (SUPERSEDED)

Binding interpretation (SUPERSEDED — repealed by R4 direction receipt):
- the V10 semantic contract structure was accepted;
- accepted projection target was `source-import/v1` plus `source-retrieval/v0`;
- operational `Suspended` and `Revoked` lifecycle authority, with monotone authority-revision binding, was a hard precondition before any live B2 retrieval adapter;
- retained implementation still requires a separately admitted implementation packet;
- no concrete source approval, network retrieval, publication, tag or release was authorized.

## Authority receipt (PR #38 — HISTORICAL)

On 2026-08-09 Develata instructed:

> 授权 merge PR #38；读取 exact-main CI 后，fresh branch 推进 M60-B2 contract-first（推荐）

PR `#38` was squash-merged only after replacement exact-head CI; its reviewed tree equals protected-main squash commit `a1b0efe33239b33afeea40e7981cf15f8a65cd1e`, and exact-main CI run `31297060989` passed.

## Source evidence used by the proposal

Repository authority:

- [`source-import/v0`](../contracts/source-import.md) §§1–9 (historical for B1 P1-1 implementation);
- [`source-import/v1`](../contracts/source-import.md) — R4 two-layer edition;
- [`source-retrieval/v0`](../contracts/source-retrieval.md) — R4 two-layer transport edition;
...
- [`module-boundaries.md`](../contracts/module-boundaries.md) §5 (R4 transport boundary);
- active acceptance `SRC-010` plus catalog-only `SRC-014`;
- the exact B1 public implementation in `crates/platform-core/src/source_registry.rs`.

External design evidence, subordinate to repository authority:

- [OWASP SSRF Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html);
- [reqwest `ClientBuilder` documentation](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html);
- [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986) and [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110).

<!-- M60_B2_RETRIEVAL_POLICY_PROPOSAL:BEGIN -->
## 1. Objective and completion claim

Freeze one reviewable R4 replacement proposal for `M60-B2 retrieval-policy` adopting the Develata-approved two-layer M60/M90 transport architecture. M60/platform-core exclusively owns retrieval admission, the crate-internal non-clone `EffectReadyRetrievalPlan`, phase-policy validation, attempt completion, `TransportStopped`, and minting the trusted `BoundedFetch` domain result. M90/crates-adapters implements only a public, framework-neutral, non-authority transport/session port that performs DNS/socket/TLS/raw HTTP byte I/O under M60-provided bounds and returns only transport observations or transport failures.

This proposal is complete only as a reviewable semantic candidate. It does not mean:

- `source-import/v1` or `source-retrieval/v0` is accepted;
- `SRC-010` or catalog-only `SRC-014` passes;
- a concrete USTC source is approved;
- a URL may be fetched;
- any Rust implementation exists;
- this R4 replacement packet itself is accepted — it is `PROPOSED_NOT_ACCEPTED` pending fresh exact-digest independent review and Develata acceptance.

## 2. Frozen source and current gap

The packet is bound to protected `main`:

```text
commit a1b0efe33239b33afeea40e7981cf15f8a65cd1e
tree   bd3f096a26baf758b58bf80874e09a1604e885c0
```

Current truth:

| Surface | Present | Missing before live retrieval |
|---|---|---|
| B1 source identity / exact canonical URL | yes | none for B1 |
| B1 review admission | `Proposed` / `Approved` | operational `Suspended` / `Revoked` and current revision |
| B1 retrieval budget | interval + body bytes | media type + total deadline |
| URL parsing | constrained exact public HTTPS | connection-time DNS/IP and connected-peer binding |
| Redirect policy | future obligation only | an exact v0 disposition |
| Credentials/proxy/compression | no B1 effect | explicit transport-negative contract |
| Rate decision | metadata only | clock/last-attempt/override semantics and atomic future use |
| HTTP transport | absent | non-authority M90 port, hostile fake and later M90 peer |

## 3. Two-layer M60/M90 transport architecture

### 3.1 Authority split

- `M60` / `platform-core` is the only owner of admission, rate/lease/idempotency, current source/revision rechecks, the crate-private non-clone `EffectReadyRetrievalPlan`, phase-policy validation, attempt completion, `TransportStopped`, and trusted `BoundedFetch` construction.
- `M90` / `crates/adapters` implements only a public, framework-neutral, non-authority transport/session port. It performs DNS/socket/TLS/raw HTTP byte I/O under M60-provided bounds and returns only transport observations or transport failures.
- M90 must never name or receive `EffectReadyRetrievalPlan`; never return or construct `BoundedFetch`, `TransportStopped`, attempt receipts, or any authority-bearing/phase-linear carrier; never choose URL/headers/policy thresholds; never normalize raw response evidence into domain truth.
- M60 coordinator consumes the one-shot effect-ready plan, derives one public private-field transport request, invokes M90, validates the complete response observation against the retained internal plan and strict phase algebra, mints `BoundedFetch` only on success, and terminalizes the attempt only after the transport call returned or cancellation/drop proved resource stop.

### 3.2 Public non-authority transport surface

The successor packet freezes exact Rust-equivalent types, visibility, fields/accessors, traits and signatures for a cross-crate port:

- one public private-field `RetrievalTransportRequest` constructed only by M60 from the effect-ready plan;
- request contains exactly the non-authority adapter inputs needed for deterministic transport: `attempt_id`, `source_id`, `authority_revision`, canonical DNS host (`RetrievalDnsName`), exact serialized request bytes (`SerializedRetrievalRequest`), expected media type (`SourceMediaType`), `maximum_response_bytes`, `maximum_elapsed_seconds`, `protocol_version`, `public_ip_policy_version`;
- `RetrievalTransportRequest` has exactly these read-only accessors and no URL/header builder conversion:

```text
attempt_id(&self) -> &RetrievalAttemptId
source_id(&self) -> &SourceId
authority_revision(&self) -> SourceAuthorityRevision
canonical_host(&self) -> &RetrievalDnsName
serialized_request(&self) -> &SerializedRetrievalRequest
expected_media_type(&self) -> &SourceMediaType
maximum_response_bytes(&self) -> u32
maximum_elapsed_seconds(&self) -> u32
protocol_version(&self) -> &SourceRetrievalProtocolVersion
public_ip_policy_version(&self) -> &PublicIpPolicyVersion
```

- one public private-field `RetrievalTransportSuccess` carrying raw response-head bytes and bounded body/framing/timing/peer/DNS observations without lossy normalization;
- one public typed `SourceTransportError` carrying transport-only failure classes and no `RetrievalPolicyError` or domain authority;
- neither success nor failure is a domain receipt or proof of acceptance;
- large byte carriers move by ownership into M60; avoid `Clone` on response/body carriers and avoid repeated buffering;
- the port is object-safe and uses only standard-library boundary types (`Future`/`Pin`, addresses, owned byte containers and M60-owned public non-authority values), with no `reqwest`/`hyper`/`tokio` type leakage;
- the adapter future owns all resolver/socket/TLS/body work, may not spawn/detach it, and success/error return occurs only after synchronous resource destruction. Dropping the future cancels and destroys resources before drop completes; no observation/result is produced on drop.

### 3.3 Public transport port signature

The non-authority M90 port is:

```rust
pub trait SourceTransportPort: Send + Sync {
    fn transport<'a>(
        &'a self,
        request: RetrievalTransportRequest,
    ) -> Pin<Box<dyn Future<Output = Result<RetrievalTransportSuccess, SourceTransportError>> + Send + 'a>>;
}
```

`RetrievalTransportRequest` is `Debug + Eq`, not `Clone`/`Copy`; its owned serialized request bytes move into M90. `RetrievalTransportSuccess` has private fields, the exact shape-only constructor/accessors/parts type frozen in §5.6, a consuming `into_parts(self)`, and no `Clone`/Serde/`Default`. It carries raw `DnsTransportObservation`, not M60-private `DnsResolutionObservation`. `SourceTransportError` is a public transport-only enum carrying no `RetrievalPolicyError`, raw payload or authority. The port carries no domain authority and no M60-internal type.

The packet defines exactly how M60 distinguishes:

- transport success followed by policy rejection (transport succeeded, M60 policy check rejected — no `BoundedFetch`);
- transport-class failure (transport itself failed — `SourceTransportError` variant returned and mapped by M60);
- explicit cancellation/drop (future dropped, resources destroyed, no observation produced);
- inability to prove resource stop (fail closed; attempt/slots remain held until exclusive-death recovery).

### 3.4 Crate dependency direction

One-way dependency: `crates/adapters -> crates/platform-core`. `platform-core` has no dependency on `adapters` and no cycle. `platform-core` remains free of production networking/runtime dependencies. M90 is replaceable and unable to mint domain authority.

### 3.5 M60-owned sealed fetch port

```rust
pub(crate) trait SourceFetchPort: sealed::SourceFetchPortSealed + Send + Sync {
    fn fetch<'a>(
        &'a self,
        plan: EffectReadyRetrievalPlan,
    ) -> Pin<Box<dyn Future<Output = Result<BoundedFetch, SourceFetchFailure>> + Send + 'a>>;
}
```

This port and the private `sealed` supertrait are crate-internal. Only an M60-owned coordinator type in `platform-core` may implement `SourceFetchPort`; no M90 adapter may implement it. Tests exercise the public pure chain only and cannot receive `EffectReadyRetrievalPlan` or return `BoundedFetch`. The M60 coordinator internally calls `SourceTransportPort::transport` with the derived `RetrievalTransportRequest`, maps `SourceTransportError` into crate-private `SourceFetchFailure`, validates the response observation, and mints `BoundedFetch` only on complete policy success plus `RetrievalTransportSuccess`.

No concrete M90 production networking, no clock adapter, and no transport execution exist.

## 4. `source-import/v1` lifecycle and authority revision

### 4.1 One monotone authority revision

Replace `SourceReviewState` with one operational `SourceStatus` and one aggregate-level `SourceAuthorityRevision`:

```text
SourceAuthorityRevision(u64) // non-zero; initial value is 1
SourceStatusEvidenceId       // nominal, same grammar/bound as SourceId

SourceStatus =
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

SourceDefinition {
  source_id:          SourceId
  owner:              SourceOwner
  url:                SourceUrl
  authority:          SourceAuthority
  retrieval_policy:   SourceRetrievalPolicy
  authority_revision: SourceAuthorityRevision
  status:             SourceStatus
}
```

Exact transitions, all with `expected_authority_revision` CAS:

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

### 4.2 Constructor surface and value-error channel

```text
SourceStatusEvidenceId::new(String) -> Result<SourceStatusEvidenceId, SourceValueError>
SourceMediaType::parse(&str) -> Result<SourceMediaType, SourceValueError>
SourceRetrievalPolicy::new(u32, u32, u32, SourceMediaType, SourceRetrievalProtocolVersion, PublicIpPolicyVersion) -> Result<SourceRetrievalPolicy, SourceValueError>
SourceDefinitionBody::new(SourceOwner, SourceUrl, SourceAuthority, SourceRetrievalPolicy) -> Result<SourceDefinitionBody, SourceValueError>
SourceDefinition::proposed(SourceId, SourceOwner, SourceUrl, SourceAuthority, SourceRetrievalPolicy) -> Result<SourceDefinition, SourceValueError>
```

`SourceStatusEvidenceId` has checked `new`, `as_str`, `into_inner`; `Clone + Debug + Eq + Ord + Hash`; no `Default`, Serde, Display, TryFrom, FromStr or unchecked constructor.

Read-only accessors:
- `SourceRetrievalPolicy::{minimum_interval_seconds, maximum_response_bytes, maximum_elapsed_seconds} -> u32`
- `SourceRetrievalPolicy::expected_media_type -> &SourceMediaType`
- `SourceRetrievalPolicy::{protocol_version, public_ip_policy_version} -> copied enum`
- `SourceDefinitionBody::{owner, url, retrieval_policy} -> reference`, `authority -> SourceAuthority`
- `SourceDefinition::{source_id, owner, url, retrieval_policy} -> reference`, `{authority, authority_revision} -> copied value`, `{status, prior_approval} -> reference`
- `SourceStatus::kind -> SourceStatusKind`
- `RetrievalSubject::{source_id, source_url, source_retrieval_policy} -> reference`, `source_authority_revision -> SourceAuthorityRevision`

### 4.3 Registry operations

```text
SourceRegistry::new() -> SourceRegistry
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

### 4.4 `SourceRetrievalPolicy` v1

Six fields:

```text
minimum_interval_seconds: u32   // 1..=604800
maximum_response_bytes:   u32   // 1..=1048576
maximum_elapsed_seconds:  u32   // 1..=60
expected_media_type:      SourceMediaType
protocol_version:         SourceRetrievalProtocolVersion
public_ip_policy_version: PublicIpPolicyVersion
```

`SourceMediaType`: lowercase ASCII `type/subtype`, each side `1..=64`, RFC token bytes only, total `3..=129` bytes.

### 4.5 `SourceStatusEvidenceId` traits

Exact traits: `Clone + Debug + Eq + Ord + Hash`. No `Default`, `Display`, `TryFrom`, `FromStr`, Serde. Constructor: `new(String) -> Result<Self, SourceValueError>`. Accessors: `as_str(&self) -> &str`, `into_inner(self) -> String`.

### 4.6 `SourceDefinition` field naming

`SourceDefinition` fields use the exact names: `source_id`, `owner`, `url`, `authority`, `retrieval_policy`, `authority_revision`, `status`. The accessor is `source_id(&self) -> &SourceId` (not a separate `id` field).

## 5. Source retrieval policy v0 — two-layer transport edition

### 5.1 Nominal identity values

All use the B1 string grammar (`1..=128` bytes, `[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?`):

```text
RetrievalAttemptId        RateOverrideId
RetrievalOverrideEvidenceId   SourceOperatorId
SourceStartAuthorizationId
```

Each has checked `new`, `as_str`, `into_inner`; `Clone + Debug + Eq + Ord + Hash`; no `Default`, Serde or unchecked constructor.

### 5.2 Attempt command

```text
RetrievalAttemptCommand {
    command_id: CommandId,
    attempt_id: RetrievalAttemptId,
    source_id: SourceId,
    expected_authority_revision: SourceAuthorityRevision,
    override_request: Option<RetrievalRateOverrideRequest>,
}
```

`RetrievalAttemptCommand::new(CommandId, RetrievalAttemptId, SourceId, SourceAuthorityRevision, Option<RetrievalRateOverrideRequest>) -> RetrievalAttemptCommand` is infallible after nominal inputs validated.

### 5.3 Authority evidence and start authorization

`RetrievalAuthorityEvidence`, `SourceStartAuthorization`, `RetrievalAttemptEnvelope`, `RetrievalReplayIdentity`, `TransportStopped` and `RetrievalAttemptCompletion` are owner-private outputs with no public constructor, Serde, `Default` or unchecked conversion.

### 5.4 Attempt state and completion

```text
RetrievalAttemptState = Admitted | Started | CompletedSuccess | CompletedFailure | Cancelled

RetrievalAttemptReceipt {
    envelope: RetrievalAttemptEnvelope,
    admitted_at: RetrievalEpochSeconds,
    started_at: Option<RetrievalEpochSeconds>,
    lease_expires_at: RetrievalEpochSeconds,
    state: RetrievalAttemptState,
    consumed_override_id: Option<RateOverrideId>,
}

RetrievalAttemptCompletion {
    attempt_id: RetrievalAttemptId,
    source_id: SourceId,
    authority_revision: SourceAuthorityRevision,
    terminal_state: CompletedSuccess | CompletedFailure | Cancelled,
    transport_stopped: Option<TransportStopped>,
}

TransportStopped {
    attempt_id: RetrievalAttemptId,
}
```

### 5.5 M60-owned ports

```rust
pub(crate) trait SourceOperatorPolicyPort: Send + Sync {
    fn authorize<'a>(&'a self, context: &'a PlatformRequestContext,
        source_id: &'a SourceId, authority_revision: SourceAuthorityRevision,
        capability: SourceRetrievalCapability, now: RetrievalEpochSeconds)
        -> Pin<Box<dyn Future<Output = Result<SourceOperatorId, RetrievalPolicyError>> + Send + 'a>>;

    fn authorize_start<'a>(&'a self, context: PlatformRequestContext,
        command: &'a RetrievalAttemptCommand, source_id: &'a SourceId,
        authority_revision: SourceAuthorityRevision, now: RetrievalEpochSeconds)
        -> Pin<Box<dyn Future<Output = Result<SourceStartAuthorization, RetrievalPolicyError>> + Send + 'a>>;
}

pub(crate) trait RetrievalClockPort: Send + Sync {
    fn now<'a>(&'a self)
        -> Pin<Box<dyn Future<Output = Result<RetrievalEpochSeconds, RetrievalPolicyError>> + Send + 'a>>;
}

pub(crate) trait RetrievalAdmissionPort: Send {
    fn admit<'a>(&'a mut self, context: PlatformRequestContext,
        candidate: RetrievalPlanCandidate)
        -> Pin<Box<dyn Future<Output = Result<RetrievalAdmissionOutcome, RetrievalPolicyError>> + Send + 'a>>;

    fn start<'a>(&'a mut self, plan: AdmittedRetrievalPlan)
        -> Pin<Box<dyn Future<Output = Result<EffectReadyRetrievalPlan, RetrievalPolicyError>> + Send + 'a>>;

    fn complete<'a>(&'a mut self, completion: RetrievalAttemptCompletion)
        -> Pin<Box<dyn Future<Output = Result<RetrievalAttemptReceipt, RetrievalPolicyError>> + Send + 'a>>;

    fn receipt<'a>(&'a mut self, context: PlatformRequestContext, command_id: CommandId)
        -> Pin<Box<dyn Future<Output = Result<RetrievalAttemptReceipt, RetrievalPolicyError>> + Send + 'a>>;
}

pub(crate) trait SourceFetchPort: sealed::SourceFetchPortSealed + Send + Sync {
    fn fetch<'a>(&'a self, plan: EffectReadyRetrievalPlan)
        -> Pin<Box<dyn Future<Output = Result<BoundedFetch, SourceFetchFailure>> + Send + 'a>>;
}
```

All ports use only standard-library `Future`/`Pin`; no external runtime/framework type crosses the boundary.

### 5.6 M90 public transport port

```rust
pub trait SourceTransportPort: Send + Sync {
    fn transport<'a>(&'a self, request: RetrievalTransportRequest)
        -> Pin<Box<dyn Future<Output = Result<RetrievalTransportSuccess, SourceTransportError>> + Send + 'a>>;
}
```

This is a public, framework-neutral, non-authority port. `RetrievalTransportRequest` carries only deterministic adapter inputs (attempt/source/revision lineage, canonical DNS host, exact serialized request bytes, expected media type, response/body/wire/deadline limits, protocol version and public-IP policy version). Before opening a socket M90 applies that exact public-IP policy version as a mandatory non-authoritative safety guard and returns `SourceTransportError::ObservationShapeRejected` if it cannot prove a public selected peer; M60 independently reruns the policy on every success. M90 returns only `SourceTransportError`; M60 maps it to crate-private `SourceFetchFailure` and alone produces policy failures.

```rust
pub fn DnsTransportObservation::new(
    queried_host: String,
    cname_chain: Vec<String>,
    complete_addresses: Vec<std::net::Ipv4Addr>,
) -> Result<DnsTransportObservation, SourceTransportError>

pub struct RetrievalTransportSuccessParts {
    pub response_head_bytes: Vec<u8>,
    pub body_bytes: Vec<u8>,
    pub wire_bytes_after_headers: u64,
    pub peer_socket_addr: std::net::SocketAddr,
    pub dns_transport_observation: DnsTransportObservation,
    pub framing: RetrievalBodyFraming,
    pub elapsed_milliseconds: u64,
}

pub fn RetrievalTransportSuccess::new(
    response_head_bytes: Vec<u8>,
    body_bytes: Vec<u8>,
    wire_bytes_after_headers: u64,
    peer_socket_addr: std::net::SocketAddr,
    dns_transport_observation: DnsTransportObservation,
    framing: RetrievalBodyFraming,
    elapsed_milliseconds: u64,
) -> Result<RetrievalTransportSuccess, SourceTransportError>

pub fn RetrievalTransportSuccess::into_parts(self) -> RetrievalTransportSuccessParts
pub fn RetrievalTransportSuccess::response_head_bytes(&self) -> &[u8]
pub fn RetrievalTransportSuccess::body_bytes(&self) -> &[u8]
pub fn RetrievalTransportSuccess::wire_bytes_after_headers(&self) -> u64
pub fn RetrievalTransportSuccess::peer_socket_addr(&self) -> std::net::SocketAddr
pub fn RetrievalTransportSuccess::dns_transport_observation(&self) -> &DnsTransportObservation
pub fn RetrievalTransportSuccess::framing(&self) -> RetrievalBodyFraming
pub fn RetrievalTransportSuccess::elapsed_milliseconds(&self) -> u64
```

Both constructors may return only transport `ObservationShapeRejected`, never `RetrievalPolicyError`. DNS shape bounds are host/CNAME bytes plus `1..=64` raw A answers; success representation bounds are head `<=32768`, body `<=1048577`, wire-after-headers `<=1114112`, IPv4 peer shape and elapsed `<=60000`. They do not apply request-specific budgets or M60 policy. `DnsTransportObservation` also exposes `queried_host(&self) -> &str`, `cname_chain(&self) -> &[String]`, `complete_addresses(&self) -> &[std::net::Ipv4Addr]`, and consuming `into_parts(self) -> (String, Vec<String>, Vec<std::net::Ipv4Addr>)`.

### 5.7 Phase method signatures (with object-safe lifetimes)

```text
RetrievalPolicy::derive_candidate(
    subject: &RetrievalSubject,
    command: &RetrievalAttemptCommand,
) -> Result<RetrievalPlanCandidate, RetrievalPolicyError>

RetrievalPolicy::evaluate_rate(
    candidate: &RetrievalPlanCandidate,
    now: RetrievalEpochSeconds,
    last_attempt_started_at: Option<RetrievalEpochSeconds>,
    override_facts: Option<&RetrievalOverrideFacts>,
    override_consumed: bool,
) -> Result<RetrievalRateDecision, RetrievalPolicyError>

RetrievalPolicy::authorize_resolution(
    candidate: RetrievalPlanCandidate,
    transport_observation: DnsTransportObservation,
) -> Result<ResolvedRetrievalCandidate, RetrievalPolicyError>

RetrievalPolicy::authorize_peer(
    plan: ResolvedRetrievalCandidate,
    peer: std::net::SocketAddr,
) -> Result<PeerBoundRetrievalCandidate, RetrievalPolicyError>

RetrievalPolicy::parse_strict_response_head(
    raw: &[u8],
) -> Result<ResponseHeadObservation, RetrievalPolicyError>

RetrievalPolicy::authorize_response_head(
    plan: PeerBoundRetrievalCandidate,
    head: ResponseHeadObservation,
) -> Result<BodyAdmissionCandidate, RetrievalPolicyError>

RetrievalPolicy::finish_body(
    admission: BodyAdmissionCandidate,
    body: BodyObservation,
) -> Result<ValidatedFetchCandidate, RetrievalPolicyError>
```

### 5.8 Closed enum families

| Enum | Variants |
|---|---|
| `SourceRetrievalCapability` | `Attempt`, `RateOverride` |
| `SourceStartCapabilities` | `AttemptOnly`, `AttemptAndRateOverride` |
| `RetrievalAttemptState` | `Admitted`, `Started`, `CompletedSuccess`, `CompletedFailure`, `Cancelled` |
| `RetrievalRateDecision` | `Allowed`, `AllowedWithOverride(RateOverrideId)` |
| `RetrievalAdmissionOutcome` | `Reserved(AdmittedRetrievalPlan)`, `Replay(RetrievalAttemptReceipt)` |
| `SourceRetrievalProtocolVersion` | `V0StrictHttpsIpv4Http11_20260809` |
| `PublicIpPolicyVersion` | `V0Ipv4Only20260809` |
| `HttpVersionClass` | `Http10`, `Http11`, `Http2`, `Http3`, `Other` |
| `RetrievalBodyFraming` | `ContentLength(u64)`, `Chunked`, `CloseDelimited` |
| `ObservedHeaderValue` | `Missing`, `One(String)`, `Repeated` |

### 5.9 Carrier trait table

| Carrier | Clone | Copy | Serde | Default | Public ctor |
|---|---|---|---|---|---|
| `RetrievalAttemptCommand` | yes | no | no | no | yes |
| `RetrievalRateOverrideRequest` | yes | no | no | no | yes |
| `RetrievalOverrideFacts` | yes | no | no | no | yes (checked) |
| `RetrievalOverrideEvidence` | no | no | no | no | no (owner-private) |
| `RetrievalAuthorityEvidence` | no | no | no | no | no (owner-private) |
| `SourceStartAuthorization` | no | no | no | no | no (owner-private) |
| `RetrievalAttemptEnvelope` | no | no | no | no | no (owner-private) |
| `RetrievalReplayIdentity` | no | no | no | no | no (owner-private) |
| `RetrievalAttemptReceipt` | no | no | no | no | no (owner-private) |
| `RetrievalAttemptCompletion` | no | no | no | no | no (owner-private) |
| `TransportStopped` | no | no | no | no | no (owner-private) |
| `SerializedRetrievalRequest` | no | no | no | no | no (private-field) |
| `RetrievalTransportRequest` | no | no | no | no | no (private-field) |
| `RetrievalTransportSuccess` | no | no | no | no | yes (shape-only) |
| `RetrievalTransportSuccessParts` | no | no | no | no | only `RetrievalTransportSuccess::into_parts` |
| `RetrievalPlanCandidate` | yes | no | no | no | only `derive_candidate` |
| `AdmittedRetrievalPlan` | no | no | no | no | no (owner-private) |
| `EffectReadyRetrievalPlan` | no | no | no | no | no (owner-private) |
| `ResolvedRetrievalCandidate` | no | no | no | no | consumed `authorize_resolution` |
| `PeerBoundRetrievalCandidate` | no | no | no | no | consumed `authorize_peer` |
| `ResponseHeadObservation` | yes | no | no | no | yes (checked) |
| `BodyObservation` | no | no | no | no | yes (checked) |
| `BodyAdmissionCandidate` | no | no | no | no | consumed `authorize_response_head` |
| `ValidatedFetchCandidate` | no | no | no | no | consumed `finish_body` |
| `BoundedFetch` | no | no | no | no | no (owner-private) |
| `DnsTransportObservation` | no | no | no | no | yes (shape-only) |
| `DnsResolutionObservation` | no | no | no | no | no (owner-private) |
| `RetrievalAdmissionOutcome` | no | no | no | no | no (linear enum) |
| `SourceTransportError` | yes | no | no | no | yes (transport-only) |
| `SourceFetchFailure` | no | no | no | no | no (owner-private) |

### 5.10 Complete named Rust inventory

Public non-authority values (directly or through checked pure construction):

```text
RetrievalAttemptId, RateOverrideId, RetrievalOverrideEvidenceId, SourceOperatorId,
SourceStartAuthorizationId, RetrievalEpochSeconds, RetrievalDnsName,
RetrievalAttemptCommand, RetrievalRateOverrideRequest, RetrievalOverrideFacts,
SourceRetrievalCapability, SourceStartCapabilities, SourceRetrievalProtocolVersion,
PublicIpPolicyVersion, HttpVersionClass, RetrievalBodyFraming, ObservedHeaderValue,
RetrievalRateDecision, ResponseHeadObservation, DnsTransportObservation,
BodyObservation, RetrievalTransportRequest, SerializedRetrievalRequest,
RetrievalTransportSuccess, RetrievalTransportSuccessParts, SourceTransportError,
RetrievalPolicyError, RetrievalPolicy
```

Crate-private/internal values (no public constructor, no Serde, no Clone unless specified):

```text
RetrievalAuthorityEvidence, SourceStartAuthorization, RetrievalAttemptEnvelope,
RetrievalReplayIdentity, RetrievalAttemptCompletion, TransportStopped,
RetrievalPlanCandidate, AdmittedRetrievalPlan, EffectReadyRetrievalPlan,
ResolvedRetrievalCandidate, PeerBoundRetrievalCandidate, BodyAdmissionCandidate,
ValidatedFetchCandidate, BoundedFetch, DnsResolutionObservation, SourceFetchFailure, RetrievalAttemptReceipt,
RetrievalOverrideEvidence, RetrievalAdmissionOutcome
```

`RetrievalTransportRequest` is a non-clone owned non-authority value constructed only by M60 from the effect-ready plan; `RetrievalTransportSuccess` is a non-clone observation with no authority and a consuming projection. They are the only M60→M90 boundary carriers.

### 5.11 Error algebra

`RetrievalPolicyError` is exactly:

```text
AttemptIdConflict, CommandIdConflict, RetrievalProtocolVersionMismatch,
AttemptSourceMismatch, ValidatedCandidateMismatch, MissingAttempt,
AttemptCompletionConflict, MissingOrTerminalSession, RequestContextMismatch,
OperatorPolicyUnavailable, UnauthorizedSourceOperator, SourceNotRetrievable,
StaleSourceAuthorityRevision, ClockUnavailable, ClockRegression,
OverrideEvidenceUnavailable, InvalidRateOverride, RateOverrideAlreadyConsumed,
RateLimitNotElapsed, LeaseUnavailable, LeaseTimeOverflow, LeaseExpired,
InvalidStartAuthorization, StartAuthorizationAlreadyConsumed, AdmissionStoreUnavailable,
PublicIpPolicyVersionMismatch, DnsAliasViolation, DnsAnswerCountViolation,
UnsupportedAddressFamily, NonPublicAddress, PeerAddressMismatch,
MalformedResponseHead, UnexpectedHttpVersion, InterimResponseDenied,
RedirectDenied, UnexpectedStatus, HeaderLimitExceeded, InvalidContentType,
UnexpectedContentType, UnsupportedContentEncoding, UnsupportedTransferCoding,
AmbiguousFraming, DeclaredBodyTooLarge, ChunkLimitExceeded, TrailerDenied,
WireLimitExceeded, BodyLimitExceeded, DeadlineExceeded
```

`SourceTransportError` is exactly `DnsUnavailable | ConnectFailed | TlsFailed | WriteFailed | ReadFailed | EofFramingFailure | ExecutionDeadline | TransportCancelled | ObservationShapeRejected` and carries no `RetrievalPolicyError` or raw/private payload. `SourceFetchFailure` is crate-private M60 domain error, exactly `Policy(RetrievalPolicyError) | Dns(SourceTransportError) | Connect(SourceTransportError) | Tls(SourceTransportError) | Transport(SourceTransportError) | Cancelled`; M90 cannot construct it.

## 6. V0 protocol and request

`SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809` freezes: GET, HTTPS, port 443, exact SourceUrl path, no query/fragment, no request body, redirects denied, automatic retries denied, proxy denied, auth/cookie absent, `Accept-Encoding: identity`, connection reuse denied, TLS 1.2/1.3, exact host as SNI, ALPN http/1.1, platform WebPKI trust store, no client certificate/0-RTT.

### 6.1 Serialized wire request

```text
GET <path> HTTP/1.1\r\n
Host: <host>\r\n
Accept: <media-type>\r\n
Accept-Encoding: identity\r\n
Connection: close\r\n
\r\n
```

`SerializedRetrievalRequest::as_bytes() -> &[u8]` is the only accessor.

## 7. Public-IP policy and peer binding

`PublicIpPolicyVersion::V0Ipv4Only20260809`: query only DNS A records; reject all IPv6. An IPv4 address is rejected iff its network-order `u32` belongs to one of the 15 CIDRs (0.0.0.0/8 through 240.0.0.0/4). M90 constructs only shape-checked raw `DnsTransportObservation`; M60's `authorize_resolution` applies host/CNAME/public-IP policy and privately creates `DnsResolutionObservation`. The one-call M90 transport also applies the same versioned table as a non-authoritative pre-connect safety guard. `authorize_peer` takes `std::net::SocketAddr` (not `Ipv4Addr`).

## 8. Strict HTTP/1.1 response-head parser

`parse_strict_response_head(raw: &[u8]) -> Result<ResponseHeadObservation, RetrievalPolicyError>` is the only constructor. CRLF only, `HttpVersionClass` recognizes `Http10`, `Http11`, `Http2`, `Http3`, `Other`. Only status `200` accepted. `ObservedHeaderValue = Missing | One(String) | Repeated`. Caps: status+headers ≤ 32768 raw bytes, ≤ 128 fields, name ≤ 64 bytes, value ≤ 8192 bytes.

## 9. Body, wire envelope and deadline

Two independent counters: delivered entity body ≤ `maximum_response_bytes`, wire bytes ≤ `maximum_response_bytes + 65536`. One total monotonic deadline `maximum_elapsed_seconds * 1000` ms. `BodyObservation::new(bytes, wire_bytes_after_headers, chunk_count, max_chunk_line_bytes, saw_chunk_extension, trailer_field_count, framing_complete, elapsed_milliseconds) -> BodyObservation`.

## 10. Acceptance and non-promotion

```text
SRC-001 implemented (unchanged bounded B1 evidence)
SRC-010 planned
SRC-011 planned
SRC-012 planned
SRC-014 catalog-only / non-admitted
M60 planned
M70 design-only
```

## 11. Future writable scopes

This R4 replacement packet names the exact 13-path scope:

```text
docs/tasks/m60-b2-retrieval-policy-readiness-proposal.md
docs/contracts/source-import.md
docs/contracts/source-retrieval.md
docs/contracts/module-boundaries.md
docs/plan/05-campus-trust-kernel.md
docs/plan/modules/00-module-map.md
docs/plan/modules/70-campus-trust-source-pipeline.md
docs/tasks/01-execution-roadmap.md
docs/acceptance/matrix.tsv
docs/acceptance/platform-baseline.md
docs/coverage-matrix.md
scripts/check_repo_contracts.py
scripts/tests/test_check_repo_contracts.py
```

The two acceptance files (`matrix.tsv`, `platform-baseline.md`) are readable reference surfaces that must remain byte-identical since they are absent from the current dirty seed and do not require status changes. `docs/coverage-matrix.md` and `docs/plan/modules/00-module-map.md` are required projections.

## 12. Stop conditions

Stop and return to Develata before authoritative mutation if:

- Develata has not accepted the exact replacement semantic packet digest;
- a reviewer finds an unresolved lifecycle, authority, permission or public-API alternative;
- implementation needs a dependency or path outside a later accepted allowlist;
- a real source would become Approved, Suspended, Revoked or fetched;
- raw source bytes, credentials, cookies, private endpoints or personal data would enter Git;
- M90 production safe-http, M60-B3 lease/snapshot or application composition is needed;
- `SRC-010`/`SRC-014` or M60 status would be promoted without complete bound evidence;
- any push, PR, merge, tag, release, deployment or publication lacks current operation-specific or active-campaign authority.
<!-- M60_B2_RETRIEVAL_POLICY_PROPOSAL:END -->

## Marker-external R11 representability clarification

The immutable R11 packet phrase “all with `expected_authority_revision` CAS” is represented by the accepted signatures, not by inventing a pre-creation revision. Initial `propose(full definition)` is the no-expected-revision creation exception and initializes `SourceAuthorityRevision` to `1`; every post-proposal lifecycle mutation (`revise`, `approve`, `suspend`, `reinstate`, `revoke`) requires exact expected-revision CAS and checked increment. The implemented bounded B1 projection also carries the closed one-variant `PublicIpPolicyVersion` inventory required by the packet's six-field retrieval policy. This clarification changes no byte inside the accepted R11 packet and grants no M60-B2 implementation authority.

## Marker-external M60-B2 implementation representability clarification

- `Decision`: `M60_B2_REPRESENTABILITY_CLARIFICATION_20260901`
- `Selecting authority`: Develata, 2026-09-01
- `Implementation taskbook semantic packet`: `sha256:19fb0e7696ffd298e34da0c52507f3b186fa50d9ee9ccc4b68657ec65cb1026e` over `26003` marker-delimited bytes
- `Independent PRE_EDIT_TASKBOOK review`: `PASS`
- `Representation`: the five pure phase carriers are public opaque non-authority outputs with private fields and no public constructors; only `RetrievalPlanCandidate` is `Clone`; pure phase methods remain public
- `Body observation`: the exact fallible `BodyObservation::new(...) -> Result<BodyObservation, SourceTransportError>` shape contract and `0..=1_048_577` exact byte-retention bound are current authority
- `Rate`: the exhaustive pure decision/precedence table installed in `source-retrieval/v0` is current authority
- `Non-claims`: the retained Rust slice is bounded offline pure policy only; no port/effect/B3 authority, no network retrieval, no concrete-source approval, no acceptance promotion, no push/PR/merge/tag/release/deployment authority

This marker-external clarification resolves Rust representability without changing any byte inside `M60_B2_RETRIEVAL_POLICY_PROPOSAL`; the separately reviewed implementation taskbook now binds the first retained offline-only projection.

### Implementation projection receipt

- `Taskbook`: [`m60-b2-offline-retrieval-policy.md`](m60-b2-offline-retrieval-policy.md)
- `Frozen semantic packet`: `sha256:19fb0e7696ffd298e34da0c52507f3b186fa50d9ee9ccc4b68657ec65cb1026e` over `26003` bytes
- `Frozen executable source phase`: controller-owned receipt `sha256:08270e618a32fc3f433971381c7fa9c01868ae84ec0e6b5188d08c6e91dfcaf9`
- `Scope`: public opaque phase values plus pure URL/DNS/IP/redirect/header/media/body/rate/error policy; synthetic observations only
- `Non-claims`: no M60/M90 port, socket, DNS lookup, HTTP client, filesystem/journal/clock effect, concrete-source approval, B3 admission or remote shipping authority

## Review receipts

Earlier reviews progressively blocked or invalidated incomplete authority revision, override, admission/effect, lease and raw-wire designs. V7's exact review found fake-observation finalization, start-authority TOCTOU, command/attempt replay algebra, missing transport accessors and wall-clock recovery release; V9 repaired those but its exact review found one recovery wording that released the very ID indexes needed for replay. V10 makes both indexes and their immutable record permanent tombstones while releasing only concurrency slots.

### M60-B2 V10 terminal product-review receipt (SUPERSEDED)

- `Receipt status`: `ISSUED` (historical — superseded by R4 replacement packet)
...
The reviewer independently recomputed the exact packet byte count/digest and source commit/tree, then found no product-level bypass through completion, cancellation, restart recovery, compaction, replay, receipt reconstruction, effect gating, transport access, request wire, DNS/IP/TLS or response framing/deadline. This receipt made V10 reviewed, not accepted. **The R4 direction receipt supersedes this entire V10 contract structure.**

### M60-B2 semantic acceptance receipt (SUPERSEDED — HISTORICAL ONLY)

- `Receipt status`: `ACCEPTED_WITH_AMENDMENT` (SUPERSEDED)
- `Accepting authority`: Develata, 2026-08-11
- `Decision`: accept V10 contract structure; operational `Suspended`/`Revoked` lifecycle precondition; separate implementation packet required — **all superseded by R4 direction**

### M60-B2 R4 replacement direction receipt (HISTORICAL — SUPERSEDED BY R11 ACCEPTANCE)

- `Receipt status`: `PROPOSED_NOT_ACCEPTED`
- `Direction`: Develata, 2026-08-12 — approved two-layer M60/M90 transport architecture
- `R4 pack`: this exact 11-path docs/checker replacement candidate
- `Non-claims`: no semantic acceptance, no retained implementation, no source approval, no network retrieval, no push/PR/merge/tag/release/publication
- `Gating`: satisfied by the R11 exact-digest review and the current acceptance receipt below

### M60-B2 R11 exact semantic acceptance receipt (CURRENT)

- `Receipt status`: `ACCEPTED`
- `Decision`: `ACCEPT_EXACT_M60_B2_R11_PACKET`
- `Accepting authority`: Develata, 2026-08-13
- `Exact semantic packet`: `sha256:34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e` over `33046` bytes
- `Bound source`: commit `a1b0efe33239b33afeea40e7981cf15f8a65cd1e`, tree `bd3f096a26baf758b58bf80874e09a1604e885c0`
- `Independent review`: `FINAL_PRODUCT_GO`
- `Semantic effect`: `source-import/v1` and `source-retrieval/v0` become current accepted contract authority while `M60` implementation state and every acceptance row remain unchanged
- `Non-claims`: no Rust implementation, concrete-source approval, network fetch, publication, commit, push, PR, merge, tag, release or deployment
- `Next gate`: retained implementation requires a separately admitted exact implementation packet

The last two bullets above are the decision-time non-claims of the R11 contract-acceptance receipt. The later marker-external implementation receipt supersedes only the “no Rust implementation” timing statement; all effect, approval, promotion and shipping non-claims remain in force.
