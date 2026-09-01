# M60-B2 bounded offline retrieval-policy implementation

## Mutable state

- `Stage`: `IMPLEMENTATION_COMPLETE_AWAITING_PARENT_GATES`
- `Disposition`: `SOURCE_PHASE_PASS`
- `Repair round`: `R2`
- `Blocker identity`: `none`
- `Next authorized mutation`: projection/checker phase may write only the remaining paths admitted by §3 while treating the four frozen executable carriers as immutable input
- `Independent review`: mandatory Hermes-native `codex-reviewer` `PASS` on taskbook `sha256:d509ef1fb95606bca0b6cc68baf810ec2a18d9471b548c8725751e5f495ad9c0` and semantic packet `sha256:19fb0e7696ffd298e34da0c52507f3b186fa50d9ee9ccc4b68657ec65cb1026e`
- `Implementation`: remote source phase passed; exact-candidate repository gates, sabotage and formal review remain pending
- `Remote shipping`: not authorized by this taskbook

## Operation-specific authority receipt

On 2026-09-01 Develata selected:

> 继续 M60-B2：先冻结 offline-only implementation packet，经独立 review 后实现；不触网、不批准真实 source

This authorizes freezing and independently reviewing the exact packet below, then implementing and verifying its bounded offline candidate. It does not authorize campus-source/product network retrieval, approval of a concrete source, an M90 production transport, M60-B3 admission/lease/snapshot work, parser/baseline/publication work, push, PR, merge, tag, release or deployment.

After mandatory v1 review identified three owning-contract blockers, Develata selected the minimal representability clarification on 2026-09-01:

> phase carriers are public opaque outputs with private fields and no public constructors; pure policy methods remain public; `BodyObservation::new` returns `Result<_, SourceTransportError>`; the taskbook rate branch table becomes owning-contract authority.

This decision authorizes the corresponding narrow `source-retrieval/v0` clarification in the writable contract/projection carriers. It does not widen effect, shipping or source authority.

## Exact source identity

- `Repository`: `Develata/ustc-campus-agent`
- `Source commit`: `ac9a38f6a979f03d88676cdb512e1103519b7bd4`
- `Source tree`: `d28ebfc3dbfb843f987fdeab6c3ba1f673e8b84e`
- `Source ref at freeze`: protected `origin/main`
- `Implementation packet digest`: `sha256:19fb0e7696ffd298e34da0c52507f3b186fa50d9ee9ccc4b68657ec65cb1026e` over `26003` marker-delimited bytes

<!-- M60_B2_OFFLINE_IMPLEMENTATION_PACKET:BEGIN -->
## 1. Mission and strongest honest claim

Implement the first retained `source-retrieval/v0` slice as a pure, deterministic, standard-library-only M60 policy kernel plus shape-only non-authority transport observations. The implementation performs no I/O and cannot produce effect authority.

The strongest terminal claim is exactly:

```text
M60-B2 has bounded offline pure retrieval-policy evidence.
It can derive and rate one non-authority retrieval candidate, validate synthetic DNS/peer/HTTP/body observations, and preserve exact v0 wire/policy semantics without performing I/O.
Live retrieval, every M60/M90 port implementation, M60-B3 admission/lease/snapshot, parser/baseline/publication composition and all concrete-source effects remain unimplemented and separately gated.
M60 remains planned.
SRC-010, SRC-011 and SRC-012 remain planned; SRC-014 remains catalog-only/non-admitted.
No concrete USTC source is approved and no network path exists.
```

This is supporting executable evidence for the accepted contract. It is not full SSRF/retrieval acceptance because it proves no real resolver, connected socket, TLS peer, HTTP transport, cancellation/resource-stop, trusted clock, durable admission transaction or hostile production adapter.

## 2. Governing authority and bounded interpretation

Normative behavior comes from current protected-main carriers in this order:

1. `docs/contracts/source-retrieval.md` (`source-retrieval/v0`), especially §§2–3, 6.1, 6.3, 7–9, 11 and the exact phase signatures;
2. the immutable accepted R11 semantic packet in `docs/tasks/m60-b2-retrieval-policy-readiness-proposal.md`, especially §§5–10;
3. `docs/contracts/source-import.md` and the implemented `RetrievalSubject`/six-field source policy in `source-import/v1`;
4. `docs/plan/05-campus-trust-kernel.md`, `docs/plan/modules/70-campus-trust-source-pipeline.md`, module map and roadmap for ownership/status;
5. `docs/acceptance/matrix.tsv` and `docs/acceptance/platform-baseline.md` for non-promotion truth.

The accepted contract states that the first separately authorized retained B2 implementation contains only pure registry/policy and non-authority observation fakes and implements none of the four M60-owned ports. This packet therefore implements only types and functions reachable in the pure phase chain or needed to construct shape-only synthetic observations. It deliberately excludes B3 authority/effect carriers and every port.

`M60_B2_REPRESENTABILITY_CLARIFICATION_20260901` freezes the selected pre-implementation representation without changing effect authority: the five pure phase carriers are public opaque non-authority outputs solely so the accepted public phase signatures are Rust-representable; `BodyObservation::new` is fallible through transport-only `ObservationShapeRejected`; and §5.5 is the exact rate branch table. The current owning contract and its marker-external accepted-packet clarification must project this decision explicitly before any implementation claim.

Where a public input struct has private fields but the contract does not freeze an accessor, do not invent one. Tests exercise it through its accepted constructor and pure-policy behavior. Exact explicit accessors in the contract must exist; no additional public constructors, mutable accessors, conversions, builders, Serde or `Default` are admitted.

If exact behavior cannot be represented without a new type, variant, field, public accessor, constructor, policy rule, error precedence, authority owner or dependency, stop with `BLOCKED_CONTRACT`. Historical V10 text may explain provenance but cannot override the current R11/source-retrieval authority.

## 3. Exact writable paths

Only these repository paths may differ from the bound source tree:

```text
CLAUDE.md
crates/platform-core/src/lib.rs
crates/platform-core/src/source_registry.rs
crates/platform-core/src/source_retrieval.rs
crates/platform-core/tests/source_retrieval.rs
docs/acceptance/platform-baseline.md
docs/contracts/source-import.md
docs/contracts/source-retrieval.md
docs/overview/architecture.md
docs/plan/05-campus-trust-kernel.md
docs/plan/modules/00-module-map.md
docs/plan/modules/70-campus-trust-source-pipeline.md
docs/tasks/01-execution-roadmap.md
docs/tasks/m60-b1-v1-lifecycle.md
docs/tasks/m60-b2-offline-retrieval-policy.md
docs/tasks/m60-b2-retrieval-policy-readiness-proposal.md
scripts/check_repo_contracts.py
scripts/checker_test_inventory.json
scripts/tests/test_check_repo_contracts.py
```

The source phase should first write only these four executable carriers:

```text
crates/platform-core/src/lib.rs
crates/platform-core/src/source_registry.rs
crates/platform-core/src/source_retrieval.rs
crates/platform-core/tests/source_retrieval.rs
```

After their bytes pass focused Rust gates and freeze, a later projection/checker phase may write only the remaining listed paths while treating the four executable carriers as immutable input. This digest staircase prevents docs/checker work from silently rewriting the policy kernel.

No Cargo manifest, lockfile, adapter crate, application, workflow, raw fixture, source-revision code, product code or configuration may change. All changed files remain regular non-symlink `100644` entries.

## 4. Protected carriers and non-effects

These remain byte-identical:

```text
Cargo.toml
Cargo.lock
crates/platform-core/Cargo.toml
crates/adapters/Cargo.toml
crates/adapters/src/lib.rs
crates/platform-core/src/source_revision.rs
docs/acceptance/matrix.tsv
docs/contracts/module-boundaries.md
docs/coverage-matrix.md
.github/workflows/ci.yml
```

Also preserve byte-for-byte the accepted R11 block between `M60_B2_RETRIEVAL_POLICY_PROPOSAL:BEGIN/END` at `sha256:34cd911e6120646a0e2e410de9987efd167e519f43e5bf64a43c96d9c3654f1e` over `33046` bytes. Only marker-external mutable status in that taskbook may change. Preserve the M60-B1 semantic block as historical B1 evidence; only its marker-external current B2 status may change.

Forbidden effects and dependencies:

- no DNS query, socket, TLS, HTTP, filesystem, process, clock read, async runtime, thread, synchronization primitive or environment/proxy read;
- no `SourceTransportPort`, `SourceOperatorPolicyPort`, `RetrievalClockPort`, `RetrievalAdmissionPort` or `SourceFetchPort` declaration or implementation in the retained slice;
- no M90 production/fake adapter implementation and no dependency from `platform-core` to `adapters`;
- no `AdmittedRetrievalPlan`, `EffectReadyRetrievalPlan`, `RetrievalAdmissionOutcome`, authority evidence, start authorization, attempt receipt/completion, `TransportStopped`, `SourceFetchFailure` or `BoundedFetch` implementation;
- no `RetrievalTransportRequest`, because its only accepted construction is from the excluded B3 effect-ready carrier;
- no concrete source proposal/approval/status mutation, raw source bytes, credentials, cookies, auth, custom headers, parser, snapshot, revision, baseline, publication or product feed.

## 5. Executable Rust surface

Create one public module `source_retrieval` in `platform-core`; it may import existing M00 `CommandId` and B1 source-registry values but must not redefine or re-export them.

### 5.1 Required public non-authority values

Implement these exact current-slice public values with private fields and accepted traits. The already implemented public `SourceRetrievalProtocolVersion` and `PublicIpPolicyVersion` remain owned by `source_registry.rs`: `source_retrieval` imports and consumes them directly, and MUST NOT redefine or re-export them:

```text
RetrievalAttemptId
RateOverrideId
RetrievalOverrideEvidenceId
SourceOperatorId
RetrievalEpochSeconds
RetrievalDnsName
RetrievalRateOverrideRequest
RetrievalOverrideFacts
RetrievalAttemptCommand
RetrievalRateDecision
HttpVersionClass
RetrievalBodyFraming
ObservedHeaderValue
ResponseHeadObservation
DnsTransportObservation
BodyObservation
RetrievalTransportSuccess
RetrievalTransportSuccessParts
SourceTransportError
RetrievalPolicyError
RetrievalPolicy
SerializedRetrievalRequest
RetrievalPlanCandidate
ResolvedRetrievalCandidate
PeerBoundRetrievalCandidate
BodyAdmissionCandidate
ValidatedFetchCandidate
```

`RetrievalPlanCandidate`, `ResolvedRetrievalCandidate`, `PeerBoundRetrievalCandidate`, `BodyAdmissionCandidate` and `ValidatedFetchCandidate` are public nameable phase outputs only so the contract's public pure signatures are representable; they have private fields, no public constructor, no `Copy`, Serde, `Default`, `Display`, or authority-bearing conversion, and safe payload-redacted `Debug`. `RetrievalPlanCandidate` alone is `Clone`; the other four phase carriers are non-Clone and are consumed by the immediately following pure method. Their only construction path is the immediately preceding pure method.

Do not implement current-slice-unused B3-only identities/capability/state values (`SourceStartAuthorizationId`, `SourceRetrievalCapability`, `SourceStartCapabilities`, `RetrievalAttemptState`) or any owner-private B3/effect carrier. Their accepted contract remains future authority, not fake offline evidence.

### 5.2 Shared B1 helper closure

The sibling module currently needs exact B1 grammar/error/media bytes without widening the public `source-import/v1` API. The only allowed edits to `source_registry.rs` are crate-private reuse helpers:

- crate-private construction of `SourceValueError` from one static value-kind and existing `SourceValueErrorKind`;
- crate-private reuse of the exact SourceId-family grammar classifier;
- crate-private read-only access to canonical `SourceMediaType` bytes.

No existing public type, trait, constructor, accessor, error variant, lifecycle behavior or test may change. Prefer one source of validation truth over copied grammar.

### 5.3 Nominal values and commands

- Four current-slice identities use the exact B1 `1..=128` grammar and expose only checked `new(String)`, `as_str`, `into_inner`; traits `Clone + Debug + Eq + Ord + Hash`; no `Copy`, `Display`, `FromStr`, `TryFrom`, Serde or `Default`.
- `RetrievalEpochSeconds` is a private-field `u64` with total `from_unix_seconds`, `get`, and `Copy + Clone + Debug + Eq + Ord + Hash`.
- `RetrievalDnsName` enforces lowercase ASCII, no trailing dot, `3..=253` total bytes, at least two labels, labels `1..=63`, alphanumeric edges, `[a-z0-9-]` interiors; no Serde/Display and fixed redacted `Debug` that cannot echo host text.
- `RetrievalRateOverrideRequest::new` and `RetrievalAttemptCommand::new` are total after nominal inputs.
- `RetrievalOverrideFacts::new` accepts the exact eight accepted fields and rejects only `issued_at > not_after` as existing `SourceValueErrorKind::InvalidOverrideWindow`; it is synthetic non-authority data.
- These input structs are `Clone + Debug + Eq`, no Serde/Default/public fields. No public field accessors are added unless explicitly frozen by the accepted contract.

### 5.4 Pure derivation and exact wire request

`RetrievalPolicy::derive_candidate(&RetrievalSubject, &RetrievalAttemptCommand)`:

1. checks protocol version before source identity/revision;
2. checks source ID and then exact authority revision;
3. parses the already-canonical B1 URL into exact lowercase DNS host and path without accepting query, fragment, explicit port or caller URL;
4. produces immutable lineage and exact serializer bytes:

```text
GET <path> HTTP/1.1\r\n
Host: <host>\r\n
Accept: <expected-media-type>\r\n
Accept-Encoding: identity\r\n
Connection: close\r\n
\r\n
```

No User-Agent, retry, proxy, cookie, authorization, referer, request body, extra header or conversion exists. `SerializedRetrievalRequest::as_bytes()` is its only public accessor. The candidate is not effect authority.

### 5.5 Pure rate evaluation

`evaluate_rate` is deterministic synthetic policy evidence only; future B3 must reload facts and recompute it atomically.

- `now < last_attempt_started_at` is `ClockRegression`.
- With no prior attempt, or when checked elapsed seconds are at least `minimum_interval_seconds`, return `Allowed`; an unnecessary caller override request does not mint an override decision.
- When the interval has not elapsed and no override request exists, return `RateLimitNotElapsed`.
- When an override request exists, require `override_facts`; absence is `OverrideEvidenceUnavailable`.
- Facts must match request evidence/override IDs plus candidate attempt/source/revision; require `issued_at <= now <= not_after`; mismatch, future-issued or expired facts are `InvalidRateOverride`.
- After exact facts validate, `override_consumed=true` is `RateOverrideAlreadyConsumed`; otherwise return `AllowedWithOverride(exact RateOverrideId)`.
- No rate path changes source/host/global concurrency, grants authority or consumes evidence.

If the mandatory reviewer determines that any one of these ordering rules is not uniquely implied by current authority, repair the owning contract before implementation rather than treating this taskbook alone as new public semantics.

### 5.6 DNS and peer pure phases

`DnsTransportObservation::new` is shape-only and may return only `SourceTransportError::ObservationShapeRejected`. Enforce exact raw bounds but do not normalize, select a peer or apply policy in its constructor. Its exact explicit read-only/consuming accessors match the contract.

`authorize_resolution` consumes the candidate and raw observation. It:

- requires exact queried canonical host;
- enforces CNAME depth `0..=8`, no loop, and aliases consistent with the accepted exact-host rule;
- sorts/deduplicates raw A answers to `1..=16` values;
- rejects every address in all 15 frozen IPv4 CIDRs and all unsupported address-family shapes;
- preserves the numerically lowest admitted IPv4 as selected peer, port `443`.

`authorize_peer` consumes the resolved candidate and accepts only IPv4 port `443` equal to the selected admitted address; mismatch fails closed. It rechecks the closed public-IP policy version without creating any connection.

### 5.7 Strict response and body pure phases

Implement the exact parser grammar/caps, status/error classes, content-type parameter rules, encoding/transfer/framing/trailer/chunk rules, independent wire/body ceilings and total deadline from `source-retrieval/v0` §§8 and 11.

- `parse_strict_response_head` is the only response-head constructor.
- `authorize_response_head` consumes peer-bound plan + head and produces one body-admission phase output only after every applicable head/framing check passes.
- `BodyObservation::new(bytes: Vec<u8>, wire_bytes_after_headers: u64, chunk_count: u32, max_chunk_line_bytes: u16, saw_chunk_extension: bool, trailer_field_count: u16, framing_complete: bool, elapsed_milliseconds: u64) -> Result<BodyObservation, SourceTransportError>` is shape-only checked construction. It accepts and stores the scalar fields over their complete Rust type domains and imposes no shape restriction on either boolean. It accepts `bytes.len()` in `0..=1_048_577`, preserving the exact vector without truncation; the one-byte overflow sentinel makes the global `maximum_response_bytes=1_048_576` policy failure representable without permitting unbounded retention. A larger vector returns only `SourceTransportError::ObservationShapeRejected` and retains none of the rejected bytes. The constructor performs no request-specific wire/body/chunk/trailer/framing/deadline policy; `finish_body` alone applies those rules.
- `finish_body` consumes body admission + body observation and enforces wire, body and deadline independently in the accepted global error order.
- The final `ValidatedFetchCandidate` is still synthetic non-authority output; there is no `BoundedFetch` conversion.

### 5.8 Shape-only transport success

Implement `SourceTransportError` with exactly the accepted nine payload-free variants and `RetrievalTransportSuccess`/parts with the exact constructor, fields, accessors, bounds and consuming projection. Only `ObservationShapeRejected` is emitted by shape constructors. No policy error, receipt, phase authority or rejected payload is embedded. Large byte/DNS carriers are non-Clone and move once through `into_parts`.

Implement the complete accepted `RetrievalPolicyError` payload-free enum even though B3-only variants are unreachable in this bounded slice; adding or removing a variant would drift the already accepted public error algebra. Rendering must not expose raw host/header/body/evidence data.

## 6. Required Rust tests and compile-time closure

Create a dedicated integration suite plus private unit tests/rustdoc compile-fail blocks. It must cover at least these exact behavior families:

1. identity grammar, traits, no-echo errors and DNS redacted Debug;
2. epoch and override-window bounds;
3. command/facts construction without authority;
4. derive precedence for protocol/source/revision and exact immutable lineage;
5. exact serialized request bytes and absence of forbidden headers/knobs;
6. first/elapsed/rate-limited decisions, clock regression, missing/mismatched/expired/future/consumed override and exact success;
7. DNS constructor shape-only behavior;
8. every one of the 15 CIDR lower/upper/neighbor boundaries, duplicate sorting, answer cap, host/CNAME depth/loop/alias failures;
9. selected-peer equality, IPv4/port binding and policy-version behavior;
10. strict CRLF/status/header grammar and all parser caps;
11. HTTP version, 1xx, 3xx, other status precedence;
12. Content-Type essence/parameter grammar, duplicate/missing/mismatch cases;
13. content/transfer encoding, content-length, ambiguous framing, trailer and chunk caps;
14. body representation constructor limits;
15. independent request-specific wire/body/deadline failures and precedence;
16. complete happy-path pure phase chain and lineage preservation;
17. transport-success shape bounds/accessors/one-shot `into_parts`;
18. complete closed error enums and redacted payload-free formatting;
19. existing `source_registry` tests remain byte-behavior compatible;
20. rustdoc `compile_fail` evidence for private fields, absent Default/Serde/unchecked constructors, absent Clone on linear carriers/large observations, absent public candidate/phase construction, absent `BoundedFetch`/port/admission APIs, and no mutable byte/host/header access.

Internal unit tests must inspect exact private serializer bytes and any private lineage not intentionally exposed. Do not widen public access merely to make integration tests easier. Every intended test command must emit non-zero tests; zero-test success is failure.

## 7. Mandatory sabotage probes

Against temporary copies outside the final candidate, each mutation must make its named focused test fail, then exact candidate bytes must be restored before the next mutation:

1. swap derivation precedence so source mismatch hides protocol mismatch;
2. add `User-Agent` or omit `Accept-Encoding: identity` from exact wire bytes;
3. let an unelapsed request pass without exact override evidence;
4. accept an expired/mismatched or already-consumed override;
5. remove one frozen CIDR or admit one reserved boundary address;
6. accept a mismatched queried host/CNAME loop;
7. accept a connected peer not equal to the selected DNS address or not on port 443;
8. accept bare LF/obs-fold or a `3xx` redirect;
9. accept duplicate Content-Type, content-length plus chunked, trailer or oversized chunk metadata;
10. conflate wire/body ceilings or ignore the total deadline;
11. make `DnsTransportObservation` or `RetrievalTransportSuccess` apply domain policy instead of shape-only validation;
12. introduce a public constructor/Clone path for one linear phase carrier or `ValidatedFetchCandidate`.

Record exact mutation, command, non-zero exit, matching test and post-restore hashes. A textual checker rejecting the mutation does not substitute for the Rust behavior test biting.

## 8. Governance/checker and projection closure

Add a new fail-closed implementation checker without weakening existing B1/R11 checks or the fingerprint moratorium. Prefer compiler/test/inventory evidence over new broad Rust-body lexical fingerprints.

The always-run checker must verify:

- exact new module/source/test/task carrier presence and module declaration;
- unchanged Cargo/dependency/adapters/CI/acceptance/boundary carriers;
- exact current integration/private test inventory and non-zero owning command;
- mandatory doctest command remains executable in CI;
- R11 packet bytes/digest unchanged;
- every current-truth carrier distinguishes bounded offline policy evidence from live retrieval/ports/B3;
- M60 stays `planned`, matrix bytes/statuses remain unchanged, `SRC-010/011/012` remain planned, `SRC-014` remains catalog-only, concrete source remains Proposed;
- no `SourceTransportPort` or four M60-owned port implementation, `BoundedFetch`, effect-ready/admission carrier, adapter/runtime/network dependency or forbidden path appears;
- checker mutation tests independently fail on false promotion, stale “no B2 implementation” projection, R11 drift, module/test omission, port/network widening and acceptance-status drift.

Update `checker_test_inventory.json` honestly for newly added checker tests; exact inventory/count drift remains fail closed. Do not reuse unrelated test IDs merely to avoid changing the inventory.

Before implementation, amend the current owning `docs/contracts/source-retrieval.md` representation text exactly enough to: (a) classify the five phase carriers as public opaque non-authority outputs with private fields/no public constructors, safe payload-redacted `Debug`, no `Copy`/Serde/`Default`/`Display`, `RetrievalPlanCandidate: Clone`, and the other four phase carriers non-Clone; (b) give the exact typed `BodyObservation::new` signature and representation contract from §5.7—including full scalar domains, unconstrained booleans, exact preserved `bytes.len() <= 1_048_577`, no truncation, and `ObservationShapeRejected` as its only constructor rejection; and (c) install §5.5 as the exact pure-rate branch/precedence table. Add one marker-external representability-clarification receipt to the accepted-contract taskbook naming Develata's 2026-09-01 selection, while preserving every byte inside `M60_B2_RETRIEVAL_POLICY_PROPOSAL`. These are contract clarifications before implementation, not acceptance promotion.

Truthful projections must say “bounded offline pure policy implemented” while preserving “live retrieval and B3+ unimplemented” in:

- `source-retrieval.md` implementation/non-claim metadata;
- `source-import.md` B2 boundary references;
- Campus Trust plan/blueprint, module map, overview and roadmap M60 rows;
- platform baseline current projection;
- B1 taskbook marker-external current B2 status only;
- M60-B2 accepted-contract taskbook marker-external mutable state only;
- concise `CLAUDE.md` crate/current-evidence guidance;
- new `docs/tasks/m60-b2-offline-retrieval-policy.md`, carrying the exact bounded implementation/non-effects/gate truth.

Do not edit `matrix.tsv`, module-boundary status, coverage tokens or acceptance statuses: the M60→M90 port remains unimplemented and `SRC-010` still requires a real hostile transport/integration binding.

## 9. Verification and parent-owned gates

Local Hermes disk is below the 10GB Rust threshold; do not run Cargo locally. Run authoritative Rust work on an isolated remote root with a run-owned target directory.

Worker/source-phase gates:

```text
cargo fmt --all -- --check
cargo test --locked -p ustc-campus-agent-core --test source_registry
cargo test --locked -p ustc-campus-agent-core --test source_retrieval
cargo test --locked -p ustc-campus-agent-core --lib source_retrieval::tests
cargo test --locked -p ustc-campus-agent-core --doc source_retrieval
cargo clippy --locked -p ustc-campus-agent-core --all-targets --all-features -- -D warnings
git diff --check
```

Parent authoritative exact-candidate gates after projection/checker import:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features
cargo test --locked --workspace --all-features --doc
python3 scripts/check_repo_contracts.py --ci
python3 scripts/run_checker_shards.py --jobs 4 --timeout-seconds 1800 --inventory scripts/checker_test_inventory.json --evidence-dir <run-owned> --require-clean --require-runner-image-identity
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
git diff --check
```

Hermes also owns exact changed-path union, staged/untracked classification, modes/symlinks, protected hashes, source→index→commit byte bridge, sabotage custody and formal POST_EDIT_CANDIDATE review. Worker exit zero is not parent acceptance.

The worker terminal receipt is exactly one of:

```text
IMPLEMENTATION_COMPLETE_AWAITING_PARENT_GATES
BLOCKED_CONTRACT
BLOCKED_VERIFICATION
BLOCKED_SCOPE
```

It must include source commit/tree, taskbook digest, final HEAD/status, exact changed paths and modes, every command/exit/non-zero test count, sabotage outcomes, non-claims and skipped parent gates. The worker must not commit, configure a remote, push, open a PR, merge or perform source/network effects.

## 10. Stop conditions

Stop before widening scope if:

- protected main differs from the bound commit/tree before candidate creation;
- the taskbook digest or R11 semantic packet drifts;
- any path outside §3 is required or any protected carrier changes;
- a Cargo dependency/manifest/lockfile, adapter, workflow or real fixture appears necessary;
- a current-contract ambiguity requires a new public semantic or precedence rule;
- an implementation needs any port, effect/admission carrier, trusted clock, persistence or network I/O;
- a concrete source would cease to be Proposed;
- M60 or an acceptance row would be promoted;
- a required gate cannot run, emits zero intended tests, or sabotage does not bite;
- the remote worker cannot prove exact run ownership, no residual process and byte/mode scope.
<!-- M60_B2_OFFLINE_IMPLEMENTATION_PACKET:END -->

## Marker-external full-gate proof-carrier repair

- `Repair`: `M60_B2_PLATFORM_IDENTITY_MODULE_INVENTORY_20260901`
- `Trigger`: the first authoritative workspace test run correctly rejected the new `pub mod source_retrieval;` declaration because `crates/platform-core/tests/platform_identity.rs` still pinned the exact pre-B2 `platform-core` module/item inventory.
- `Exact scope addition`: `crates/platform-core/tests/platform_identity.rs`, regular non-symlink `100644`, limited to admitting `source_retrieval` in the existing exact `lib.rs` module and item tables.
- `Authority`: Develata's 2026-09-01 instruction to continue the accepted M60-B2 plan toward MVP; this is a necessary fail-closed proof-carrier repair, not a new product semantic.
- `Frozen source staircase`: the four §3 executable carriers remain byte-identical to the source-phase receipt; this repair changes no production code, manifest, dependency, adapter, acceptance row or protected transport carrier.
- `Non-claims`: no source approval, network/transport effect, B3 admission, M60 status promotion or remote shipping authority.

## Marker-external formal-review repair R1

- `Review`: DeepSeek reviewer label `m60-b2-formal-review-c46030a3` returned `VERDICT: BLOCKED`; result `sha256:7e0c196cfffcd33ebf592b87e2ae6b1865bd52067cd97de333610fa1156fd101` over `9779` bytes. The wrapper did not parse the prefixed verdict, but the finding text is retained as formal review evidence rather than discarded.
- `Blocker`: the accepted exact serialized request bytes and absence of `User-Agent` had no direct Rust behavioral assertion, so taskbook sabotage probe 2 could not bite independently of co-mutable checker digests.
- `Correctness repair`: add one private unit test that derives the candidate and compares the complete request bytes; reorder `finish_body` so `AmbiguousFraming` precedes chunk/trailer failures as required by the frozen global error order; add combined-precedence, parser-cap, chunk-metadata and DNS-alias/depth boundary assertions to the existing focused tests.
- `Exact repair carriers`: `crates/platform-core/src/source_retrieval.rs`, `crates/platform-core/tests/source_retrieval.rs`, this marker-external receipt and the existing checker/hash/test-inventory proof carriers; no manifest, dependency, adapter, port, acceptance row or protected transport carrier.
- `Repaired source hashes`: `source_retrieval.rs sha256:06a2652fa1f26ccfbb144aebbb0d75319766af85c3abe8df2b034be06337e58e`; `tests/source_retrieval.rs sha256:d99dea0fd7da81351c5debc20012d500313e58778b95d4934cac013eefef5dcf`.
- `Required closure`: rerun focused/full Rust gates, execute taskbook sabotage probe 2 with a controller-owned mutation/restore receipt, rerun the complete checker shard suite, and obtain a fresh exact-candidate formal verdict.
- `Non-claims`: this repair neither authorizes effects nor changes M60/SRC projection truth or shipping authority.

## Marker-external formal-review repair R2

- `Review`: DeepSeek reviewer label `m60-b2-formal-review-99af9d9b` returned `VERDICT: PASS`; result `sha256:3f05b458e5ac5b93d95bfb1490fb8db14dbb7b4df31d8a331964e520d85f2a31` over `5489` bytes. The wrapper again did not parse the prefixed verdict, but the explicit review result is retained.
- `Should-fix accepted`: the reviewer found one combined response-head precedence case where `Transfer-Encoding: gzip` plus `Content-Length` returned `AmbiguousFraming` even though frozen error order places `UnsupportedTransferCoding` first.
- `Repair`: validate a single transfer-coding value before repeated/coexisting framing checks, retain `chunked + Content-Length -> AmbiguousFraming`, and add the exact `gzip + Content-Length -> UnsupportedTransferCoding` focused case.
- `Repaired source hashes`: `source_retrieval.rs sha256:131fb476fc04b3cc900946a8153dc79d3d701cb107254b8e80fe6bb2cdf9f1e7`; `tests/source_retrieval.rs sha256:91eccecc7a40da1e608ea4c78fc3fc02657621bf9881ee84c82543a006cfffe8`.
- `Required closure`: rerun focused/full Rust gates, complete checker shards and obtain one final exact-candidate formal verdict; the wire-sabotage behavioral test remains retained and must continue to pass.
- `Non-claims`: no API/effect/authority/projection/shipping widening.

## Marker-external pull-request review repair R3

- `Review`: GitHub Codex review `5076822986` on exact head `ad342163c21a5903f8f01ebd42527847d5e7e5de` produced two P2 correctness findings: discussion `3903119091` rejected unquoted `Content-Type` parameter spaces, and discussion `3903119098` identified the missing `1..=16` chunk-size digit enforcement.
- `Contract-preserving repair`: unquoted parameter values now accept RFC `tchar` bytes only while quoted values may additionally contain ASCII spaces; focused tests reject `charset=utf 8` and admit `charset="utf 8"`.
- `Representability closure`: `max_chunk_line_bytes` is the pre-`CRLF` width. Once chunk extensions are rejected, that width is exactly the hexadecimal digit count; policy therefore rejects `0` and `> 16` and tests both failing edges plus width `16` success without adding a public field or widening the accepted phase API.
- `Exact repair carriers`: `crates/platform-core/src/source_retrieval.rs`, `crates/platform-core/tests/source_retrieval.rs`, `docs/contracts/source-retrieval.md`, this marker-external receipt, and the existing checker/hash proof carrier.
- `Repaired source hashes`: `source_retrieval.rs sha256:a53777861a15e4099674b48902383284cbf30df9f077dabecde18053a39db85f`; `tests/source_retrieval.rs sha256:8c287f29399f54818e569acb1373e56ead06d84c2cbac6e1d51254918cdc63a9`.
- `Required closure`: rerun focused/full Rust gates, complete checker shards, obtain a fresh exact-head formal review and resolve both GitHub review conversations before merge authorization is requested.
- `Non-claims`: no public API, dependency, transport/effect, source approval, acceptance projection or shipping widening.

## Review receipts

- `Pre-edit review`: `PASS`; taskbook `sha256:d509ef1fb95606bca0b6cc68baf810ec2a18d9471b548c8725751e5f495ad9c0`; marker packet `sha256:19fb0e7696ffd298e34da0c52507f3b186fa50d9ee9ccc4b68657ec65cb1026e` over `26003` bytes.
- `Source phase`: `PASS`; controller-owned receipt `sha256:08270e618a32fc3f433971381c7fa9c01868ae84ec0e6b5188d08c6e91dfcaf9`; `17` source-registry integration tests, `12` source-retrieval integration tests, `2` private unit tests and `3` compile-fail doctests passed with zero failures, plus `cargo check`, rustfmt and focused clippy under the exact-base remote lane.
- `Formal repair R1 source gates`: `PASS`; controller-owned receipt `sha256:e9a7650111c0832e3583c56a12c69f4d0925000dc8554c5278c8814a1edc68ae`; baseline focused gates passed with `17` source-registry integration tests, `12` source-retrieval integration tests, `3` private unit tests and `3` compile-fail doctests; exact-wire sabotage changed the source to `sha256:85d4f1cd7b8c5c67c8cc8913afd4c3948106e492141652aa1368494b7b300e55`, the named unit test bit with exit `101`, restoration returned the exact repaired source hash, and the post-restore named test passed.
- `Formal repair R2 source gates`: `PASS`; controller-owned receipt `sha256:46fd36c88617c925739a80605fe291320b3cb33d0355b00db594da9c0e183b69`; focused format/check/tests/doc/clippy remained green after the transfer-coding precedence repair, exact-wire sabotage again bit with exit `101`, and post-restore source/test hashes matched `131fb476fc04b3cc900946a8153dc79d3d701cb107254b8e80fe6bb2cdf9f1e7` / `91eccecc7a40da1e608ea4c78fc3fc02657621bf9881ee84c82543a006cfffe8`.
- `Pull-request repair R3 source gates`: `PASS`; controller-owned receipt `sha256:0ac5d44baef367c61075f11796f3ad37763ecb5a549819ae52860a6286e25724`; focused format/check/tests/doc/clippy remained green after both Codex P2 repairs, exact-wire sabotage bit with exit `101`, and post-restore source/test hashes matched `a53777861a15e4099674b48902383284cbf30df9f077dabecde18053a39db85f` / `8c287f29399f54818e569acb1373e56ead06d84c2cbac6e1d51254918cdc63a9`.
- `R3 DeepSeek delta review`: `PASS` with no blocker; result `sha256:9d62d023c16d35d2b12430d4c652eb6a6f38a32b4a9c423d6109cde7a6fc665c` over the exact `f0ea21447a8c7375b66ed45d6f2c085a1fd0bd7f` packet. Its single `SHOULD_FIX` requested an explicit width-`1` success boundary; the focused test now admits both widths `1` and `16`, with repaired test hash `sha256:fdec042855125f6713762b1379f471250f59ca317653bc0b6ce539aa30a0acf3`. A fresh controller-owned focused-gate receipt `sha256:89693e240391594f9e5005cf43f52bd01cd29a62db4efb1793cb75c6ac02cab2` confirms format/check/tests/doc/clippy and exact-wire sabotage remain green after this test-only hardening. This marker-external repair changes no production source, public API, authority or effect boundary.
- `Exact candidate`: pending authoritative full gates, checker sabotage and formal review.
