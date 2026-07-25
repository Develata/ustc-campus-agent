# M80 — Dioxus Fullstack Multi-client

## Metadata

- `Module ID`: `M80`
- `Status`: Accepted blueprint; no Dioxus dependency, Fullstack app or target artifact implemented
- `Implementation State`: `planned`
- `Version`: `m80-dioxus-fullstack/v1`
- `Last Review`: `2026-07-25`
- `Decision`: [`../../adr/0009-dioxus-multi-client-shell.md`](../../adr/0009-dioxus-multi-client-shell.md)
- `Owning Contract`: [`../../contracts/client-shell.md`](../../contracts/client-shell.md)
- `Primary code area`: future `apps/ustc-client/`, attached server-side through `apps/ustc-agentd/`

## 1. Purpose

M80 provides one shared Rust/Dioxus Fullstack product interface for mandatory Web/PWA and Android targets. It owns routes, components, accessible presentation, form drafts, client-side event reduction, Web SSR/CSR/hydration, generated first-party client calls and narrow target adapters.

Web is the first proof surface. Android is a required peer target after the shared ingress/event/recovery contract is executable. iOS and desktop are later scope.

M80 displays server-owned state and submits typed user intent through M10 Fullstack ingress. It does not perform canonical product calculations or mutations.

## 2. Non-goals

- owning Agent, Market, grant, Plugin, source, product or audit rules;
- direct database, repository, filesystem, process, executor or provider access;
- placing business logic in components, hooks, reducers or server-function transport adapters;
- letting a server function bypass M00/M10 admission or application command/query ports;
- claiming task success from optimistic/local UI state;
- implementing iOS/desktop before their target gates enter scope;
- claiming Web/Android support from compilation alone.

## 3. Owned objects and state

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

These are presentation facts only. Losing them may harm UX but cannot corrupt backend truth.

## 4. Public inputs and outputs

Input from M10:

```text
versioned server-function/HTTP response and error values
compatibility envelope and minimum-supported-client outcome
typed event/stream values with monotone cursor
server-owned run/market/product projections
```

Output to M10:

```text
versioned query/command values
user intent and current preconditions
idempotency/correlation identity
reconnect cursor
client build/target/protocol identity
```

Target ports:

```text
ExternalNavigation
NotificationPort
SecureSessionPort
LocalArchivePort
PlatformInfo
ServerEndpointPort
```

Unsupported capability returns typed unavailable state. It does not silently switch execution location.

## 5. Fullstack source boundary

Shared Dioxus server-function declarations may live beside the generated client facade so Web and Android compile against the same Rust types. Their server-only implementation is an M10 ingress adapter and may call one admitted application command/query port after authentication, authorization, bounds, idempotency and audit.

Forbidden from M80/shared Fullstack code:

- backend domain crate types as UI state;
- concrete repository/database/queue/provider/MCP/executor SDKs;
- Dioxus signals/hooks as durable authority;
- direct server-only business fallback from components;
- unversioned request/event types crossing an independently deployed Android boundary.

Dioxus types terminate in the Fullstack application. Application/domain/runtime/Plugin contracts remain framework-neutral.

## 6. Dependency direction

Allowed dependencies:

- exact-pinned Dioxus/DX release and target features selected for server, Web and Android;
- versioned M10 ingress DTO/error/event/compatibility values;
- generated Dioxus server-function client calls and typed stream handles;
- target-specific Web/Android adapters;
- presentation-only libraries after size/security review.

Server-only M10 adapter features may depend on public application ports but never on concrete infrastructure. Client target features must not compile backend domain, repository, provider or executor implementations.

## 7. Lifecycle

```text
app/page bootstrap
→ validate client build/protocol/server compatibility
→ restore target-appropriate admitted session
→ load server projection
→ Initial/Loading
→ Ready | Empty | Error | Offline | ReauthRequired | UpgradeRequired
→ submit correlated user intent
→ Pending presentation only
→ accept typed response/events
→ reduce monotone authoritative projection
→ reconnect/refresh/re-auth/upgrade when required
```

Transport disconnect is not run completion. Unknown event versions/variants or non-monotone sequences require refresh or upgrade.

## 8. Failure and recovery

- Server unavailable/offline: preserve safe drafts, show explicit state, invent no success.
- Unknown schema/event or stale projection: request refresh and block conflicting intent.
- Unsupported client/server version: show `UpgradeRequired`; no unsafe application dispatch.
- Reauthentication: use target SecureSessionPort; no raw password in shared app state.
- Timeout after possible acceptance: query by idempotency/correlation ID before retry.
- Client cache loss: reload from backend.
- SSR/renderer/WebView failure: backend state remains unchanged.
- Unsafe Markdown/HTML/tool output: sanitize or render as bounded text/artifact.
- Unsupported native capability: typed unavailable UI, no arbitrary bridge fallback.

## 9. Configuration and secrets

Client config contains a validated HTTPS server origin, supported protocol/schema versions, build/target identity, non-secret capabilities and presentation defaults. Web and Android use separate secure-session adapters. Model/provider/Plugin/source secrets never enter client config or logs.

Android server URL cannot default to loopback in production. Build/profile configuration and runtime admission both validate the intended origin.

## 10. Observability

Client diagnostics include build/target/protocol/server version, route, safe UI state, event cursor, reconnect and stable error code. No raw credentials, prompts, profile data or tool payloads in normal telemetry. Accessibility, performance and lifecycle evidence is target-specific.

## 11. Extension and replacement

Web and Android are required peer adapters beneath one application/presentation model. iOS and desktop may join later without changing backend domain contracts. Native bridges expose narrow typed commands, never arbitrary eval/process access.

The Dioxus renderer/transport may be replaced while preserving application command/query semantics and durable state. Replacement does not imply that an old Android artifact can ignore protocol compatibility.

## 12. Performance path

Hot paths are initial Web payload, SSR/hydration, event reduction, large lists, Android startup and bounded untrusted content. Avoid duplicating full backend histories in component state. Keep reducers deterministic and target branches isolated. Set target-specific Web bundle/SSR latency, Android startup/memory and event-backpressure budgets before acceptance.

## 13. Scope boundary

**Required initial product scope**

- exact-pinned Dioxus/DX and target features after source revalidation;
- cohesive apps/ustc-client Fullstack source and M10 server attachment;
- Web/PWA page/asset delivery with SSR/hydration or explicit CSR decision;
- generated first-party query/command calls and typed event reducer;
- one Market/run/product journey;
- loading/empty/error/offline/re-auth/upgrade/pending/terminal states;
- Docker Compose server startup/readiness/restart/read-back;
- Android emulator and real-device build/launch/remote journey;
- responsive keyboard/accessibility and reconnect behavior.

**Later**

- iOS package after macOS/Xcode/signing/device evidence;
- desktop package after real native-desktop demand;
- opt-in notifications and local archive;
- richer product/public views over unchanged application ports.

**Explicit non-goals**

- backend business logic in Dioxus components or transport adapters;
- offline peer authority or local database truth;
- direct Plugin/provider execution;
- target support claimed only from compilation.

## 14. Small-module decomposition

1. `fullstack-contract` — versioned DTO/error/event/compatibility values.
2. `server-function-client` — generated typed call facade; server implementation owned by M10 adapter.
3. `event-client` — typed SSE/stream cursor/reconnect/version handling.
4. `app-state` — deterministic presentation reducer.
5. `routes` — navigation and page composition.
6. `design-system` — accessible reusable display/form components.
7. `market-ui` — browse/detail/install intent only.
8. `agent-ui` — finite run/plan/tool/review projection and intents.
9. product UI modules — render typed M70/M71/M72 projections only.
10. `platform-web` — SSR/CSR/PWA/session behavior.
11. `platform-android` — endpoint/session/lifecycle/Custom Tab/package behavior.
12. later `platform-ios` and `platform-desktop` peers.
13. `client-conformance` — fake ingress/event/version fixtures shared across targets.

## 15. Exit gate

M80 is standalone-ready when its shared reducer/generated client facade passes fake-ingress fixtures for normal, stale, incompatible-version, reconnect, pending, denial and terminal states.

Web/PWA is accepted only after browser smoke proves Fullstack page delivery, one real query/command/event journey, accessibility/console/network cleanliness and no direct backend/domain dependency. Android is accepted only after emulator and real-device launch, validated HTTPS server configuration, secure session, the same semantic journey, lifecycle/reconnect and Custom Tab evidence. Docker Compose server and Android/server version-skew cases must pass before the required target set is accepted.
