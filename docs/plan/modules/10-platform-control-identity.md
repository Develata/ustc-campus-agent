# M00 — Platform Control and Identity

## Metadata

- `Module ID`: `M00`
- `Status`: Accepted blueprint; `M00-B1 identity-types` and `M00-B2 session-domain` implemented, remaining batches planned
- `Implementation State`: `partial-evidence`
- `Version`: `m00-platform-control/v2`
- `Last Review`: `2026-08-02`
- `Composition`: `apps/ustc-agentd`
- `Primary code area`: `crates/platform-core/src/identity.rs` for `M00-B1`; `crates/platform-core/src/session.rs` for `M00-B2`; future cohesive modules under `crates/platform-core/`; adapter implementations under `M90`
- `Primary Contract`: [`platform-identity/v0`](../../contracts/platform-identity.md) for `M00-B1`; [`platform-session/v0`](../../contracts/platform-session.md) for `M00-B2`; accepted semantic [`platform-account/v0`](../../contracts/platform-account.md) and [`user-context-profile/v0`](../../contracts/user-context-profile.md) for planned account/auth/profile batches; [`module-boundaries.md`](../../contracts/module-boundaries.md) for cross-module actor/profile values
- `Acceptance`: implemented `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015`, `AUTH-016`, `AUTH-017`, `AUTH-018`, `AUTH-019`, `AUTH-020`; catalog-only `AUTH-001`–`AUTH-010`, `AUTH-013`, `AUTH-021`–`AUTH-030` and `PROFILE-001`–`PROFILE-012` remain non-active

## 1. Purpose

`M00` owns the platform's runtime human account, external authentication links, tenant membership, user-context profile, session and exact tenant/user/request/policy identity. It defines the platform-wide command envelope and causation chain used by other modules. It does not decide the internal rules of Market, Agent, sources or products.

The module answers:

- who is acting and under which tenant;
- which stable platform account an external authentication subject is explicitly linked to;
- which tenant membership admits that account without conflating membership with role or profile;
- which authenticated session admitted the request;
- which purpose-bound context-profile fields a consumer may read or propose;
- which stable request/command caused a state transition;
- which platform policy snapshot applies;
- how one command/event/effect is correlated in audit evidence.

## 2. Non-goals

- implementing Dioxus, HTTP routes or UI sessions;
- storing raw passwords, USTC CAS credentials or provider keys;
- treating a school number, GID, email, telephone, campus-card UID, profile value or provider login alias as `UserId`;
- owning package grants, Agent phases, source revisions or product facts;
- making administrator/service actors account kinds or deriving authorization from profile facts;
- becoming a generic authorization engine that replaces each module's domain checks;
- implementing database-specific identity rows in domain types.

## 3. Owned objects and state

```text
TenantId
UserId
SessionId
UserAccount / UserAccountStatus
ExternalIdentity / ExternalIdentityAlias
TenantMembership
ActorIdentity: User | future separately contracted ServicePrincipal
RequestId
CommandId
CausationId / CorrelationId
PlatformPolicySnapshotId
SessionStatus: Active | Expired | Revoked
ProfileFieldDefinition / ProfileFact / ProfileProposal
CurrentProfileProjection / ProfileAccessGrant / ProfileAuditReceipt
```

`M00` owns account/external-identity/membership validity, the general user-context profile, session lifecycle and platform-wide request causation. A downstream module still owns whether that actor may perform its specific operation. `UserAccount`, `TenantMembership`, role/grant policy and user-context profile are separate authority classes even though M00 owns their platform boundary.

`SessionStatus` is spelled as [`platform-session/v0`](../../contracts/platform-session.md) §2.4 freezes it, for the same reason the command names in §4 below are: an earlier `SessionState` spelling here predates that contract and a blueprint does not override it. Two of its three variants carry fields — `Expired { expired_at, observed_at, cause }` and `Revoked { revoked_at }` — so the three-way summary above is the lifecycle shape, not the field list.

## 4. Public inputs and outputs

Inbound values:

```text
SessionCredentialEvidence       # adapter-produced, never raw credential in domain logs
AuthAssertion                   # adapter-produced canonical issuer/subject and bounded claims
AdmitAuthentication / LinkExternalIdentity
SetTenantMembershipStatus
OpenSession
RefreshSession
RevokeSession
BuildRequestContextCommand
Propose/Accept/Reject/Supersede/DeleteProfileFact
ReadPurposeBoundProfile
```

The three session command names are exactly those frozen by [`platform-session/v0`](../../contracts/platform-session.md) §4 and §4.1; the earlier `…Command`-suffixed spelling here did not match the contract that owns them, and a blueprint is not authority for a name an accepted contract has frozen. `ExpireSession` is deliberately absent from this list rather than overlooked: §4 of that contract makes it an M00-internal lifecycle command issued only through the future ports batch, so it is not a public module input. `BuildRequestContextCommand` keeps its provisional name because `M00-B3 request-context` owns it and no accepted contract has frozen it yet. Account/profile operation names are semantic placeholders owned by their later exact batch contracts.

The values that actually cross the pure session boundary are the wrapper enums `SessionCommand` and `SessionEvent` frozen in §4.1 and §5.1 of that contract — `decide` and `evolve` take `&SessionCommand` and `&SessionEvent`. The four command and four event names above and in §4 are their payloads; this list names them individually because that is what a reader looking for one operation searches for.

Outbound values:

```text
AuthenticatedActor
PlatformRequestContext
SessionSnapshot
AccountLinkDecision / UserAccountView / TenantMembershipView
CurrentProfileProjection / ProfileMutationReceipt
PlatformControlEvent
PlatformControlError
```

`PlatformRequestContext` contains stable IDs and policy references only. It is not a mutable bag for arbitrary module state and never embeds the profile payload. M30/M72 and clients request separate purpose-bound projections under [`user-context-profile/v0`](../../contracts/user-context-profile.md).

## 5. Dependency direction

Allowed dependencies:

- stable value libraries and cryptographic digest primitives;
- `M90` clock, account/external-identity/membership/session/profile repository, secret-reference and event-journal ports through interfaces declared by `M00`, implemented under [`storage-profiles/v0`](../../contracts/storage-profiles.md).

Allowed callers:

- `M10` for admitted API requests;
- backend composition/application services that need an actor/request context.

Forbidden dependencies:

- Dioxus/client types;
- Market package/install/grant types;
- Agent run/checkpoint types;
- Plugin executor/provider/MCP implementations;
- concrete database/cloud/session-framework types in public contracts.

## 6. Lifecycle

```text
credential evidence admitted by an auth adapter
→ canonical external subject resolves through an explicit create/link decision
→ active platform account and tenant membership admitted
→ session opened under local idle/absolute/reauthentication deadlines
→ request contexts created with monotone/unique command identities
→ optional purpose-bound profile projection or typed profile proposal
→ refresh under policy | expire | revoke
→ historical causation retained under audit policy
```

Revoke blocks new request contexts immediately. It does not erase already committed receipts or rewrite historical events.

## 7. Failure and recovery

- Unknown/inactive tenant/account/membership/session: reject before calling another module.
- Missing/ambiguous provider subject or conflicting account link: reject without creating or merging an account.
- Profile field/purpose/consumer/sensitivity/registry mismatch: reject without reading or changing profile payload.
- Expired/revoked session: return stable reauthentication/revocation error.
- Clock or repository unavailable: reject mutation; never create an untracked anonymous command.
- Duplicate command ID with identical envelope: return prior recorded disposition where supported.
- Duplicate command ID with conflicting envelope: reject as a conflict.
- Audit causation write failure for a durable mutation: do not acknowledge acceptance.

Session-store recovery rebuilds current state from durable session transitions. UI cookies or in-memory middleware state never repair domain identity.

## 8. Configuration and secrets

Typed configuration includes session duration ceilings, local reauthentication policy, accepted auth adapter IDs/issuers, account-creation/link policy, profile field/consumer policy, cookie/token transport policy, storage profile and public origin bindings. Secret material enters only by `SecretRef`; it is absent from domain values, normal logs and fixtures.

The MVP uses a clearly labelled demo identity adapter with SQLite only under `local-demo`. It must not be described as production USTC authentication. Hosted/production requires PostgreSQL and rejects the development identity adapter; selecting PostgreSQL alone is not a production-readiness claim.

## 9. Observability

Stable evidence includes request/command/correlation IDs, tenant/user/session IDs, provider configuration and profile projection/fact IDs where needed, policy snapshot, admitted/denied disposition and redacted reason code. Metrics count account/link/profile/session outcomes and downstream latency by operation class without recording credentials, aliases, profile values or private claims.

## 10. Extension and replacement

Replaceable adapters may include demo identity, future OIDC/CAS-compatible browser flow or self-hosted identity integration. All produce the same bounded `AuthAssertion`; M00 alone resolves it to an account/membership and then an `AuthenticatedActor`/session. Replacing an adapter does not change downstream module APIs. SQLite and PostgreSQL implement the same M00 repository contracts without exposing backend rows as authority.

## 11. Performance path

The hot path is request-context admission: session/account/membership lookup, expiry/revoke check, command identity validation and audit correlation. It must be bounded, avoid broad profile loads and use indexed tenant/session/external-subject IDs. A profile read uses a separate purpose-bound indexed projection; security checks are not skipped for latency.

## 12. Scope boundary

**MVP**

- stable IDs and request context;
- one honest demo/auth adapter boundary;
- durable runtime account, external-identity link and tenant-membership semantics behind fakes/adapters;
- session open/expire/revoke;
- a basic extensible context-profile registry/fact/proposal/current-projection path with unknown fields allowed and sensitive fields denied from prompts by default;
- command/correlation identity and redacted audit linkage.

**Later**

- reviewed institutional/browser auth integration;
- key rotation and multi-device session management;
- administrator roles and separately typed service principals under separate policy;
- richer profile fields/derivation/connectors without changing the fact/projection authority split.

**Explicit non-goals**

- raw USTC password handling;
- cross-tenant super-session;
- silent identity fallback;
- domain permission checks centralized into one generic boolean.

## 13. Small-module decomposition

1. `identity-types` — the six bounded tenant/user/session/request/command/correlation IDs and their shared validation error.
2. `session-domain` — legal session transitions, frozen by [`platform-session/v0`](../../contracts/platform-session.md).
3. `request-context` — immutable request/command/causation envelope.
4. `policy-reference` — pinned platform policy identity.
5. `session-port` — repository/clock/secret-ref interfaces and fakes.
6. `control-evidence` — stable events/errors and redaction rules.
7. `account-directory` — human `UserAccount` lifecycle, distinct service-principal negative space and tenant membership.
8. `external-identity` — canonical provider subject, aliases and explicit link/conflict lifecycle.
9. `auth-admission` — bounded `AuthAssertion`, account/membership resolution and session opening; no raw credential.
10. `profile-field-registry` — extensible field schema, source/sensitivity/consumer and retention policy.
11. `profile-fact-domain` — fact/proposal/confirmation/supersession/deletion and deterministic conflict semantics.
12. `profile-projection` — purpose-bound current views and M30/M72/client consumer boundaries.
13. `account-profile-ports` — narrow repositories/audit ports and equal-contract fakes; M90 supplies SQLite/PostgreSQL adapters.

Each small module receives a separate reviewable commit with unit tests before the composition adapter.

**These list positions are not batch numbers.** [`01-execution-roadmap.md`](../../tasks/01-execution-roadmap.md) retains `M00-B1` through `M00-B5` for the already accepted identity/session/request-context/admission path, then schedules account/auth/membership/profile work as later contract-gated batches. A batch may land more than one adjacent small module only when the roadmap names both and each still has independently reviewable evidence. The roadmap owns grouping; this section owns decomposition.

## 14. First approved batch — `M00-B1 identity-types`

[`platform-identity/v0`](../../contracts/platform-identity.md) is the exact construction contract for the first small module. It freezes six opaque nominal ID kinds, a shared bounded grammar, deterministic non-echoing errors, Serde behavior and convergence of the existing invocation-local tenant/user values.

`CausationId` and any tenant-scoped actor key remain with `request-context`; `PlatformPolicySnapshotId` remains with `policy-reference`. The M20-owned invocation `PolicySnapshotId` identifies a different fact and must not alias the future platform-policy identity.

`M00-B1` deliberately does not create an authenticated actor, session lifecycle, request context, policy decision, ID generator or storage port. Those claims remain blocked behind later batches. `AUTH-013` stays catalog-only until request-context work.

`M00-B1` is implemented in `crates/platform-core/src/identity.rs`, with evidence in `crates/platform-core/tests/platform_identity.rs` and rustdoc `compile_fail` API proofs; `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015` and `AUTH-016` pass. Invocation authority now consumes the M00 tenant/user definitions. That is bounded partial evidence for one small module. `M00-B2` is now implemented alongside it under §15; every later batch is planned, and the module remains short of the §17 exit gate, so it is neither `StandaloneReady` nor accepted.

## 15. Second approved batch — `M00-B2 session-domain`

[`platform-session/v0`](../../contracts/platform-session.md) is the exact contract for the second small module. It freezes the pure, replayable lifecycle kernel for one session: immutable open scope, resolved idle/absolute/credential deadline algebra, the open/refresh/expire/revoke transition table, expected-revision event ordering, deterministic decision/evolution/replay, typed non-echoing failures, the credential and dependency negative space, and — in its §4.1 and §5.1 — the exact public command/event topology, constructor signatures, accessors, derive set and Serde tagging that this blueprint's §3 and §4 name only semantically.

The contract is **accepted and implemented**. `crates/platform-core/src/session.rs` and `crates/platform-core/tests/platform_session.rs` exist and carry the four bound tests of that contract's §12. `AUTH-017`, `AUTH-018`, `AUTH-019` and `AUTH-020` are `implemented`; two of their §13 fixtures are private library-target fixtures inside the session module, for the reason that contract's §17 records. `M00` stays `partial-evidence`: this batch creates no port, adapter, request context or admission behavior.

`M00-B2` deliberately does not create a clock, repository, journal, database, secret resolver, ID generator, authenticated actor, request context, policy reference, session port, control-evidence projection, cookie/token/auth adapter or `M10` integration. It authenticates no credential and persists nothing: `expected_revision` is validated as optimistic-concurrency *intent*, and the compare-and-append that would make it durable belongs to `M00-B4`. Those claims stay blocked behind their own batches and contracts.

`M00-B2` hands three obligations forward that it cannot discharge itself. They are recorded here, in the module that owns all three batches, so they are carried rather than asserted in a contract whose successors have not been written:

1. **`M00-B3 request-context`** consumes `SessionSnapshot::admits_at` as the single validity question. It must not recompute admission from `effective_expires_at`, which a revoked session deliberately preserves and which therefore reads as still-valid for exactly the revocation case.
2. **`M00-B4`** owns producing `CredentialEvidenceDigest` — the domain separation and the adapter-side material it is computed over — under `platform-session/v0` §2.2's prohibition on computing it over raw credential text. B2 pins the value into immutable evidence and cannot verify how it was made.
3. **`M00-B5 demo-api-admission-integration`** owns the composition rule that untrusted callers cannot reach `OpenSession` with self-asserted evidence. B2 makes the transport half structural by giving commands no `Deserialize`, but admission itself is a composition decision.

Two boundary facts are worth stating here rather than only in the contract. The session module imports the canonical `SessionId`, `TenantId` and `UserId` without renaming and re-exports none of them, so it adds no externally reachable API to a `platform-identity/v0` kind and mints no seventh kind. Adding it to `crates/platform-core/` extends the frozen surface `platform-identity/v0` §4 accounts for; that extension is deliberate registered drift, listed in `platform-session/v0` §11.1, and it changes no accepted grammar, bound, precedence, Serde shape or kind set.

## 16. Planned account, authentication and profile batches

[`platform-account/v0`](../../contracts/platform-account.md) and [`user-context-profile/v0`](../../contracts/user-context-profile.md) are accepted semantic boundaries, not exact Rust API/implementation contracts. They add no production source or active acceptance evidence.

The roadmap groups the remaining work as:

1. `M00-B3 request-context` and `M00-B4 ports-and-fakes` retain their accepted identity/session scope;
2. `M00-B5 demo-api-admission-integration` proves one development-only assertion-to-session path and one denied request, without claiming a durable account directory;
3. `M00-B6 account-directory-and-membership` freezes exact account/membership states, commands, errors, repository ports and semantic fakes;
4. `M00-B7 external-identity-and-auth-admission` freezes canonical issuer/subject uniqueness, explicit create/link/conflict, adapter assertion and local reauthentication semantics;
5. `M00-B8 profile-registry-and-facts` freezes extensible field definitions, facts/proposals, source/verification/conflict/deletion and repository fakes;
6. `M00-B9 profile-projection-and-consumers` freezes purpose-bound current projections plus M10/M30/M72 boundaries;
7. `M00-B10 durable-account-profile-integration` attaches M90 SQLite `local-demo` and PostgreSQL hosted/production adapters only after repository conformance and active acceptance bindings exist.

USTC CAS production integration is later than B7: it requires a reviewed institutional protocol/attribute agreement. A mock/fixture can test one stable subject with current/historical aliases, but mock-only attributes and login names cannot become production identity authority.

## 17. Exit gate

`M00` is integration-ready when standalone tests prove account/external-subject/membership isolation, explicit link conflicts, profile purpose/sensitivity boundaries, tenant/session scope, expire/revoke, duplicate/conflicting command behavior, redaction and deterministic replay through fake ports. It is accepted only after M90 adapter conformance proves the selected storage profiles and `M10` proves admitted and denied authentication/profile/application requests without invoking a downstream fake on denial.
