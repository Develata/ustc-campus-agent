# M00 — Platform Control and Identity

## Metadata

- `Module ID`: `M00`
- `Status`: Accepted blueprint; implementation planned
- `Implementation State`: `planned`
- `Version`: `m00-platform-control/v1`
- `Last Review`: `2026-07-26`
- `Composition`: `apps/ustc-agentd`
- `Primary code area`: future cohesive modules under `crates/platform-core/`; adapter implementations under `M90`
- `Primary Contract`: [`platform-identity/v0`](../../contracts/platform-identity.md) for `M00-B1`; [`module-boundaries.md`](../../contracts/module-boundaries.md) for later cross-module actor/context values
- `Acceptance`: active planned `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015`, `AUTH-016`; catalog-only `AUTH-013` and later session/admission cases remain deferred

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
2. `session-domain` — legal session transitions.
3. `request-context` — immutable request/command/causation envelope.
4. `policy-reference` — pinned platform policy identity.
5. `session-port` — repository/clock/secret-ref interfaces and fakes.
6. `control-evidence` — stable events/errors and redaction rules.

Each small module receives a separate reviewable commit with unit tests before the composition adapter.

## 14. First approved batch — `M00-B1 identity-types`

[`platform-identity/v0`](../../contracts/platform-identity.md) is the exact construction contract for the first small module. It freezes six opaque nominal ID kinds, a shared bounded grammar, deterministic non-echoing errors, Serde behavior and convergence of the existing invocation-local tenant/user values.

`CausationId` and any tenant-scoped actor key remain with `request-context`; `PlatformPolicySnapshotId` remains with `policy-reference`. The M20-owned invocation `PolicySnapshotId` identifies a different fact and must not alias the future platform-policy identity.

`M00-B1` deliberately does not create an authenticated actor, session lifecycle, request context, policy decision, ID generator or storage port. Those claims remain blocked behind later batches. The active `AUTH-011`, `AUTH-012`, `AUTH-014`, `AUTH-015` and `AUTH-016` rows are `planned`; `AUTH-013` stays catalog-only until request-context work. This contract-ready slice does not promote the module from `planned` and is not implementation evidence.

## 15. Exit gate

`M00` is integration-ready when standalone tests prove tenant/session scope, expire/revoke, duplicate/conflicting command behavior, redaction and deterministic replay through fake ports. It is accepted only after `M10` proves one admitted and one denied request without invoking a downstream fake on denial.
