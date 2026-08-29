# Platform session-domain contract

## Metadata

- `Status`: Accepted `M00-B2 session-domain` contract; implemented under §17
- `Version`: `platform-session/v0`
- `Last Review`: `2026-07-29`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Depends On`: implemented [`platform-identity/v0`](platform-identity.md) values and [`module-boundaries.md`](module-boundaries.md)
- `Authority Defers To`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md) for authority partition
- `Acceptance`: active `AUTH-017`, `AUTH-018`, `AUTH-019` and `AUTH-020`, all `implemented` under §17
- `Primary Code`: `crates/platform-core/src/session.rs`, with evidence in `crates/platform-core/tests/platform_session.rs`

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

Authentication adapters produce bounded `SessionCredentialEvidence`; implemented [`platform-session-port/v0`](platform-session-port.md) supplies the B4a clock/repository/secret-reference interfaces, deterministic fakes and one durable DemoReviewed current-session read/bootstrap vendor; `M00-B4 control-evidence` still later owns stable external event/error/redaction projections; `M00-B3 request-context` owns admitted actor/request/command/causation semantics. A caller cannot reinterpret this pure kernel or B4a read vendor as proof that credential authentication or durable lifecycle mutation already happened.

## 2. Required public semantic values

The implementation must expose nominal Rust values with exactly the names, fields and invariants below; §2.0 fixes which of them are private-field structs and which are public enums, because Rust does not offer the same closure for both. The names are frozen by acceptance of this contract; changing one is a `platform-session/v0` change under §16, not an implementation detail. No field or invariant may be silently dropped.

### 2.0 Binding to the merged `platform-identity/v0` implementation

`M00-B1` is merged, so the following are read back from the implementation rather than proposed.

**Placement.** The session domain is one cohesive module at `crates/platform-core/src/session.rs`, declared `pub mod session;` in `crates/platform-core/src/lib.rs`, with evidence at `crates/platform-core/tests/platform_session.rs`. It is a sibling of `identity` and `invocation`, not a submodule of either, and it creates no new crate: `module-work-policy/v1.3` §5 prefers a small internal module until a compiler-enforced dependency, independent release or a second real consumer exists, and none does.

**Identity values are imported, never redefined and never re-exported.** The session module binds exactly:

```rust
use crate::identity::{SessionId, TenantId, UserId};
```

spelled without renaming. `platform-identity/v0` §4 requires that every governed source's `use`/`type`/`mod` items, `impl` self types, macro definitions and macro invocations be accounted for against an exact allowlist; the session module joins that governed set on the same terms and gains no exemption. It publishes **no** `pub use` of any identity kind: `invocation.rs`'s §6 compatibility re-export is the only admitted second path to an admitted kind, and B2 does not receive one. B2 therefore adds no externally reachable API to any `platform-identity/v0` kind — no inherent method, no trait implementation, no alias, no second path.

**B2 mints no seventh identity kind, and structurally cannot.** The `identity_value!` generator is private to `identity.rs`, so it is unreachable from a sibling module. `AuthAdapterId` and `CredentialEvidenceDigest` are `platform-session/v0` values defined by this contract, not `platform-identity/v0` values, and they do not widen, alias or re-spell one.

**Representation is part of the rule, inherited from `platform-identity/v0` §4, and it is a rule about structs.** **Every public B2 struct is a named-field struct with private fields**, never a tuple struct — stated for all of them rather than only for those carrying a validated invariant, because the property it buys is uniform and costs nothing. A tuple struct's constructor is a *value*, not only syntax: `let ctor = AuthAdapterId; ctor(text)` fills the private field while writing no construction expression a scan can find, and that was demonstrated against real evidence during B1. A named-field struct has no constructor function item at all, so a struct literal — syntax, which cannot be bound, aliased, passed or returned — is the only way to produce one, whatever that struct happens to validate.

**The rule stops at structs because Rust gives it nowhere else to stand.** This contract also freezes seven public enums — `SessionStatus`, `SessionExpiryCause`, `SessionCommand`, `SessionEvent`, `SessionValueErrorKind`, `SessionDomainError` and `EventDerivedField` — and an enum variant's fields are exactly as public as the enum itself, with no per-variant privacy to ask for. A universal "every B2 value has private fields" sentence would therefore not be a strict rule but an **unsatisfiable** one, and an unsatisfiable rule specifies nothing: an implementer cannot obey it and a reviewer cannot tell a deliberate shape from a violation. `platform-identity/v0` scopes the identical rule the identical way — its §4 states it for the six ID kinds, its §5 states `IdentityValueErrorKind` as a public enum owning its variants and payloads — and B2 inherits that scoping rather than widening it. `#[non_exhaustive]` is not the escape: it would buy external non-constructibility at the cost of forcing `..` and `_` arms on every consumer of sets whose whole purpose (§2.4, §9) is exhaustive reading, and §2.5 and §11.1 freeze an attribute closure that does not contain it.

What an enum must satisfy instead is a property of the **set**, not of a field, and is stated where each is defined: it is closed, carries exactly the variants and payloads listed, and its payloads are non-secret (§9). Where a caller's ability to name a variant might be mistaken for authority, the section that defines the enum says what actually holds — §2.4 for `SessionStatus`, §4.1 for the two wrapper enums.

**One constructor per constructible struct, checked exactly when checking is meaningful at that point.** Construction is partitioned three ways, and the partition is frozen:

1. **Structs a caller may build.** `SessionInstant`, `SessionDuration`, `AuthAdapterId`, `CredentialEvidenceDigest`, `SessionCredentialEvidence`, `SessionPolicy`, the four command structs of §4.1 and the four event structs of §5.1 each have exactly one inherent constructor and no second path.
2. **Structs a caller may not build.** `SessionSnapshot` and `SessionValueError` have **no** public constructor at all. A snapshot arises only from validated evolution and replay (§8); an error arises only from a rejecting validator inside the module. This is not an omission from rule 1 — they are read-only outputs, and §10 records the same fact as negative space.
3. **Enums.** The seven above are constructed by naming a variant, which is Rust's own construction path and admits no inherent constructor in its place.

Across all three: no public unchecked path, no `Default`, no `Deref`, no mutable backing access and no cross-kind conversion. Under rule 1, where the value owns an invariant that constructor is checked: string-backed values use `parse(value: impl Into<String>) -> Result<Self, SessionValueError>`, matching `platform-identity/v0` §4, and integer-backed and aggregate values use the exact constructor named with them below. The command and event families of §§4.1 and 5.1 instead take a **total** constructor returning `Self`, and those sections state the specific reason for each family — reasons of precedence and of decidable-but-not-meaningful comparison, not a claim that no constructible input is ever wrong.

Two rules govern any constructor added later, and they are separate:

1. a constructor whose error arm no input can reach must not exist, because it forces an `expect` at every construction site and reads to the next reviewer like a checked path;
2. a constructor must not check a relation whose *correctness* depends on state the value cannot see. Such a check is decidable but not meaningful: it enforces agreement between two caller-supplied fields that may both be wrong, and it silently removes from the input space exactly the adversarial values the aggregate guard has to be able to reject. §5.1 is the worked case.

**Serde delegates rather than re-implements.** No B2 value implements a hand-written `Visitor`, a `visit_*` method or `deserialize_any`. Each `Deserialize` deserializes the canonical primitive once through that primitive's own implementation and hands the result to the same checked constructor, so whichever entry point a deserializer chooses there is exactly one construction path. Aggregate structs carry `#[serde(deny_unknown_fields)]`; enums carry `#[serde(rename_all = "snake_case")]`.

**An aggregate carrying a cross-field invariant deserializes through its constructor, not field by field.** `deny_unknown_fields` plus per-field delegation validates every field and still admits a struct that no constructor would have built, because a cross-field invariant belongs to no field. Any aggregate carrying such an invariant therefore decodes a private shadow struct and hands it to the named constructor; the derived field-by-field decode is insufficient and is forbidden for those values. In this contract that is exactly one value — `SessionCredentialEvidence`, whose credential window spans two fields — and §5.1 records why no event joins it. The rule is load-bearing rather than stylistic: §9.2 removes two error variants on the argument that a malformed evidence or policy cannot reach `decide` or `evolve`, and that argument is only true if every deserialization path routes through the constructor that enforces the invariant.

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

**A missing `credential_not_after` is a rejection, not an absent deadline.** This is the one place where `deny_unknown_fields` plus per-field delegation does not give what §10's two-halves argument promises, so it is specified rather than left to the derive: serde's derived `Deserialize` fills a *missing* field of `Option` type with `None` instead of failing, and it does so inside the shadow struct too. The consequence is not cosmetic. A persisted payload that simply omits the field would decode as "this credential never expires", deleting the `Credential` term from §3's `min(...)` and disarming §3's already-expired-evidence open check — a downgrade performed by omission, on the one field whose whole purpose is to cap a session's life. So the shadow struct must reject an absent `credential_not_after` and accept an explicit null as the only spelling of "no credential deadline", which an explicit field attribute achieves and the bare derive does not. Serialization is the matching half: the field is always written, and `skip_serializing_if` is forbidden on it, so no B2-produced payload can ever be the omitted form. §13 binds the fixture.

Invariants:

- all fields are private and construction/Serde is validating;
- unknown fields fail closed, and so do missing ones — including `credential_not_after`, which needs an explicit rule because serde's derived decode gives `Option` fields the opposite default;
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

**Both are public enums, so a caller may name `SessionStatus::Active` or assemble `SessionStatus::Expired { .. }` from instants and a cause it already holds.** That is not a hole, and no section of this contract claims otherwise. Neither value carries authority by itself; a status is authoritative only as a *field of a snapshot*, and the invariant that holds is exactly this: **a caller-built `SessionStatus` cannot be injected into, substituted inside or read back out of a `SessionSnapshot` as that snapshot's own status.** The snapshot's fields are private, it has no public constructor, no `Deserialize` and no setter, and §8's evolution is its only producer. What must be unforgeable is the snapshot, and that is what is closed. `SessionStatus` also appears as a payload of `SessionDomainError::TerminalSession`, where it is a reported fact rather than an input, for the same reason.

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

§2.5 freezes this type's traits and accessors along with every other B2 type's.

Adding, removing or reinterpreting a persisted/public snapshot field requires a versioned contract and acceptance update; implementations cannot append convenience or framework state to this snapshot.

**`effective_expires_at` is not a validity predicate, and the snapshot exposes one so that no consumer has to invent it.** A revoked session keeps its `effective_expires_at` unchanged — §8 preserves it — so that field alone still reads as "not yet expired" while `status` says `Revoked`. The obvious consumer test `observed_at < effective_expires_at` is therefore correct for `Active`, correct for `Expired`, and **wrong exactly for `Revoked`**: it fails open on the logout/revocation path, which is the security-critical one catalog `AUTH-008` names. That is the worst shape of defect, because it passes casual testing.

The one sanctioned validity question is therefore part of this contract:

```text
SessionSnapshot::admits_at(&self, observed_at: SessionInstant) -> bool
```

**The question it answers is frozen as *current admission*, never historical validity.** It asks "may this snapshot admit an operation observed at `observed_at`?" It does not answer "was this session valid at instant `t`?", and B2 offers no method that does. The distinction decides the method's shape rather than decorating it: the two readings want opposite answers for a stale instant, and one method serving both would have to take the permissive answer, which is the answer no admission path may use. A later audit or forensic reconstruction that genuinely needs the historical question answers it by replaying the event sequence under §8, which is the only place the past is authoritative.

It returns `true` only when all three of these hold:

```text
status == Active
observed_at >= last_transition_at
observed_at < effective_expires_at
```

**The middle conjunct makes the read model fail closed on stale time**, and it is load-bearing for the same reason the first one is. `M00-B3`'s admission path supplies `observed_at` from an adapter observation, exactly as a command does, and §7 rejects a command whose `observed_at` precedes `last_transition_at` as `NonMonotoneTime`. A read model that admitted that same instant would be *more permissive than the decide path it guards*: a replayed stale observation, or a snapshot that has advanced past the instant a caller is still holding, would be admitted on evidence the state machine itself refuses. The concrete shape is a session refreshed or observed forward at `t₁` and an admission asked at `t₀ < t₁`, which a status-plus-upper-bound predicate answers `true`. Fail closed instead: an instant the aggregate has already moved past is not evidence of present admission.

The conjunct never makes a live session unreachable. §8's `Active` invariant `effective_expires_at > last_transition_at` guarantees the admitting window `[last_transition_at, effective_expires_at)` is non-empty for every `Active` snapshot. So `admits_at` is `true` at exactly `last_transition_at` and `false` at exactly `effective_expires_at`: the lower bound admits, the upper bound expires, matching §3's rule that equality is expired. A freshly opened session has `last_transition_at == opened_at`, so an observation before open is refused by the same conjunct rather than by a separate rule.

`M00-B3`'s request-context admission is the intended consumer; it asks this and does not recompute.

The internal evidence digest is not part of a client-safe projection by default. B3/B4 define later projections explicitly rather than serializing the whole domain snapshot across module boundaries.

### 2.5 Frozen trait and accessor closure

A public algebra whose derive lists do not close is not frozen, only described: `#[derive(Eq)]` on a struct whose field type never received `Eq` does not compile, and a contract that names the outer derive while leaving the inner one to taste has specified nothing. So the closure is stated once, exhaustively, for every public B2 type.

| Type | Derived | Hand-written |
|---|---|---|
| `SessionInstant` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` | — |
| `SessionDuration` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Serialize` | `Deserialize` — validating, §2.1 |
| `AuthAdapterId` | `Debug`, `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Serialize` | `Deserialize` — validating, §2.2 |
| `CredentialEvidenceDigest` | `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Serialize` | `Debug` — redacting, §10; `Deserialize` — validating, §2.2 |
| `SessionCredentialEvidence` | `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize` | `Deserialize` — shadow struct, §2.2 |
| `SessionPolicy` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize` | — |
| `SessionExpiryCause` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize` | — |
| `SessionStatus` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize` | — |
| `SessionSnapshot` | `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize` | — |
| the four commands, `SessionCommand` | `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize` | — |
| the four events, `SessionEvent` | `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, `Deserialize` | — |
| `SessionValueErrorKind`, `EventDerivedField` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` | — |
| `SessionValueError` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` | `Display`, `Error` |
| `SessionDomainError` | `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` | `Display`, `Error` |

**The two Serde columns are not symmetric, and the asymmetry is the rule rather than an inconsistency.** `Serialize` is derived for every value in this contract, because nothing here constrains serialization beyond the wire form §§2.1–2.2 already fix: the four scalar values carry `#[serde(transparent)]` and therefore emit exactly one `u64` or one string, which is byte-for-byte what a hand-written `serialize_u64`/`serialize_str` produces. Writing one by hand would add an implementation that proves nothing.

`Deserialize` is different because §2.0 requires every decode to reach the value's own checked constructor, and a derive cannot do that for a value that owns an invariant. `#[serde(transparent)]` fills the private field directly, so a zero `SessionDuration` or an ungrammatical `AuthAdapterId` would be admitted; `#[serde(try_from = "…")]` would route through a constructor but only by adding a public `TryFrom`, which is a second construction path under §2.0 rule 1 and a public associated function outside the closure this section ends with. So exactly those values whose constructor can reject — `SessionDuration`, `AuthAdapterId`, `CredentialEvidenceDigest`, and `SessionCredentialEvidence` through its shadow struct — hand-write a `Deserialize` that reads the canonical primitive through that primitive's own implementation and hands the result to `parse`/`from_millis`/`new`. `SessionInstant` is the one scalar whose constructor is total, so its derived decode and `from_unix_millis` are the same function and it stays fully derived. Everything else derives both. A hand-written `Visitor`, `visit_*` method or `deserialize_any` remains forbidden everywhere.

**`Display` and `Error` belong to the wrapper, never to the kind.** `SessionValueError` and `SessionDomainError` are the two values a caller propagates, so they carry the rendering and the `std::error::Error` implementation. `SessionValueErrorKind` and `EventDerivedField` are payloads read by pattern matching and implement neither, exactly as `platform-identity/v0` §5 gives `IdentityValueError` a `Display` while `IdentityValueErrorKind` has none. Giving a kind enum its own `Display` would publish a second rendering path for the same failure and widen the public trait surface for nothing.

`SessionId`, `TenantId` and `UserId` bring their own derives from `platform-identity/v0` §4; B2 neither adds to them nor relies on any trait that contract does not already give them.

Four consequences of the table are load-bearing rather than incidental, and are stated so a reader does not have to re-derive them:

- **`Copy` closes downward.** `SessionDomainError` is `Copy` and carries `TerminalSession { status: SessionStatus }` and `EventDerivedFieldMismatch { field: EventDerivedField }`, so both of those are `Copy`; `SessionStatus` in turn carries only `SessionInstant` and `SessionExpiryCause`, so both of those are too. `Copy` also requires `Clone`, which is why every `Copy` row above lists it.
- **`PartialEq`/`Eq` close downward from `SessionSnapshot`.** §8's replay obligation is structural equality of every field, and it cannot be stated otherwise. Note that the snapshot stores adapter, digest, instants and durations *flattened* rather than holding a `SessionCredentialEvidence` or a `SessionPolicy`, so those two aggregates get `PartialEq`/`Eq` from the command and event families that do hold them, not from the snapshot.
- **No B2 type derives `Hash`, `Default`, `Deref` or `PartialOrd`/`Ord` beyond the four scalar rows above.** Nothing in this contract keys a map or sorts a session, and `Default` is prohibited outright by §10.
- **The error types are `Copy` and carry no borrowed data**, which is what lets §9 keep them small and non-echoing.

**Accessors are uniform, and they are a struct rule.** Every public B2 struct exposes exactly one read-only accessor per field, named exactly as the field, returning the field by value where its type is `Copy` and by shared reference otherwise. There is no setter, no `_mut`, no owned-field extraction, no whole-struct destructuring accessor and no accessor that returns a field not named above. The seven public enums expose no accessors at all: their variants and payloads are read by pattern matching, which is what makes the closed-set property of §2.0 legible at the use site.

The only public **methods** — associated functions taking a `self` receiver — that are not field accessors are `SessionSnapshot::admits_at`, the `as_str`/`as_millis`/`as_unix_millis` readers named in §§2.1–2.2, the `value_kind()`/`kind()` readers of §9.1, and the three uniform projections each of `SessionCommand` and `SessionEvent` in §§4.1 and 5.1. The constructors of §2.0's rule 1 take no receiver and are therefore not methods; that section is where they are enumerated, and no public associated function exists outside these two lists.

**`SessionStatus`'s Serde representation is frozen too**, because it is a public serialized read model and leaving one of the three serialized shapes unfrozen while freezing the other two would be arbitrary. It is externally tagged with `#[serde(rename_all = "snake_case")]`, so `Active` serializes as the string `"active"` and the two struct variants as `{"expired": {...}}` and `{"revoked": {...}}`, with their fields under exactly the names §2.4 lists.

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

The four command structs have private fields and exactly one constructor each; the `SessionCommand` wrapper enum is §2.0's third construction case and §4.1 states why its variant payload is public. No command carries a raw credential, arbitrary metadata map, downstream permission, UI state, database handle or framework session object. §4.1 freezes the exact Rust shape.

`expected_revision` is optimistic-concurrency intent. B2 validates it during pure decision; B4 later binds the same value to journal/repository compare-and-append. B2 does not claim that an in-memory decision alone persisted anything.

The command set deliberately has no generic `SetState`, `Touch`, `Patch`, `Restore`, `Unexpire` or `Unrevoke` operation.

`ExpireSession` is an M00-internal lifecycle command issued only through the future B4 session application/port path. It is not an inbound cross-module command, is never decoded directly from M10, and does not expand the M00 blueprint's public input list.

For an accepted open, `SessionOpened.opened_at` is exactly `OpenSession.observed_at`; the evolved snapshot initializes `last_transition_at` to that same instant.

### 4.1 Frozen public algebra

The semantic set above is realized as exactly four named-field structs plus one wrapper enum:

```text
struct OpenSession    { session_id, credential_evidence, policy, observed_at, expected_revision }
struct RefreshSession { session_id, observed_at, expected_revision }
struct ExpireSession  { session_id, observed_at, expected_revision }
struct RevokeSession  { session_id, observed_at, expected_revision }

enum SessionCommand {
    Open(OpenSession),
    Refresh(RefreshSession),
    Expire(ExpireSession),
    Revoke(RevokeSession),
}
```

**Structs, not tuple structs; wrapper variants, not inlined fields.** §2.0's named-field rule is stated for every public B2 struct, so it binds the four command structs whether or not they validate anything. It does **not** extend to `SessionCommand`'s variants, which are single-field tuple variants, and the reason is not that a variant constructor is somehow unlike a tuple-struct constructor — it is a bindable function value in the same way. The reason is that §2.0's objection is specifically to *filling a private field* without writing a construction expression, and an enum variant has no private field to fill: its payload is public, as the next paragraph says, and the payload's own privacy is enforced one level down by `OpenSession` itself. A bound `SessionCommand::Open` can therefore produce nothing that its argument did not already legitimize.

Inlining each command's fields into the enum instead would be the real loss: it would erase the four nominal types whose field sets §16 freezes, and it would put private fields directly under a tuple-shaped constructor, which is the combination §2.0 rules out.

**Payload reachability is intended and is not a hole.** Enum variant fields are public in Rust, so a caller may both build `SessionCommand::Open(open)` from a command it validly constructed and match a `&SessionCommand` back down to `&OpenSession`. Neither bypasses anything: the payload's own fields stay private and the wrapper carries no invariant. This contract therefore makes no claim of a "private variant" anywhere, because Rust has none to offer.

**Constructors are total**, taking fields in the order listed:

```text
OpenSession::new(session_id: SessionId, credential_evidence: SessionCredentialEvidence,
                 policy: SessionPolicy, observed_at: SessionInstant,
                 expected_revision: u64) -> Self
RefreshSession::new(session_id: SessionId, observed_at: SessionInstant,
                    expected_revision: u64) -> Self
ExpireSession::new(session_id: SessionId, observed_at: SessionInstant,
                   expected_revision: u64) -> Self
RevokeSession::new(session_id: SessionId, observed_at: SessionInstant,
                   expected_revision: u64) -> Self
```

Every argument is an already-validated nominal value or a `u64` denoting an instant or a revision claim, so nothing a constructor could reject remains — for three of the four. `RefreshSession`, `ExpireSession` and `RevokeSession` carry only a session identity, an instant and a revision claim, and every question about them (is the revision current? is the observation stale? is the session terminal?) is a fact about the *aggregate*, which a constructor holding one command cannot see. §7 decides each at the only point where the aggregate is in hand.

**`OpenSession` is the case that needs an argument, and it is a precedence argument rather than a decidability one.** §3's open failures — `opened_at < authenticated_at`, an already-expired `credential_not_after`, checked-add overflow, a derived deadline that is not strictly future — are all computable from `OpenSession`'s own fields, so a checked constructor *could* reject them. It must not, for two reasons that point the same way:

1. **It would invert §7.** That section's open precedence puts `SessionAlreadyExists` second and `RevisionMismatch` third, ahead of time ordering, credential expiry and overflow arithmetic. A constructor necessarily runs before all of them, because the command must exist before it can be decided against an aggregate. So a checked `OpenSession::new` would answer a lower-precedence question first, and an input that is both stale-timed *and* aimed at an existing session would report the wrong one of the two — exactly what §7's closing rule forbids: "No lower-precedence failure may hide a higher-precedence one."
2. **It would split one failure across both §9 taxonomies.** `InvalidTimeOrder`, `CredentialEvidenceExpired` and `DeadlineOverflow` are `SessionDomainError` variants. Reporting them from a constructor means either a value constructor returning a domain error, or duplicate variants in `SessionValueError` — and §9 exists precisely to keep the two taxonomies split along §7's precedence line.

So all four constructors are declared `-> Self`, and §8's open evolution re-derives every one of those invariants from the event, which §8 already requires. This is a deliberate placement of a reachable check, not a claim that nothing is checkable.

**Accessors and traits** are §2.5's uniform rule and command rows, applied here without exception: one read-only accessor per field, named exactly as the field, by value for the `Copy` field types and by shared reference for `SessionId`, `SessionCredentialEvidence` and `SessionPolicy`.

`SessionCommand` additionally exposes the three projections §7's precedence needs uniformly across command kinds, so identity and revision checks are written once rather than per variant:

```text
SessionCommand::session_id(&self) -> &SessionId
SessionCommand::observed_at(&self) -> SessionInstant
SessionCommand::expected_revision(&self) -> u64
```

`OpenSession`'s derived `Debug` is safe despite the credential evidence it carries: it delegates, ultimately reaching `CredentialEvidenceDigest`'s hand-written redacting `Debug` (§10), so the digest is redacted once, at the value that *is* the digest, rather than separately at every holder.

**Serde representation.** `SessionCommand` is externally tagged — serde's default, stated because it is frozen rather than incidental — and carries `#[serde(rename_all = "snake_case")]`, so its variants serialize under exactly the tags `open`, `refresh`, `expire` and `revoke`. Each command struct carries `#[serde(deny_unknown_fields)]` and serializes its fields under their exact names.

**Commands implement `Serialize` but not `Deserialize`.** §10 admits Serde only where command handling and event *replay* need it, and replay reads events, never commands. Making that explicit converts a rule someone must remember into one the compiler enforces: §2.2 requires that untrusted callers cannot invoke `OpenSession` with self-asserted evidence, and with no `Deserialize` there is no way to decode one from a transport payload at all. A future ingress maps its own validated DTO through these constructors, which is where the admission decision belongs.

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

Every event sequence is the exact next positive integer. The four event structs have private fields; the `SessionEvent` wrapper enum is §2.0's third construction case, on §4.1's terms. External callers cannot construct an accepted snapshot by setting fields directly.

Events implement both `Serialize` and `Deserialize`, because replay reads them back. §5.1 freezes the exact Rust shape.

A `sequence` is **not** validated at construction. Whether a sequence is the exact next integer is a question about the aggregate, not about the event, and §8 answers it with `EventSequenceMismatch`; there is deliberately no `SessionValueErrorKind` variant for a zero or out-of-order sequence, because a value-level check could only ever repeat what the aggregate must decide anyway.

Events retain only bounded provenance. They never contain raw credentials, secret values, cookies, authorization headers, provider payloads, arbitrary reason strings or client-supplied metadata.

`SessionRefreshed.effective_expires_at` plus `SessionExpired.expired_at` and `cause` are redundant verification fields: evolution recomputes them from prior state and rejects a mismatch. `expired_at` is the effective deadline at which the session became invalid, while `observed_at` is when expiry was detected and persisted; a late observation must not rewrite historical validity. Their presence makes persisted evidence inspectable without allowing a forged derived value to become authority.

Final stable platform-facing event/error names and reason codes remain owned by B4 `control-evidence`; these domain events are the state-machine authority that such projections must map one-to-one.

### 5.1 Frozen public algebra

```text
struct SessionOpened    { sequence, session_id, credential_evidence, policy, opened_at }
struct SessionRefreshed { sequence, session_id, observed_at, effective_expires_at }
struct SessionExpired   { sequence, session_id, observed_at, expired_at, cause }
struct SessionRevoked   { sequence, session_id, observed_at }

enum SessionEvent {
    Opened(SessionOpened),
    Refreshed(SessionRefreshed),
    Expired(SessionExpired),
    Revoked(SessionRevoked),
}
```

Topology, payload reachability and the named-field rule are exactly as §4.1 states for commands, for the same reasons.

**All four constructors are total**, taking fields in the order listed:

```text
SessionOpened::new(sequence: u64, session_id: SessionId,
                   credential_evidence: SessionCredentialEvidence,
                   policy: SessionPolicy, opened_at: SessionInstant) -> Self
SessionRefreshed::new(sequence: u64, session_id: SessionId,
                      observed_at: SessionInstant,
                      effective_expires_at: SessionInstant) -> Self
SessionExpired::new(sequence: u64, session_id: SessionId, observed_at: SessionInstant,
                    expired_at: SessionInstant, cause: SessionExpiryCause) -> Self
SessionRevoked::new(sequence: u64, session_id: SessionId,
                    observed_at: SessionInstant) -> Self
```

**`SessionExpired`'s totality is a decision, not an omission.** A draft revision gave it a checked constructor rejecting `observed_at < expired_at`. That is withdrawn. It opened an error channel belonging to neither §9 taxonomy — `SessionValueErrorKind` has no variant for it, and `SessionDomainError` is the decision/evolution taxonomy, not a construction one — and the available repairs were all worse than the defect: a new value-error variant for one event's self-consistency, a third error type, or a Serde-only message with no typed peer.

It is withdrawn on a stronger ground than taxonomy tidiness, and the ground is **not** that the comparison is undecidable — `observed_at >= expired_at` compares two `u64`-backed fields of one struct and is perfectly decidable. It is that the comparison is decidable *without being meaningful*, which is §2.0's second constructor rule. `expired_at` is a **derived** field whose correctness means "exactly equals the aggregate's pre-existing effective deadline" (§8). An event holding two caller-supplied instants does not know that deadline, so a constructor comparing them enforces agreement between two numbers that may both be forged, while admitting every pair that is consistently wrong.

What that check costs is concrete. §13 requires a persisted `SessionExpired` whose `observed_at` precedes the effective deadline, with sequence, `SessionId` and **every derived field exact**, to be rejected as `EventTimeOutsideValidity`. Exactness of `expired_at` is what makes that fixture the load-bearing one, and it forces `observed_at < expired_at` — so a checked constructor makes precisely that fixture unconstructible. The claim is not that the `EventTimeOutsideValidity` arm would be wholly dead: §13's forged-*low* `expired_at` case reaches it from the other side, and §8 reaches it through `SessionRefreshed` and `SessionRevoked` as well. The claim is narrower and sufficient — only an event with an exact `expired_at` can show that guard 1 rejects on its own rather than being shadowed by guard 2, and a checked constructor is the one thing that removes that event from the input space.

The invariant is enforced where it is decidable. §8's `Active`/`SessionExpired` guards run in exactly this order, so two implementations report the same failure for a multiply-forged event:

1. `observed_at >= effective_expires_at`, else `EventTimeOutsideValidity`;
2. `expired_at == effective_expires_at`, else `EventDerivedFieldMismatch { ExpiredAt }`;
3. `cause` equals the §3 tie-precedence result, else `EventDerivedFieldMismatch { ExpiryCause }`.

Together those imply `observed_at >= expired_at` for every event evolution accepts, which is the property the withdrawn constructor was reaching for — obtained from the aggregate that can actually establish it.

**No event owns a cross-field invariant, so no event needs §2.0's shadow-struct decode**; `SessionCredentialEvidence` is the only value in this contract that does, and it qualifies because its credential window is a relation between two fields the value fully owns.

**Owning one is narrower than computing one, and `SessionOpened` is the case that shows the difference.** It carries `credential_evidence`, `policy` and `opened_at`, which is exactly enough to evaluate all four of §3's open failure conditions — `opened_at < authenticated_at`, an already-expired `credential_not_after`, checked-add overflow, and a derived deadline that is not strictly later than `opened_at`. Those relations are computable from the struct alone, and this contract says so plainly rather than claiming there was nothing here to check. The constructor is nonetheless total, for the two reasons §4.1 gives for `OpenSession`, in the same order:

1. **It would invert §8.** For an empty aggregate, evolution requires `sequence == 1` before it revalidates any open invariant. A constructor necessarily runs before evolution sees an aggregate at all, so a checked `SessionOpened::new` would answer the lower-precedence open-invariant question ahead of `EventSequenceMismatch` — the same inversion §4.1 refuses for `OpenSession` against §7.
2. **It would make §8's own re-derivation unfalsifiable.** §8 requires evolution to revalidate every §2–§3 open invariant *from the persisted event* rather than trust it, and §9.2 keeps `InvalidTimeOrder`, `CredentialEvidenceExpired` and `DeadlineOverflow` as reachable variants. A checked constructor removes from the input space every event that could exercise that requirement — and by §2.0's shadow-struct rule it would close the Serde path too, since it would then own a cross-field invariant — leaving §8's obligation untestable and those three variants dead on the evolve path. That is precisely the dead-arm defect §9.2 refused when it removed `InvalidCredentialEvidence` and `InvalidSessionPolicy`. `evolve` owns the domain-error taxonomy, and a malformed persisted `SessionOpened` must remain constructible so that fail-closed evolution can be shown rejecting it.

The remaining three events own nothing to check. `SessionRevoked` carries no derived field at all. `SessionRefreshed` and `SessionExpired` each carry one, and a relation between a derived field and an instant beside it is decidable without being meaningful — §2.0's second rule, worked in full for `SessionExpired` above and identical in shape for `SessionRefreshed`, whose `effective_expires_at` is correct only against a deadline the event cannot see.

Events therefore derive `Deserialize` with `#[serde(deny_unknown_fields)]`. That is not an *unchecked* decode in §10's sense: every field is a `u64` or a nominal value whose own `Deserialize` delegates to its checked constructor, so the invalid-value half is closed field by field and `deny_unknown_fields` closes the unknown-field half. What remains — sequence order, cross-session identity, derived-field agreement, event time inside the guard window — is an aggregate question, and §8 answers all of it.

**Accessors** follow §4.1: one read-only accessor per field, named exactly as the field, by value for `Copy` fields and by reference otherwise. `SessionEvent` exposes the three projections §8 needs uniformly:

```text
SessionEvent::sequence(&self) -> u64
SessionEvent::session_id(&self) -> &SessionId
SessionEvent::observed_at(&self) -> SessionInstant
```

`observed_at` maps `SessionEvent::Opened` to that event's `opened_at`, which §4 already fixes as exactly the open command's `observed_at`.

**Accessors and traits** are the §2.5 rows for the event family; nothing here adds to or departs from that closure. `SessionOpened`'s derived `Debug` is safe by the same delegation §4.1 states for `OpenSession`.

**Serde representation.** `SessionEvent` is externally tagged with `#[serde(rename_all = "snake_case")]`, so its variants serialize under exactly `opened`, `refreshed`, `expired` and `revoked`. `SessionExpiryCause` carries the same `rename_all`, giving exactly `credential`, `absolute` and `idle`. Each event struct carries `#[serde(deny_unknown_fields)]` and serializes its fields under their exact names.

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

1. malformed value shape is rejected by the §2 value constructors before a command can be built, and so before decision — the command constructors themselves are total (§4.1) and reject nothing;
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

1. malformed evidence/policy/value shape, rejected by the §2 value constructors before `OpenSession::new` is reachable;
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

`SessionExpired.observed_at` is greater than or equal to `SessionExpired.expired_at` for every event this table accepts. That is a *consequence* of the three guards in the `SessionExpired` row, run in the order §5.1 fixes, not a fourth check and not a constructor invariant: the first guard puts `observed_at` at or after the effective deadline and the second pins `expired_at` to that same deadline. §5.1 states why the property is enforced here rather than at construction. The refresh and revoke apply guards use the same effective-expiry function as command decision, and the expire guard additionally uses the same §3 cause function; evolution may not accept a persisted event that `decide` could never have emitted. No event can replace tenant/user/session/adapter/evidence/policy scope or change a terminal state.

**An apply-guard violation has its own named failure.** A forged or corrupted event can carry an exact sequence, an exact `SessionId`, a forward `observed_at` and — for `SessionRevoked`, which has no derived field at all — nothing else to check, while still sitting on the wrong side of the effective deadline for its kind. None of `EventSequenceMismatch`, `SessionIdMismatch`, `NonMonotoneTime` or `EventDerivedFieldMismatch` describes that, and `IllegalEventForState` would contradict this table, which lists the pair as legal. It is `EventTimeOutsideValidity`, and it covers all three `Active` rows: a `SessionRefreshed` or `SessionRevoked` whose `observed_at` is at or after the effective deadline, and a `SessionExpired` whose `observed_at` precedes it.

So the complete fail-closed set for evolution is: revision exhaustion (`RevisionOverflow`), a gap, a duplicate sequence, an out-of-order event (`EventSequenceMismatch`), a cross-session event (`SessionIdMismatch`), a backward event time (`NonMonotoneTime`), an illegal event/state pair (`IllegalEventForState`), an event time outside the guard's validity window (`EventTimeOutsideValidity`), a forged derived field (`EventDerivedFieldMismatch`), a persisted refresh whose recomputed deadline does not strictly advance (`NoEffectiveRefresh`), a persisted refresh whose own checked deadline arithmetic overflows (`DeadlineOverflow`), or a failed open invariant (`InvalidTimeOrder`, `CredentialEvidenceExpired`, `DeadlineOverflow`). Each returns no partial snapshot.

The last two are reachable on this path and are named here rather than folded into "a forged derived field", because reporting a forged field for either would be false. `NoEffectiveRefresh` answers when the recomputed deadline does not advance — under a policy whose idle candidate is already clipped by the absolute or credential cap, the event's own `effective_expires_at` may equal that recomputation exactly, so nothing was forged and the refresh is simply impossible. The `Active`/`SessionRefreshed` guards therefore run in a fixed order — validity window, then strict advance, then exact agreement with the recomputed deadline — and two implementations report the same failure for a multiply-invalid event.

While a session is `Active`, `effective_expires_at > last_transition_at` is an invariant: open requires strictly-future derived deadlines and a credential deadline later than `opened_at`, and refresh requires both `observed_at < effective_expires_at` and a strictly advancing deadline. This is what makes §7's ordering of non-monotone time before time-derived expiry unambiguous rather than merely conventional — `observed_at < last_transition_at` implies `observed_at < effective_expires_at`, so the two conditions have an empty overlap and can never compete.

Replaying the same validated `SessionOpened` plus the same ordered events must reconstruct a structurally equal `SessionSnapshot`: every field compares equal. Replay never reads the current clock, reloads policy, resolves a credential, calls an adapter or writes evidence. A canonical persisted byte encoding and checksum are deferred to B4's journal contract; B2 does not invent a format-independent “canonical Serde byte representation”.

A duplicate event sequence is an error, not an apply-time no-op. B4's journal may recognize an identical append retry under its own accepted contract, but it must not ask the domain aggregate to apply the same sequence twice.

## 9. Typed failures and diagnostic safety

There are exactly two taxonomies, split along §7's own precedence: value construction is rejected **before** decision, so a shape failure cannot reach the state machine and the state machine carries no shape variant.

### 9.1 Construction — `SessionValueError`

`SessionValueError` mirrors `IdentityValueError`: a small `Copy` value carrying exactly the static Rust value-kind name that rejected the input and one `SessionValueErrorKind`, with private fields, no public constructor per §2.0's rule 2, exactly two read-only accessors `value_kind()` and `kind()`, and no `source`. `SessionValueErrorKind` is the public enum owning exactly:

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

`EventTimeOutsideValidity` is the §8 apply-guard failure: the event's `observed_at` is on the wrong side of the effective deadline for that event kind. It is payload-free — the two instants involved are already in the caller's own snapshot and event. It is also the variant that carries `SessionExpired`'s `observed_at >= expired_at` property, which §5.1 declines to check at construction because an event alone cannot know the deadline that makes the comparison meaningful. No third error channel exists for that case, and none may be introduced for it.

The draft revision of this contract also listed `InvalidCredentialEvidence` and `InvalidSessionPolicy` here. They are deliberately **not** domain variants: `SessionCredentialEvidence` and `SessionPolicy` have validating constructors and validating Serde, so a malformed one cannot be built, cannot be deserialized and therefore cannot reach `decide` or `evolve` — including from a persisted event, whose fields are already those types. Retaining an unreachable variant would be a permanent dead arm that no adversarial fixture could exercise, and §13 requires every fixture to affect an executable assertion. The corresponding real failures are `SessionValueErrorKind::MalformedDigest`, `ZeroDuration` and `CredentialWindowNotAfterAuthentication` in §9.1.

`EventDerivedField` is a closed non-secret enum containing exactly `RefreshEffectiveExpiresAt`, `ExpiredAt` and `ExpiryCause`. The error does not carry arbitrary field names, source payloads or rendered event values.

`CredentialEvidenceExpired` has no credential/evidence payload. `InvalidTimeOrder` covers `opened_at < authenticated_at`; `DeadlineOverflow` covers checked deadline arithmetic. §3's further open condition — that the derived absolute and idle deadlines be strictly later than `opened_at` — is an invariant that both `decide` and `evolve` assert, not a separate reachable variant: with `SessionDuration` non-zero by construction and every addition checked, the only way to fail it is the overflow `DeadlineOverflow` already names. It is stated as an invariant so evolution has an explicit postcondition to re-derive rather than trust, not so that a dead error variant exists.

Errors are small typed values. They may report a failure kind, safe expected/actual revision and terminal status. They must not retain or render:

- credential/token/cookie/password text;
- a provider subject or arbitrary adapter payload;
- rejected secret-derived bytes or fragments;
- a complete serialized evidence value;
- arbitrary caller-provided reason text.

**That guarantee is scoped to the diagnostics B2 produces, and the boundary is stated rather than left implicit.** It binds every `SessionValueError` and `SessionDomainError`, every `Display` and `Debug` rendering of a B2 value, and the validation mapping each `Deserialize` applies when it hands a decoded primitive to a checked constructor. It does **not** bind the deserializer's own syntax and type diagnostics, and cannot: because every decode reads the canonical primitive first, a payload of the wrong shape is rejected by the serde implementation before any B2 validator runs, and `deny_unknown_fields` names the field it refused. Those messages are produced outside this contract's values, and §10's prohibition on a hand-written `Visitor` means B2 has no interception point at which to rewrite them.

They are therefore **untrusted boundary diagnostics**, not B2 errors, and carrying them across a trust boundary is a redaction obligation assigned to `M00-B4 control-evidence` together with whatever transport surface renders it: that batch owns the stable external error projection, and it must map a decode failure to a bounded reason code rather than forward the deserializer's message. No `AUTH-017..020` row asserts anything about those messages, and none may be read as doing so.

Failed commands and failed event application leave the previous snapshot unchanged and produce no partial event/snapshot.

## 10. Serde and public API negative space

The exact B2 Serde surface is:

- nominal scalar values, `SessionCredentialEvidence`, `SessionPolicy` and events implement validating `Serialize` and `Deserialize`, which is what event replay needs;
- commands implement `Serialize` only, per §4;
- deserialization delegates to the same decision-independent value validators; unknown and missing fields fail closed;
- `SessionSnapshot` is a serialization-only read model: it implements no public `Deserialize`, `Default` or direct constructor, and can arise only from validated `SessionOpened` evolution plus legal replay;
- `SessionStatus` is serialization-only in the same sense — no `Deserialize`, no `Default` — but it is a public enum, so a caller may name a variant, and this contract does not pretend otherwise. §2.4 states the property that actually holds: what cannot be forged is a `SessionSnapshot` carrying a caller-chosen status, and the snapshot's private fields, absent constructor and absent `Deserialize` are what close it;
- `SessionValueError`, `SessionValueErrorKind`, `SessionDomainError` and `EventDerivedField` implement neither `Serialize` nor `Deserialize`; B4 owns stable external error/event projections.

Mechanically that means `#[serde(deny_unknown_fields)]` on every aggregate struct, `#[serde(rename_all = "snake_case")]` on every enum **that derives Serde** — the three error enums derive none, and a bare `serde` attribute without the derive in scope does not compile — and no hand-written visitor anywhere, exactly as §2.0 requires. A derived **unchecked** field decode on an authority-bearing value is forbidden: `deny_unknown_fields` closes the unknown-field half, and delegation to the checked constructor closes the invalid-value half. Neither closes the other, so both are required.

"Unchecked" is the operative word, and §§4.1 and 5.1 make the boundary exact rather than leaving it to judgement. A derived decode whose every field is itself a validating `Deserialize` *is* checked, field by field, and is what events use. What is forbidden is a decode that reaches a private field without passing that field's own validator, and a derived decode of an aggregate whose invariant spans fields — which §2.0 routes through a shadow struct instead, for the one value that has one.

A decode failure carries two kinds of message, and only one is B2's. A value this contract owns rejected through its checked constructor renders a `SessionValueError` under §9's non-echo rule; a payload the deserializer itself refused — wrong JSON type, malformed syntax, unknown field — renders that deserializer's diagnostic, which B2 neither produces nor can suppress. §9 assigns the second class to `M00-B4 control-evidence` as an untrusted boundary diagnostic. A consumer must not forward it verbatim across a trust boundary.

Serde proves only shape and invariant validity, never credential authenticity or caller admission. B3/B4/B5 composition must not expose `OpenSession` deserialization as an untrusted transport endpoint. Derived unchecked decoding is forbidden on every authority-bearing value.

External compile-fail/API proofs must show that callers cannot:

- construct or mutate snapshots/events through public fields;
- set `revision`, deadlines, status or expiry cause directly on a `SessionSnapshot` or an event struct;
- replace tenant/user/session/evidence/policy after open;
- default a session into an active state;
- convert one identity kind into another;
- obtain mutable backing access;
- bypass a value's validation through public fields or a second, unchecked constructor;
- pass raw credential text to any B2 evidence constructor or conversion API;
- call a public unchecked constructor or generic state setter.

"Unchecked constructor" here means a second construction path on a value that *has* an invariant — an `AuthAdapterId` or `SessionCredentialEvidence` reachable without its validator. It does not refer to the deliberately total command and event constructors of §§4.1 and 5.1, which are each the single path to their value and skip no check that belongs to them; §§4.1 and 5.1 record where those checks live instead.

Read-only, zero-copy accessors are allowed. `Display` for client/log use must not expose credential evidence internals.

**`Debug` redaction is owned by B2, not deferred.** The obligation is a property of the whole module: no `Debug` rendering of any B2 value may contain the digest bytes. It is discharged at exactly one place — `CredentialEvidenceDigest` itself implements `Debug` by hand and renders a fixed redaction token instead of its bytes. Every value that can reach a digest reaches it only through that type, so `SessionCredentialEvidence`, `SessionSnapshot`, `OpenSession`, `SessionOpened`, `SessionCommand` and `SessionEvent` all derive `Debug` and inherit the redaction, which is why §2.5 lists a derived `Debug` for each of them.

Redacting at the digest rather than at its holders is the point, not a convenience. A hand-written `Debug` repeated at each holder is one forgotten holder away from leaking, and the set of holders grows with every later batch; a redaction on the type itself cannot be forgotten by a holder that has not been written yet. `Display` is not implemented on `CredentialEvidenceDigest` at all, so there is no second rendering path to keep in step. An earlier revision deferred this to B4 `control-evidence`, which does not exist; that would have left an implementer with no rule to satisfy while the repository's default convention is a derived `Debug`, and a derived `Debug` prints the digest on every `assert_eq!` failure, panic message and trace line. It would also have made B2 weaker than its own sibling: `AUTH-014` governs B1's identity errors on `Display` **and** `Debug`. B4 may later widen redaction to further surfaces; it is not a prerequisite for this one.

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
- the admitted item declarations, with `pub mod session;` in `lib.rs` and the session module's own complete item list. That list is its five `use` items **plus the two items the §12 library fixtures introduce**: the inline `#[cfg(test)] mod tests` and its `use super::*;`. Both are ordinary accounted items, not an exemption — the module-declaration carrier named in the bullet above governs *non-inline* `mod name;` declarations only, and `session.rs` still admits none of those, so the checker forbids the file-backed form outright while positively requiring exactly one registered inline fixture module. Neither carrier is loosened to admit them: the fixture module carries no `pub`, so the public-declaration allowlist is unchanged by its presence, and its `#[test]` and `#[cfg]` attribute names join the per-source attribute allowlist below by enumeration;
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

`AUTH-017`, `AUTH-018`, `AUTH-019` and `AUTH-020` are active rows in [`../acceptance/matrix.tsv`](../acceptance/matrix.tsv) at gate `pr`, carrying exactly the bindings below. All four are `implemented`; §17 records the evidence and the two §13 entries whose fixtures are library-target rather than integration tests. They are simultaneously retained in the long-horizon catalog [`platform-baseline.md`](../acceptance/platform-baseline.md), which is a catalog and confers no currency by itself.

`planned` is a non-pass state, and no row here carries it any longer: each named test exists, runs unconditionally and passes. This section remains the specification those tests satisfy, and §17 is the record of what was proved.

| Case | Binding |
|---|---|
| `AUTH-017` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_open_pins_immutable_scope_and_checked_deadlines -- --exact` |
| `AUTH-018` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_lifecycle_precedence_is_deterministic_and_terminal -- --exact && cargo test --locked -p ustc-campus-agent-core --lib session::tests::terminal_precedence_holds_at_the_revision_ceiling -- --exact` |
| `AUTH-019` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_revision_and_replay_are_exact_and_fail_closed -- --exact && cargo test --locked -p ustc-campus-agent-core --lib session::tests::revision_ceiling_fails_closed_on_decide_and_evolve -- --exact` |
| `AUTH-020` | `python3 scripts/check_repo_contracts.py && cargo test --locked -p ustc-campus-agent-core --test platform_session session_domain_has_no_credential_or_adapter_surface -- --exact && cargo test --locked -p ustc-campus-agent-core --doc session` |

Each binding runs the repository checker before its Rust leg for the reason `platform-identity/v0` §4 gives: redirecting a `[[test]]` target or renaming a bound function makes `--exact` match nothing, which cargo reports as `running 0 tests` at exit zero, and a guard written inside the suite is exactly what such a change replaces. Only an out-of-band carrier detects that, so the checker is part of each binding rather than a courtesy check.

`AUTH-020`'s `--doc` leg is covered by CI's separate unconditional `cargo test --locked --all-features --doc` step, since `--all-targets` does not run doctests.

**`AUTH-018` and `AUTH-019` each carry a second exact leg against the library target**, and the reason is a property of this contract's own algebra rather than a preference. §2.4 gives `SessionSnapshot` no public constructor, §10 gives it no `Deserialize`, and §8 sets `revision` only to `current + 1` from a base of `1`, so an aggregate at `revision == u64::MAX` is not reachable from an integration test at any feasible cost — while §7 item 5 and §8's checked increment both have observable behaviour there that a wrapping increment would silently change. §13 therefore binds those cases to private `#[cfg(test)]` fixtures inside `crates/platform-core/src/session.rs`, which may build that snapshot directly because they are in the module that owns the fields, and which then call the real `decide` and `evolve`. The fixtures add no public or feature-gated hook, change no production item, and are frozen by the same surface accounting as the rest of the module — a `pub` on any of them fails the public-declaration allowlist, and `#[cfg(test)]`/`#[test]` are the only attribute names the registration admits beyond those the production code already uses.

### `AUTH-017` — immutable open scope and deadline algebra

Prove that open pins exact tenant/user/session/adapter/evidence/policy scope; derives separate idle, policy-absolute and optional credential deadlines with checked arithmetic; rejects stale evidence/time/overflow; and validating Serde rejects incomplete/unknown input.

### `AUTH-018` — refresh/expire/revoke precedence

Prove that refresh extends only idle expiry, never credential/policy-absolute expiry or scope; equality is expired; `Credential > Absolute > Idle` resolves equal deadlines; late observation preserves the effective `expired_at`; expired sessions cannot refresh/relabel; revoke blocks immediately; terminal states cannot mutate or resurrect; and `admits_at` answers current admission under §2.4's three conjuncts, refusing a revoked session, a stale observation and the expiry boundary alike.

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
- decision and evolution at revision `u64::MAX`, including a forged wrapped sequence `0` that a wrapping increment would accept as the next sequence — proved by the private same-module fixtures §12 binds to `AUTH-019`'s library leg, since no reachable public call sequence produces that aggregate;
- equal, backward and forward observed instants;
- event sequence gap, duplicate and reorder;
- cross-session event injection, and a decide-side `SessionIdMismatch` where the command names a different session than the supplied state;
- forged refreshed deadline, effective `expired_at` and expiry cause;
- a persisted `SessionRefreshed` and a persisted `SessionRevoked` whose `observed_at` is at or after the effective deadline, and a persisted `SessionExpired` whose `observed_at` precedes it — each `EventTimeOutsideValidity`, each with sequence, `SessionId` and every derived field otherwise exact. Each must also satisfy `observed_at >= last_transition_at`, or §8's earlier universal non-decreasing-time check answers `NonMonotoneTime` first and the fixture proves nothing about the row guard it was written for. That constraint is satisfiable in every case because the `Active` invariant leaves the window `[last_transition_at, effective_expires_at)` non-empty;
- that same `SessionExpired` reached two ways, proving §5.1's totality decision rather than assuming it: **direct construction** with `observed_at` strictly less than `expired_at`, which must *succeed* because the constructor is total, and **deserialization** of the byte-equal payload, which must succeed identically — with both results then rejected by `evolve` as the same `EventTimeOutsideValidity`. A test that only exercised the evolve path would still pass if an implementer quietly reintroduced a fallible constructor and a third error channel;
- a `SessionExpired` whose `expired_at` is forged *below* the true effective deadline, with `last_transition_at <= observed_at < effective_expires_at` and `observed_at` above the forged value — which the total constructor also admits, and which `evolve` rejects as `EventTimeOutsideValidity` rather than `EventDerivedFieldMismatch`, pinning §5.1's guard order. Without the `last_transition_at` bound this case is reachable through `NonMonotoneTime` instead and pins nothing;
- conversely, reaching `EventDerivedFieldMismatch { ExpiredAt }` or `{ ExpiryCause }` at all requires `observed_at >= effective_expires_at`, since guard 1 answers first otherwise; the forged-derived-field fixtures above must be built that way or they silently test guard 1 twice;
- late expiry observation that must retain the earlier effective deadline;
- expiry observed exactly at and strictly after each deadline, with equal derived `expired_at` but distinct `observed_at`;
- a refresh whose own deadline arithmetic overflows because `effective_expires_at` sits at `u64::MAX`, returning `DeadlineOverflow`;
- deserialization of a `SessionCredentialEvidence` payload whose `credential_not_after` is not strictly later than `authenticated_at`, rejected as `CredentialWindowNotAfterAuthentication` — this is the fixture that proves §2.0's shadow-struct rule is in force and that §9.2's removal of `InvalidCredentialEvidence` was safe;
- deserialization of a `SessionCredentialEvidence` payload that **omits** `credential_not_after` entirely, which must be rejected, alongside one that spells it as an explicit null, which must be accepted as "no credential deadline". These are the fixtures for §2.2's downgrade-by-omission rule, and they fail against the bare derive rather than against a plausible mistake: serde's default for a missing `Option` field is `None`, so an implementation that skips the explicit attribute passes every other fixture in this list;
- round-tripping a `SessionCredentialEvidence` that has no credential deadline, confirming the serialized form writes the field rather than skipping it;
- `admits_at` false for a revoked session at an instant strictly before its preserved `effective_expires_at`, and for an expired one;
- `admits_at` false for an `Active` snapshot observed strictly before `opened_at`;
- `admits_at` false for an `Active` snapshot that has been refreshed, observed strictly between `opened_at` and `last_transition_at` — the stale-time case a status-plus-upper-bound predicate wrongly admits, and the one that distinguishes §2.4's three-conjunct rule from the two-conjunct one;
- `admits_at` at both boundaries of an `Active` snapshot: true at exactly `last_transition_at`, false at exactly `effective_expires_at`;
- replay after each legal prefix and across the full lifecycle;
- dual-fault precedence cases; the reachable orientations belong to the integration tests, and the one that is not reachable — a terminal session whose `current_revision` is `u64::MAX`, which must return `TerminalSession` and not `RevisionOverflow` — is proved by the private same-module fixture §12 binds to `AUTH-018`'s library leg;
- secret-like canary strings absent from every **B2-owned** error, `Display` and `Debug` surface, and from every serialized event/snapshot surface where forbidden — `Debug` included because §10 makes its redaction a B2 obligation. §9 scopes this to the diagnostics B2 produces: a deserializer's own syntax or type message is an untrusted boundary diagnostic owned by `M00-B4 control-evidence`, so no fixture here asserts anything about it and no acceptance row may be read as covering it.

**Two entries above name a private same-module fixture rather than an integration test, and that is a deliberate amendment rather than a concession.** An acceptance floor entry that no test can express is worse than a missing one: it leaves an implementer nothing to satisfy and a reviewer nothing to measure, which is the defect §15's third round already corrected once elsewhere in this contract. A source-text assertion — that one guard is written above another, or that a call site spells `checked_add` — is not an acceptable substitute either, because it proves the shape of the code rather than the behaviour of the algebra. The realizable shape is the one that runs the real `decide` and `evolve` against the state in question, and the only way to hold that state is from inside the module that owns the fields. Those fixtures are held to the same standard as every other entry here: each must fail when its production guard or guard order is broken.

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

**This section is the historical record of the acceptance-entry gate, stated as it stood when the contract was accepted.** Its conditions describe that moment, not the present: §17 is the current-state carrier, and where the two differ §17 is the later fact. The eight conditions this contract carried while it was a draft were discharged, in the same order they were stated:

1. `M00-B1` is merged at `c347e689aa23ee777b95e0989e633a9d91041161` and its public surface is read back in §2.0;
2. the M00 blueprint, module map, roadmap and coverage matrix project B2 alongside — not over — B1's implemented evidence;
3. `AUTH-017..020` are in the long-horizon catalog and are active `planned` matrix rows with the exact future bindings in §12;
4. the repository checker registers this contract as a fail-closed key file and cross-validates §12 against the active matrix, so a stale or missing projection carrier fails the run;
5. transition, deadline and error precedence received independent blocker review across three lanes — contract/dependency direction, acceptance evidence, and semantics/security — and every accepted finding is folded into this revision, including a frozen error set that could not express a specified apply-guard rejection, a precedence list that contradicted §6.3 at `u64::MAX`, `Debug` redaction deferred to an unstarted batch, an unowned digest-provenance obligation, a read model that was fail-open for `Revoked`, and an incomplete §11.1 carrier list;
6. public type names and Serde shapes are frozen in §2, §4.1 and §5.1 against the merged B1 API;
7. no current-status carrier claims B2 implementation evidence — every affected carrier says `planned`, and `M00` stays `partial-evidence`;
8. documentation and checker gates pass on the exact final head.

A second merge-review round returned `NO-GO` on the revision that discharged those eight, and required four repairs before acceptance. Each is folded in above, and each is recorded here rather than only in a pull-request comment, because a reader of this contract otherwise cannot tell a deliberate shape from an arbitrary one:

1. **`admits_at` was underspecified and fail-open on stale time.** §2.4 now freezes the question it answers as *current admission* rather than historical validity, and adds the conjunct `observed_at >= last_transition_at`, so the read model is never more permissive than the decide path it guards. §13 binds before-open, before-last-transition and both-boundary fixtures.
2. **The public command and event families had semantic field lists but no Rust-representable algebra.** §4.1 and §5.1 now freeze struct-versus-variant topology, wrapper-variant payload reachability, exact constructor signatures and the external Serde tagging for `OpenSession`/`RefreshSession`/`ExpireSession`/`RevokeSession`/`SessionCommand` and `SessionOpened`/`SessionRefreshed`/`SessionExpired`/`SessionRevoked`/`SessionEvent`, and §2.5 closes the trait and accessor set over **every** public B2 type — because a derive list that names an outer `Eq` while leaving the field type's `Eq` unstated does not compile and therefore freezes nothing. The M00 blueprint's public-input list is aligned to those names in the same change.
3. **`SessionExpired`'s checked constructor implied an error channel in neither §9 taxonomy.** It is withdrawn: §5.1 makes the constructor total and §8 enforces `observed_at >= expired_at` as a consequence of ordered apply guards. The reason is that the comparison is decidable but not *meaningful* at construction — only the aggregate knows the deadline that gives `expired_at` a correct value — and that enforcing it removes §13's exact-derived-field fixture from the input space. No new error type, variant or Serde-only message was introduced, and §13 binds a direct-construction and a deserialization fixture so the decision is executable rather than asserted.
4. **Command constructors returned an unreachable `Result`.** All four command and all four event constructors are now total. For five of the eight nothing was computable at construction; for `OpenSession`, `SessionOpened` and `SessionExpired` something was, and §§4.1 and 5.1 state the specific reason each check belongs to `decide`/`evolve` instead — precedence inversion, against §7 for the first and §8 for the second, and decidable-but-not-meaningful comparison for the third. §2.0 records both governing rules for any constructor added later, and deliberately does not record the weaker claim that a total constructor implies nothing was checkable.

Two independent review lanes then read the repaired revision before it was offered back, and three of their findings changed substance rather than wording. The trait closure of repair 2 did not compile as first written — `SessionCredentialEvidence` and `SessionPolicy` are stored flattened in `SessionSnapshot`, so §2.4's transitive `PartialEq`/`Eq` clause never reached them — which is why §2.5 is an exhaustive table rather than a transitive rule. Repair 3's first justification claimed the `EventTimeOutsideValidity` arm would be *permanently* dead under a checked constructor, which this round's own forged-low fixture refutes; the claim is now the narrower and sufficient one. And §2.2's "missing fields fail closed" was false for `credential_not_after`, because serde's derived decode reads an absent `Option` field as `None` — a downgrade by omission on the field that caps a session's life, now specified explicitly and bound by two fixtures.

A third merge-review round returned `NO-GO` on one contract-representability blocker plus one adjacent reasoning defect, and both are folded in above.

The blocker was that §2.0's representation and construction rules were written as universal statements over "every B2 value", and **no Rust program satisfies them**. An enum variant's fields are exactly as public as its enum, so `SessionStatus`, `SessionExpiryCause`, `SessionCommand`, `SessionEvent` and the three error enums cannot have private fields at all; `SessionSnapshot` and `SessionValueError` deliberately have no public constructor, so "each value has exactly one" was false in the other direction; and §10 compounded it by asserting that `SessionStatus` itself could arise only from evolution, which is false for a public enum and was never the property that mattered. An unsatisfiable rule is worse than a missing one, because it leaves an implementer nothing to obey and a reviewer nothing to measure. §2.0 now scopes representation to public structs, says why the rule stops there and why `#[non_exhaustive]` is not the escape, and partitions construction three ways — buildable structs, unbuildable structs, enums. §2.4 states the invariant that does hold, that a caller-built status cannot be injected into a snapshot. §2.5, §4.1 and §10 are reconciled to the same scoping, and §10 states the two read models separately.

The adjacent defect was in the totality reasoning rather than in the algebra: §5.1 and this section claimed nothing beyond `SessionExpired` was checkable at event construction, while `SessionOpened` carries exactly the fields §3's four open conditions are computed from. The signature stays total, now on stated grounds — a checked one would answer ahead of §8's sequence check, and would remove from the input space every event that could falsify §8's obligation to re-derive open invariants from the persisted event, killing three `SessionDomainError` variants on the evolve path. The count in repair 4 above is corrected with it. Neither correction changes a name, field set, transition, precedence order, deadline formula, error variant or Serde shape, so neither is a `platform-session/v0` change under §16.

This contract is therefore accepted, and accepted is the *entry* condition for implementation, not evidence of it. What acceptance authorizes is exactly one thing: `M00-B2 session-domain` may be implemented against this specification, under `module-work-policy/v1.3` §3 Path B, on its own branch, with its own review and its own exact-head CI.

What acceptance does **not** establish:

- no `AUTH-017..020` row may be promoted from documentation alone; each is promoted only when its named test exists and every assertion in §§3–13 that it covers is executable;
- `M00` does not advance past `partial-evidence`, and neither `StandaloneReady` nor any later readiness state is reachable from this batch; bounded B3 is separately implemented by `platform-request-context/v0`, B4a session-port/read-vendor evidence is separately implemented by `platform-session-port/v0`, while B4b control-evidence and B5 composition remain unstarted;
- this B2 contract alone establishes no actor, request-context, policy-reference, port, adapter, journal or M10 admission behavior. The later bounded B3 kernel is recorded below without retroactively enlarging B2.

## 16. Change rule

Changing a public value name, field set, transition table entry, precedence order, deadline formula, error variant set or Serde shape frozen above changes `platform-session/v0`. Such a change requires an owning-contract update, acceptance-row and fixture review, and — once implementation exists — implementation and consumer evidence on the same revision.

Adding or changing request-context semantics belongs to [`platform-request-context/v0`](platform-request-context.md), not to this contract. Production ports, control-evidence, credential-verification and transport semantics remain later owning contracts. §14 lists those boundaries.

**`platform-session/v0` is retained across the implementation revision, and the reason is stated rather than assumed.** That revision corrected four statements in this document and amended two §13 fixture entries. Measured against the list above: no public value name changed, no field set changed, no transition-table entry changed, no precedence order changed, no deadline formula changed, and the error variant sets of §9.1 and §9.2 are identical. Nor did any **Serde shape** change — §2.5's Serde columns now record which `Deserialize` is hand-written, and the wire form each value produces and accepts is byte-for-byte what §§2.1–2.2 already fixed.

Two of the four corrections touch the public trait surface, so they are called out rather than folded in silently. §2.5 previously implied a `Display`/`Error` pair on `SessionValueErrorKind`; that implication is withdrawn, which **narrows** the described surface to what `platform-identity/v0` §5's wrapper-versus-kind precedent already gave B1 and what §9.1's accessor list already assumed. And §9's non-echo guarantee is now scoped to the diagnostics B2 produces, with the deserializer's own messages assigned to `M00-B4`; that narrows a *stated* guarantee without weakening any behaviour, because no implementation of this contract could ever have bound a message it has no interception point for. Neither correction adds a trait, a variant, a field or a construction path, so neither meets this section's threshold. A future change that widens the trait surface — a `Display` on a kind enum, a public `TryFrom`, a `Deserialize` on `SessionSnapshot` — would.

## 17. Implementation record

`M00-B2 session-domain` is implemented. This section projects the resulting evidence state; §§1–16 remain the authority and this section overrides none of them.

The carriers are `crates/platform-core/src/session.rs`, declared `pub mod session;` in `crates/platform-core/src/lib.rs`, with integration evidence in `crates/platform-core/tests/platform_session.rs` and the two private library fixtures §12 binds inside the module itself. The batch added no crate, no manifest dependency and no change to `crates/platform-core/Cargo.toml`. §11.1's carrier extensions were made in `scripts/check_repo_contracts.py` and the mirroring Rust guard in `crates/platform-core/tests/platform_identity.rs` together, including the enumerated cross-file identity binding, the per-source attribute allowlist, the session-scoped forbidden-carrier scan, the fixture module's own registration and the bound test surfaces.

**All four rows — `AUTH-017`, `AUTH-018`, `AUTH-019`, `AUTH-020` — are `implemented`.** Every §13 fixture is an executable assertion under the tests §12 binds, and each was shown to fail when the production guard or guard order it covers is broken.

Four statements in this document were corrected in the same revision that implemented it, because an implementation cannot be conformant to text that no implementation can satisfy:

1. **§2.5's Serde columns** listed `Deserialize` as derived for `SessionDuration`, `AuthAdapterId` and `CredentialEvidenceDigest`. A derived decode cannot reach those values' checked constructors without adding a public `TryFrom`, which §2.0 rule 1 and §2.5's own closure both forbid, so the column was not satisfiable alongside §2.1's "a zero cannot arrive through Serde" and §2.2's grammars. §2.5 now separates a derived `Serialize` from a validating hand-written `Deserialize` and says which values need which, and why.
2. **§2.5's error row** implied `Display` and `Error` on `SessionValueErrorKind`. `platform-identity/v0` §5 gives those to the wrapper and not to the kind, and §2.0 states B2 inherits that shape. The row now names `SessionValueError` and `SessionDomainError` as the two types that carry them.
3. **§8's evolution failure enumeration** omitted `NoEffectiveRefresh` and `DeadlineOverflow`, both reachable when a persisted `SessionRefreshed` cannot advance or overflows its own recomputation. Reporting a forged derived field for either would be false, so §8 now names them and fixes the guard order that decides between them.
4. **§9's non-echo guarantee** was unqualified and therefore unachievable at its edge. Because every decode reads the canonical primitive first, a deserializer rejects a wrong-typed or malformed payload with its own message before any B2 validator runs, and §10 forbids the hand-written `Visitor` that would be needed to intercept it. §9 now scopes the guarantee to the diagnostics B2 produces and assigns the rest to `M00-B4 control-evidence` as untrusted boundary diagnostics; §10 states the same boundary where the Serde surface is frozen, §13 scopes its canary fixture to B2-owned surfaces, and `AUTH-020` carries the same scope in its own assertion text rather than an explanation attached to it.

`platform-session/v0` is **retained**, and §16 states the reasoning: none of the four corrections changes a public value name, field set, transition-table entry, precedence order, deadline formula, error variant set or Serde wire shape. Two of them touch the public trait surface, and both narrow it rather than widen it — one withdraws an implied `Display`/`Error` pair the implementation never had, the other narrows a stated guarantee to what any implementation could actually deliver.

**§13 was amended in the same revision, for two entries.** Both named a `SessionSnapshot` already at `revision == u64::MAX`. That aggregate is not reachable from an integration test at any feasible cost: §2.4 gives the type no public constructor, §10 gives it no `Deserialize`, and §8 sets `revision` only to `current + 1` from a base of `1`, so arriving there needs on the order of 1.8e19 accepted evolutions. The behaviour at that value is nonetheless real — §7 item 5 puts `RevisionOverflow` below terminal state and above time, and §8's checked increment is what makes a wrapped sequence `0` rejectable rather than acceptable — so an unprovable requirement would have left the most dangerous arithmetic in the contract unguarded.

The amendment binds those two entries to private `#[cfg(test)]` fixtures inside the module that owns the fields, which construct the ceiling snapshot from a real opened snapshot with only `revision` and `status` overridden and then call the production `decide` and `evolve`. §12 binds each by an exact library-target command alongside the existing integration command. The fixtures add no public or feature-gated hook and change no production item: the public-declaration, `impl`, derive and macro allowlists that freeze this module are unchanged by their presence, so a `pub` on any of them fails the gate. A source-text assertion — that one guard is written above another — was considered and rejected as the acceptance proof, because it pins the shape of the code rather than the behaviour of the algebra; the fixtures execute the real paths, and mutation of each guard and of each increment fails them.

## 18. B3 request-context integration

[`platform-request-context/v0`](platform-request-context.md) consumes `SessionSnapshot` through the transaction-current `AdmissionPorts::load_session` observation. For an authenticated request it requires exact requested/loaded `SessionId` equality and then calls the production `SessionSnapshot::admits_at(observed_at)` predicate before constructing `M00AdmittedActor::Authenticated`. Public admission carries no session. This integration changes no session command, event, transition, deadline, error, or Serde shape, so `platform-session/v0` is retained. The bounded kernel is `AUTH-013`; B4a now supplies one secure durable DemoReviewed current-session read/bootstrap vendor under `AUTH-021`, while formal authentication, durable lifecycle mutation, B4b control evidence and B5 administration remain planned.

## 19. B4a session-port implementation record

[`platform-session-port/v0`](platform-session-port.md) is the separate owning contract for the B4a port surface and durable current-session read/bootstrap vendor. It replays this contract's immutable `SessionEvent` values through production `evolve`; it never decodes or persists `SessionSnapshot`, changes no transition rule here and adds no raw credential path. The three-plugin composition now reads authenticated current-session authority from the retained file, with exact missing/scope-mismatch startup denials and no fixture fallback.

M00 remains `partial-evidence`: B4a is bounded implemented, B4b `control-evidence` and B5 composition are planned, and formal USTC SSO/public session lifecycle transport remains deferred.
