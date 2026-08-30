# M10 — Application Ingress Host

## Metadata

- `Module ID`: `M10`
- `Status`: Accepted blueprint with bounded executable evidence for M10-owned client-protocol/application-ingress carriers and one loopback three-plugin HTTP/Web composition; production public HTTP/Dioxus/stream hosting remains planned
- `Implementation State`: `partial-evidence`
- `Version`: `m10-application-ingress/v2.1`
- `Last Review`: `2026-08-24`
- `Composition`: `apps/ustc-agentd`
- `Primary code area`: `apps/ustc-agentd/`, M10-owned `crates/client-protocol/` and `crates/application-ingress/`, plus future shared Dioxus server-function declarations in the Fullstack application boundary

## 1. Purpose

`M10` is the admitted network/application boundary between M80 peer clients, public integrations and backend modules. It owns:

- Axum/Dioxus Fullstack server-function ingress for Web and Android;
- versioned HTTP JSON endpoints for `ustc-agent`, inbound MCP and other intentionally admitted heterogeneous clients;
- typed SSE/stream event delivery;
- authentication/session admission;
- request/response/error/compatibility mapping;
- idempotency/precondition and audit context;
- one-ingress-to-one-application-operation dispatch.
- one canonical operation/schema registry from which HTTP, Dioxus, CLI and inbound-MCP allowlists are projected.

It translates and coordinates. It does not become a second implementation of domain rules.

M10 owns the versioned public wire schema and compatibility carrier (`client-protocol` when extraction is justified). M80 produces request instances and consumes M10 results/events through that carrier, but cannot redefine the schema. M10 server code MUST NOT depend on M80 `client-core`; both sides may depend on the M10-owned protocol carrier, preventing a server↔client cycle.

### Current bounded evidence

The retained `client-protocol` and `application-ingress` carriers prove checked request/value/error envelopes, M00 admission, capability and owner/operator lookup projections, idempotent submit/finalize behavior and typed M71 result mapping. `apps/ustc-agentd` composes one fixture-backed Affairs operation over numeric loopback TCP, with durable fixture record/idempotency files and real subprocess coverage through `ustc-agent`. It also hosts one loopback-only Axum route and embedded page for the exact `affairs.get` result; the route consumes the submit capability internally and emits only the public-redacted typed lookup result.

This evidence is deliberately narrower than the module exit gate. It does not establish remotely exposed production HTTP/TLS, a Dioxus server-function surface, broad operation registry, stream cursor/reconnect semantics, supported-version matrix, graceful drain, Docker Compose deployment, inbound MCP projection or Web/Android parity journey. The loopback fixture transports are integration proof surfaces, not the final heterogeneous-client transport contract.

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

First-party Dioxus inputs may arrive through generated server-function clients. `ustc-agent`, inbound MCP and other admitted heterogeneous clients arrive through explicit versioned HTTP/SSE routes. Every path carries admitted session context, bounded versioned values and optional idempotency/precondition identity, and every path terminates at the same application command/query ownership.

Outputs are typed accepted/denied responses, compatibility outcomes and monotone events/streams. Initial operation families remain those registered in [`../../contracts/interfaces.md`](../../contracts/interfaces.md):

```text
health/build/protocol compatibility
Market browse/detail and installation commands
HarnessRun create/read/answer/cancel/events
first-party product queries/actions added only with their contracts
```

Operation IDs are transport-neutral. A CLI command tree, MCP tool registry or GUI action registry is an allowlisted projection, not a new application service. If two adapters expose the same operation, authorization, result/error/provenance/audit semantics remain equal; the adapters do not need identical command sets. M10 binds exact schema identity/digest so a data, permission, effect or target widening can stale old grants before dispatch.

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

Dioxus server functions, `ustc-agent`/inbound-MCP HTTP/SSE routes, other public REST/SSE routes and future internal transports are peer ingress adapters over the same application ports. Adding or replacing one transport must not duplicate domain logic or make one client shell a subprocess dependency of another.

Dioxus clients may bind to versioned generated server-function routes and typed event handles. `ustc-agent` and inbound MCP bind to only the explicit endpoint subset registered for their real heterogeneous consumption; the MCP adapter remains outside M10 and calls those routes through M80 client-core. The first inbound-MCP profile is public-read Streamable HTTP; private read/draft projections are later explicit delegated-profile slices. Public clients bind only to endpoints explicitly marked public in the interface registry. Independently released Android, CLI and MCP artifacts retain declared compatibility windows; shared source types alone are not compatibility proof.

## 12. Performance path

Hot paths are request decode/admission, compatibility checks, read-model serialization and event fan-out. Bound body/event sizes, connection counts and per-client queues. Backpressure or a slow client must not block domain journals or other tenants.

## 13. Scope boundary

**Required initial product scope**

- health/build/protocol compatibility for every admitted client target;
- one explicit read-only HTTP/JSON operation consumed by `ustc-agent` and projected by one least-privilege inbound MCP tool/resource;
- exact operation, schema digest, permission/effect class and per-adapter allowlist with stale-grant behavior on widening;
- Dioxus Fullstack server-function query/command/event ingress for one real Web journey;
- the same semantic client-core result/error/event behavior consumed by Android, CLI and inbound MCP where the operation applies;
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

1. `ingress-registry` — canonical operation/schema plus server-function/public-route/adapter-allowlist/version/error registry.
2. `ingress-contract` — first-party/public request/response/event/compatibility values.
3. `request-admission` — bounds, M00 actor, client version and precondition mapping.
4. `server-function-adapter` — Dioxus/Axum endpoint declarations and dispatch.
5. `user-integration-http-adapter` — exact REST/SSE routes required by `ustc-agent`, inbound MCP or another named heterogeneous consumer.
6. `application-dispatch` — one ingress to one use case.
7. `error-projection` — stable safe error and upgrade envelopes.
8. `event-stream` — cursor, heartbeat, reconnect and backpressure.
9. `server-lifecycle` — config preflight, Dioxus attachment, readiness and drain.
10. `ingress-conformance` — black-box and import-boundary fixtures.

## 15. Exit gate

M10 is integration-ready when black-box tests prove versioning, malformed input, auth denial, supported/unsupported client versions, one accepted fake command, stable error mapping, monotone event reconnect and graceful shutdown. Dependency tests prove every ingress adapter cannot reach concrete repositories/executors. It is accepted only after Web and Android complete the same semantic Fullstack journey, `ustc-agent` completes one real versioned read path, and one inbound MCP projection reaches that admitted path without operator/domain/M51 reach-through; all run against the Docker Compose server without direct backend access.
