# Dioxus Fullstack multi-client contract

## Metadata

- `Status`: Accepted target architecture; no Dioxus dependency or client/server Fullstack slice implemented yet
- `Version`: `client-shell/v1`
- `Last Review`: `2026-07-25`
- `Owning Plan`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md)
- `Large-module Blueprints`: [`M10`](../plan/modules/20-application-api-host.md), [`M80`](../plan/modules/80-dioxus-multi-client.md), [`M90`](../plan/modules/90-infrastructure-operations.md)
- `Decision`: [`ADR-0009`](../adr/0009-dioxus-multi-client-shell.md)
- `Counterpart Interfaces`: [`interfaces.md`](interfaces.md)
- `Acceptance`: planned `WEB-*`, `I18N-*`, `CLIENT-*`, applicable `DEP-*`

## 1. Purpose and required targets

USTC Campus Agent uses Dioxus Fullstack as the long-lived first-party application stack. The required product targets are:

- Web/PWA;
- a native Linux Fullstack server deployed through Docker Compose;
- Android.

Web is implemented and proven first. Android is a mandatory peer target once the shared Web ingress, event and recovery contract is executable. iOS and desktop are later scope.

The Fullstack application shares Rust components, presentation state, endpoint values and generated client calls while keeping platform authority on the server:

```text
shared Dioxus components / routes / presentation reducer
        ├── Web SSR/CSR + hydration + PWA
        ├── Android WebView package
        ├── iOS later
        └── desktop later
                 │
                 │ versioned server functions / HTTP / typed streams
                 ▼
M10 ingress in ustc-agentd
        │ admission + application command/query mapping
        ▼
server-owned domain/runtime state
```

Changing the Dioxus renderer or target adapter must not change platform domain contracts. Changing Agent or Plugin internals must not require client rewrites while the supported first-party ingress contract remains compatible.

## 2. What Dioxus Fullstack owns

Dioxus Fullstack provides the application-level integration of:

- shared Rust UI components, routes and presentation reducer;
- Web SSR, page delivery, hydration/CSR and assets;
- Android packaging of the shared application;
- Axum-compatible typed server-function declarations and generated client calls;
- typed forms, request/response values, errors, SSE/stream/WebSocket handles;
- target adapters for session storage, external navigation, notifications, archive and platform facts.

It does not own databases, caches, sessions, mailers or platform domain semantics merely because they are reachable from an Axum handler.

## 3. Authority boundary

The shared app owns only presentation facts:

```text
Route / PageId
ViewModel / FormDraft
ClientIntentCorrelation
ApiConnectionState
EventCursor and reduced server projection
PresentationState:
  Initial | Loading | Ready | Empty | Error | Offline | ReauthRequired | UpgradeRequired
TargetCapabilityState
Theme/locale/accessibility preferences
```

It must not own or decide:

- tenant/user/session authority;
- Market package/install/grant state;
- HarnessRun/AgentRun transitions;
- Agent tool routing or Plugin execution;
- source/revision/publication truth;
- effect intent/receipt or audit truth;
- canonical product calculations or mutations.

Client cache and optimistic UI are projections. They may show `pending`, but cannot claim an authoritative transition before the server accepts it.

## 4. Fullstack ingress contract

A Dioxus server function is an Axum-compatible HTTP endpoint and a generated Rust client call. It is a valid first-party ingress adapter, not merely SSR plumbing.

A first-party request follows:

```text
Web/Android typed call
→ version/size/shape decode
→ M00 actor/session admission
→ M10 authorization, precondition/idempotency and audit context
→ one owning application command/query port
→ typed result/error/event projection
→ Web/Android reducer
```

A server function MAY call an application command/query port after all M00/M10 gates pass. It MUST NOT call concrete repositories, database clients, Plugin executors, provider SDKs, domain internals or durable journals directly.

Public REST/SSE endpoints may coexist for CLI, integrations or intentionally public resources. They are peer transport adapters over the same application ports. They do not duplicate business logic or create a second authority.

`ClientApi` names the semantic first-party client facade. Its Dioxus implementation SHOULD use generated server-function calls and typed stream handles rather than a parallel hand-written request layer. Exact public/heterogeneous routes remain registered in [`interfaces.md`](interfaces.md).

## 5. Source and artifact topology

Start with one cohesive Fullstack application surface and internal modules. One source/workspace does not mean one artifact:

```text
apps/ustc-client/                  # shared Dioxus Fullstack application source
  src/app/                         # routes/components/view models
  src/ingress/                     # shared declarations + generated client facade
  src/platform/                    # web/android; iOS/desktop later
  assets/

apps/ustc-agentd/                  # server composition root
  attaches Dioxus SSR/assets/server functions to Axum
  supplies admitted application ports and infrastructure
```

The exact Cargo package/feature shape is finalized by the first implementation slice. Begin with modules and target features; extract a shared crate only for a real independent consumer, privilege/deployment boundary or measured build isolation.

Dioxus component/router/signal/WebView types terminate in the Fullstack application boundary. Domain/runtime/Plugin crates do not depend on Dioxus. Server-only ingress adapters may depend on Dioxus/Axum transport types plus public application ports, never concrete backend adapters.

Separate build/release surfaces are mandatory:

- server feature/native Linux image for Docker Compose;
- Web assets/WASM and optional SSR/hydration;
- Android package and signing/release metadata;
- later iOS/desktop artifacts.

## 6. Inputs, outputs and events

Inputs from M10/application ingress include:

```text
versioned response/error/event DTOs
server/application compatibility envelope
monotone event cursor
server-owned run/market/product projections
minimum/supported client version facts
```

Outputs from Web/Android include:

```text
versioned query/command DTOs
user intent and current preconditions
idempotency/correlation identity
reconnect cursor
client build/target/protocol identity
```

The shared reducer handles at least:

- initial/loading/ready/empty/error/offline states;
- monotone event sequence and reconnect cursor;
- stale projection and reauthentication;
- pending intent without invented success;
- upgrade-required/incompatible-server outcomes;
- terminal run state distinct from transport disconnect.

Unknown schema/event variants or non-monotone sequences fail closed and request refresh or upgrade; they are not silently interpreted as a nearby state.

## 7. Platform ports

Shared components access target capability only through narrow typed ports:

- `ExternalNavigation`: browser tab or Android Custom Tab for USTC/iCourse link-out;
- `NotificationPort`: opt-in local notification projection, never task-completion authority;
- `SecureSessionPort`: browser-appropriate session and Android secure credential/token storage;
- `LocalArchivePort`: optional user-controlled export distinct from durable server memory;
- `PlatformInfo`: build, target and capability facts;
- `ServerEndpointPort`: validated HTTPS server origin for independently packaged clients.

Shared UI cannot invoke filesystem, WebView JavaScript, keychain/keystore, notifications or process APIs directly. Unsupported capability returns a typed unavailable state.

## 8. Lifecycle and target order

```text
app/page bootstrap
→ validate client build/protocol/server compatibility
→ restore admitted session through target port
→ load server projection
→ Initial/Loading
→ Ready | Empty | Error | Offline | ReauthRequired | UpgradeRequired
→ submit correlated user intent
→ Pending presentation only
→ accept typed response/events
→ reduce monotone authoritative projection
→ reconnect/refresh/re-auth/upgrade when required
```

Target order:

1. **Web/PWA** — prove Fullstack server start, page/asset delivery, optional SSR/hydration, authentication, typed query/command, event reconnect, accessibility and responsive behavior.
2. **Android required target** — reuse components/reducer/server functions; prove emulator and real-device launch, validated HTTPS endpoint, secure session, reconnect, lifecycle, Custom Tab and release package.
3. **iOS later** — enter scope only with macOS/Xcode/signing/device acceptance.
4. **Desktop optional later** — enter scope when a real window/local integration need exists.

Compilation alone never proves target support.

## 9. Mobile/server compatibility

Shared source types prevent mismatch only for artifacts built from the same revision. Installed Android clients can lag the server, so every independently deployed boundary has an explicit compatibility policy:

- versioned server-function route/DTO/error/event contract;
- stable unknown-field/unknown-variant behavior;
- client build and protocol identity on admission;
- supported version window and migration policy;
- typed `UpgradeRequired` before unsafe application dispatch;
- compatibility fixtures for at least one supported older Android protocol;
- no server rollout that silently reinterprets a request from an installed client.

Web may deploy atomically with the server; Android compatibility is still tested independently.

## 10. Failure and security

- API/server unavailable: preserve safe drafts, show explicit offline state, invent no success.
- Timeout after possible acceptance: query by correlation/idempotency identity before retry.
- Reauthentication: use the target `SecureSessionPort`; no raw password storage in shared app state.
- Unsupported client/server protocol: return `UpgradeRequired` before application mutation.
- Renderer/WebView failure: backend truth remains unchanged.
- Unsafe Markdown/HTML/tool output: sanitize or render as bounded text/artifact.
- Unsupported native capability: typed unavailable state, no arbitrary bridge fallback.
- Server-function adapter import/reach-through: fail dependency/architecture checks.
- Logs exclude raw credentials, prompts, private profile data and tool payloads.

## 11. Configuration and deployment

Public client configuration contains only:

- validated server HTTPS origin;
- supported protocol/schema versions;
- client build/target identity;
- non-secret feature/capability facts;
- presentation defaults.

The Docker Compose profile owns the server process, dependencies, readiness, persistent volumes, migration/backup/restore and reverse-proxy/TLS wiring. It does not own the Android artifact. The Android package points to the deployed server and uses target secure-session facilities.

Dioxus Fullstack does not provide database/cache/session/mailer implementations; these remain explicit M90/Axum infrastructure choices.

## 12. Scope and decomposition

**Required initial product scope**

- exact-pinned Dioxus/DX feature set after source revalidation;
- one Fullstack app source with server, Web and Android target features;
- Web/PWA page/asset delivery and typed first-party query/command/event journey;
- Docker Compose server startup/readiness/restart/read-back;
- Android emulator plus real-device journey against the deployed server;
- stable errors, compatibility envelope, reconnect and upgrade-required behavior;
- accessible responsive shared UI with target-specific QA.

**Later**

- iOS target after macOS/Xcode/signing/device gate;
- desktop packaging after a real native capability requirement;
- opt-in notifications and local archive;
- richer public REST/SSE surfaces over unchanged application ports.

**Small-module decomposition**

1. `fullstack-contract` — versioned DTO/error/event/compatibility values.
2. `server-function-ingress` — Dioxus/Axum declarations and admitted M10 mapping.
3. `event-client` — typed SSE/stream cursor/reconnect/version handling.
4. `app-state` — deterministic presentation reducer.
5. `routes` — navigation and page composition.
6. `design-system` — accessible reusable display/form components.
7. `market-ui` — browse/detail/install intent only.
8. `agent-ui` — finite run/plan/tool/review projection and intents.
9. product UI modules — typed M70/M71/M72 projections only.
10. `platform-web` — browser/PWA/session behavior.
11. `platform-android` — endpoint/session/lifecycle/Custom Tab/package behavior.
12. later `platform-ios` and `platform-desktop` peers.
13. `client-conformance` — fake ingress/event/compatibility fixtures shared across targets.

## 13. Replacement and acceptance

The boundary is accepted only when:

- Web/PWA passes `CLIENT-001`, applicable `WEB-*`, one typed server-function query/command and one typed event stream;
- Android passes `CLIENT-002` on emulator and real device against the deployed server;
- archive/memory and offline behavior pass `CLIENT-003/004`;
- installed-client/server compatibility and typed upgrade behavior pass `CLIENT-005`;
- server-function admission and no-repository/executor reach-through pass `CLIENT-006`;
- Docker Compose Fullstack read-back passes the applicable `DEP-*` case;
- equivalent server fixtures reduce to equivalent semantic state on Web and Android;
- replacing Dioxus transport/UI does not change domain/runtime/Plugin crates or persisted authority schemas.

## 14. Current status

Implemented now: none of the Dioxus application, server-function ingress, HTTP/SSE host, Web journey, Android package, auth/session flow or Compose Fullstack profile.

Accepted now: Dioxus Fullstack as the long-lived Web/Android stack; Web-first then mandatory Android rollout; Axum-compatible server functions as admitted first-party ingress; optional public HTTP peer adapters; target ports; independent artifact/version compatibility; authority and dependency constraints.

This contract does not claim a runnable client or promote planned acceptance rows into the active matrix.
