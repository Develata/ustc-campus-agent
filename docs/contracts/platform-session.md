# Platform session-domain contract

## Metadata

- `Status`: Accepted `M00-B2 session-domain` contract; no implementation exists yet
- `Version`: `platform-session/v0`
- `Last Review`: `2026-07-28`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Depends On`: implemented [`platform-identity/v0`](platform-identity.md) values and [`module-boundaries.md`](module-boundaries.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for authority partition
- `Acceptance`: active `planned` `AUTH-017`, `AUTH-018`, `AUTH-019`, `AUTH-020`; none is `implemented`, and no session implementation evidence exists
- `Primary Code`: `crates/platform-core/src/session.rs`, with evidence in `crates/platform-core/tests/platform_session.rs`; neither file exists yet

## 1. Scope and authority

`platform-session/v0` specifies the pure, replayable lifecycle kernel for one platform session. It owns:

- immutable tenant/user/session scope after open;
- resolved idle and hard-expiry deadlines;
- legal open, refresh, expire and revoke transitions;
- expected-revision event ordering;
- deterministic decision, evolution and replay;
- fail-closed handling of stale revision, stale time, terminal mutation and forged event payloads;
- credential-evidence provenance without raw credential retention.

It does not authenticate a credential, parse a cookie or token, call a clock, persist an event, generate an identifier, build an admitted request context, assign a role, decide a downstream permission or emit the final cross-module `PlatformControlEvent`/`PlatformControlError` projection.

Authentication adapters produce bounded `SessionCredentialEvidence`; `M00-B4 session-port` later supplies clock/repository/secret-reference interfaces and deterministic fakes; `M00-B4 control-evidence` later owns stable external event/error/redaction projections; `M00-B3 request-context` later owns admitted actor/request/command/causation semantics. A caller cannot reinterpret this pure kernel as proof that credential authentication or durable persistence already happened.

## 2. Required public semantic values

The implementation must expose nominal, private-field Rust values with exactly the names, fields and invariants below. The names are frozen by acceptance of this contract; changing one is a `platform-session/v0` change under §16, not an implementation detail. No field or invariant may be silently dropped.

### 2.0 Binding to the merged `platform-identity/v0` implementation

`M00-B1` is merged, so the following are read back from the implementation rather than proposed.

**Placement.** The session domain is one cohesive module at `crates/platform-core/src/session.rs`, declared `pub mod session;` in `crates/platform-core/src/lib.rs`, with evidence at `crates/platform-core/tests/platform_session.rs`. It is a sibling of `identity` and `invocation`, not a submodule of either, and it creates no new crate: `module-work-policy/v1.3` §5 prefers a small internal module until a compiler-enforced dependency, independent release or a second real consumer exists, and none does.

**Identity values are imported, never redefined and never re-exported.** The session module binds exactly:

```rust
use crate::identity::{SessionId, TenantId, UserId};
```

spelled without renaming. `platform-identity/v0` §4 requires that every governed source's `use`/`type`/`mod` items, `impl` self types, macro definitions and macro invocations be accounted for against an exact allowlist; the session module joins that governed set on the same terms and gains no exemption. It publishes **no** `pub use` of any identity kind: `invocation.rs`'s §6 compatibility re-export is the only admitted second path to an admitted kind, and B2 does not receive one. B2 therefore adds no externally reachable API to any `platform-identity/v0` kind — no inherent method, no trait implementation, no alias, no second path.

**B2 mints no seventh identity kind, and structurally cannot.** The `identity_value!` generator is private to `identity.rs`, so it is unreachable from a sibling module. `AuthAdapterId` and `CredentialEvidenceDigest` are `platform-session/v0` values defined by this contract, not `platform-identity/v0` values, and they do not widen, alias or re-spell one.

**Representation is part of the rule, inherited from `platform-identity/v0` §4.** Every B2 value carrying a validated invariant is a **named-field struct with private fields**, never a tuple struct. A tuple struct's constructor is a *value*, not only syntax: `let ctor = AuthAdapterId; ctor(text)` fills the private field while writing no construction expression a scan can find, and that was demonstrated against real evidence during B1. A named-field struct has no constructor function item, so a struct literal — syntax, which cannot be bound, aliased, passed or returned — is the only way to produce one.

**One checked constructor per value.** Each value has exactly one inherent validating constructor and no public unchecked path, no `Default`, no `Deref`, no mutable backing access and no cross-kind conversion. String-backed values use `parse(value: impl Into<String>) -> Result<Self, SessionValueError>`, matching `platform-identity/v0` §4. Integer-backed and aggregate values use the exact constructor named with them below.

**Serde delegates rather than re-implements.** No B2 value implements a hand-written `Visitor`, a `visit_*` method or `deserialize_any`. Each `Deserialize` deserializes the canonical primitive once through that primitive's own implementation and hands the result to the same checked constructor, so whichever entry point a deserializer chooses there is exactly one construction path. Aggregate structs carry `#[serde(deny_unknown_fields)]`; enums carry `#[serde(rename_all = "snake_case")]`.

**An aggregate deserializes through its constructor, not field by field.** `deny_unknown_fields` plus per-field delegation validates every field and still admits a struct that no constructor would have built, because a cross-field invariant belongs to no field. Any aggregate carrying such an invariant therefore decodes a private shadow struct and hands it to the named constructor; the derived field-by-field decode is insufficient and is forbidden for those values. This is load-bearing rather than stylistic: §9.2 removes two error variants on the argument that a malformed evidence or policy cannot reach `decide` or `evolve`, and that argument is only true if every deserialization path routes through the constructor that enforces the invariant.

**Errors follow the merged B1 shape.** `SessionValueError` mirrors `IdentityValueError`: a small `Copy` value naming the Rust value kind that rejected the input plus one `SessionValueErrorKind`, with no `source`, no rejected-input field and no input-derived rendering. §9 names the decision/evolution taxonomy separately.

This section states what B2 inherits. It does not restate B1's construction-site, function-body, sweep and lexer closure; §11.2 states exactly how much of that apparatus B2 carries and why the difference is deliberate.

### 2.1 Time and duration

`SessionInstant` is a non-negative `u64` count of Unix-epoch milliseconds. The domain never reads wall-clock time itself; a command carries one adapter-observed instant. Every addition is checked and overflow fails before an event is produced.

Every `u64` denotes an instant, so its constructor is total and is honestly declared so rather than returning a `Result` that cannot fail:

```text
SessionInstant::from_unix_millis(millis: u64) -> Self
SessionInstant::as_unix_millis(&self) -> u64
```

Totality here is a statement about representation, not about admissibility: §3 still rejects an instant that is stale, non-monotone or arithmetically out of range for the transition being decided.

`SessionDuration` is a non-zero `u64` millisecond duration. Session-policy/config loading owns deployment ceilings; the pure kernel still rejects zero and arithmetic overflow. A duration is not a retry count, token lifetime or UI timeout.

```text
SessionDuration::from_millis(millis: u64) -> Result<Self, SessionValueError>
SessionDuration::as_millis(&self) -> u64
```

Time is compared only as an integer ordering. No timezone, locale, leap-second or formatted timestamp enters the state machine.

Both values serialize as one `u64`. `SessionDuration` deserialization delegates to `from_millis`, so a zero cannot arrive through Serde; `SessionInstant` deserialization delegates to `from_unix_millis`, which admits every `u64` by construction.

### 2.2 Adapter and credential-evidence provenance

`AuthAdapterId` is a bounded opaque adapter identity. The spelling carries no trust level, provider semantics or authorization result. Its accepted UTF-8 bytes are:

```regex
^[A-Za-z0-9](?:[-A-Za-z0-9._:]{0,126}[A-Za-z0-9])?$
```

Normative consequences, each an anchored line the implementation is cross-checked against:

1. encoded length is `1..=128` bytes;
2. the first and last byte are ASCII alphanumeric;
3. interior bytes are ASCII alphanumeric or one of `.`, `_`, `:`, `-`;
4. whitespace, control characters, non-ASCII text and every other punctuation byte are rejected;
5. case is significant;
6. no trimming, Unicode normalization, case folding, delimiter rewriting or alternate spelling occurs.

**That grammar is owned by this contract, not borrowed from `platform-identity/v0`.** It is deliberately byte-identical to `platform-identity/v0` §3 so that an operator reads one identifier shape across `M00`, but the agreement is a recorded design choice, not a derivation: neither document is authority for the other, neither is obliged to follow the other if one changes, and each is bound to its own implementation by its own carriers. `platform-identity/v0` §4 is explicit that agreement among mutable carriers is not evidence; a claim here that B2 "uses B1's grammar" would be exactly such an unbound claim, because nothing would compare them. Stating the bytes here instead gives B2's grammar a root of its own.

Consequently `AuthAdapterId` is not one of the six `platform-identity/v0` kinds, does not widen them, and must not be converted to or from one. An operator who wants the two grammars to stay equal changes both documents, and the change is visible as two changes.

`CredentialEvidenceDigest` is exactly lowercase `sha256:` followed by 64 lowercase hexadecimal digits:

```regex
^sha256:[0-9a-f]{64}$
```

Uppercase hexadecimal, a bare 64-character digest with no prefix, another algorithm prefix and any other length are rejected; there is no normalization, lower-casing or prefix-insertion path. It fingerprints already-admitted evidence for replay/audit correlation; it is not a credential, bearer token, password hash, refresh token or authorization result, and B2 never computes one — see §11.

**The producer carries an obligation B2 cannot check, so it is stated normatively rather than assumed.** A `CredentialEvidenceDigest` MUST be a domain-separated, non-invertible fingerprint over adapter-side material that is not raw credential text. Computing it directly over a password, cookie, bearer token or other secret is forbidden. The reason is concrete: this value is pinned into an immutable event, retained in a serializable snapshot and preserved across replay, so a digest taken over a low-entropy campus password would be an unsalted, offline-attackable hash embedded permanently in audit evidence — which catalog `AUTH-010` exists to prevent. B2 receives the value and validates its shape only; it has no way to observe how it was produced. The obligation is therefore inherited by the authentication-adapter contract named in §14, and this paragraph is what that contract must satisfy.

`SessionCredentialEvidence` contains exactly:

```text
tenant_id:             TenantId
user_id:               UserId
auth_adapter_id:       AuthAdapterId
evidence_digest:       CredentialEvidenceDigest
authenticated_at:      SessionInstant
credential_not_after:  Option<SessionInstant>
```

Its single checked constructor is:

```text
SessionCredentialEvidence::new(
    tenant_id, user_id, auth_adapter_id, evidence_digest,
    authenticated_at, credential_not_after,
) -> Result<Self, SessionValueError>
```

`Deserialize` decodes a private shadow struct and hands it to that constructor, per §2.0, because the temporal invariant below spans two fields and so belongs to none of them.

Invariants:

- all fields are private and construction/Serde is validating;
- unknown or missing fields fail closed;
- `credential_not_after`, when present, is strictly later than `authenticated_at`;
- no raw credential, token, cookie, provider subject, email, username, display name, role, secret reference value or arbitrary adapter payload is retained;
- the digest and adapter ID may appear in internal replay evidence, but no validation, decision, evolution or serialization error ever echoes rejected source text or secret-derived material;
- a platform `UserId` is not inferred from provider-subject text by this kernel.

The pure domain verifies the shape and temporal consistency of this value. It does not verify the external credential represented by it.

Structural validity is not authentication. A syntactically valid or successfully deserialized `SessionCredentialEvidence` is only a claim from a trusted M00 authentication-adapter/application boundary; it is never sufficient evidence at an untrusted M10/transport boundary. B2 exposes no raw-credential-to-evidence conversion and does not hash credential text. Final composition must prevent untrusted callers from invoking `OpenSession` directly with self-asserted evidence.

`TenantId`, `UserId` and `SessionId` are the exact canonical types and 1–128-byte ASCII values defined by `platform-identity/v0`; B2 neither wraps them again nor widens their grammar.

### 2.3 Resolved session policy

`SessionPolicy` contains exactly:

```text
idle_timeout:      SessionDuration
absolute_timeout:  SessionDuration
```

Its constructor is `SessionPolicy::new(idle_timeout, absolute_timeout) -> Self`. It is total: both fields are already non-zero by type and the two durations carry no relation to each other, so there is nothing left to reject.

The values are resolved and pinned when the session opens. Refresh never reloads policy and never changes either duration. A later platform-policy version affects new sessions unless a separately accepted migration contract says otherwise.

Deliberately, `idle_timeout` may equal or exceed `absolute_timeout`. Such a policy is well-formed and simply means the idle candidate never binds, so every refresh returns `NoEffectiveRefresh`. The same is true of any session whose effective deadline has already reached its absolute or credential cap. `NoEffectiveRefresh` is therefore an ordinary steady state, not a liveness failure, and a consumer must not treat it as one.

### 2.4 Session status and snapshot

`SessionStatus` is exactly one of:

```text
Active
Expired {
    expired_at: SessionInstant,
    observed_at: SessionInstant,
    cause: SessionExpiryCause,
}
Revoked { revoked_at: SessionInstant }
```

`SessionExpiryCause` is exactly:

```text
Credential
Absolute
Idle
```

`SessionSnapshot` contains exactly:

```text
session_id:             SessionId
tenant_id:              TenantId
user_id:                UserId
auth_adapter_id:        AuthAdapterId
evidence_digest:        CredentialEvidenceDigest
authenticated_at:       SessionInstant
credential_not_after:   Option<SessionInstant>
opened_at:              SessionInstant
last_transition_at:     SessionInstant
idle_timeout:           SessionDuration
absolute_timeout:       SessionDuration
effective_expires_at:   SessionInstant
absolute_expires_at:    SessionInstant
status:                 SessionStatus
revision:               u64
```

All fields are private and read-only. Tenant, user, session, adapter, evidence digest, authentication time, credential deadline, policy durations and policy-absolute expiry are immutable after open. `revision` is the last applied event sequence and starts at `1` after `SessionOpened`.

Adding, removing or reinterpreting a persisted/public snapshot field requires a versioned contract and acceptance update; implementations cannot append convenience or framework state to this snapshot.

**`effective_expires_at` is not a validity predicate, and the snapshot exposes one so that no consumer has to invent it.** A revoked session keeps its `effective_expires_at` unchanged — §8 preserves it — so that field alone still reads as "not yet expired" while `status` says `Revoked`. The obvious consumer test `observed_at < effective_expires_at` is therefore correct for `Active`, correct for `Expired`, and **wrong exactly for `Revoked`**: it fails open on the logout/revocation path, which is the security-critical one catalog `AUTH-008` names. That is the worst shape of defect, because it passes casual testing.

The one sanctioned validity question is therefore part of this contract:

```text
SessionSnapshot::admits_at(&self, observed_at: SessionInstant) -> bool
```

It returns `true` only when `status` is `Active` **and** `observed_at < effective_expires_at`. `M00-B3`'s request-context admission is the intended consumer; it asks this and does not recompute.

The internal evidence digest is not part of a client-safe projection by default. B3/B4 define later projections explicitly rather than serializing the whole domain snapshot across module boundaries.

## 3. Deadline algebra

For an admitted open command observed at `opened_at`:

```text
absolute_expires_at = checked_add(opened_at, absolute_timeout)
idle_candidate = checked_add(opened_at, idle_timeout)
effective_expires_at = min(
    idle_candidate,
    absolute_expires_at,
    credential_not_after when present,
)
```

Open fails if:

- `opened_at < authenticated_at`;
- `credential_not_after` is present and `opened_at >= credential_not_after`, meaning credential evidence is already expired;
- any checked addition overflows;
- the resulting absolute or idle deadline is not strictly later than `opened_at`.

The effective expiry deadline is exactly `effective_expires_at`; its open/refresh derivation already takes the minimum of the idle candidate, policy-absolute deadline and optional credential deadline. No second expiry calculation exists.

A session is expired at `observed_at >= effective_expires_at`; equality is expired, not a grace interval.

Expiry cause is selected from the deadline that equals `effective_expires_at`, with deterministic tie precedence:

1. `Credential` when `credential_not_after` is present and equals the effective deadline;
2. otherwise `Absolute` when `absolute_expires_at` equals the effective deadline;
3. otherwise `Idle`.

Thus ties use `Credential > Absolute > Idle`. A session whose idle and policy-absolute deadlines are equal expires as `Absolute`; if credential `not_after` shares that deadline, it expires as `Credential`. This classification does not change authority: every cause blocks all new request contexts.

A refresh observed before expiry computes:

```text
candidate = checked_add(observed_at, idle_timeout)
new_effective_expires_at = min(
    candidate,
    absolute_expires_at,
    credential_not_after when present,
)
```

Refresh never extends `absolute_expires_at` or `credential_not_after`, changes tenant/user/evidence/policy, or replaces credential evidence. If `new_effective_expires_at <= current_effective_expires_at`, the command is an explicit `NoEffectiveRefresh` failure and produces no event.

Credential rotation, replacement evidence, multi-device coordination and session migration are later contracts, not hidden refresh behavior.

## 4. Commands

The semantic command set is exactly:

```text
OpenSession {
    session_id,
    credential_evidence,
    policy,
    observed_at,
    expected_revision,
}

RefreshSession {
    session_id,
    observed_at,
    expected_revision,
}

ExpireSession {
    session_id,
    observed_at,
    expected_revision,
}

RevokeSession {
    session_id,
    observed_at,
    expected_revision,
}
```

Command values have private fields and one checked constructor each — `OpenSession::new(...)`, `RefreshSession::new(...)`, `ExpireSession::new(...)`, `RevokeSession::new(...)` — returning `Result<Self, SessionValueError>`. No command carries a raw credential, arbitrary metadata map, downstream permission, UI state, database handle or framework session object.

**Commands implement `Serialize` but not `Deserialize`.** §10 admits Serde only where command handling and event *replay* need it, and replay reads events, never commands. Making that explicit converts a rule someone must remember into one the compiler enforces: §2.2 requires that untrusted callers cannot invoke `OpenSession` with self-asserted evidence, and with no `Deserialize` there is no way to decode one from a transport payload at all. A future ingress maps its own validated DTO through these constructors, which is where the admission decision belongs.

`expected_revision` is optimistic-concurrency intent. B2 validates it during pure decision; B4 later binds the same value to journal/repository compare-and-append. B2 does not claim that an in-memory decision alone persisted anything.

The command set deliberately has no generic `SetState`, `Touch`, `Patch`, `Restore`, `Unexpire` or `Unrevoke` operation.

`ExpireSession` is an M00-internal lifecycle command issued only through the future B4 session application/port path. It is not an inbound cross-module command, is never decoded directly from M10, and does not expand the M00 blueprint's public input list.

For an accepted open, `SessionOpened.opened_at` is exactly `OpenSession.observed_at`; the evolved snapshot initializes `last_transition_at` to that same instant.

## 5. Events

The immutable semantic event set is exactly:

```text
SessionOpened {
    sequence,
    session_id,
    credential_evidence,
    policy,
    opened_at,
}

SessionRefreshed {
    sequence,
    session_id,
    observed_at,
    effective_expires_at,
}

SessionExpired {
    sequence,
    session_id,
    observed_at,
    expired_at,
    cause,
}

SessionRevoked {
    sequence,
    session_id,
    observed_at,
}
```

Every event sequence is the exact next positive integer. Event fields are private. External callers cannot construct an accepted snapshot by setting fields directly.

Events implement both `Serialize` and `Deserialize`, because replay reads them back. Each has one checked constructor, and `SessionExpired`'s is the only one with a cross-field invariant — `observed_at >= expired_at` — so it is the one event that deserializes through a shadow struct per §2.0.

A `sequence` is **not** validated at construction. Whether a sequence is the exact next integer is a question about the aggregate, not about the event, and §8 answers it with `EventSequenceMismatch`; there is deliberately no `SessionValueErrorKind` variant for a zero or out-of-order sequence, because a value-level check could only ever repeat what the aggregate must decide anyway.

Events retain only bounded provenance. They never contain raw credentials, secret values, cookies, authorization headers, provider payloads, arbitrary reason strings or client-supplied metadata.

`SessionRefreshed.effective_expires_at` plus `SessionExpired.expired_at` and `cause` are redundant verification fields: evolution recomputes them from prior state and rejects a mismatch. `expired_at` is the effective deadline at which the session became invalid, while `observed_at` is when expiry was detected and persisted; a late observation must not rewrite historical validity. Their presence makes persisted evidence inspectable without allowing a forged derived value to become authority.

Final stable platform-facing event/error names and reason codes remain owned by B4 `control-evidence`; these domain events are the state-machine authority that such projections must map one-to-one.

## 6. Transition table

Decision dispatch first resolves the aggregate by `session_id`. If no aggregate exists, §6.1 applies. On an existing aggregate, `OpenSession` uses the dedicated open precedence in §7 and returns `SessionAlreadyExists`; `RefreshSession`, `ExpireSession` and `RevokeSession` use the existing-aggregate precedence in §7, with §§6.2–6.3 defining status-specific legality after earlier identity/revision/terminal/time checks.

### 6.1 Empty aggregate

| Current | Command | Result |
|---|---|---|
| no session | valid `OpenSession`, expected revision `0` | `SessionOpened(sequence=1)` |
| no session | otherwise-valid `OpenSession`, expected revision not `0` | `RevisionMismatch { expected: command value, actual: 0 }` |
| no session | `RefreshSession` / `ExpireSession` / `RevokeSession` | `SessionNotFound` |

An empty aggregate has no applied events, so its conceptual revision is `0`; the first accepted event has sequence `1`.

An `OpenSession` against an existing aggregate returns `SessionAlreadyExists`; it is not treated as an idempotent no-op. Command-level duplicate/conflict identity is owned by later request-context/application work.

### 6.2 Active session

| Command condition | Result |
|---|---|
| `RefreshSession`, observed before effective expiry and §3 recomputation yields `new_effective_expires_at > current_effective_expires_at` | `SessionRefreshed` |
| `RefreshSession`, observed before effective expiry but §3 recomputation yields `new_effective_expires_at <= current_effective_expires_at` | `NoEffectiveRefresh`; no event |
| `RefreshSession`, observed at/after effective expiry | `SessionExpired`; requested refresh is not applied |
| `ExpireSession`, observed before effective expiry | `SessionNotYetExpired`; no event |
| `ExpireSession`, observed at/after effective expiry | `SessionExpired` |
| `RevokeSession`, observed before effective expiry | `SessionRevoked` |
| `RevokeSession`, observed at/after effective expiry | `SessionExpired`; requested revoke is not applied |

For any accepted transition, `observed_at` must be greater than or equal to `last_transition_at`. A backward observation is `NonMonotoneTime` and produces no event. Revocation may occur at the same millisecond as the prior transition. Under immutable evidence/policy, refresh at the same instant recomputes the same effective deadline and therefore returns `NoEffectiveRefresh`.

Time-derived expiry is checked before refresh or revoke semantics once identity, revision, terminal-state and monotone-time checks pass. This prevents an already expired session from being refreshed or relabeled by a later command.

Every `SessionExpired` produced by this table records the exact effective deadline as `expired_at`, not the later command observation. `last_transition_at` advances to `observed_at`, preserving both validity time and detection order.

Reclassifying a revoke observed at/after expiry as `SessionExpired` is deliberate: the session had already lost validity at the earlier effective deadline, so revocation cannot replace that historical cause. B2 emits the same `SessionExpired` domain event as explicit expiry and does not encode the caller's original revoke intent; B4 control-evidence may record that redacted command disposition separately without forging a `SessionRevoked` domain event.

### 6.3 Terminal session

`Expired` and `Revoked` are terminal. Every subsequent `RefreshSession`, `ExpireSession` or `RevokeSession` returns `TerminalSession` and produces no event, including repeated expire/revoke. `OpenSession` against any existing aggregate follows its dedicated precedence and returns `SessionAlreadyExists`. Historical events and prior evidence remain unchanged.

B2 defines no resurrection. Opening a new session requires a new `SessionId` and fresh admitted evidence.

## 7. Deterministic decision precedence

For `RefreshSession`, `ExpireSession` and `RevokeSession` on an existing aggregate, command decision uses this global precedence:

1. malformed command/value shape is rejected by constructors before decision;
2. command `session_id` mismatch;
3. `expected_revision` mismatch;
4. terminal session mutation;
5. `current_revision == u64::MAX` returns `RevisionOverflow`;
6. non-monotone observed time;
7. time-derived expiry at the observed instant;
8. command-specific legality — not-yet-expired, no-effective-refresh, and `DeadlineOverflow` when a refresh's own checked deadline arithmetic overflows.

Terminal state is checked **before** revision exhaustion, so §6.3's flat statement — every later `RefreshSession`, `ExpireSession` or `RevokeSession` on a terminal session returns `TerminalSession` — is exactly true with no exception at `u64::MAX`. That ordering is also the more useful answer: a terminal session will never emit another event, so reporting its exhausted counter would describe the less decisive of two facts. `RevisionOverflow` keeps precedence over time and command legality, because an exhausted counter makes every remaining transition impossible regardless of the clock.

Item 8's `DeadlineOverflow` is reachable and belongs here rather than at open: `effective_expires_at` may sit at `u64::MAX` under an absolute or credential cap, with `observed_at` just below it, so `checked_add(observed_at, idle_timeout)` overflows while the session is still validly `Active`. No other failure can co-occur with it, so its position is unambiguous.

For open, precedence is:

1. malformed evidence/policy/value shape;
2. existing aggregate returns `SessionAlreadyExists`;
3. on an empty aggregate, non-zero `expected_revision` returns `RevisionMismatch { expected: command value, actual: 0 }`;
4. authentication/open time ordering;
5. already-expired credential evidence;
6. arithmetic overflow or non-future derived deadline.

No lower-precedence failure may hide a higher-precedence one. Tests include dual-fault cases in both orientations where relevant.

## 8. Append, evolution and replay

The state loop is:

```text
command
→ validate values
→ decide against current snapshot and expected revision
→ immutable next event
→ B4 journal append under expected revision
→ evolve only from the persisted event
→ equal replayed snapshot
```

Both halves are free functions over caller-supplied state, with no hidden aggregate registry:

```text
decide(state: Option<&SessionSnapshot>, command: &SessionCommand)
    -> Result<SessionEvent, SessionDomainError>
evolve(state: Option<&SessionSnapshot>, event: &SessionEvent)
    -> Result<SessionSnapshot, SessionDomainError>
```

`None` is the empty aggregate. Because the caller supplies the state rather than the domain looking it up, a command naming a different session than the state it was given is a real, reachable `SessionIdMismatch` on the decide path — not only on the replay path — and §13 binds a fixture for each.

Applying an event requires:

```text
next_revision = current_revision.checked_add(1).ok_or(RevisionOverflow)
event.sequence == next_revision
```

Decision uses the same checked increment before producing an event. Revision exhaustion produces no event, applies no event and leaves the prior snapshot unchanged; wrapping from `u64::MAX` to sequence `0` is always rejected.

For an empty aggregate, only `SessionOpened(sequence=1)` is legal. Evolution revalidates every §2–§3 open invariant before creating a snapshot: credential/evidence/policy shape, `opened_at >= authenticated_at`, credential still valid at open, checked deadline arithmetic and strictly future derived deadlines. It then derives the deadlines itself; it never trusts a serialized snapshot or caller-supplied derived deadline.

For every existing aggregate, evolution first requires exact next sequence, exact `SessionId`, non-decreasing event time and immutable scope. It then uses this exhaustive event-by-state table:

| Current state | Event | Additional apply guard | Result |
|---|---|---|---|
| `Active` | `SessionRefreshed` | `observed_at < effective_expires_at`; recomputed effective deadline strictly advances the current deadline and exactly equals the event field | set `effective_expires_at` to the event field, set `last_transition_at = event.observed_at`, advance revision |
| `Active` | `SessionExpired` | `observed_at >= effective_expires_at`; event `expired_at` exactly equals the pre-existing effective deadline; event cause exactly equals the §3 tie-precedence result | enter `Expired`, preserve immutable scope and current `effective_expires_at`, set `last_transition_at = observed_at` |
| `Active` | `SessionRevoked` | `observed_at < effective_expires_at` | enter `Revoked { revoked_at: event.observed_at }`, preserve immutable scope and current `effective_expires_at`, set `last_transition_at = observed_at` |
| `Active` | `SessionOpened` | never legal | fail closed |
| `Expired` or `Revoked` | any event | never legal | fail closed |

`SessionExpired.observed_at` must be greater than or equal to `SessionExpired.expired_at`. The refresh and revoke apply guards use the same effective-expiry function as command decision, and the expire guard additionally uses the same §3 cause function; evolution may not accept a persisted event that `decide` could never have emitted. No event can replace tenant/user/session/adapter/evidence/policy scope or change a terminal state.

**An apply-guard violation has its own named failure.** A forged or corrupted event can carry an exact sequence, an exact `SessionId`, a forward `observed_at` and — for `SessionRevoked`, which has no derived field at all — nothing else to check, while still sitting on the wrong side of the effective deadline for its kind. None of `EventSequenceMismatch`, `SessionIdMismatch`, `NonMonotoneTime` or `EventDerivedFieldMismatch` describes that, and `IllegalEventForState` would contradict this table, which lists the pair as legal. It is `EventTimeOutsideValidity`, and it covers all three `Active` rows: a `SessionRefreshed` or `SessionRevoked` whose `observed_at` is at or after the effective deadline, and a `SessionExpired` whose `observed_at` precedes it.

So the complete fail-closed set for evolution is: a gap, a duplicate sequence, an out-of-order event, a cross-session event, a forged derived field, an event time outside the guard's validity window, an illegal event/state pair, or a failed open invariant. Each returns no partial snapshot.

While a session is `Active`, `effective_expires_at > last_transition_at` is an invariant: open requires strictly-future derived deadlines and a credential deadline later than `opened_at`, and refresh requires both `observed_at < effective_expires_at` and a strictly advancing deadline. This is what makes §7's ordering of non-monotone time before time-derived expiry unambiguous rather than merely conventional — `observed_at < last_transition_at` implies `observed_at < effective_expires_at`, so the two conditions have an empty overlap and can never compete.

Replaying the same validated `SessionOpened` plus the same ordered events must reconstruct a structurally equal `SessionSnapshot`: every field compares equal. Replay never reads the current clock, reloads policy, resolves a credential, calls an adapter or writes evidence. A canonical persisted byte encoding and checksum are deferred to B4's journal contract; B2 does not invent a format-independent “canonical Serde byte representation”.

A duplicate event sequence is an error, not an apply-time no-op. B4's journal may recognize an identical append retry under its own accepted contract, but it must not ask the domain aggregate to apply the same sequence twice.

## 9. Typed failures and diagnostic safety

There are exactly two taxonomies, split along §7's own precedence: value construction is rejected **before** decision, so a shape failure cannot reach the state machine and the state machine carries no shape variant.

### 9.1 Construction — `SessionValueError`

`SessionValueError` mirrors `IdentityValueError`: a small `Copy` value carrying exactly the static Rust value-kind name that rejected the input and one `SessionValueErrorKind`, with private fields, exactly two read-only accessors `value_kind()` and `kind()`, and no `source`. `SessionValueErrorKind` is the public enum owning exactly:

```text
Empty
TooLong { max_bytes: usize }
InvalidStart
InvalidCharacter { byte_index: usize }
InvalidEnd
MalformedDigest
ZeroDuration
CredentialWindowNotAfterAuthentication
```

The first five apply to `AuthAdapterId` under §2.2's grammar, in `platform-identity/v0` §5's precedence — empty, over-length, invalid first byte, first invalid interior byte scanned left to right, invalid final byte. `MalformedDigest` is the single, payload-free rejection for `CredentialEvidenceDigest`: the value has one fixed shape, so a positional index would describe secret-derived text without adding a usable distinction. `ZeroDuration` rejects a zero `SessionDuration`. `CredentialWindowNotAfterAuthentication` rejects `SessionCredentialEvidence` whose `credential_not_after` is present and not strictly later than `authenticated_at`.

### 9.2 Decision and evolution — `SessionDomainError`

`SessionDomainError` is a `Copy` enum owning exactly:

```text
CredentialEvidenceExpired
InvalidTimeOrder
DeadlineOverflow
SessionNotFound
SessionAlreadyExists
SessionIdMismatch
RevisionMismatch { expected: u64, actual: u64 }
RevisionOverflow
TerminalSession { status: SessionStatus }
NonMonotoneTime
SessionNotYetExpired
NoEffectiveRefresh
EventSequenceMismatch { expected: u64, actual: u64 }
EventTimeOutsideValidity
IllegalEventForState
EventDerivedFieldMismatch { field: EventDerivedField }
```

Field roles are pinned so two implementations cannot report them oppositely. In `RevisionMismatch`, `expected` is the caller's claim and `actual` is the aggregate's truth, as §6.1 already fixes. In `EventSequenceMismatch` the roles are the same shape: `expected` is the derived `next_revision` and `actual` is the event's own `sequence`.

`EventTimeOutsideValidity` is the §8 apply-guard failure: the event's `observed_at` is on the wrong side of the effective deadline for that event kind. It is payload-free — the two instants involved are already in the caller's own snapshot and event.

The draft revision of this contract also listed `InvalidCredentialEvidence` and `InvalidSessionPolicy` here. They are deliberately **not** domain variants: `SessionCredentialEvidence` and `SessionPolicy` have validating constructors and validating Serde, so a malformed one cannot be built, cannot be deserialized and therefore cannot reach `decide` or `evolve` — including from a persisted event, whose fields are already those types. Retaining an unreachable variant would be a permanent dead arm that no adversarial fixture could exercise, and §13 requires every fixture to affect an executable assertion. The corresponding real failures are `SessionValueErrorKind::MalformedDigest`, `ZeroDuration` and `CredentialWindowNotAfterAuthentication` in §9.1.

`EventDerivedField` is a closed non-secret enum containing exactly `RefreshEffectiveExpiresAt`, `ExpiredAt` and `ExpiryCause`. The error does not carry arbitrary field names, source payloads or rendered event values.

`CredentialEvidenceExpired` has no credential/evidence payload. `InvalidTimeOrder` covers `opened_at < authenticated_at`; `DeadlineOverflow` covers checked deadline arithmetic. §3's further open condition — that the derived absolute and idle deadlines be strictly later than `opened_at` — is an invariant that both `decide` and `evolve` assert, not a separate reachable variant: with `SessionDuration` non-zero by construction and every addition checked, the only way to fail it is the overflow `DeadlineOverflow` already names. It is stated as an invariant so evolution has an explicit postcondition to re-derive rather than trust, not so that a dead error variant exists.

Errors are small typed values. They may report a failure kind, safe expected/actual revision and terminal status. They must not retain or render:

- credential/token/cookie/password text;
- a provider subject or arbitrary adapter payload;
- rejected secret-derived bytes or fragments;
- a complete serialized evidence value;
- arbitrary caller-provided reason text.

Failed commands and failed event application leave the previous snapshot unchanged and produce no partial event/snapshot.

## 10. Serde and public API negative space

The exact B2 Serde surface is:

- nominal scalar values, `SessionCredentialEvidence`, `SessionPolicy` and events implement validating `Serialize` and `Deserialize`, which is what event replay needs;
- commands implement `Serialize` only, per §4;
- deserialization delegates to the same decision-independent value validators; unknown and missing fields fail closed;
- `SessionSnapshot` and `SessionStatus` are serialization-only read models: they implement no public `Deserialize`, `Default` or direct constructor and can arise only from validated `SessionOpened` evolution plus legal replay;
- `SessionValueError`, `SessionValueErrorKind`, `SessionDomainError` and `EventDerivedField` implement neither `Serialize` nor `Deserialize`; B4 owns stable external error/event projections.

Mechanically that means `#[serde(deny_unknown_fields)]` on every aggregate struct, `#[serde(rename_all = "snake_case")]` on every enum, and no hand-written visitor anywhere, exactly as §2.0 requires. A derived unchecked field decode on an authority-bearing value is forbidden: `deny_unknown_fields` closes the unknown-field half, and delegation to the checked constructor closes the invalid-value half. Neither closes the other, so both are required.

Serde proves only shape and invariant validity, never credential authenticity or caller admission. B3/B4/B5 composition must not expose `OpenSession` deserialization as an untrusted transport endpoint. Derived unchecked decoding is forbidden on every authority-bearing value.

External compile-fail/API proofs must show that callers cannot:

- construct or mutate snapshots/events through public fields;
- set `revision`, deadlines, status or expiry cause directly;
- replace tenant/user/session/evidence/policy after open;
- default a session into an active state;
- convert one identity kind into another;
- obtain mutable backing access;
- bypass evidence/command validation through public fields or an unchecked constructor;
- pass raw credential text to any B2 evidence constructor or conversion API;
- call a public unchecked constructor or generic state setter.

Read-only, zero-copy accessors are allowed. `Display` for client/log use must not expose credential evidence internals.

**`Debug` redaction is owned by B2, not deferred.** Every value carrying an `evidence_digest` — `SessionCredentialEvidence`, `SessionOpened` and `SessionSnapshot` — implements `Debug` by hand and renders that field as a fixed redaction token rather than its bytes. An earlier revision deferred this to B4 `control-evidence`, which does not exist; that would have left an implementer with no rule to satisfy while the repository's default convention is a derived `Debug`, and a derived `Debug` prints the digest on every `assert_eq!` failure, panic message and trace line. It would also have made B2 weaker than its own sibling: `AUTH-014` governs B1's identity errors on `Display` **and** `Debug`. B4 may later widen redaction to further surfaces; it is not a prerequisite for this one.

## 11. Dependency and side-effect boundary

The pure B2 implementation may depend on:

- `platform-identity/v0` values;
- Rust standard-library value traits;
- existing workspace Serde support;
- lightweight digest-shape validation already admitted by repository policy.

It must not import or expose:

- clock/timezone crates or a system clock;
- RNG/UUID generation;
- Dioxus, Axum or browser/session middleware;
- cookie, OAuth, OIDC, CAS or token libraries;
- database, cache, queue, filesystem, network or process APIs;
- concrete M90 repositories/journals/secret resolvers;
- M10, Market, Agent, Plugin, provider or MCP types;
- raw secret storage or logging behavior.

Two of those are concrete in-repository names rather than categories, and are called out because `platform-core` already depends on them for M20 work: the session module must not import **or reference by path** `ustc_agent_tool_protocol` — the Agent-facing tool family §4 of [`module-boundaries.md`](module-boundaries.md) confines to the M30/M40 seam — and must not import or reference `semver`, which carries package-version semantics owned by M20.

"Or reference" is the load-bearing half. Both crates are already real dependencies of `platform-core`, so `semver::Version::parse(…)` written inside a function body compiles, declares no item, and would therefore be invisible to an item-level allowlist. §11.1 accordingly requires a per-file forbidden-carrier scan rather than relying on the item accounting alone.

Adding either import, or any new entry to `crates/platform-core/Cargo.toml`, is out of scope for B2; the batch introduces **no** manifest dependency change.

`B2` also computes no digest. `CredentialEvidenceDigest` is validated for shape and stored; nothing in the module hashes, derives or verifies it. The "lightweight digest-shape validation" admitted above is byte-class validation of an already-supplied string, not a cryptographic dependency, and no digest crate is added.

### 11.1 Required carrier extensions in the frozen `platform-core` surface

`platform-identity/v0` §4 freezes `platform-core`'s compiled surface by **total accounting**, not by pattern search: the crate's file inventory, each governed source's `mod`/`use`/`type`/`extern` items in source order with their attribute envelopes, each source's `impl` self types, macro definitions, macro invocation names, derive list and manifest key sets are each compared against an exact allowlist, and *an added item fails exactly as a removed one does*. Adding a module is therefore not a silent act — which is the intended design, and also means B2 cannot begin until the extension is made deliberately.

Adding the session module is admitted drift of that surface, and B2's first implementation commit must extend, in `scripts/check_repo_contracts.py` and its mirroring Rust guard together:

- the governed source-file inventory and the package file inventory, with `src/session.rs` and `tests/platform_session.rs`;
- the admitted `mod` declarations, with `session` under `lib.rs` and none under `session.rs`;
- the admitted item declarations, with `pub mod session;` in `lib.rs` and the session module's own complete `use` list;
- **the enumerated cross-file identity-binding exception.** This is a *separate* rule from the item allowlist and is the one that actually admits §2.0's import. It is currently a hard-coded single exception — admitted only when the file is `invocation.rs` **and** the normalized text equals that one re-export string — carried in both `scripts/check_repo_contracts.py` and the mirroring Rust guard. It must be widened to an enumerated set keyed by **exact file name and exact normalized text**, adding `session.rs` with exactly `use crate::identity::{SessionId, TenantId, UserId};`. It must **never** be relaxed into a predicate over `crate::identity::`, a prefix match or a regex: the rule's whole purpose is to refuse the alias class, and a pattern would re-open it. The failure message an implementer will see first reads `platform identity value alias or import outside the M00 identity module`, which describes a prohibition rather than a missing registration — that message is not an invitation to loosen the predicate;
- the admitted sibling `impl` self types, macro definitions and macro invocation names for `session.rs`;
- **the per-source attribute-name allowlist** for `session.rs`. There is no per-sibling derive-body carrier; the rule that fires is an exact-set comparison over every attribute name in the file. `session.rs` will carry at least `serde`, from §2.0's `deny_unknown_fields` and `rename_all`, plus `derive`, `doc` and `must_use` — none of which is admitted for a sibling today;
- a **forbidden-carrier scan over `session.rs`**, listing at least `ustc_agent_tool_protocol` and `semver` alongside the §11 categories. B1 has such a scan but applies it only to `identity.rs`, necessarily, since `invocation.rs` legitimately uses both crates. Without a session-scoped equivalent, §11's prohibition has no carrier at all once B2 exists, because a path-qualified call inside a function body declares no item. One scan closes it, and it does not require the frozen function bodies §11.2 declines;
- the admitted test-file items and attribute envelope for `tests/platform_session.rs`.

One trap in the Rust mirror is worth naming, because "extend the table" does not describe it: several of its lookups dispatch by **fallthrough default** rather than by exhaustive match, so a `session.rs` that is never added explicitly is silently checked against `lib.rs`'s lists instead of failing closed. Each extension must be verified by running the gate against a real `session.rs`, not by inspecting the table.

This is drift of the frozen **surface registration**, not of `platform-identity/v0` itself. It changes no accepted byte grammar, maximum length, error precedence, Serde shape or nominal kind set, so by `platform-identity/v0` §9 it is not a version change of that contract — the same reasoning that document already applies in its §5 to the `IdentityValueError` representation, and the same cost it already accepts in its §4 for `invocation.rs`'s import list. One sentence of `platform-identity/v0` §4 is amended by acceptance of this contract, from admitting a single cross-file identity binding to admitting an enumerated set of them; the substance of the rule — no renaming, and complete accounting of every governed source — is unchanged, and B2 receives no re-export.

If a future reviewer prefers a separate crate to this extension, that is a `platform-session/v0` change under §16 and a `module-work-policy/v1.3` §5 extraction decision, not an implementation choice available at commit time.

### 11.2 How much of B1's closure B2 carries, and why less

`platform-identity/v0` §4 closes its grammar against a co-mutating adversary: contract-rooted regex parsing, construction-site and spelling accounting, frozen function bodies, effective-use elimination for the length bound, guard-as-one-structural-unit, control-transfer prohibitions, name-resolution shadowing rules, a runtime sweep whose carriers are pinned as values, and a shared lexical corpus differential between two independent implementations. That apparatus exists because those six values are the repository's root identity authority: every tenant, user, session, request, command and correlation reference in every module is whatever that grammar admits.

B2 carries the **root**, not the whole apparatus. Required: §2.2's two grammars exist as fenced normative carriers in this document; the implementation's admitted byte classes, length bound and digest shape are extracted from source and cross-checked against them by the repository checker; the acceptance rows in §12 each run that checker before their Rust leg, because a Rust test cannot prove it ran; and the negative-space and API proofs in §10 and §13 are executable.

Deliberately **not** required: frozen per-function body fingerprints, construction-expression counting, the effective-use elimination chain, statement-position and shadowing rules, and a second lexer with a corpus differential.

The reason is a difference in blast radius, stated rather than implied. A defect in B1's grammar mints or admits identities repository-wide. A defect in `AuthAdapterId`'s grammar admits a malformed adapter label into one session's provenance record; it authenticates nothing, authorizes nothing, mints no identity, and cannot widen a `platform-identity/v0` value, because §2.0 gives B2 no path to construct one. The session values whose failure *would* matter — revision algebra, deadline algebra, terminal precedence, replay equality — are closed by executable adversarial fixtures under §13 rather than by source-shape accounting, because they are behavioural properties that a fixture can actually falsify.

This is a stated scope limit, not a claim of equivalent rigor. A later batch that raises `AuthAdapterId` to an authority-bearing value must revisit it.

## 12. Acceptance coverage

`AUTH-017`, `AUTH-018`, `AUTH-019` and `AUTH-020` are active rows in [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv) with status `planned`, gate `pr`, and the exact future bindings below. They are simultaneously retained in the long-horizon catalog [`platform-baseline.md`](../acceptance/platform-baseline.md), which is a catalog and confers no currency by itself.

`planned` is a non-pass state. No row here is `implemented`, no named test exists, and this section is the specification those tests must satisfy — not a report that they do.

| Case | Binding |
|---|---|
| `AUTH-017` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_open_pins_immutable_scope_and_checked_deadlines -- --exact` |
| `AUTH-018` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_lifecycle_precedence_is_deterministic_and_terminal -- --exact` |
| `AUTH-019` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_revision_and_replay_are_exact_and_fail_closed -- --exact` |
| `AUTH-020` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_domain_has_no_credential_or_adapter_surface -- --exact && cargo test --locked -p ustc-campus-agent-core --doc session` |

Each binding runs the repository checker before its Rust leg for the reason `platform-identity/v0` §4 gives: redirecting a `[[test]]` target or renaming a bound function makes `--exact` match nothing, which cargo reports as `running 0 tests` at exit zero, and a guard written inside the suite is exactly what such a change replaces. Only an out-of-band carrier detects that, so the checker is part of each binding rather than a courtesy check.

`AUTH-020`'s `--doc` leg is covered by CI's separate unconditional `cargo test --locked --all-features --doc` step, since `--all-targets` does not run doctests.

### `AUTH-017` — immutable open scope and deadline algebra

Prove that open pins exact tenant/user/session/adapter/evidence/policy scope; derives separate idle, policy-absolute and optional credential deadlines with checked arithmetic; rejects stale evidence/time/overflow; and validating Serde rejects incomplete/unknown input.

### `AUTH-018` — refresh/expire/revoke precedence

Prove that refresh extends only idle expiry, never credential/policy-absolute expiry or scope; equality is expired; `Credential > Absolute > Idle` resolves equal deadlines; late observation preserves the effective `expired_at`; expired sessions cannot refresh/relabel; revoke blocks immediately; and terminal states cannot mutate or resurrect.

### `AUTH-019` — expected revision and deterministic replay

Prove exact checked sequence/revision behavior including exhaustion, gap/duplicate/out-of-order/cross-session/wrapped-zero/forged-derived-field rejection, immutable event application and equal replay without clock/policy/adapter I/O.

### `AUTH-020` — credential and dependency negative space

Prove that commands/events/errors/Serde/debug paths never retain or echo raw credentials or arbitrary adapter payloads, external callers cannot bypass authority-bearing field validation or call a raw-credential conversion surface, structural evidence decoding is not represented as authentication, and the pure module declares no clock/RNG/transport/database/framework/auth-adapter dependency or ID-generation surface.

Existing catalog `AUTH-008` remains the later demo/integration assertion for idle/absolute expiry and logout invalidation. Existing `AUTH-010` remains the broader manual-security assertion that the platform never stores/logs raw USTC passwords or tokens. B2 unit evidence supports but does not complete either case.

## 13. Required adversarial fixtures

Acceptance of this contract does not bind these; the implementation does. `AUTH-017..020` may be promoted to `implemented` only once every case below is an executable assertion under §12's named tests. The list is a floor, not a ceiling:

- open at, before and after credential `not_after`;
- zero duration and checked-add overflow;
- idle deadline before, equal to and after policy-absolute and credential deadlines;
- refresh before, exactly at and after effective expiry;
- refresh that advances and does not advance idle expiry;
- revoke before and at expiry;
- repeated refresh/expire/revoke on terminal states;
- stale and future expected revisions;
- decision and evolution at revision `u64::MAX`, including forged wrapped sequence `0`;
- equal, backward and forward observed instants;
- event sequence gap, duplicate and reorder;
- cross-session event injection, and a decide-side `SessionIdMismatch` where the command names a different session than the supplied state;
- forged refreshed deadline, effective `expired_at` and expiry cause;
- a persisted `SessionRefreshed` and a persisted `SessionRevoked` whose `observed_at` is at or after the effective deadline, and a persisted `SessionExpired` whose `observed_at` precedes it — each `EventTimeOutsideValidity`, each with sequence, `SessionId` and every derived field otherwise exact, so no other variant can answer;
- late expiry observation that must retain the earlier effective deadline;
- expiry observed exactly at and strictly after each deadline, with equal derived `expired_at` but distinct `observed_at`;
- a refresh whose own deadline arithmetic overflows because `effective_expires_at` sits at `u64::MAX`, returning `DeadlineOverflow`;
- deserialization of a `SessionCredentialEvidence` payload whose `credential_not_after` is not strictly later than `authenticated_at`, rejected as `CredentialWindowNotAfterAuthentication` — this is the fixture that proves §2.0's shadow-struct rule is in force and that §9.2's removal of `InvalidCredentialEvidence` was safe;
- `admits_at` false for a revoked session at an instant strictly before its preserved `effective_expires_at`, and true only while `Active` and before that deadline;
- replay after each legal prefix and across the full lifecycle;
- dual-fault precedence cases, including a terminal session whose `current_revision` is `u64::MAX`, which must return `TerminalSession` and not `RevisionOverflow`;
- secret-like canary strings absent from every error, `Display` and `Debug` surface, and from every serialized event/snapshot surface where forbidden — `Debug` included because §10 makes its redaction a B2 obligation.

Fixture names or expected-output edits must affect executable assertions; a checker that merely counts fixture files is insufficient.

## 14. Explicit non-goals

This contract does not define or claim:

- production USTC CAS/OIDC/browser authentication;
- credential verification, token refresh or cookie rotation;
- role/RBAC/domain permission decisions;
- tenant-scoped `ActorIdentity` or `PlatformRequestContext`;
- `CausationId`, command duplicate/conflict disposition or policy-reference identity;
- ID generation/collision policy;
- concrete clock/repository/journal/secret-ref adapters;
- durable exactly-once append or distributed session consistency;
- CSRF/origin/cookie transport security;
- multi-device session management, administrator/service actors or session migration;
- deletion/erasure of already committed audit history;
- a client-safe full snapshot projection.

These remain owned by later M00 batches, M10, M90 or release/security integration as named in their contracts.

One of them is an obligation rather than a feature, so it is named with an owner rather than only excluded. **Producing** a `CredentialEvidenceDigest` — choosing the domain separation and the adapter-side material it is computed over, and never computing it over raw credential text — belongs to the authentication-adapter contract that `M00-B4` introduces alongside `session-port` and `control-evidence`. §2.2 states the obligation normatively because B2 pins the resulting value into immutable evidence and cannot verify it; that batch's contract must discharge it explicitly rather than inherit it silently.

## 15. Acceptance record and implementation-entry gate

The eight conditions this contract carried while it was a draft are now discharged, in the same order they were stated:

1. `M00-B1` is merged at `c347e689aa23ee777b95e0989e633a9d91041161` and its public surface is read back in §2.0;
2. the M00 blueprint, module map, roadmap and coverage matrix project B2 alongside — not over — B1's implemented evidence;
3. `AUTH-017..020` are in the long-horizon catalog and are active `planned` matrix rows with the exact future bindings in §12;
4. the repository checker registers this contract as a fail-closed key file and cross-validates §12 against the active matrix, so a stale or missing projection carrier fails the run;
5. transition, deadline and error precedence received independent blocker review across three lanes — contract/dependency direction, acceptance evidence, and semantics/security — and every accepted finding is folded into this revision, including a frozen error set that could not express a specified apply-guard rejection, a precedence list that contradicted §6.3 at `u64::MAX`, `Debug` redaction deferred to an unstarted batch, an unowned digest-provenance obligation, a read model that was fail-open for `Revoked`, and an incomplete §11.1 carrier list;
6. public type names and Serde shapes are frozen in §2 against the merged B1 API;
7. no current-status carrier claims B2 implementation evidence — every affected carrier says `planned`, and `M00` stays `partial-evidence`;
8. documentation and checker gates pass on the exact final head.

This contract is therefore accepted, and accepted is the *entry* condition for implementation, not evidence of it. What acceptance authorizes is exactly one thing: `M00-B2 session-domain` may be implemented against this specification, under `module-work-policy/v1.3` §3 Path B, on its own branch, with its own review and its own exact-head CI.

What acceptance does **not** establish:

- no `AUTH-017..020` row may be promoted from documentation alone; each is promoted only when its named test exists and every assertion in §§3–13 that it covers is executable;
- `M00` does not advance past `partial-evidence`, and neither `StandaloneReady` nor any later readiness state is reachable from this batch — `module-work-policy/v1.3` §7 owns that gate and B3, B4 and B5 are unstarted;
- no session, actor, request-context, policy-reference, port, adapter, journal or M10 admission behavior becomes operational, and none of it may be described as such.

## 16. Change rule

Changing a public value name, field set, transition table entry, precedence order, deadline formula, error variant set or Serde shape frozen above changes `platform-session/v0`. Such a change requires an owning-contract update, acceptance-row and fixture review, and — once implementation exists — implementation and consumer evidence on the same revision.

Adding request-context, policy-reference, port, control-evidence, credential-verification or transport semantics is a later owning contract, not an incidental extension of these values. §14 lists what those later contracts own.
