# M00 — Platform Control and Identity

## Metadata

- `Module ID`: `M00`
- `Status`: Accepted blueprint; `M00-B1 identity-types` implemented, `M00-B2 session-domain` contract accepted and unimplemented, remaining batches planned
- `Implementation State`: `partial-evidence`
- `Version`: `m00-platform-control/v1`
- `Last Review`: `2026-07-28`
- `Composition`: `apps/ustc-agentd`
- `Primary code area`: `crates/platform-core/src/identity.rs` for `M00-B1`; `crates/platform-core/src/session.rs` for `M00-B2`, which does not exist yet; future cohesive modules under `crates/platform-core/`; adapter implementations under `M90`
- `Primary Contract`: [`platform-identity/v0`](../../contracts/platform-identity.md) for `M00-B1`; [`platform-session/v0`](../../contracts/platform-session.md) for `M00-B2`; [`module-boundaries.md`](../../contracts/module-boundaries.md) for later cross-module actor/context values
- `Acceptance`: implemented `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015`, `AUTH-016`; active `planned` `AUTH-017`, `AUTH-018`, `AUTH-019`, `AUTH-020`; catalog-only `AUTH-013` and later admission cases remain deferred

## 1. Purpose

`M00` gives every backend operation an exact tenant, user, session, request and policy identity. It defines the platform-wide command envelope and causation chain used by other modules. It does not decide the internal rules of Market, Agent, sources or products.

The module answers:

- who is acting and under which tenant;
- which authenticated session admitted the request;
- which stable request/command caused a state transition;
- which platform policy snapshot applies;
- how one command/event/effect is correlated in audit evidence.

## 2. Non-goals

- implementing Dioxus, HTTP routes or UI sessions;
- storing raw passwords, USTC CAS credentials or provider keys;
- owning package grants, Agent phases, source revisions or product facts;
- becoming a generic authorization engine that replaces each module's domain checks;
- implementing database-specific identity rows in domain types.

## 3. Owned objects and state

```text
TenantId
UserId
SessionId
ActorIdentity
RequestId
CommandId
CausationId / CorrelationId
PlatformPolicySnapshotId
SessionState: Active | Expired | Revoked
```

`M00` owns identity validity, session lifecycle and platform-wide request causation. A downstream module still owns whether that actor may perform its specific operation.

## 4. Public inputs and outputs

Inbound values:

```text
SessionCredentialEvidence       # adapter-produced, never raw credential in domain logs
OpenSessionCommand
RefreshSessionCommand
RevokeSessionCommand
BuildRequestContextCommand
```

Outbound values:

```text
AuthenticatedActor
PlatformRequestContext
SessionSnapshot
PlatformControlEvent
PlatformControlError
```

`PlatformRequestContext` contains stable IDs and policy references only. It is not a mutable bag for arbitrary module state.

## 5. Dependency direction

Allowed dependencies:

- stable value libraries and cryptographic digest primitives;
- `M90` clock, session repository, secret-reference and event-journal ports through interfaces declared by `M00`.

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
→ session opened
→ request contexts created with monotone/unique command identities
→ refresh under policy | expire | revoke
→ historical causation retained under audit policy
```

Revoke blocks new request contexts immediately. It does not erase already committed receipts or rewrite historical events.

## 7. Failure and recovery

- Unknown tenant/user/session: reject before calling another module.
- Expired/revoked session: return stable reauthentication/revocation error.
- Clock or repository unavailable: reject mutation; never create an untracked anonymous command.
- Duplicate command ID with identical envelope: return prior recorded disposition where supported.
- Duplicate command ID with conflicting envelope: reject as a conflict.
- Audit causation write failure for a durable mutation: do not acknowledge acceptance.

Session-store recovery rebuilds current state from durable session transitions. UI cookies or in-memory middleware state never repair domain identity.

## 8. Configuration and secrets

Typed configuration includes session duration ceilings, refresh policy, accepted auth adapter IDs, cookie/token transport policy and public origin bindings. Secret material enters only by `SecretRef`; it is absent from domain values, normal logs and fixtures.

The MVP may use a clearly labelled demo identity adapter. It must not be described as production USTC authentication.

## 9. Observability

Stable evidence includes request/command/correlation IDs, tenant/user/session IDs, policy snapshot, admitted/denied disposition and redacted reason code. Metrics count session opens, expiry, revoke, duplicate/conflict and downstream latency by operation class without recording credentials or private payloads.

## 10. Extension and replacement

Replaceable adapters may include demo identity, future OIDC/CAS-compatible browser flow or self-hosted identity integration. All produce the same `AuthenticatedActor` and session transitions. Replacing an adapter does not change downstream module APIs.

## 11. Performance path

The hot path is request-context admission: session lookup, expiry/revoke check, command identity validation and audit correlation. It must be bounded, avoid broad profile loads and use indexed tenant/session IDs. Security checks are not skipped for latency.

## 12. Scope boundary

**MVP**

- stable IDs and request context;
- one honest demo/auth adapter boundary;
- session open/expire/revoke;
- command/correlation identity and redacted audit linkage.

**Later**

- reviewed institutional/browser auth integration;
- key rotation and multi-device session management;
- administrator/service actors under separate policy.

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

Each small module receives a separate reviewable commit with unit tests before the composition adapter.

## 14. First approved batch — `M00-B1 identity-types`

[`platform-identity/v0`](../../contracts/platform-identity.md) is the exact construction contract for the first small module. It freezes six opaque nominal ID kinds, a shared bounded grammar, deterministic non-echoing errors, Serde behavior and convergence of the existing invocation-local tenant/user values.

`CausationId` and any tenant-scoped actor key remain with `request-context`; `PlatformPolicySnapshotId` remains with `policy-reference`. The M20-owned invocation `PolicySnapshotId` identifies a different fact and must not alias the future platform-policy identity.

`M00-B1` deliberately does not create an authenticated actor, session lifecycle, request context, policy decision, ID generator or storage port. Those claims remain blocked behind later batches. `AUTH-013` stays catalog-only until request-context work.

`M00-B1` is implemented in `crates/platform-core/src/identity.rs`, with evidence in `crates/platform-core/tests/platform_identity.rs` and rustdoc `compile_fail` API proofs; `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015` and `AUTH-016` pass. Invocation authority now consumes the M00 tenant/user definitions. That is bounded partial evidence for one small module: `M00-B3` through `M00-B5` are planned, `M00-B2` has an accepted contract and no implementation, and the module remains short of the §16 exit gate, so it is neither `StandaloneReady` nor accepted.

## 15. Second approved batch — `M00-B2 session-domain`

[`platform-session/v0`](../../contracts/platform-session.md) is the exact contract for the second small module. It freezes the pure, replayable lifecycle kernel for one session: immutable open scope, resolved idle/absolute/credential deadline algebra, the open/refresh/expire/revoke transition table, expected-revision event ordering, deterministic decision/evolution/replay, typed non-echoing failures, and the credential and dependency negative space.

The contract is **accepted and unimplemented**. `crates/platform-core/src/session.rs` and `crates/platform-core/tests/platform_session.rs` do not exist; `AUTH-017`, `AUTH-018`, `AUTH-019` and `AUTH-020` are active rows at status `planned` with the exact future bindings in that contract's §12. `planned` is a non-pass state, so nothing in this section is evidence of behavior.

`M00-B2` deliberately does not create a clock, repository, journal, database, secret resolver, ID generator, authenticated actor, request context, policy reference, session port, control-evidence projection, cookie/token/auth adapter or `M10` integration. It authenticates no credential and persists nothing: `expected_revision` is validated as optimistic-concurrency *intent*, and the compare-and-append that would make it durable belongs to `M00-B4`. Those claims stay blocked behind their own batches and contracts.

Two boundary facts are worth stating here rather than only in the contract. The session module imports the canonical `SessionId`, `TenantId` and `UserId` without renaming and re-exports none of them, so it adds no externally reachable API to a `platform-identity/v0` kind and mints no seventh kind. Adding it to `crates/platform-core/` extends the frozen surface `platform-identity/v0` §4 accounts for; that extension is deliberate registered drift, listed in `platform-session/v0` §11.1, and it changes no accepted grammar, bound, precedence, Serde shape or kind set.

## 16. Exit gate

`M00` is integration-ready when standalone tests prove tenant/session scope, expire/revoke, duplicate/conflicting command behavior, redaction and deterministic replay through fake ports. It is accepted only after `M10` proves one admitted and one denied request without invoking a downstream fake on denial.
