# Platform session-domain contract

## Metadata

- `Status`: Draft, non-mergeable `M00-B2 session-domain` target contract; must be rebased onto merged M00-B1 and re-reviewed before acceptance or merge
- `Version`: `platform-session/v0`
- `Last Review`: `2026-07-26`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Depends On`: accepted [`platform-identity/v0`](platform-identity.md) values and [`module-boundaries.md`](module-boundaries.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for authority partition
- `Acceptance`: proposed catalog-only `AUTH-017`, `AUTH-018`, `AUTH-019`, `AUTH-020`; no active matrix row or implementation evidence yet
- `Primary Code`: future cohesive session-domain module under `crates/platform-core/`

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

The final implementation contract must expose nominal, private-field Rust values equivalent to the shapes below. Naming may be refined before this draft becomes accepted, but no field or invariant may be silently dropped.

### 2.1 Time and duration

`SessionInstant` is a non-negative `u64` count of Unix-epoch milliseconds. The domain never reads wall-clock time itself; a command carries one adapter-observed instant. Every addition is checked and overflow fails before an event is produced.

`SessionDuration` is a non-zero `u64` millisecond duration. Session-policy/config loading owns deployment ceilings; the pure kernel still rejects zero and arithmetic overflow. A duration is not a retry count, token lifetime or UI timeout.

Time is compared only as an integer ordering. No timezone, locale, leap-second or formatted timestamp enters the state machine.

### 2.2 Adapter and credential-evidence provenance

`AuthAdapterId` is a bounded opaque adapter identity. Its accepted spelling uses the same 1–128-byte ASCII grammar as `platform-identity/v0`; the spelling carries no trust level or provider semantics.

`CredentialEvidenceDigest` is exactly lowercase `sha256:<64 hex>`. It fingerprints already-admitted evidence for replay/audit correlation; it is not a credential, bearer token, password hash, refresh token or authorization result.

`SessionCredentialEvidence` contains exactly:

```text
tenant_id:             TenantId
user_id:               UserId
auth_adapter_id:       AuthAdapterId
evidence_digest:       CredentialEvidenceDigest
authenticated_at:      SessionInstant
credential_not_after:  Option<SessionInstant>
```

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

The values are resolved and pinned when the session opens. Refresh never reloads policy and never changes either duration. A later platform-policy version affects new sessions unless a separately accepted migration contract says otherwise.

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

Command values have private fields and validating constructors/Serde. Unknown fields fail closed. No command carries a raw credential, arbitrary metadata map, downstream permission, UI state, database handle or framework session object.

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
4. `current_revision == u64::MAX` returns `RevisionOverflow` before any event-producing transition;
5. terminal session mutation;
6. non-monotone observed time;
7. time-derived expiry at the observed instant;
8. command-specific legality such as not-yet-expired or no-effective-refresh.

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
| `Active` | `SessionRefreshed` | `observed_at < effective_expires_at`; recomputed effective deadline strictly advances the current deadline and exactly equals the event field | update effective deadline, `last_transition_at`, revision |
| `Active` | `SessionExpired` | `observed_at >= effective_expires_at`; event `expired_at` exactly equals the pre-existing effective deadline; event cause exactly equals the §3 tie-precedence result | enter `Expired`, preserve immutable scope and current `effective_expires_at`, set `last_transition_at = observed_at` |
| `Active` | `SessionRevoked` | `observed_at < effective_expires_at` | enter `Revoked { revoked_at: event.observed_at }`, preserve immutable scope and current `effective_expires_at`, set `last_transition_at = observed_at` |
| `Active` | `SessionOpened` | never legal | fail closed |
| `Expired` or `Revoked` | any event | never legal | fail closed |

`SessionExpired.observed_at` must be greater than or equal to `SessionExpired.expired_at`. Refresh and revoke apply guards use the same effective-expiry and cause functions as command decision; evolution may not accept a persisted event that `decide` could never have emitted. No event can replace tenant/user/session/adapter/evidence/policy scope or change a terminal state.

A gap, duplicate sequence, out-of-order event, cross-session event, forged derived field, illegal event/state pair or failed open invariant fails closed and returns no partial snapshot.

Replaying the same validated `SessionOpened` plus the same ordered events must reconstruct a structurally equal `SessionSnapshot`: every field compares equal. Replay never reads the current clock, reloads policy, resolves a credential, calls an adapter or writes evidence. A canonical persisted byte encoding and checksum are deferred to B4's journal contract; B2 does not invent a format-independent “canonical Serde byte representation”.

A duplicate event sequence is an error, not an apply-time no-op. B4's journal may recognize an identical append retry under its own accepted contract, but it must not ask the domain aggregate to apply the same sequence twice.

## 9. Typed failures and diagnostic safety

The domain distinguishes at least:

```text
InvalidCredentialEvidence
CredentialEvidenceExpired
InvalidSessionPolicy
InvalidTimeOrder
DeadlineOverflow
SessionNotFound
SessionAlreadyExists
SessionIdMismatch
RevisionMismatch { expected, actual }
RevisionOverflow
TerminalSession { status }
NonMonotoneTime
SessionNotYetExpired
NoEffectiveRefresh
EventSequenceMismatch { expected, actual }
IllegalEventForState
EventDerivedFieldMismatch { field: EventDerivedField }
```

`EventDerivedField` is a closed non-secret enum containing exactly `RefreshEffectiveExpiresAt`, `ExpiredAt` and `ExpiryCause`. The error does not carry arbitrary field names, source payloads or rendered event values.

`CredentialEvidenceExpired` has no credential/evidence payload. `InvalidTimeOrder` covers `opened_at < authenticated_at`; `DeadlineOverflow` covers checked deadline arithmetic. Errors are small typed values. They may report a failure kind, safe expected/actual revision and terminal status. They must not retain or render:

- credential/token/cookie/password text;
- a provider subject or arbitrary adapter payload;
- rejected secret-derived bytes or fragments;
- a complete serialized evidence value;
- arbitrary caller-provided reason text.

Failed commands and failed event application leave the previous snapshot unchanged and produce no partial event/snapshot.

## 10. Serde and public API negative space

The exact B2 Serde surface is:

- nominal scalar values, `SessionCredentialEvidence`, `SessionPolicy`, commands and events support validating serialization/deserialization needed by command handling and event replay;
- deserialization delegates to the same decision-independent value validators; unknown and missing fields fail closed;
- `SessionSnapshot` and `SessionStatus` are serialization-only read models: they implement no public `Deserialize`, `Default` or direct constructor and can arise only from validated `SessionOpened` evolution plus legal replay;
- domain errors need not implement wire/persistence Serde; B4 owns stable external error/event projections.

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

Read-only, zero-copy accessors are allowed. `Display` for client/log use must not expose credential evidence internals; internal debug output remains redacted according to B4 control-evidence rules.

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

A source-level checker eventually binds these negative-space claims to the implementation. The contract itself does not claim those checker carriers exist yet.

## 12. Proposed acceptance coverage

These cases remain catalog-only while this contract is draft. Exact active `matrix.tsv` commands and test names must be reviewed before implementation begins.

### `AUTH-017` — immutable open scope and deadline algebra

Prove that open pins exact tenant/user/session/adapter/evidence/policy scope; derives separate idle, policy-absolute and optional credential deadlines with checked arithmetic; rejects stale evidence/time/overflow; and validating Serde rejects incomplete/unknown input.

### `AUTH-018` — refresh/expire/revoke precedence

Prove that refresh extends only idle expiry, never credential/policy-absolute expiry or scope; equality is expired; `Credential > Absolute > Idle` resolves equal deadlines; late observation preserves the effective `expired_at`; expired sessions cannot refresh/relabel; revoke blocks immediately; and terminal states cannot mutate or resurrect.

### `AUTH-019` — expected revision and deterministic replay

Prove exact checked sequence/revision behavior including exhaustion, gap/duplicate/out-of-order/cross-session/wrapped-zero/forged-derived-field rejection, immutable event application and equal replay without clock/policy/adapter I/O.

### `AUTH-020` — credential and dependency negative space

Prove that commands/events/errors/Serde/debug paths never retain or echo raw credentials or arbitrary adapter payloads, external callers cannot bypass authority-bearing field validation or call a raw-credential conversion surface, structural evidence decoding is not represented as authentication, and the pure module declares no clock/RNG/transport/database/framework/auth-adapter dependency or ID-generation surface.

Existing catalog `AUTH-008` remains the later demo/integration assertion for idle/absolute expiry and logout invalidation. Existing `AUTH-010` remains the broader manual-security assertion that the platform never stores/logs raw USTC passwords or tokens. B2 unit evidence supports but does not complete either case.

## 13. Required adversarial fixtures before acceptance

The accepted contract/implementation gate must bind at least:

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
- cross-session event injection;
- forged refreshed deadline, effective `expired_at` and expiry cause;
- late expiry observation that must retain the earlier effective deadline;
- expiry observed exactly at and strictly after each deadline, with equal derived `expired_at` but distinct `observed_at`;
- replay after each legal prefix and across the full lifecycle;
- dual-fault precedence cases;
- secret-like canary strings absent from every error and serialized event/snapshot surface where forbidden.

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

## 15. Draft-to-accepted gate

This exact draft revision MUST NOT merge. Passing draft-local documentation/checker gates does not waive the required post-B1 rebase, authority reconciliation, active planned bindings and final exact-head review.

Before this draft may become accepted or authorize Rust implementation:

1. B1 canonical identity implementation is merged and its exact public surface is read back;
2. the M00 blueprint and roadmap project B2 without overwriting B1 evidence;
3. `AUTH-017..020` exist in the long-horizon catalog and active matrix as `planned` with exact non-vacuous future bindings;
4. the repository checker registers this contract and rejects missing/stale projection carriers;
5. transition/deadline/error precedence receives independent blocker review;
6. public type names and Serde shapes are frozen against the merged B1 API;
7. no current-status carrier claims B2 implementation evidence;
8. docs/checker gates pass on the exact final head.

Until all eight are true, this file is design evidence only and MUST NOT be cited as implementation authorization or module readiness.
