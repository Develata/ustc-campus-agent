# Typed multi-client core and peer shell contract

## Metadata

- `Status`: Accepted target architecture with an approved Affairs-first compatibility/operation-registry prerequisite plus bounded client-core, ordinary-user Affairs CLI, operation-specific loopback thin-Web and Android debug-bridge proof; production auth/remote HTTP/stream, inbound MCP and Dioxus clients remain planned
- `Version`: `client-shell/v2.3`
- `Last Review`: `2026-09-04`
- `Owning Plan`: [`M80 Client Core and Interaction Shells`](../plan/modules/80-dioxus-multi-client.md)
- `Counterpart Plans`: [`M10 Application Ingress Host`](../plan/modules/20-application-api-host.md), [`platform authority`](../plan/03-platform-authority.md)
- `Decisions`: [`ADR-0009`](../adr/0009-dioxus-multi-client-shell.md), [`ADR-0010`](../adr/0010-typed-client-peer-adapters.md)
- `Counterpart Interfaces`: [`interfaces.md`](interfaces.md), [`cli.md`](cli.md), [`module-boundaries.md`](module-boundaries.md)
- `Acceptance`: planned `CLIENT-007` through `CLIENT-010`; long-horizon `CLIENT-001` through `CLIENT-006`, `WEB-*` and applicable `DEP-*`

## 1. Purpose

USTC Campus Agent uses one framework-neutral typed client core beneath three peer outer adapters:

```text
                         M10 admitted application API
                    versioned call/result/error/event seam
                                      │
                           framework-neutral client core
                     ┌────────────────┼────────────────┐
                     │                │                │
              Dioxus Web/Android   ustc-agent      inbound MCP
              presentation shell   user/automation external Agent tools/resources
```

The common core owns client-side transport semantics, compatibility reaction, authentication ports, correlation/idempotency propagation, event reconnect and safe projection reduction. It does not own server/domain decisions.

Peer adapters reuse code through the typed core. They do not invoke one another as subprocesses. In particular, Dioxus Web/Android MUST NOT use `GUI → ustc-agent executable → server` as its production path.

`ustc-agentctl` remains a separate operator/developer CLI and is not part of this contract's least-privilege user/automation surface.

### Current bounded evidence

`crates/client-core` and the real `apps/ustc-agent` binary retain one typed `affairs get/lookup` path over numeric loopback framing. `ustc-agentd serve-web` additionally retains one loopback-only, operation-specific Axum/HTTP presentation proof whose static page renders a public-redacted server-owned `ClientResponseDto::Available` without making domain decisions. The CLI core validates endpoint/protocol/value bounds, propagates correlation/idempotency/provenance, reduces typed M10 responses/errors, emits one canonical JSON result and maps stable exit classes without importing backend or operator implementations. The colocated Web proof is not a replacement for that core and does not count as the future peer Dioxus shell. This is partial evidence only: no production profile/auth, remote HTTP/TLS, NDJSON/SSE, reconnect/cancellation/version-skew matrix, inbound MCP or Dioxus target exists.

The bounded Web page is deliberately not a second client authority. It accepts only a stable procedure ID, calls the colocated same-origin endpoint and presents prerequisites, ordered steps, explicit unknown effective/deadline time, entry points, contacts, safe public lineage (`source_id`, `evidence_set_digest`, `materialization_receipt_id`, `revision_count`), freshness, conflict and uncertainty already present in the typed response. The server consumes the response-only public capability internally and returns only the public-redacted lookup result; JavaScript never receives a capability or raw revision identity. It never computes procedure truth, freshness, conflict resolution, authorization or eligibility. Its loopback bind, embedded assets and retained source-grounded noncanonical fixture make it demo evidence, not the final Dioxus peer adapter or a remotely deployable public service.

The bounded `apps/ustc-android-demo` package hosts that same-origin Web MVP in a native Android `WebView` after validating one path-free origin. HTTP is admitted only for `127.0.0.1`/`localhost`; remote origins require HTTPS. The Activity owns loading/offline/retry, endpoint preference, back navigation and WebView lifecycle only. It exposes no JavaScript bridge, local product/tool execution or operator path; SSL errors, mixed content, file/content access, remote cleartext, malformed origins and non-Web navigation fail closed. ADB reverse is the explicit local-demo transport and does not create a LAN/public server claim.

This bounded debug artifact is supporting partial evidence only for long-horizon `CLIENT-002`, which still requires the Dioxus/shared-client contract, authenticated HTTPS, secure session storage, real-device lifecycle/reconnect, version compatibility and Custom Tab evidence. It is not projected as an active acceptance row.

## 2. Required client surfaces

### 2.1 Dioxus

Required product targets remain:

- Web/PWA;
- a native Linux Fullstack server deployed through Docker Compose;
- Android.

Web is proven first. Android is a mandatory peer target after the shared ingress/event/recovery path is executable. Windows is an explicitly admitted later desktop peer target, not a current required release gate; other desktop targets and iOS remain later candidates.

Dioxus owns routes, accessible components, forms, presentation reduction, Web SSR/CSR/hydration and target adapters. It may use generated Dioxus server-function calls when they preserve this contract's M00/M10 admission and result semantics.

### 2.2 `ustc-agent`

`ustc-agent` is the user-facing and automation-facing headless client. It consumes explicit versioned M10 HTTP/JSON and typed event streams through client-core. Its noninteractive machine modes are stable public contracts; they are not debug printouts.

The initial CLI is read-oriented and least privilege. It does not inherit `ustc-agentctl` administrative commands or credentials.

The retained fixture proof precedes that production transport: it exposes only public `affairs get` and capability-stdin result lookup over numeric loopback. It accepts no raw session/operator authority and makes no remote-client or authentication claim.

### 2.3 Inbound MCP adapter

The inbound MCP adapter exposes selected USTC Campus Agent tools/resources to external Agent clients. It maps MCP protocol values at the outer boundary, invokes admitted M10 operations through client-core, and maps bounded typed outcomes back to MCP.

This direction is distinct from M51:

```text
external Agent → inbound MCP adapter → client-core → M10
platform M40   → M51 outbound MCP binding/executor → external MCP server
```

The two surfaces share neither lifecycle authority nor credential/session state merely because both speak MCP.

## 3. Authority boundary

The client core and outer adapters MAY own:

```text
client build/target/protocol support
validated server endpoint
client auth-profile reference
request correlation/idempotency identity
connection/reconnect state
last accepted event cursor
safe reduced server projection
form draft / command input
presentation/serialization preferences
MCP-visible selected capability projection
```

They MUST NOT own or decide:

- tenant/user/session authority;
- package/install/grant state;
- HarnessRun/AgentRun transitions;
- Agent tool routing or Plugin execution;
- source/revision/publication truth;
- effect intent/receipt or audit truth;
- canonical product calculation or mutation;
- operator/admin authorization.

Client cache, CLI output and MCP results are projections. They may report `pending`, but cannot report authoritative success before a typed M10 result/event establishes it.

## 4. Directional boundary values

`B-M80-M10-CALL` carries request instances produced by M80 client behavior under the M10-owned versioned `client-protocol` schema:

```text
ClientBuild / ClientTarget / ClientProtocolVersion
AuthenticatedClientSession projection
ClientRequestCorrelation / IdempotencyKey
versioned query or command intent
current precondition identity
reconnect cursor where applicable
```

`B-M10-M80-RESULT` carries M10-produced values:

```text
versioned accepted/denied result
stable safe error
compatibility envelope / UpgradeRequired
server-owned read projection
correlation/idempotency outcome
```

`B-M10-M80-EVENT` carries M10-produced values:

```text
versioned event projection
monotone cursor
heartbeat/resync/refresh outcome
terminal versus nonterminal state
```

Shared Rust source does not merge authority. M10 owns the public wire schema and result/event production; M80 owns client behavior and produces request instances that conform to that schema. `client-core` depends on the M10-owned protocol carrier, while M10 server code never depends on `client-core`. M10 still cannot reinterpret client intent as a domain command until admission succeeds.

Unknown versions, variants or non-monotone cursors fail closed and yield refresh, resync or upgrade—not a nearby interpretation.

The first retained compatibility seam is exact: M10 advertises current/supported/minimum protocol major `1` through header-free `GET /api/v1/server/info`; `GET /api/v1/client/capabilities` and `GET /api/v1/affairs/{procedure_id}?as_of=<unix-ms>` require `X-USTC-Client-Protocol-Major`. M10 returns typed `upgrade_required` for an older major and typed `incompatible_protocol` for a newer, absent or unparseable major before application dispatch. M80 reduces the returned variant as-is; it MUST NOT infer one from HTTP `426`/`409`, compare majors as authority or flatten an M71 terminal into transport status. The machine outer result remains `ustc-client-result/v1`.

## 5. Common client-core behavior

For every peer adapter, M80 `client-core` consumes the M10-owned framework-neutral `client-protocol` values and performs the same semantic sequence:

```text
validate local endpoint/profile/protocol
→ obtain target-appropriate admitted session projection
→ send client build/target/protocol and bounded request
→ preserve correlation/idempotency/precondition identity
→ receive typed accepted/denied/compatibility outcome
→ subscribe/resume from monotone event cursor when required
→ reduce safe server projection
→ reconcile timeout-after-possible-acceptance before retry
```

The core exposes typed operations and failures. It does not expose command-line strings, Dioxus signals/components or MCP SDK objects.

Required common failure classes include:

```text
InvalidClientInput
InvalidEndpoint
AuthenticationRequired
Forbidden
CapabilityUnavailable
IncompatibleProtocol / UpgradeRequired
StalePrecondition
Conflict
RateLimited
TransportUnavailable
TimeoutOutcomeUnknown
MalformedOrUnknownServerValue
CursorRequiresRefresh
ServerDenied
```

Transport and shell adapters may add outer failure detail, but cannot weaken or silently remap these semantics.

## 6. Dioxus Fullstack boundary

One shared Dioxus application provides:

```text
shared components / routes / presentation reducer
        ├── Web SSR/CSR + hydration + PWA
        ├── Android WebView package
        ├── iOS later
        └── desktop later
```

A Dioxus server function is an Axum-compatible M10 HTTP endpoint and generated client call. Its server-only body MAY:

1. extract admitted request/session facts;
2. validate version, bounds, authorization, idempotency and preconditions;
3. call one public application command/query port;
4. map the typed result/error/event.

It MUST NOT call concrete repositories, databases, Plugin/MCP executors, provider SDKs or journals directly.

Dioxus component/router/signal/WebView types terminate in the Dioxus outer adapter. They do not enter client-core, M10 application ports or backend contracts.

## 7. User CLI contract

`ustc-agent` human mode may provide readable rendering. Machine mode MUST provide:

- an explicit versioned JSON result envelope or NDJSON event stream;
- protocol data only on stdout;
- redacted diagnostics only on stderr;
- stable exit classes mapped from typed client failures;
- `--non-interactive` behavior with no prompt or terminal dependency;
- bounded output and deterministic ordering where the owning result contract is ordered;
- explicit build/server/protocol identity in diagnostic or result metadata;
- typed cancellation and reconciliation rather than process-kill-as-cancellation.

Secrets MUST NOT be accepted in argv, printed in output or inherited from `ustc-agentctl` operator configuration. Human and machine renderers consume the same typed result; neither reparses the other's output.

The exact command tree is owned by [`cli.md`](cli.md), while application-operation identities and per-adapter allowlists are owned by [`interfaces.md`](interfaces.md). CLI commands, MCP tools and graphical actions that project the same operation preserve its typed result, permission, provenance, audit and failure semantics, but the adapters need not expose identical registries. Authentication and target-local maintenance are not MCP business tools. Implementation enters only with active planned acceptance rows and future bindings.

## 8. Inbound MCP contract

The inbound MCP adapter is a server surface consumed by an external personal Agent acting as MCP client. Its initial remote profile uses reviewed MCP Streamable HTTP. Local stdio/relay is later and requires a separately accepted deployment/session contract.

The adapter MAY expose only explicitly registered selected tools/resources whose underlying M10 operation and exact schema digest are admitted for the delegated profile. It MUST:

- advertise bounded names, descriptions, schemas and result sizes;
- bind every request to external caller/session plus delegated tenant/user/profile context;
- revalidate current server capability and authorization on every call;
- preserve typed denial and no-fallback behavior;
- label returned campus/source content as data, not executable instruction;
- return provenance/freshness/uncertainty fields when the owning product result carries them;
- keep operator/admin commands absent;
- reach no domain module, repository, ToolGateway executor or M51 session directly.

The first retained MCP slice exposes only a reviewed `public-read` projection, initially `market.package.list`. Campus operations enter only after their owning product/source contracts exist; private read/draft operations require a later explicit delegated-profile slice. Schema, permission, result-data or effect widening stales prior grants and removes the changed operation from discovery until re-approval.

The adapter is not a second Agent loop. External Agents may call deterministic resources/tools or an explicitly admitted central Agent operation; the adapter itself does not own planning, model invocation or run completion.

## 9. Authentication and credentials

Each peer uses a target-appropriate `ClientAuthPort`:

- Web: browser-appropriate session handling;
- Android: secure token/session storage through `SecureSessionPort`;
- `ustc-agent`: a least-privilege user auth profile/reference outside argv and machine output; the preferred later contract candidate is server-mediated browser pairing with one-time bounded exchange, never direct CAS-password/ticket handling;
- inbound MCP: an explicitly delegated user/service profile scoped to selected tools/resources.

No adapter receives raw operator credentials, provider secrets, Plugin credentials, raw USTC passwords or CAS sessions. Authentication failure reaches no application operation.

A shared token cache across peer adapters is not implied. Shared semantics and separate secure storage are compatible.

## 10. Platform ports, configuration and deployment

Shared/client-core code accesses target capability only through narrow typed ports:

- `ClientTransport` and `ClientEventTransport`: versioned M10 calls/events only;
- `ClientAuthPort`: target-appropriate admitted session projection;
- `ExternalNavigation`: browser tab or Android Custom Tab for USTC/iCourse link-out;
- `NotificationPort`: opt-in local notification projection, never task-completion authority;
- `SecureSessionPort`: browser-appropriate session and Android secure credential/token storage;
- `LocalArchivePort`: optional user-controlled export distinct from durable server memory;
- `PlatformInfo`: build, target and capability facts;
- `ServerEndpointPort`: validated HTTPS server origin for independently packaged clients.

Framework-neutral client-core cannot directly invoke filesystem, WebView JavaScript, keychain/keystore, notifications, process or MCP/terminal APIs. Unsupported capability returns a typed unavailable state.

Public client configuration contains only validated server HTTPS origin, supported protocol/schema versions, client build/target identity, non-secret capability facts, bounded transport defaults and presentation/serialization defaults. `ustc-agent` and inbound MCP add only profile references, never raw credentials or operator config.

The Docker Compose profile owns the server process, dependencies, readiness, persistent volumes, migration/backup/restore and reverse-proxy/TLS wiring. It does not own Android, CLI, MCP or Windows artifacts. Dioxus Fullstack does not provide database/cache/session/mailer implementations; these remain explicit M90/Axum infrastructure choices.

## 11. Lifecycle and concurrency

```text
bootstrap
→ validate config/build/protocol
→ establish admitted session
→ compatibility/capability preflight
→ Ready | Offline | ReauthRequired | UpgradeRequired
→ submit correlated operation
→ Pending
→ typed result/events
→ reconcile or terminal projection
```

Multiple concurrent operations remain separated by correlation identity. A shell may stop observing without cancelling the accepted server operation. Cancellation is a separate typed intent. Reconnect uses the last server cursor and follows the server's resync policy.

The inbound MCP session, CLI process and Dioxus page lifecycle are transport/session projections only. None is a HarnessRun or AgentRun identity.

## 12. Failure and recovery

- API unavailable: explicit unavailable state; no hidden local execution fallback.
- Timeout after possible acceptance: query/reconcile by correlation or idempotency identity before retry.
- Older client major: server-typed `UpgradeRequired` before unsafe dispatch; M80 preserves the typed relation and recovery hint.
- Newer, absent or malformed client major: server-typed `IncompatibleProtocol` before unsafe dispatch; M80 does not relabel it as an upgrade.
- Unknown event/result: fail closed and refresh/upgrade.
- Cursor gap/expiry: explicit resync or full projection reload.
- Reauthentication: use the adapter auth port; preserve no raw secret in common state.
- CLI partial/malformed machine frame: non-success; consumers cannot accept partial bytes as complete.
- MCP disconnect: does not cancel an accepted platform operation.
- Dioxus renderer/WebView failure: backend truth remains unchanged.
- Unsafe Markdown/HTML/tool content: sanitize or emit bounded typed content.

## 13. Source and artifact topology

Start with modules until actual boundaries justify crates/artifacts. The accepted target shape is:

```text
crates/client-protocol/          # M10-owned versioned wire DTO/error carrier; bounded Affairs evidence
crates/client-core/              # M80-owned client behavior; bounded Affairs evidence
apps/ustc-agent/                 # bounded ordinary-user Affairs CLI evidence; production adapters planned
future apps/ustc-client/         # Dioxus Web/Android source; later admitted Windows target
apps/ustc-android-demo/          # bounded debug WebView bridge; not final Dioxus/CLIENT-002
future inbound MCP adapter       # exact package/process placement chosen by first slice
apps/ustc-agentctl/              # existing separate operator/developer CLI
apps/ustc-agentd/                # M10 server composition and ingress
```

The three real peer consumers justify a client-core crate when its first retained slice lands. Do not create empty crates/binaries before exact accepted batch contracts and active planned acceptance bindings exist.

One workspace does not mean one artifact. Server, Web assets, Android package, CLI binary, any MCP process/entrypoint and a later Windows package have independent packaging, version and release evidence.

## 14. Dependency confinement

The final dependency shape MUST satisfy:

```text
M10 client-protocol
  owns versioned wire DTO/error/event/compatibility schemas
  may use only narrow deterministic data-format/digest helpers required to construct those carriers
  must not depend on M80 client-core, peer shells, network/async runtimes or backend domain implementations

M80 client-core
  may depend on M10 client-protocol + narrow transport abstractions
  must not depend on Dioxus, CLI parser, terminal, MCP SDK,
  backend domain implementations, repositories, executors, providers or journals

Dioxus adapter
  may depend on client-core + Dioxus target features

ustc-agent
  may depend on client-core + CLI/rendering/auth-profile adapters

inbound MCP adapter
  may depend on client-core + MCP protocol adapter
```

No peer shell may import or spawn another peer shell as its production path. `ustc-agentctl` and M51 are forbidden dependencies of user CLI/MCP/Dioxus client targets.

## 15. Compatibility

Every independently deployed request carries client build/target/protocol identity. The server declares supported versions and minimum client requirements. An installed Android artifact, CLI binary or MCP adapter may lag the server and therefore receives typed compatibility behavior.

Shared source protects only artifacts built from the same revision. It does not replace:

- versioned DTO/error/event schemas;
- stable unknown-field/unknown-variant rules;
- at least one supported older-client fixture;
- typed unsupported-version rejection before application dispatch;
- explicit migration and rollout policy.

Web may deploy atomically with the server, but still exercises equivalent semantic fixtures.

## 16.1 Windows later-target admission

Windows is an admitted future M80 peer target so the architecture need not be reopened merely to begin a bounded desktop proposal. It is intentionally not part of the current required-target gate. Promotion to a required delivery target needs a separate accepted amendment plus active acceptance rows covering:

- signed installer and update/rollback identity;
- secure session storage and browser login callback/pairing;
- sleep/resume, proxy and reconnect behavior;
- crash recovery and local cache non-authority;
- explicit desktop-only demand such as tray, notification or long-lived run observation;
- protocol compatibility and remote-server read-back on real Windows hosts.

Until promotion, `ustc-agent.exe` is the supported low-cost Windows automation shape once the cross-platform CLI is implemented. Framework support or successful compilation alone does not claim a Windows GUI release.

## 17. Conformance and acceptance

The framework-neutral conformance suite runs every peer adapter against equivalent fake-M10 cases:

- normal read result;
- accepted mutation and pending event projection;
- authentication/policy denial;
- stale precondition/conflict;
- timeout-after-possible-acceptance reconciliation;
- monotone reconnect and cursor refresh;
- cancellation distinct from transport/process closure;
- supported older protocol;
- unsupported/unknown protocol;
- bounded malformed/untrusted result.

`CLIENT-007` proves shared typed-core and peer equivalence. `CLIENT-008` proves no shell-out path and operator privilege isolation. `CLIENT-009` proves the user CLI machine contract. `CLIENT-010` proves the least-privilege inbound MCP boundary.

These rows remain `planned` until every assertion in their executable bindings passes. The bounded Affairs client/CLI path and debug Android bridge are supporting partial evidence, not substitutes for completion of the planned rows. Existing long-horizon Dioxus `CLIENT-001` through `CLIENT-010`, `WEB-*` and deployment cases remain non-active until projected into the active matrix.

## 18. Current status

Accepted now:

- one framework-neutral client core beneath Dioxus, `ustc-agent` and inbound MCP peer adapters;
- separate `ustc-agentctl` operator surface;
- no GUI-to-CLI production subprocess path;
- M10 remains the admitted application boundary and backend authority remains unchanged;
- inbound MCP and M51 outbound MCP are directionally separate;
- required Web/PWA, Docker Compose server and Android targets remain unchanged.
- one bounded debug Android bridge may provide competition/demo access without being relabelled as final Dioxus or `CLIENT-002` acceptance.
- Windows is admitted as a later desktop peer target but is not a current required release gate.

Implemented now as bounded non-production evidence: one framework-neutral client-core and real `ustc-agent` public Affairs get/capability-lookup JSON path against the fixture-only loopback composition, including dependency confinement and real subprocess tests; plus one source-bound debug Android WebView bridge as partial evidence toward planned `CLIENT-002`.

Not implemented: production profile/auth, HTTP/TLS ingress, typed event stream, full peer conformance, inbound MCP adapter, Dioxus application, peer Web/PWA journey, production Android/Windows package or deployment, or complete long-horizon `CLIENT-002`. This partial evidence does not promote those planned rows to pass.
