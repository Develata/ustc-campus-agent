# Source retrieval policy contract

## Metadata

- `Status`: Accepted contract as `source-retrieval/v0` under the R11 M60-B2 two-layer transport architecture, with the first bounded offline pure-policy implementation retained; supersedes the accepted V10 `DEC-M60-B2-ACCEPTANCE`
- `Version`: `source-retrieval/v0`
- `Last Review`: `2026-09-01`
- `Accepted Per`: `ACCEPT_EXACT_M60_B2_R11_PACKET` — Develata accepted the exact `33046`-byte semantic packet (`sha256:34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e`) on 2026-08-13
- `Owning Blueprint`: [`M60 Campus Trust and Source Pipeline`](../plan/modules/70-campus-trust-source-pipeline.md)
- `Depends On`: [`source-import.md`](source-import.md), [`module-boundaries.md`](module-boundaries.md)
- `Superseded Packet Digest`: `sha256:ba36425adc164ca9b3ec75addd4be2e4b299b5f8a8cfb75cf6a710679acd32ab` over `77276` bytes — historical evidence only; this R4 packet supersedes it
- `Acceptance`: no acceptance rows promoted by this contract; `SRC-010` remains `planned`, `SRC-014` catalog-only/non-admitted
- `Implementation`: `crates/platform-core/src/source_retrieval.rs` and `crates/platform-core/tests/source_retrieval.rs` retain the bounded offline pure-policy implementation admitted by `M60_B2_REPRESENTABILITY_CLARIFICATION_20260901` and the independently reviewed taskbook packet `sha256:19fb0e7696ffd298e34da0c52507f3b186fa50d9ee9ccc4b68657ec65cb1026e`; no transport port, network effect, real source approval or B3 admission carrier is implemented

## 1. Scope and authority

`source-retrieval/v0` owns the pure request, rate, DNS, response, framing and deadline decision algebra for bounded HTTPS retrieval of approved sources. It defines:

- exact request candidate derivation from a current `RetrievalSubject`;
- pure rate evaluation with override semantics;
- DNS resolution observation, public-IP policy validation and peer binding;
- strict HTTP/1.1 response-head parsing and framing enforcement;
- body observation, wire/deadline bounds and validation chain;
- semantic outbound ports (`RetrievalAdmissionPort`, `SourceOperatorPolicyPort`, `RetrievalClockPort`, `SourceFetchPort`) that M60 owns;
- the public, framework-neutral, non-authority M90 transport port (`SourceTransportPort`).

It does **not** perform DNS resolution, open a socket, establish TLS, send bytes, receive bytes, read a clock, load a persistent store, mint ACL tokens, authenticate a user, create a raw snapshot or produce a source revision. These effects belong to M60-B3 (admission/lease/journal), M90 (transport adapter) and later M60 slices.

### 1.1 Two-layer M60/M90 transport boundary

- **M60** / `platform-core` exclusively owns retrieval admission, the crate-internal effect-ready carrier, phase-policy validation, attempt completion, `TransportStopped`, and minting the trusted `BoundedFetch` domain result.
- **M90** / `crates/adapters` implements only a public, framework-neutral, non-authority transport/session port (`SourceTransportPort`). It performs DNS, socket, TLS and bounded raw HTTP I/O under M60-provided bounds and returns only transport observations (`RetrievalTransportSuccess`) or transport failures (`SourceTransportError`).
- M90 never receives `EffectReadyRetrievalPlan` and never returns or constructs `BoundedFetch`, a domain receipt, or an authority-bearing carrier.
- M60 consumes the one-shot effect-ready plan, derives one public `RetrievalTransportRequest`, invokes M90 via `SourceTransportPort::transport`, validates the response observation against the retained internal plan and strict phase algebra, mints `BoundedFetch` only on success, and terminalizes the attempt only after the transport call returned or cancellation/drop proved resource stop.
- Dependency direction: `crates/adapters -> crates/platform-core`. No `platform-core` dependency on `adapters`, no cycle, `platform-core` free of production networking/runtime dependencies.

Pure B2 decision output is `RetrievalPlanCandidate`, which is explicitly not effect authority. Only a separate M60-B3 atomic admission transaction can produce `EffectReadyRetrievalPlan`. Only the crate-private sealed `SourceFetchPort` exchange can produce `BoundedFetch`.

## 2. Nominal identity values

All new nominal values use the B1 string grammar and bound (`1..=128` bytes, `[a-z0-9](?:[-a-z0-9._:]{0,126}[a-z0-9])?`):

```text
RetrievalAttemptId
RateOverrideId
RetrievalOverrideEvidenceId
SourceOperatorId
SourceStartAuthorizationId
```

Each has checked `new`, `as_str`, `into_inner`; `Clone + Debug + Eq + Ord + Hash`; no `Default`, Serde or unchecked constructor.

`RetrievalEpochSeconds(u64)` has private `u64`, `from_unix_seconds(u64) -> RetrievalEpochSeconds` (total, not accepted by any caller command), `get()`, and `Copy + Clone + Debug + Eq + Ord + Hash`.

`RetrievalDnsName` is lowercase ASCII DNS text without a trailing dot: total `3..=253` bytes, at least two labels, labels `1..=63`, label edge alphanumeric, interior `[a-z0-9-]`. Constructor: `parse(&str) -> Result<RetrievalDnsName, SourceValueError>`. Traits: `Clone + Eq + Ord + Hash`, no Serde/Display, redacted `Debug`.

`CommandId` and `PlatformRequestContext` are M00-owned imported types; M60 neither redefines nor re-exports them.

## 3. Attempt command and override command

```text
SourceRetrievalCapability = Attempt | RateOverride
SourceStartCapabilities = AttemptOnly | AttemptAndRateOverride

RetrievalRateOverrideRequest {
    override_id: RateOverrideId,
    evidence_id: RetrievalOverrideEvidenceId,
}

RetrievalOverrideEvidence {
    evidence_id: RetrievalOverrideEvidenceId,
    override_id: RateOverrideId,
    attempt_id: RetrievalAttemptId,
    operator: SourceOperatorId,
    source_id: SourceId,
    authority_revision: SourceAuthorityRevision,
    issued_at: RetrievalEpochSeconds,
    not_after: RetrievalEpochSeconds,
}

RetrievalOverrideFacts {
    // exact same eight fields; public non-authority pure-test value
}

RetrievalAttemptCommand {
    command_id: CommandId,
    attempt_id: RetrievalAttemptId,
    source_id: SourceId,
    expected_authority_revision: SourceAuthorityRevision,
    override_request: Option<RetrievalRateOverrideRequest>,
}
```

`RetrievalRateOverrideRequest::new(RateOverrideId, RetrievalOverrideEvidenceId) -> RetrievalRateOverrideRequest` and `RetrievalAttemptCommand::new(CommandId, RetrievalAttemptId, SourceId, SourceAuthorityRevision, Option<RetrievalRateOverrideRequest>) -> RetrievalAttemptCommand` are infallible after nominal inputs are validated.

`RetrievalOverrideFacts::new(RetrievalOverrideEvidenceId, RateOverrideId, RetrievalAttemptId, SourceOperatorId, SourceId, SourceAuthorityRevision, RetrievalEpochSeconds, RetrievalEpochSeconds) -> Result<RetrievalOverrideFacts, SourceValueError>` is checked only for `issued_at <= not_after` and creates non-authority test data.

`RetrievalOverrideEvidence` is owner-private (no public constructor). `RetrievalRateOverrideRequest` is not a receipt and confers no authority.

## 4. Authority evidence and start authorization

```text
RetrievalAuthorityEvidence {
    // owner-private projection from admitted PlatformRequestContext:
    tenant_id, user_id, session_id, session_revision,
    request_id, command_id, correlation_id, policy_reference,
    source_operator_id
}

SourceStartAuthorization {
    start_authorization_id: SourceStartAuthorizationId,
    command_id, attempt_id, source_id, authority_revision,
    tenant_id, user_id, session_revision,
    source_operator_id, operator_policy_revision,
    capabilities: SourceStartCapabilities,
    issued_at, not_after: RetrievalEpochSeconds
}

RetrievalAttemptEnvelope {
    command: RetrievalAttemptCommand,
    authority: RetrievalAuthorityEvidence,
}

RetrievalReplayIdentity {
    command: RetrievalAttemptCommand,
    tenant_id, user_id, source_operator_id,
}
```

Identity and operator authority never come from the caller command. [`platform-request-context/v0`](platform-request-context.md) owns the sealed `M00AdmittedActor::{Public, Authenticated}`, `PlatformRequestContext`, and request-scoped immutable operation descriptor projection; M10 will pass one admitted context beside the typed command over existing `B-M10-APP-CALL`. This retrieval contract consumes only the exact authenticated arm where private source authority is required and never upgrades `Public` into synthetic tenant/user/session identity. The bounded M00 kernel is implemented, while that M10 composition remains planned.

`SourceStartAuthorization` is the effect-time current authority witness. `SourceOperatorPolicyPort::authorize_start` consumes the sealed context continuation, validates current session/operation and exact capability, then mints one owner-private witness bound to the complete command/source/revision/tenant/user/operator with a trusted `issued_at..=not_after` window ≤ 5 seconds. The witness is non-clone, no Copy/Serde/Default/Display/public constructor, and the B3 start transaction atomically consumes its unique ID.

`RetrievalAttemptEnvelope`, `RetrievalAuthorityEvidence`, `RetrievalReplayIdentity`, `RetrievalAttemptCompletion` and `TransportStopped` are owner-private outputs with no public constructor or Serde.

## 5. Attempt state and completion

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

`RetrievalAttemptReceipt` fields and constructor are owner-private; accessors are read-only. It is the idempotent B3 result. `RetrievalAttemptCompletion` is owner-private. `TransportStopped` is owner-private, constructed only after synchronous resource destruction.

## 6. Admission and rate semantics

### 6.1 Pure candidate

`RetrievalPolicy::derive_candidate` produces `RetrievalPlanCandidate` from a current `RetrievalSubject` and complete `RetrievalAttemptCommand`. It checks source ID/revision and derives exact immutable request fields. It is explicitly not effect authority.

V0 concurrency: ≤ `1` per `SourceId`, ≤ `1` per canonical host, ≤ `4` globally. Rate override never bypasses these caps.

### 6.2 Atomic admission transaction (B3-only port)

```text
RetrievalAdmissionOutcome = Reserved(AdmittedRetrievalPlan) | Replay(RetrievalAttemptReceipt)
```

`RetrievalAdmissionPort` (M60-owned, B3-implemented):

```text
admit(context: PlatformRequestContext, candidate: RetrievalPlanCandidate) -> Result<RetrievalAdmissionOutcome, RetrievalPolicyError>
start(plan: AdmittedRetrievalPlan) -> Result<EffectReadyRetrievalPlan, RetrievalPolicyError>
complete(completion: RetrievalAttemptCompletion) -> Result<RetrievalAttemptReceipt, RetrievalPolicyError>
receipt(context: PlatformRequestContext, command_id: CommandId) -> Result<RetrievalAttemptReceipt, RetrievalPolicyError>
```

`admit` transaction precedence:
1. `command.command_id == context.command_id` and operation `source.retrieval.attempt/v0`;
2. trusted `now` from `RetrievalClockPort`;
3. sealed context/session admission + `SourceOperatorPolicyPort` for `Attempt` + optional `RateOverride`;
4. derive owner-private `RetrievalAuthorityEvidence`, envelope, `RetrievalReplayIdentity`;
5. load authoritative attempt/override/lease records;
6. dual unique indexes: equal IDs + equal replay = `Replay(reconstructed_receipt)`;
7. reload source: non-`Approved` or revision mismatch rejects;
8. read `last_attempt_started_at`; validate rate with optional override;
9. compute reservation expiry `now + 30`;
10. atomically commit: consume override, acquire slots, append record, mint `AdmittedRetrievalPlan`.

`start` consumes `AdmittedRetrievalPlan`, calls `authorize_start` for a witness, reloads source at commit time, atomically consumes the witness and returns one `EffectReadyRetrievalPlan`.

Durable journal: unique indexes on both `CommandId` and `RetrievalAttemptId`; permanent tombstones. One admitted command produces at most one outbound attempt. Replay reconstructs a receipt without I/O.

### 6.3 Pure rate evaluation

```text
RetrievalRateDecision = Allowed | AllowedWithOverride(RateOverrideId)

RetrievalPolicy::evaluate_rate(candidate, now, last_attempt_started_at, override_facts, override_consumed) -> Result<RetrievalRateDecision, RetrievalPolicyError>
```

Pure B2 helper; output is never accepted by `RetrievalAdmissionPort`. B3 reloads authoritative facts and recomputes it in the transaction.

The exact pure decision table and branch-local precedence are:

1. if `last_attempt_started_at = Some(last)` and `now < last`, return `ClockRegression`;
2. if there is no prior attempt, or checked `now - last >= minimum_interval_seconds`, return `Allowed`; an unnecessary override request is ignored and cannot mint an override decision;
3. otherwise the interval has not elapsed:
   - no override request returns `RateLimitNotElapsed`;
   - an override request with no `override_facts` returns `OverrideEvidenceUnavailable`;
   - facts whose evidence ID, override ID, attempt ID, source ID or source-authority revision do not exactly match the request/candidate, or whose window does not satisfy `issued_at <= now <= not_after`, return `InvalidRateOverride`;
   - exact valid facts with `override_consumed = true` return `RateOverrideAlreadyConsumed`;
   - exact valid unconsumed facts return `AllowedWithOverride` carrying the exact requested `RateOverrideId`.

This table is exhaustive. The helper neither consumes evidence nor changes concurrency. Future B3 loads authoritative facts and executes the same table inside its transaction.

## 7. V0 protocol and public-IP policy

`SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809` freezes:

```text
method             GET
scheme             https
port               443 (implicit only)
host/path          exact SourceUrl bytes
query/fragment     absent
request body       absent
redirects          denied
automatic retries  denied
proxy              denied, including environment/system proxy
auth/cookie        absent
referer            absent
content encoding   request Accept-Encoding: identity
connection reuse   denied; one fresh connection per attempt
TLS identity       TLS 1.2 or 1.3; exact host as SNI/certificate name/Host
ALPN               exactly http/1.1
TLS trust          platform WebPKI root store; no custom CA
TLS session        no client certificate, 0-RTT or cross-attempt connection reuse
```

### 7.1 Serialized wire request

Headers: `Host`, `Accept` (expected media type), `Accept-Encoding: identity`, `Connection: close`. No `User-Agent`. The serializer emits exactly:

```text
GET <path> HTTP/1.1\r\n
Host: <host>\r\n
Accept: <media-type>\r\n
Accept-Encoding: identity\r\n
Connection: close\r\n
\r\n
```

`SerializedRetrievalRequest::as_bytes() -> &[u8]` is the only accessor. No public constructor, mutation, Serde, `Display`, header map or URL conversion.

### 7.2 Public-IP policy v0

`PublicIpPolicyVersion::V0Ipv4Only20260809`: query only DNS A records; reject all IPv6.

An IPv4 address is rejected iff its network-order `u32` belongs to:

```text
0.0.0.0/8        10.0.0.0/8       100.64.0.0/10    127.0.0.0/8
169.254.0.0/16    172.16.0.0/12    192.0.0.0/24     192.0.2.0/24
192.88.99.0/24    192.168.0.0/16   198.18.0.0/15    198.51.100.0/24
203.0.113.0/24    224.0.0.0/4      240.0.0.0/4
```

Deny-only, most-specific-first. Any address outside is admitted. Table edits require a new `PublicIpPolicyVersion`, contract/checker/mutation update and review.

### 7.3 DNS resolution and peer binding

M90 reports raw DNS shape through `DnsTransportObservation`, not a policy verdict:

```rust
pub fn DnsTransportObservation::new(
    queried_host: String,
    cname_chain: Vec<String>,
    complete_addresses: Vec<std::net::Ipv4Addr>,
) -> Result<DnsTransportObservation, SourceTransportError>
```

This shape-only constructor accepts no policy version and may reject only as `SourceTransportError::ObservationShapeRejected`: queried-host bytes `1..=253`; at most `64` CNAME strings, each `1..=253` bytes; and `1..=64` raw A answers. It does not apply the v0 public-IP table, sort/deduplicate addresses, reject alias loops, select a peer or return `RetrievalPolicyError`. Read-only accessors are `queried_host(&self) -> &str`, `cname_chain(&self) -> &[String]`, `complete_addresses(&self) -> &[std::net::Ipv4Addr]`; `into_parts(self) -> (String, Vec<String>, Vec<std::net::Ipv4Addr>)` consumes the raw carrier.

M60 alone consumes that carrier in `RetrievalPolicy::authorize_resolution(candidate, transport_observation)`: it requires the exact approved host, applies CNAME depth `0..=8`/loop/alias rules, sorts and deduplicates to `1..=16` IPv4 values, applies the v0 public-IP table and creates crate-private `DnsResolutionObservation`. The selected peer is the numerically lowest admitted IPv4 address plus port `443`. TLS SNI and HTTP Host remain the exact canonical DNS host. Because the approved boundary is one transport call, M90 must also apply the same versioned deny table as a non-authoritative pre-connect safety guard and return `SourceTransportError::ObservationShapeRejected` without opening a socket if it cannot prove a public selected peer. M60 independently reruns the policy over every successful raw observation; the M90 guard neither returns `RetrievalPolicyError` nor mints a domain candidate.

## 8. Response policy

### 8.1 Status and redirects

Only HTTP status `200` is accepted in v0. Every `3xx` is `RedirectDenied`. Conditional `304` belongs with B3 validators and is not smuggled into B2.

### 8.2 Strict HTTP/1.1 response-head parser

`RetrievalPolicy::parse_strict_response_head(raw: &[u8]) -> Result<ResponseHeadObservation, RetrievalPolicyError>` is the only `ResponseHeadObservation` constructor. It enforces:

- `CRLF` only; bare CR/LF/NUL/obs-fold reject;
- one status line `HTTP/DIGIT.DIGIT SP 3DIGIT SP reason CRLF`;
- field names of RFC `tchar` bytes followed by `:`; no whitespace before colon;
- field values: ASCII space or visible ASCII only;
- exactly one empty `CRLF` terminator.

Parser caps: status+headers ≤ 32768 raw bytes, ≤ 128 header fields, field name ≤ 64 bytes, field value ≤ 8192 bytes.

### 8.3 Framing and body validation

`Content-Type`: exactly one syntactically valid `type/subtype` essence required. Parameters ≤ 16, name/value `1..=64`, no duplicates. Optional ASCII space is admitted around `;`, but not on either side of a parameter `=`. An unquoted parameter value is an RFC `tchar` token and therefore contains no spaces; a quoted value may contain ASCII spaces in addition to `tchar` bytes. Case-insensitive essence must equal `expected_media_type`.

Framing rules:
- `Content-Encoding`: absent or exactly `identity`;
- `Transfer-Encoding`: absent or exactly `chunked` (one token, no parameters/chains);
- `Content-Length`: `0` or `[1-9][0-9]*`, no coexistence with `Transfer-Encoding`;
- absent both means close-delimited under `Connection: close`;
- `Trailer`: absent;
- chunk-size lines ≤ 128 raw bytes, `1..=16` hex digits, non-final count ≤ 4096.

`max_chunk_line_bytes` counts bytes before `CRLF` in the widest observed chunk-size line. Because any extension sets `saw_chunk_extension = true` and is rejected, an accepted line consists only of its hexadecimal size digits; the policy therefore rejects widths `0` and `> 16` directly, which also satisfies the broader 128-byte line cap.

Body/wire counters operate independently. Wire ≤ `maximum_response_bytes + 65536`. Delivered entity body ≤ `maximum_response_bytes`. Adapter retains at most `maximum_response_bytes + 1` entity bytes.

The only public body-observation constructor is:

```rust
BodyObservation::new(
    bytes: Vec<u8>,
    wire_bytes_after_headers: u64,
    chunk_count: u32,
    max_chunk_line_bytes: u16,
    saw_chunk_extension: bool,
    trailer_field_count: u16,
    framing_complete: bool,
    elapsed_milliseconds: u64,
) -> Result<BodyObservation, SourceTransportError>
```

It is shape-only. It accepts every scalar value in the complete Rust type domain and imposes no shape restriction on either boolean. It accepts `bytes.len()` in `0..=1_048_577`, preserves the exact vector without truncation, and returns only `SourceTransportError::ObservationShapeRejected` for a larger vector while retaining none of the rejected bytes. The one-byte overflow sentinel makes the global `maximum_response_bytes = 1_048_576` failure representable without unbounded retention. Request-specific wire/body/chunk/trailer/framing/deadline rules are applied only by `finish_body`.

### 8.4 Deadline

One total monotonic deadline of `maximum_elapsed_seconds * 1000` milliseconds, from immediately before DNS resolution through final framing/EOF verification. All phases share the same deadline. The fetch future directly owns every resource; dropping it closes them synchronously.

## 9. Non-authority transport boundary (M60→M90)

### 9.1 RetrievalTransportRequest

`RetrievalTransportRequest` is a non-authority, private-field, owned struct containing exactly the adapter inputs needed for deterministic transport. It owns all values (`RetrievalAttemptId`, `SourceId`, `SourceAuthorityRevision`, `RetrievalDnsName`, `SerializedRetrievalRequest`, `SourceMediaType`, `u32 maximum_response_bytes`, `u32 maximum_elapsed_seconds`, `SourceRetrievalProtocolVersion`, `PublicIpPolicyVersion`) and carries no lifetime parameter. It is `Debug + Eq`, not `Clone`/`Copy` — the owned serialized request bytes move into the adapter. Constructor is `pub(crate)` in platform-core and constructed only from `EffectReadyRetrievalPlan` by the M60 coordinator. No public constructor, Serde, `Default`, URL/header builder conversion, arbitrary headers or retry/proxy/cookie/auth knobs.

The exact read-only accessor surface is:

```rust
pub fn RetrievalTransportRequest::attempt_id(&self) -> &RetrievalAttemptId
pub fn RetrievalTransportRequest::source_id(&self) -> &SourceId
pub fn RetrievalTransportRequest::authority_revision(&self) -> SourceAuthorityRevision
pub fn RetrievalTransportRequest::canonical_host(&self) -> &RetrievalDnsName
pub fn RetrievalTransportRequest::serialized_request(&self) -> &SerializedRetrievalRequest
pub fn RetrievalTransportRequest::expected_media_type(&self) -> &SourceMediaType
pub fn RetrievalTransportRequest::maximum_response_bytes(&self) -> u32
pub fn RetrievalTransportRequest::maximum_elapsed_seconds(&self) -> u32
pub fn RetrievalTransportRequest::protocol_version(&self) -> &SourceRetrievalProtocolVersion
pub fn RetrievalTransportRequest::public_ip_policy_version(&self) -> &PublicIpPolicyVersion
```

Borrowed accessors expose owned nominal values without cloning; scalar fields/enums are copied. There are no additional accessors or conversions.

### 9.2 RetrievalTransportSuccess

`RetrievalTransportSuccess` is a private-field success observation carrying raw response-head bytes, body bytes, wire bytes after headers, peer socket address, raw `DnsTransportObservation`, framing, and elapsed milliseconds. Traits: `Debug`, no `Clone`, no `Copy`, no Serde, no `Default`.

```rust
pub fn RetrievalTransportSuccess::new(
    response_head_bytes: Vec<u8>,
    body_bytes: Vec<u8>,
    wire_bytes_after_headers: u64,
    peer_socket_addr: std::net::SocketAddr,
    dns_transport_observation: DnsTransportObservation,
    framing: RetrievalBodyFraming,
    elapsed_milliseconds: u64,
) -> Result<RetrievalTransportSuccess, SourceTransportError>

pub struct RetrievalTransportSuccessParts {
    pub response_head_bytes: Vec<u8>,
    pub body_bytes: Vec<u8>,
    pub wire_bytes_after_headers: u64,
    pub peer_socket_addr: std::net::SocketAddr,
    pub dns_transport_observation: DnsTransportObservation,
    pub framing: RetrievalBodyFraming,
    pub elapsed_milliseconds: u64,
}

pub fn RetrievalTransportSuccess::into_parts(self) -> RetrievalTransportSuccessParts

pub fn RetrievalTransportSuccess::response_head_bytes(&self) -> &[u8]
pub fn RetrievalTransportSuccess::body_bytes(&self) -> &[u8]
pub fn RetrievalTransportSuccess::wire_bytes_after_headers(&self) -> u64
pub fn RetrievalTransportSuccess::peer_socket_addr(&self) -> std::net::SocketAddr
pub fn RetrievalTransportSuccess::dns_transport_observation(&self) -> &DnsTransportObservation
pub fn RetrievalTransportSuccess::framing(&self) -> RetrievalBodyFraming
pub fn RetrievalTransportSuccess::elapsed_milliseconds(&self) -> u64
```

The constructor may return only `SourceTransportError::ObservationShapeRejected` for representation bounds: response-head bytes `<=32768`, body bytes `<=1048577`, wire bytes after headers `<=1114112`, IPv4 peer shape and elapsed milliseconds `<=60000`. It does not compare request-specific budgets, apply the public-IP table, parse/authorize response semantics or return `RetrievalPolicyError`. Read-only accessors have the exact field names above and return borrowed byte slices/raw DNS observation or copied scalar/enum/socket values. The consuming parts value moves raw head/body/DNS owners into M60 without repeat buffering. No observation constructor returns or embeds `RetrievalPolicyError`, `BoundedFetch`, receipt, phase carrier or authority witness.

### 9.3 SourceTransportPort (M90 public port)

```rust
pub trait SourceTransportPort: Send + Sync {
    fn transport<'a>(
        &'a self,
        request: RetrievalTransportRequest,
    ) -> Pin<Box<dyn Future<Output = Result<RetrievalTransportSuccess, SourceTransportError>> + Send + 'a>>;
}
```

This is a public, framework-neutral, non-authority port. It uses only standard-library `Future`/`Pin` types. The adapter future owns all resolver/socket/TLS/body work and may not spawn/detach it. Before opening a socket it applies the request's exact `PublicIpPolicyVersion` as the mandatory non-authoritative safety guard described in §7.3; M60 reruns the same policy on every success. Success/error return occurs only after synchronous resource destruction. Dropping the future cancels and destroys resources before drop completes; no observation/result is produced on drop.

## 10. M60-owned ports

```rust
pub(crate) trait SourceOperatorPolicyPort: Send + Sync {
    fn authorize(&self, context: &PlatformRequestContext, source_id: &SourceId,
        authority_revision: SourceAuthorityRevision, capability: SourceRetrievalCapability,
        now: RetrievalEpochSeconds) -> Pin<Box<dyn Future<Output = Result<SourceOperatorId, RetrievalPolicyError>> + Send + '_>>;
    fn authorize_start(&self, context: PlatformRequestContext, command: &RetrievalAttemptCommand,
        source_id: &SourceId, authority_revision: SourceAuthorityRevision,
        now: RetrievalEpochSeconds) -> Pin<Box<dyn Future<Output = Result<SourceStartAuthorization, RetrievalPolicyError>> + Send + '_>>;
}

pub(crate) trait RetrievalClockPort: Send + Sync {
    fn now(&self) -> Pin<Box<dyn Future<Output = Result<RetrievalEpochSeconds, RetrievalPolicyError>> + Send + '_>>;
}

pub(crate) trait RetrievalAdmissionPort: Send {
    fn admit(&mut self, context: PlatformRequestContext, candidate: RetrievalPlanCandidate)
        -> Pin<Box<dyn Future<Output = Result<RetrievalAdmissionOutcome, RetrievalPolicyError>> + Send + '_>>;
    fn start(&mut self, plan: AdmittedRetrievalPlan)
        -> Pin<Box<dyn Future<Output = Result<EffectReadyRetrievalPlan, RetrievalPolicyError>> + Send + '_>>;
    fn complete(&mut self, completion: RetrievalAttemptCompletion)
        -> Pin<Box<dyn Future<Output = Result<RetrievalAttemptReceipt, RetrievalPolicyError>> + Send + '_>>;
    fn receipt(&mut self, context: PlatformRequestContext, command_id: CommandId)
        -> Pin<Box<dyn Future<Output = Result<RetrievalAttemptReceipt, RetrievalPolicyError>> + Send + '_>>;
}

pub(crate) trait SourceFetchPort: sealed::SourceFetchPortSealed + Send + Sync {
    fn fetch(&self, plan: EffectReadyRetrievalPlan)
        -> Pin<Box<dyn Future<Output = Result<BoundedFetch, SourceFetchFailure>> + Send + '_>>;
}
```

M60 owns context mapping, command, admission, trusted-time use, plan, policy, result, error and conformance. M90 returns only raw non-authority transport observations or `SourceTransportError`; it cannot mint, broaden or reinterpret authority and never names `EffectReadyRetrievalPlan`, domain phase carriers, `TransportStopped`, attempt receipt, `SourceFetchFailure` or `BoundedFetch`. The internal M60 coordinator maps `SourceTransportError` into crate-private/domain `SourceFetchFailure`. All ports use only standard-library `Future`/`Pin`; no external runtime/framework type crosses the boundary.

The first retained B2 implementation contains only pure registry/policy and non-authority observation values exercised by synthetic fakes. It implements none of the four ports above.

## 11. Error algebra

`RetrievalPolicyError` is exactly:

```text
AttemptIdConflict          CommandIdConflict          RetrievalProtocolVersionMismatch
AttemptSourceMismatch      ValidatedCandidateMismatch  MissingAttempt
AttemptCompletionConflict  MissingOrTerminalSession    RequestContextMismatch
OperatorPolicyUnavailable  UnauthorizedSourceOperator  SourceNotRetrievable
StaleSourceAuthorityRevision  ClockUnavailable         ClockRegression
OverrideEvidenceUnavailable  InvalidRateOverride       RateOverrideAlreadyConsumed
RateLimitNotElapsed        LeaseUnavailable            LeaseTimeOverflow
LeaseExpired              InvalidStartAuthorization   StartAuthorizationAlreadyConsumed
AdmissionStoreUnavailable  PublicIpPolicyVersionMismatch  DnsAliasViolation
DnsAnswerCountViolation    UnsupportedAddressFamily    NonPublicAddress
PeerAddressMismatch        MalformedResponseHead       UnexpectedHttpVersion
InterimResponseDenied      RedirectDenied              UnexpectedStatus
HeaderLimitExceeded        InvalidContentType          UnexpectedContentType
UnsupportedContentEncoding  UnsupportedTransferCoding  AmbiguousFraming
DeclaredBodyTooLarge       ChunkLimitExceeded          TrailerDenied
WireLimitExceeded          BodyLimitExceeded           DeadlineExceeded
```

### 11.1 Global primary-error precedence

Within the phase chain, the earliest reachable phase that produces an error wins. Across phases, global precedence is:

1. `RetrievalProtocolVersionMismatch` (derivation)
2. `AttemptSourceMismatch` (derivation)
3. `RateLimitNotElapsed` (rate — before no-override rejection)
4. `InvalidRateOverride` (rate — override evidence invalid)
5. `RateOverrideAlreadyConsumed` (rate — override already consumed)
6. `DnsAliasViolation` (resolution — CNAME alias mismatch)
7. `DnsAnswerCountViolation` (resolution — too many or too few A records)
8. `UnsupportedAddressFamily` (resolution — IPv6 or non-A record)
9. `NonPublicAddress` (peer — reserved/private IP)
10. `PeerAddressMismatch` (peer — selected address not in observed set)
11. `PublicIpPolicyVersionMismatch` (peer — policy version mismatch)
12. `MalformedResponseHead` (response — HTTP grammar violation)
13. `UnexpectedHttpVersion` (response — not HTTP/1.1)
14. `InterimResponseDenied` (response — 1xx)
15. `RedirectDenied` (response — 3xx)
16. `UnexpectedStatus` (response — not 200)
17. `HeaderLimitExceeded` (response — too many header fields)
18. `InvalidContentType` (response — malformed Content-Type)
19. `UnexpectedContentType` (response — Content-Type mismatch)
20. `UnsupportedContentEncoding` (response — unsupported Content-Encoding)
21. `UnsupportedTransferCoding` (response — unsupported Transfer-Encoding)
22. `AmbiguousFraming` (response — conflicting length/framing signals)
23. `DeclaredBodyTooLarge` (response — Content-Length exceeds cap)
24. `ChunkLimitExceeded` (response — chunk-size or chunk-count limit)
25. `TrailerDenied` (response — Trailer header present)
26. `WireLimitExceeded` (body — total wire bytes)
27. `BodyLimitExceeded` (body — entity body bytes)
28. `DeadlineExceeded` (body — monotonic deadline)

Per-phase admission/lease errors (`LeaseUnavailable`, `LeaseTimeOverflow`, `LeaseExpired`, `AdmissionStoreUnavailable`, `OverrideEvidenceUnavailable`) precede the rate phase but follow identity/command conflicts. Identity conflicts (`AttemptIdConflict`, `CommandIdConflict`) precede everything. Session errors (`MissingAttempt`, `MissingOrTerminalSession`, `RequestContextMismatch`) follow identity conflicts but precede admission errors.

`SourceTransportError` is the public M90 transport-only error (exactly `DnsUnavailable | ConnectFailed | TlsFailed | WriteFailed | ReadFailed | EofFramingFailure | ExecutionDeadline | TransportCancelled | ObservationShapeRejected`), carrying no `RetrievalPolicyError`, raw URL/header/body/DNS payload, credential, framework error string or private peer text.

`SourceFetchFailure` is the crate-private/domain M60 error family mapped from `SourceTransportError`, exactly `Policy(RetrievalPolicyError) | Dns(SourceTransportError) | Connect(SourceTransportError) | Tls(SourceTransportError) | Transport(SourceTransportError) | Cancelled`. M90 cannot construct domain policy failure.

No variant carries rejected raw URL/header/body/DNS payload, credential, framework error string or private peer text.

### 11.2 Carrier linearity rules

| Carrier | Clone | Copy | Serde | Default | Public ctor | Safe Debug | Display |
|---|---|---|---|---|---|---|---|
| `RetrievalAttemptCommand` | yes | no | no | no | yes (all fields infallible) | yes | no |
| `RetrievalRateOverrideRequest` | yes | no | no | no | yes (all fields infallible) | yes | no |
| `RetrievalOverrideFacts` | yes | no | no | no | yes (checked) | yes | no |
| `RetrievalOverrideEvidence` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalAuthorityEvidence` | no | no | no | no | no (owner-private) | yes | no |
| `SourceStartAuthorization` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalAttemptEnvelope` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalReplayIdentity` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalAttemptReceipt` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalAttemptCompletion` | no | no | no | no | no (owner-private) | yes | no |
| `TransportStopped` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalPlanCandidate` | yes | no | no | no | only `derive_candidate` | yes | no |
| `AdmittedRetrievalPlan` | no | no | no | no | no (owner-private) | yes | no |
| `ResolvedRetrievalCandidate` | no | no | no | no | no (owner-private) | yes | no |
| `PeerBoundRetrievalCandidate` | no | no | no | no | no (owner-private) | yes | no |
| `ResponseHeadObservation` | yes | no | no | no | yes (checked) | yes | no |
| `BodyAdmissionCandidate` | no | no | no | no | no (owner-private) | yes | no |
| `ValidatedFetchCandidate` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalTransportRequest` | no | no | no | no | no (private-field) | yes | no |
| `RetrievalTransportSuccess` | no | no | no | no | yes (shape-only) | yes | no |
| `RetrievalTransportSuccessParts` | no | no | no | no | only `RetrievalTransportSuccess::into_parts` | yes | no |
| `EffectReadyRetrievalPlan` | no | no | no | no | no (owner-private) | yes | no |
| `BoundedFetch` | no | no | no | no | no (owner-private) | yes | no |
| `DnsTransportObservation` | no | no | no | no | yes (shape-only) | yes | no |
| `DnsResolutionObservation` | no | no | no | no | no (owner-private) | yes | no |
| `BodyObservation` | no | no | no | no | yes (checked) | yes | no |
| `SerializedRetrievalRequest` | no | no | no | no | no (owner-private) | yes | no |
| `RetrievalAdmissionOutcome` | no | no | no | no | no (linear enum) | yes | no |

All nominal identity values follow the rule stated in §2: `Clone + Debug + Eq + Ord + Hash`, no `Copy`/`Default`/Serde. `RetrievalEpochSeconds` carries `Copy`. `RetrievalPlanCandidate`, `ResolvedRetrievalCandidate`, `PeerBoundRetrievalCandidate`, `BodyAdmissionCandidate` and `ValidatedFetchCandidate` are public opaque non-authority output types with private fields and no public constructors. All five have safe payload-redacted `Debug` and no `Copy`, Serde, `Default` or `Display`; only `RetrievalPlanCandidate` is `Clone`, while the other four are non-Clone. Owner-private effect-authority carriers remain non-public and never render raw payload, credential, header or semantic content.

### 11.3 Closed enum families

| Enum | Variants |
|---|---|
| `SourceRetrievalCapability` | `Attempt`, `RateOverride` |
| `SourceStartCapabilities` | `AttemptOnly`, `AttemptAndRateOverride` |
| `RetrievalAttemptState` | `Admitted`, `Started`, `CompletedSuccess`, `CompletedFailure`, `Cancelled` |
| `RetrievalRateDecision` | `Allowed`, `AllowedWithOverride(RateOverrideId)` |
| `RetrievalAdmissionOutcome` | `Reserved(AdmittedRetrievalPlan)`, `Replay(RetrievalAttemptReceipt)` |
| `SourceRetrievalProtocolVersion` | `V0StrictHttpsIpv4Http11_20260809` |
| `PublicIpPolicyVersion` | `V0Ipv4Only20260809` |
| `SourceTransportError` | `DnsUnavailable`, `ConnectFailed`, `TlsFailed`, `WriteFailed`, `ReadFailed`, `EofFramingFailure`, `ExecutionDeadline`, `TransportCancelled`, `ObservationShapeRejected` |
| `SourceFetchFailure` | `Policy(RetrievalPolicyError)`, `Dns(SourceTransportError)`, `Connect(SourceTransportError)`, `Tls(SourceTransportError)`, `Transport(SourceTransportError)`, `Cancelled` |
| `HttpVersionClass` | `Http10`, `Http11`, `Http2`, `Http3`, `Other` |
| `RetrievalBodyFraming` | `ContentLength(u64)`, `Chunked`, `CloseDelimited` |
| `ObservedHeaderValue` | `Missing`, `One(String)`, `Repeated` |

Every enum is closed. No caller-built variant may be injected.

### 11.4 Phase method signatures

```rust
// Public pure decision methods, non-clone carrier consumption:
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

`RetrievalPlanCandidate` may be cloned only to exercise independent pure branches; `evaluate_rate` borrows it and `authorize_resolution` consumes one candidate. Every later phase method consumes its non-Clone predecessor and produces the next opaque non-Clone phase output. No phase output has a public constructor or effect-authority conversion.

### 11.5 Complete named Rust inventory

Public non-authority values (directly or through checked pure construction):

| Type | § |
|---|---|
| `RetrievalAttemptId` | §2 |
| `RateOverrideId` | §2 |
| `RetrievalOverrideEvidenceId` | §2 |
| `SourceOperatorId` | §2 |
| `SourceStartAuthorizationId` | §2 |
| `RetrievalEpochSeconds` | §2 |
| `RetrievalDnsName` | §2 |
| `RetrievalAttemptCommand` | §3 |
| `RetrievalRateOverrideRequest` | §3 |
| `RetrievalOverrideFacts` | §3 |
| `SourceRetrievalCapability` | §3 |
| `SourceStartCapabilities` | §3 |
| `SourceRetrievalProtocolVersion` | §7 |
| `PublicIpPolicyVersion` | §7 |
| `HttpVersionClass` | §7 |
| `RetrievalBodyFraming` | §8 |
| `ObservedHeaderValue` | §8 |
| `RetrievalRateDecision` | §6 |
| `ResponseHeadObservation` | §8 |
| `DnsTransportObservation` | §7 |
| `BodyObservation` | §8 |
| `RetrievalTransportRequest` | §9 |
| `SerializedRetrievalRequest` | §7 |
| `RetrievalTransportSuccess` | §9 |
| `RetrievalTransportSuccessParts` | §9 |
| `SourceTransportError` | §11 |
| `RetrievalPolicyError` | §11 |
| `RetrievalPolicy` | §6 |
| `RetrievalPlanCandidate` | §6 |
| `ResolvedRetrievalCandidate` | §7 |
| `PeerBoundRetrievalCandidate` | §7 |
| `BodyAdmissionCandidate` | §8 |
| `ValidatedFetchCandidate` | §9 |

Crate-private/internal values (no public constructor, no Serde, no Clone unless specified):

| Type | § |
|---|---|
| `RetrievalAuthorityEvidence` | §4 |
| `SourceStartAuthorization` | §4 |
| `RetrievalAttemptEnvelope` | §4 |
| `RetrievalReplayIdentity` | §4 |
| `RetrievalAttemptCompletion` | §5 |
| `TransportStopped` | §5 |
| `AdmittedRetrievalPlan` | §6 |
| `EffectReadyRetrievalPlan` | §6 |
| `BoundedFetch` | §11 |
| `DnsResolutionObservation` | §7 |
| `SourceFetchFailure` | §11 |
| `RetrievalOverrideEvidence` | §3 |
| `RetrievalAttemptReceipt` | §5 |
| `RetrievalAdmissionOutcome` | §6

## 12. Non-claims and stop conditions

This contract and its retained bounded implementation:
- does not approve a concrete USTC source;
- does not authorize network retrieval;
- implements only pure Rust policy/observation algebra in `platform-core`, not an adapter, port implementation, clock, lease, journal or effect carrier;
- does not promote `SRC-010`, `SRC-014` or M60 status;
- does not authorize push, PR, merge, tag, release or publication.

The separately admitted implementation packet retains only the bounded offline pure policy and shape-only observations described above. Operational `Suspended`/`Revoked` lifecycle and monotone `SourceAuthorityRevision` must be present before any live B2 retrieval adapter; both prerequisites are already present, but every live B2 retrieval adapter, M60/M90 port implementation and B3 admission/effect carrier remains separately gated.

## 13. Change rule

Changing request semantics, DNS/IP policy table, HTTP grammar, framing rules, error taxonomy, port signatures, phase chain or body envelope changes `source-retrieval/v0` and requires contract, checker, mutation-test and downstream review on the same revision.