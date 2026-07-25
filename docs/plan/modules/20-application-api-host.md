# M10 — Application API Host

## Metadata

- `Module ID`: `M10`
- `Status`: Accepted blueprint; daemon skeleton exists, API implementation planned
- `Version`: `m10-application-api/v0`
- `Last Review`: `2026-07-25`
- `Composition`: `apps/ustc-agentd`
- `Primary code area`: `apps/ustc-agentd/` plus a future cohesive API/application crate only if multiple hosts justify it

## 1. Purpose

`M10` is the explicit network boundary between clients/integrations and backend modules. It owns versioned HTTP JSON/SSE transport, authentication admission, request/response DTO mapping, stable error projection and event-stream delivery.

It is intentionally boring. It translates and coordinates; it does not become a second implementation of domain rules.

## 2. Non-goals

- rendering Dioxus components or owning client navigation;
- deciding grants, package lifecycle, Agent phases, source acceptance or product facts;
- exposing direct database, container, filesystem or Plugin executor operations;
- using Dioxus server functions as a hidden alternate business API;
- turning internal Rust structs into an accidental unversioned public protocol.

## 3. Owned objects and state

```text
ApiVersion
RouteId
RequestEnvelope / ResponseEnvelope
ErrorEnvelope
EventCursor / EventEnvelope
ConnectionState
IdempotencyHeader policy
```

The API host owns connection/event delivery state only. Domain state remains in its owning module.

## 4. Public inputs and outputs

Client inputs are versioned HTTP JSON requests with an admitted session and optional idempotency/precondition identity. Outputs are typed accepted/denied responses and monotone SSE event envelopes.

Initial route families remain those registered in `docs/contracts/interfaces.md`:

```text
health
market browse/detail and installation commands
Agent/Harness run create/read/answer/cancel/events
first-party product queries/actions added only with their contracts
```

Public DTOs contain stable IDs, explicit status and safe summaries. Unknown fields/versions follow each route contract; they are never silently interpreted as a nearby variant.

## 5. Dependency direction

Allowed dependencies:

- `M00` request/session admission;
- public application-service interfaces of `M20`, `M30`, `M60`, `M70`, `M71` and `M72`;
- `M90` HTTP runtime, config, telemetry and event-subscription adapters.

Forbidden dependencies:

- Dioxus component/router/signal types;
- concrete database repositories in route handlers;
- Plugin executor/provider SDK calls from handlers;
- private fields or internal events of another large module;
- business fallback implemented in transport code.

## 6. Lifecycle

```text
startup config preflight
→ bind reviewed listener
→ health/readiness available
→ admit request/session
→ decode and validate envelope
→ call one typed application operation
→ map typed result/error
→ emit response and correlated event cursor
→ graceful drain and shutdown
```

SSE reconnect resumes from a server-owned monotone cursor. Disconnect is not task cancellation or terminal completion.

## 7. Failure and recovery

- Unsupported API/schema version: explicit version error and no domain call.
- Malformed/oversized request: reject before application dispatch.
- Missing/invalid session: `M00` denial and no downstream call.
- Stale precondition/idempotency conflict: typed conflict response.
- Downstream typed denial: stable API error code; no transport-level success disguise.
- Event cursor too old or unknown: explicit refresh/resync response.
- Client disconnect: preserve accepted backend operation; reconnect reads authoritative state.
- Partial startup/config failure: readiness remains false and no mutation routes are exposed.

## 8. Configuration and secrets

Typed config covers listener addresses, public origins, TLS/proxy trust, request/body/time limits, SSE heartbeat/buffer limits, route-version enablement and telemetry redaction. Secret values are references supplied by `M90`; no route config embeds credentials.

## 9. Observability

Record route ID, API version, request/correlation IDs, response class, latency, payload size class and stream reconnect/drop counters. Do not log raw auth headers, prompts, tool payloads, profile data or private source content by default.

## 10. Extension and replacement

HTTP runtime, reverse proxy, serialization implementation and deployment profile are replaceable behind route/DTO contracts. A future CLI or internal transport may call the same application interfaces without pretending to be HTTP. Public clients remain bound to versioned HTTP/SSE, not internal Dioxus server-function signatures.

## 11. Performance path

The hot paths are request decode/admission, read-model serialization and SSE fan-out. Bound body/event sizes, connection counts and per-client queues. Backpressure or a slow client must not block domain journals or other tenants.

## 12. Scope boundary

**MVP**

- health/version;
- read-only Market browse/detail;
- minimal finite run create/read/cancel/events needed by one Dioxus journey;
- stable errors, request correlation and reconnect cursor;
- strict request size/version/session gates.

**Later**

- installation/grant mutations when `M20` is durable;
- first-party product-specific routes;
- richer event filtering and operator APIs under separate authority.

**Explicit non-goals**

- generic database query endpoint;
- generic URL/process/container proxy;
- business logic inside handlers;
- transport-specific state used as completion truth.

## 13. Small-module decomposition

1. `api-registry` — route/version/error registry.
2. `api-dto` — public request/response/event values.
3. `request-admission` — bounds, session and precondition mapping.
4. `application-dispatch` — one-handler-to-one-use-case mapping.
5. `error-projection` — stable safe error envelopes.
6. `event-stream` — cursor, heartbeat, reconnect and backpressure.
7. `server-lifecycle` — config preflight, readiness, drain/shutdown.
8. `api-conformance` — black-box fixtures and fake downstreams.

## 14. Exit gate

`M10` is integration-ready when a black-box server test proves versioning, malformed input, auth denial, one accepted fake command, stable error mapping, monotone SSE reconnect and graceful shutdown. It is accepted only after the Dioxus client completes one real API/event journey without direct backend access.
