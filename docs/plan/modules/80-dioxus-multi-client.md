# M80 — Client Core and Interaction Shells

## Metadata

- `Module ID`: `M80`
- `Status`: Accepted blueprint with an approved Affairs-first M10 compatibility/operation-registry prerequisite plus bounded client-core, ordinary-user CLI evidence and one operation-specific loopback Web presentation proof; inbound MCP, Dioxus targets and full client conformance remain planned
- `Implementation State`: `partial-evidence`
- `Version`: `m80-client-shells/v2.2`
- `Last Review`: `2026-09-01`
- `Decisions`: [`ADR-0009`](../../adr/0009-dioxus-multi-client-shell.md), [`ADR-0010`](../../adr/0010-typed-client-peer-adapters.md)
- `Owning Contract`: [`client-shell/v2.1`](../../contracts/client-shell.md)
- `Primary code areas`: `crates/client-core/`, `apps/ustc-agent/`, future `apps/ustc-client/`, and one future inbound MCP adapter surface finalized by its first accepted slice

## 1. Purpose

M80 owns the framework-neutral client behavior shared by three peer interaction surfaces:

1. the Dioxus Web/PWA and Android presentation application;
2. the `ustc-agent` user/automation CLI;
3. the inbound MCP adapter through which external Agents consume selected platform tools/resources.

It owns client-side protocol use, compatibility reaction, authentication adapters, command/query transport, event subscription/reconnect, correlation/idempotency propagation, safe projection reduction and outer-shell conformance. The peer shells reuse this typed client core; they do not invoke one another as subprocesses.

Web is the first graphical proof surface. Android remains a required peer target. The headless CLI and one least-privilege inbound MCP read path are also required initial client surfaces. Windows is an explicitly admitted later desktop peer target but not a current required release gate; iOS and other desktop targets remain later candidates.

M80 displays or serializes server-owned state and submits typed intent through M10. It performs no canonical product calculation or mutation.

### Current bounded evidence

The retained framework-neutral `client-core` constructs and reduces the M10-owned typed Affairs requests/responses, confines transport to numeric loopback endpoints, preserves correlation/idempotency/provenance, emits one canonical `ustc-client-result/v1` JSON value and maps typed/transport failures to stable exit classes. The real `ustc-agent` binary exposes only `affairs get`, `affairs lookup`, help and version; dependency guards keep backend domain, repository, executor, `ustc-agentd` and `ustc-agentctl` implementations out of the client.

`ustc-agentd serve-web` separately provides one operation-specific loopback presentation proof over the same M10-owned typed DTO: conditions, steps, explicit unknown time bounds, entries, contacts, safe lineage, freshness, conflict and uncertainty. It consumes the public capability internally and exposes no raw revision identity. Because this proof is colocated in the composition root and does not use the shared client core or Dioxus, it does not establish the peer Web/PWA target.

The first retained prerequisite consumes M10 protocol major `1`, header-free `server.info`, the closed Web/CLI `server.info`/`capability.list`/`affairs.get` projection and typed `upgrade_required`/`incompatible_protocol` outcomes. M80 now reduces those server-owned values into client state and retains `ustc-client-result/v1`; it does not derive compatibility from HTTP status or recalculate Affairs authority. This is still not the complete `CLIENT-007` or `CLIENT-009` claim. It has no production HTTP/TLS profile/auth adapter, NDJSON/SSE stream, cursor/reconnect/cancellation or version-skew host matrix, and it does not implement inbound MCP, Dioxus Web/PWA, Android, Windows or shared graphical presentation state. The loopback fixture paths are bounded composition evidence and must not be projected as production remote-client support.

## 2. Non-goals

- owning identity, Agent, Market, grant, Plugin, source, product or audit rules;
- direct database, repository, filesystem, process, executor or provider access;
- making `GUI → CLI executable → server` the normal Web/mobile path;
- treating subprocess exit, transport disconnect or local optimistic state as operation completion;
- exposing `ustc-agentctl` operator/admin commands through `ustc-agent` or MCP;
- treating the inbound MCP client adapter as M51, which instead owns outbound platform-to-external-MCP binding and execution;
- embedding an Agent/model loop in the user CLI or MCP adapter;
- placing business logic in components, command parsers, MCP handlers, reducers or transport adapters;
- claiming Web, Android, CLI or MCP support from compilation or schema declarations alone.

## 3. Owned objects and state

```text
ClientProtocolSupport
ClientBuild / ClientTarget
ClientRequestCorrelation / IdempotencyKey
ClientConnectionState
ClientEventCursor and reduced server projection
ClientCapabilityProjection
ClientAuthProfileRef (never raw secret)
ClientIntent / ClientOutcome
PresentationState:
  Initial | Loading | Ready | Empty | Error | Offline |
  ReauthRequired | UpgradeRequired | Pending
CliOutputMode / CliExitClass
InboundMcpToolResourceProjection
TargetCapabilityState
Theme/locale/accessibility preferences
```

These are client facts and rebuildable projections only. Losing them may harm UX or require reauthentication/reconciliation; it cannot corrupt backend truth.

Request instances crossing `B-M80-M10-CALL` are produced by M80 client behavior, while their versioned wire schema is owned by M10's public `client-protocol` carrier. Result/event schemas and instances crossing `B-M10-M80-RESULT` and `B-M10-M80-EVENT` are M10-produced. Shared source placement does not merge authority.

## 4. Public inputs and outputs

Inputs from M10:

```text
versioned response/error/event values
server/API compatibility envelope
minimum-supported-client or UpgradeRequired outcome
monotone event cursor and resync outcome
server-owned run/market/product projection
safe capability availability
```

Outputs to M10:

```text
versioned query/command values
user intent and current preconditions
correlation/idempotency identity
reconnect cursor
client build/target/protocol identity
admitted authentication/session material through a target-specific port
```

Target ports:

```text
ClientTransport
ClientEventTransport
ClientAuthPort
ExternalNavigation
NotificationPort
SecureSessionPort
LocalArchivePort
PlatformInfo
ServerEndpointPort
```

Unsupported capability returns a typed unavailable state. It never silently changes the execution location or selects another endpoint/tool.

## 5. Framework-neutral client-core boundary

`client-core` owns common client behavior, not backend semantics. It depends on the M10-owned framework-neutral `client-protocol` carrier; M10 server code never depends on `client-core`. It MAY:

1. construct bounded versioned requests from validated shell input;
2. attach build/protocol/session/correlation/precondition facts;
3. call the declared M10 transport;
4. map typed results/errors/events without weakening them;
5. resume a stream from a server cursor;
6. reconcile timeout-after-possible-acceptance by correlation/idempotency identity;
7. expose deterministic fake-M10 fixtures to every outer adapter.

It MUST NOT:

- import Dioxus component/router/signal/WebView types;
- import CLI parser/terminal formatting types;
- import MCP SDK protocol types;
- import backend domain, repository, provider, executor or journal implementations;
- infer successful mutation from local cache, child-process exit or transport closure;
- redefine M10-owned wire DTO/error/event schemas or make M10 depend on client behavior;
- duplicate server validation or policy as client authority.

Framework-specific values terminate in their outer adapters. The common core exposes Rust-owned typed inputs, outputs, events and failures.

## 6. Peer adapter contract

| Adapter | Purpose | Canonical transport | Additional constraint |
|---|---|---|---|
| Dioxus Web/Android | graphical presentation and intent capture | generated admitted server-function calls and typed events, or an equivalent versioned M10 transport | no process/CLI bridge; presentation state only |
| `ustc-agent` | end-user and noninteractive automation client | explicit versioned M10 HTTP/JSON plus SSE/typed streams | machine mode is data-only stdout with stable schema/exit semantics |
| inbound MCP adapter | expose selected platform tools/resources to external Agents | reviewed MCP Streamable HTTP mapped through client-core to admitted M10 API | public-read first; exact operation/schema allowlist; no operator surface or direct domain/M51 reach-through |

All three adapters consume the same semantic fake-M10 conformance suite. Transport-specific encoding may differ, but equivalent accepted input must reduce to equivalent typed client state and preserve the same denial/compatibility outcome.

`ustc-agentctl` is not a fourth M80 peer. It remains an operator/developer surface with separately admitted local/administrative commands.

## 7. Dependency direction

Allowed dependencies:

- M10-owned versioned `client-protocol` request/result/error/event/compatibility values;
- HTTP/TLS/JSON/SSE plumbing behind owned transport adapters;
- exact-pinned Dioxus/DX target features in the Dioxus adapter only;
- an exact-pinned MCP protocol implementation in the inbound MCP adapter only;
- target-specific Web/Android/session/navigation adapters;
- presentation-only libraries after size/security review.

Forbidden dependencies:

- backend domain/application implementations in client targets;
- concrete databases, repositories, queues, provider SDKs, Plugin/MCP executors or journals;
- `ustc-agentctl` command handlers or operator credentials;
- one peer adapter importing or spawning another peer executable;
- M51 outbound MCP binding/session/executor internals.

Server-only Dioxus declarations may attach through M10, but that feature cannot make M80 client code depend on backend implementation. Cyclic M10↔M80 code dependencies remain forbidden: M10 and M80 may both consume the M10-owned `client-protocol` carrier, M80 `client-core` depends on it, and M10 never depends on `client-core`.

## 8. Lifecycle

```text
client/shell bootstrap
→ validate local config and supported protocol
→ establish target-appropriate admitted session
→ query server compatibility/capabilities
→ Initial/Loading
→ Ready | Empty | Error | Offline | ReauthRequired | UpgradeRequired
→ submit correlated typed intent
→ Pending projection only
→ accept typed response/events
→ reduce monotone server projection
→ reconnect/reconcile/refresh/re-auth/upgrade when required
```

CLI noninteractive mode performs the same lifecycle and emits one versioned result envelope or an NDJSON event sequence. The inbound MCP adapter maps selected tool/resource requests into the same client operations and returns bounded, instruction-isolated results. Adapter registries are allowlisted projections of the M10-owned application-operation registry: semantic equivalence is required only where two adapters expose the same operation; login and target-local maintenance are not MCP business tools.

Transport disconnect, CLI termination and MCP session closure are not server-task cancellation or terminal completion.

## 9. Failure and recovery

- Server unavailable/offline: preserve only safe drafts/cursors; return explicit unavailable state and invent no success.
- Unknown schema/event or non-monotone sequence: fail closed and require refresh/upgrade.
- Server-typed `upgrade_required`: reduce to `UpgradeRequired` without recalculating the major relation; the M10 HTTP adapter projects `426`.
- Server-typed `incompatible_protocol`: reduce to `IncompatibleProtocol` without treating a newer, absent or malformed major as an upgrade case; the M10 HTTP adapter projects `409`.
- Timeout after possible acceptance: reconcile by correlation/idempotency identity before retry.
- Cancellation: send a typed server cancellation intent; killing a shell/process is not cancellation evidence.
- Reauthentication: use the adapter's secure auth port; no raw password/token in shared state or command arguments.
- CLI partial output: machine framing fails and exits non-success; partial bytes are never treated as a complete result.
- MCP tool/resource denial: preserve the exact typed policy/capability failure; do not fall back to a same-name tool or operator command.
- Dioxus renderer/WebView failure: backend truth remains unchanged.
- Unsafe Markdown/HTML/tool output: sanitize or render/emit bounded typed content.

## 10. Configuration and secrets

Common public config contains only validated HTTPS server origin, supported protocol versions, build/target identity, non-secret capabilities and bounded transport defaults.

Each adapter owns a separate secret/session projection:

- Web uses browser-appropriate admitted session handling;
- Android uses `SecureSessionPort` and a non-loopback validated production origin;
- `ustc-agent` uses a least-privilege user auth profile/ref and never places a secret in argv or machine output;
- inbound MCP uses an explicitly delegated user/service profile and cannot inherit operator credentials.

The preferred future CLI authentication contract candidate is server-mediated browser pairing with a one-time bounded exchange; no CLI or personal Agent receives a raw USTC password, CAS ticket or complete CAS session.

Model/provider/Plugin/source secrets never enter client config, logs or output envelopes.

## 11. Observability

Client diagnostics may include build/target/protocol/server version, operation family, route/tool/resource ID, correlation ID, safe state, cursor, reconnect count, latency and stable error code. They exclude credentials, prompts, private profile data, tool payloads and source content unless an explicit bounded user output contract requires the content.

CLI diagnostics go to stderr; machine stdout remains protocol data. MCP logs distinguish external caller/session identity from the platform's outbound M51 execution sessions.

## 12. Extension and replacement

Dioxus, CLI and inbound MCP are replaceable peers over the same semantic client contract. Adding another shell must not change backend domain contracts or cause an existing shell to become its process dependency.

The client-core transport may be replaced while preserving request/result/event semantics and compatibility fixtures. The Dioxus renderer may be replaced without changing CLI/MCP behavior. The MCP protocol library may be replaced without changing M51 or platform tool authority.

A future desktop client MAY offer an explicitly optional local sidecar/debug mode, but Web/Android production paths remain direct typed clients of M10.

## 13. Performance path

Common hot paths are request encoding/admission latency, event framing/reduction, reconnect and bounded list/result projection. Reuse connections, bound request/result/event sizes and queues, and prevent a slow shell from blocking server journals or other tenants.

Dioxus separately budgets initial Web payload, SSR/hydration, Android startup/memory and large-list rendering. CLI/MCP separately budget process startup, first result, stream throughput and bounded concurrent sessions. No adapter-specific optimization may bypass common compatibility, authorization or result bounds.

## 14. Scope boundary

**Required initial product scope**

- framework-neutral client-core contract and fake-M10 conformance fixtures;
- one read-only `ustc-agent` health/capability/product query with stable JSON output and typed failures;
- one reviewed read-only inbound MCP tool/resource projection through the same client core;
- exact application-operation and schema-digest projection from M10, with grant invalidation on data/permission/effect widening;
- exact-pinned Dioxus/DX after source revalidation;
- cohesive Dioxus Web/PWA and Android source over the common client semantics;
- one Market/run/product query-command-event journey;
- explicit loading/empty/error/offline/re-auth/upgrade/pending/terminal states;
- Docker Compose server startup/readiness/restart/read-back;
- Android emulator and real-device HTTPS/session/lifecycle/reconnect/Custom Tab proof;
- adapter privilege/dependency confinement and cross-adapter semantic conformance.

**Later**

- richer CLI command families and MCP resources/tools over already admitted application operations;
- delegated tenant-private reads and tenant-local drafts after explicit consent/ownership acceptance;
- iOS package after macOS/Xcode/signing/device evidence;
- Windows Dioxus package only after a separate promotion amendment and installer/signing/update, secure-session, login-callback, sleep/resume/proxy/reconnect and real-host evidence; optional local sidecar remains separately admitted;
- opt-in notifications and local archive;
- richer public/product views over unchanged application ports.

**Explicit non-goals**

- operator/admin mutation through `ustc-agent` or MCP;
- backend business logic in client-core or shell adapters;
- offline peer authority or local database truth;
- direct Plugin/provider/M51 execution;
- generic Agent-to-Agent federation implied by the MCP adapter;
- automatic enrollment, registration, payment or external campus transaction submission;
- arbitrary shell, URL, filesystem, database, container or third-party MCP capability;
- target support claimed only from compilation.

## 15. Small-module decomposition

1. `client-contract-adoption` — consume the M10-owned `client-protocol` schema and closed operation registry; the first retained subset is protocol major `1` plus Web/CLI `server.info`, `capability.list` and `affairs.get`.
2. `client-core` — compatibility, auth-port, command/query, correlation/idempotency, typed failure and event-reconnect behavior; compatibility reduction consumes the M10 terminal and does not infer authority from transport status.
3. `client-conformance` — fake-M10 normal, denial, stale, reconnect, cancellation, timeout-reconciliation and version-skew fixtures.
4. `user-cli-shell` — `ustc-agent` command tree, human/machine rendering and stable exit/output semantics.
5. `inbound-mcp-shell` — selected least-privilege tool/resource discovery and invocation over client-core; first slice is public-read `market.package.list` using the M10 operation/schema registry.
6. `dioxus-fullstack-contract` — generated call facade and target feature confinement.
7. `app-state` — deterministic presentation reducer.
8. `routes` and `design-system` — accessible navigation, display and forms.
9. `market-ui`, `agent-ui` and product UI modules — typed projection and intent only.
10. `platform-web` — SSR/CSR/PWA/session behavior.
11. `platform-android` — endpoint/session/lifecycle/Custom Tab/package behavior.
12. later `platform-windows` admitted peer and later-candidate `platform-ios`/other desktop peers; Windows promotion to required scope is a separate acceptance amendment.

## 16. Exit gate

M80 client-core is standalone-ready only when fake-M10 fixtures prove compatibility admission, typed normal/denial outcomes, monotone reconnect, timeout reconciliation, cancellation semantics and adapter-independent equivalent reduction.

The bounded Affairs-first prerequisite may prove only bootstrap/capability/compatibility reduction and exact Web HTTP projections. It leaves this standalone-ready gate and planned `CLIENT-007`/`CLIENT-009` rows open until the omitted peer/event/reconnect/cancellation/host-matrix evidence exists.

The user CLI is integration-ready only when a real M10 read-only path proves versioned JSON/NDJSON, stdout/stderr separation, stable exit classes, auth isolation and no operator/domain dependency. The inbound MCP adapter is integration-ready only when one external-client conformance path proves bounded discovery/invocation, tenant/grant isolation and no `ustc-agentctl`, domain or M51 reach-through.

Dioxus Web/PWA is accepted only after browser smoke proves page delivery, one real query/command/event journey, accessibility and console/network cleanliness. Android is accepted only after emulator and real-device launch, validated HTTPS origin, secure session, the same semantic journey, lifecycle/reconnect and Custom Tab evidence.

The module is accepted only when all required initial peer surfaces pass their bound active acceptance rows, equivalent server fixtures produce equivalent semantic state, no shell spawns another as its production path, and replacing one outer adapter leaves the other adapters and backend domain/runtime/Plugin crates unchanged. Windows is excluded from this current module acceptance gate until explicitly promoted.
