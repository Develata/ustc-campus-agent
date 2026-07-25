# M10 — Application Ingress Host

## Metadata

- `Module ID`: `M10`
- `Status`: Accepted blueprint; daemon skeleton exists, Fullstack/application ingress implementation planned
- `Version`: `m10-application-ingress/v1`
- `Last Review`: `2026-07-25`
- `Composition`: `apps/ustc-agentd`
- `Primary code area`: `apps/ustc-agentd/` plus shared Dioxus server-function declarations in the Fullstack application boundary

## 1. Purpose

`M10` is the admitted network/application boundary between first-party Dioxus clients, public integrations and backend modules. It owns:

- Axum/Dioxus Fullstack server-function ingress for Web and Android;
- versioned HTTP JSON endpoints when public or heterogeneous clients need them;
- typed SSE/stream event delivery;
- authentication/session admission;
- request/response/error/compatibility mapping;
- idempotency/precondition and audit context;
- one-ingress-to-one-application-operation dispatch.

It translates and coordinates. It does not become a second implementation of domain rules.

## 2. Non-goals

- rendering Dioxus components or owning client navigation/presentation state;
- deciding grants, package lifecycle, Agent phases, source acceptance or product facts;
- exposing direct database, container, filesystem or Plugin executor operations;
- letting a server-function macro bypass admission or create a hidden authority path;
- forcing an admitted server function through a redundant loopback HTTP request before the same application port;
- turning internal Rust/domain structs into an accidental unversioned mobile protocol;
- duplicating business logic between Fullstack and public HTTP adapters.

## 3. Owned objects and state

```text
ApiVersion / ClientProtocolVersion
IngressId / RouteId / ServerFunctionId
RequestEnvelope / ResponseEnvelope
ErrorEnvelope / CompatibilityEnvelope
EventCursor / EventEnvelope
ConnectionState
Idempotency and precondition policy
ClientBuild/Target admission facts
```

M10 owns ingress, compatibility and event-delivery state only. Domain state remains in its owning module.

## 4. Public inputs and outputs

First-party Web/Android inputs may arrive through generated Dioxus server-function clients. Public/integration inputs may arrive through explicit HTTP routes. Both carry admitted session context, bounded versioned values and optional idempotency/precondition identity.

Outputs are typed accepted/denied responses, compatibility outcomes and monotone events/streams. Initial operation families remain those registered in [`../../contracts/interfaces.md`](../../contracts/interfaces.md):

```text
health/build/protocol compatibility
Market browse/detail and installation commands
HarnessRun create/read/answer/cancel/events
first-party product queries/actions added only with their contracts
```

Public DTOs contain stable IDs, explicit status and safe summaries. Unknown fields/versions follow the endpoint contract; they are never silently interpreted as a nearby variant.

## 5. Server-function boundary

A Dioxus server function is an Axum-compatible HTTP endpoint. Its server-only body MAY:

1. extract admitted request/session facts;
2. validate version, bounds, idempotency and preconditions;
3. call one public application command/query port;
4. map its typed result/error/event into the first-party contract.

It MUST NOT directly call:

- concrete repositories or database clients;
- Plugin/MCP executors;
- model provider SDKs;
- private fields/events of another module;
- journal/evidence implementations;
- business fallback logic in transport code.

Dioxus component/router/signal/WebView types remain forbidden inside M10. Dioxus/Axum server-function transport types are allowed only in the ingress adapter and terminate there.

## 6. Dependency direction

Allowed dependencies:

- `M00` request/session/client compatibility admission;
- public application command/query interfaces of `M20`, `M30`, `M60`, `M70`, `M71` and `M72`;
- `M90` HTTP runtime, config, telemetry, event-subscription and deployment adapters;
- Dioxus/Axum server-function transport declarations confined to the Fullstack ingress adapter.

Forbidden dependencies:

- Dioxus UI/component/router/signal state;
- concrete database/repository implementations in handlers;
- Plugin executor/provider SDK calls from handlers;
- private fields or internal events of another large module;
- transport-specific business fallback.

## 7. Lifecycle

```text
startup config and protocol preflight
→ bind reviewed listener
→ attach Dioxus SSR/assets/server functions and any public routes
→ health/readiness available
→ admit client build/protocol/session
→ decode and validate bounded envelope
→ reject incompatible client before application dispatch
→ call one typed application operation
→ map typed result/error/event
→ emit response and correlated event cursor
→ graceful drain and shutdown
```

SSE/typed-stream reconnect resumes from a server-owned monotone cursor. Disconnect is not task cancellation or terminal completion.

## 8. Failure and recovery

- Unsupported client/API/schema version: typed compatibility/upgrade error and no domain call.
- Malformed/oversized request: reject before application dispatch.
- Missing/invalid session: M00 denial and no downstream call.
- Stale precondition/idempotency conflict: typed conflict response.
- Downstream typed denial: stable error code; no transport-level success disguise.
- Event cursor too old or unknown: explicit refresh/resync outcome.
- Client disconnect: preserve an accepted backend operation; reconnect reads authoritative state.
- Partial startup/config/migration failure: readiness remains false and no mutation ingress is exposed.
- Fullstack/public adapter disagreement: fail conformance; do not choose one transport as hidden fallback.

## 9. Configuration and secrets

Typed config covers listener addresses, public/Web origin, Android server origin policy, TLS/proxy trust, body/time/connection limits, stream heartbeat/buffer limits, supported client/API versions, minimum client version and telemetry redaction. Secret values are references supplied by M90; endpoint config embeds no credential.

Web same-origin session and Android secure-session/token transport are separate target adapters under one admission policy.

## 10. Observability

Record ingress/route ID, protocol/API version, client build/target, request/correlation IDs, response class, latency, payload size class and stream reconnect/drop counters. Do not log raw auth headers, prompts, tool payloads, profile data or private source content.

## 11. Extension and replacement

Dioxus server functions, public REST/SSE, future CLI/internal transport and their serialization runtimes are peer ingress adapters over the same application ports. Adding or replacing one transport must not duplicate domain logic.

First-party clients may bind to versioned Dioxus server-function routes and typed event handles. Public clients bind only to endpoints explicitly marked public in the interface registry. Independently released Android clients retain a declared compatibility window; shared source types alone are not compatibility proof.

## 12. Performance path

Hot paths are request decode/admission, compatibility checks, read-model serialization and event fan-out. Bound body/event sizes, connection counts and per-client queues. Backpressure or a slow client must not block domain journals or other tenants.

## 13. Scope boundary

**Required initial product scope**

- health/build/protocol compatibility;
- Dioxus Fullstack server-function query/command/event ingress for one real Web journey;
- the same semantic ingress consumed by Android;
- read-only Market browse/detail;
- minimal finite run create/read/cancel/events;
- stable errors, request correlation, reconnect cursor and upgrade-required behavior;
- strict size/version/session/idempotency gates;
- Docker Compose startup/readiness/restart/read-back.

**Later**

- installation/grant mutations when M20 is durable;
- first-party product-specific operations;
- public/heterogeneous REST surface beyond real consumers;
- richer event filtering and operator APIs.

**Explicit non-goals**

- generic database query endpoint;
- generic URL/process/container proxy;
- business logic inside ingress handlers;
- transport state used as completion truth.

## 14. Small-module decomposition

1. `ingress-registry` — server-function/public route/version/error registry.
2. `ingress-contract` — first-party/public request/response/event/compatibility values.
3. `request-admission` — bounds, M00 actor, client version and precondition mapping.
4. `server-function-adapter` — Dioxus/Axum endpoint declarations and dispatch.
5. `public-http-adapter` — only routes with real heterogeneous consumers.
6. `application-dispatch` — one ingress to one use case.
7. `error-projection` — stable safe error and upgrade envelopes.
8. `event-stream` — cursor, heartbeat, reconnect and backpressure.
9. `server-lifecycle` — config preflight, Dioxus attachment, readiness and drain.
10. `ingress-conformance` — black-box and import-boundary fixtures.

## 15. Exit gate

M10 is integration-ready when black-box tests prove versioning, malformed input, auth denial, supported/unsupported Android client versions, one accepted fake command, stable error mapping, monotone event reconnect and graceful shutdown. Dependency tests prove server-function adapters cannot reach concrete repositories/executors. It is accepted only after Web and Android complete the same semantic Fullstack journey against the Docker Compose server without direct backend access.
