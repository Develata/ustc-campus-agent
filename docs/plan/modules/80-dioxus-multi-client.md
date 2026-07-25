# M80 — Dioxus Multi-client

## Metadata

- `Module ID`: `M80`
- `Status`: Accepted blueprint; no client crate or Dioxus dependency implemented
- `Version`: `m80-dioxus-client/v0`
- `Last Review`: `2026-07-25`
- `Decision`: [`../../adr/0009-dioxus-multi-client-shell.md`](../../adr/0009-dioxus-multi-client-shell.md)
- `Owning Contract`: [`../../contracts/client-shell.md`](../../contracts/client-shell.md)
- `Primary code area`: future `apps/ustc-client/`

## 1. Purpose

`M80` provides one shared Rust/Dioxus product interface for Web/PWA first and later desktop/mobile. It owns routes, components, accessible presentation, form drafts, client-side event reduction, SSR/page delivery and narrow target adapters.

It displays server-owned state and submits typed user intent through `M10` HTTP JSON/SSE. It does not perform product calculations or canonical mutations.

## 2. Non-goals

- owning Agent, Market, grant, Plugin, source, product or audit rules;
- direct database, filesystem, process, executor or provider access;
- using Dioxus server functions as an alternate business API;
- claiming task success from optimistic/local UI state;
- duplicating backend planning, validation or ranking logic;
- implementing all targets before Web/PWA lifecycle proof.

## 3. Owned objects and state

```text
Route / PageId
ViewModel / FormDraft
ClientIntentCorrelation
ApiConnectionState
EventCursor and reduced server projection
PresentationState:
  Initial | Loading | Ready | Empty | Error | Offline | ReauthRequired
TargetCapabilityState
Theme/locale/accessibility preferences
```

These are presentation facts only. Losing them may harm UX but cannot corrupt backend truth.

## 4. Public inputs and outputs

Input from `M10`:

```text
versioned response/error/event DTOs
monotone event cursor
server-owned run/market/product projections
```

Output to `M10`:

```text
versioned query/command DTOs
user intent and current preconditions
idempotency/correlation identity
reconnect cursor
```

Target ports:

```text
ExternalNavigation
NotificationPort
SecureSessionPort
LocalArchivePort
PlatformInfo
```

Unsupported target capabilities return typed unavailable state. They do not switch execution location silently.

## 5. Dependency direction

Allowed dependencies:

- Dioxus stable release/features selected for the current target;
- generated or hand-written client DTO/API/event contracts from `M10`;
- target-specific browser/desktop/mobile adapters;
- presentation-only libraries after size/security review.

Forbidden dependencies:

- backend domain crates (`platform-core`, `agent-runtime`, Plugin domains);
- database/queue/provider/MCP/executor SDKs;
- server repository implementations;
- Dioxus types crossing into backend contracts;
- business computation in components, hooks, reducers or server functions.

## 6. Lifecycle

```text
app/page bootstrap
→ load public/session projection
→ Initial/Loading
→ Ready | Empty | Error | Offline | ReauthRequired
→ submit correlated user intent
→ Pending presentation only
→ accept server response/events
→ reduce monotone authoritative projection
→ reconnect/refresh/re-auth when required
```

SSE disconnect is transport state, not run completion. Unknown event versions/variants or non-monotone sequences require refresh; they are not silently ignored.

## 7. Failure and recovery

- API unavailable/offline: preserve safe local drafts, show explicit state, do not invent success.
- Unknown schema/event or stale projection: request refresh and block conflicting intent.
- Reauthentication required: use target `SecureSessionPort`; no password form storage outside contract.
- Intent timeout after possible acceptance: query by idempotency/correlation ID before retry.
- Client cache loss: reload from backend.
- SSR/renderer/WebView failure: backend state remains unchanged.
- Unsafe Markdown/HTML/tool output: sanitize or render as bounded text/artifact.
- Unsupported native capability: typed unavailable UI, no arbitrary bridge fallback.

## 8. Configuration and secrets

Client config contains public API origin, supported schema versions, build/target identity, non-secret feature availability and presentation defaults. Browser/native credentials use target secure session mechanisms. Model/provider/Plugin/source secrets never enter client config or logs.

## 9. Observability

Client diagnostics include build/target/API version, route, safe UI state, event cursor, reconnect and stable error code. No raw credentials, prompts, profile data or tool payloads in normal logs/telemetry. Accessibility and performance evidence is target-specific.

## 10. Extension and replacement

Web, desktop and mobile target adapters are peers beneath one shared application/view model. The renderer can be replaced without changing backend domain contracts while the `M10` API remains compatible. Native bridges expose narrow typed commands, never arbitrary eval/process access.

## 11. Performance path

Hot paths are initial payload, event reduction, large list rendering and bounded untrusted content. Avoid duplicating full backend histories in component state. Keep reducers deterministic, event cursors monotone and target-specific branches isolated. Web bundle/SSR latency and desktop/mobile startup receive target budgets before acceptance.

## 12. Scope boundary

**MVP**

- simple `apps/ustc-client` Dioxus initialization after contracts are approved;
- Web/PWA primary shell with SSR/page host;
- typed `M10` API client and SSE reducer;
- one Market/run/product journey;
- loading/empty/error/offline/re-auth/pending/terminal states;
- responsive keyboard/accessibility and reconnect behavior.

**Later**

- desktop packaging with narrow navigation/session adapters;
- mobile shell after real-device lifecycle proof;
- opt-in notifications and user-controlled local archive;
- richer product views over unchanged API contracts.

**Explicit non-goals**

- backend business logic in Dioxus server functions;
- offline peer authority or local database truth;
- direct Plugin/provider execution;
- desktop/mobile support claimed only because compilation succeeds.

## 13. Small-module decomposition

1. `client-contract` — versioned DTO/error/event mapping.
2. `api-client` — HTTP query/command/correlation/retry classification.
3. `event-client` — SSE cursor/reconnect/version handling.
4. `app-state` — deterministic presentation reducer.
5. `routes` — navigation and page composition.
6. `design-system` — accessible reusable display/form components.
7. `market-ui` — browse/detail/install intent only.
8. `agent-ui` — finite run/plan/tool/review projection and intents.
9. product UI modules — render typed `M70/M71/M72` projections only.
10. `platform-web`, later `platform-desktop`, `platform-mobile` — narrow target ports.
11. `ssr-host` — page delivery/session bootstrap plumbing, no domain API.
12. `client-conformance` — fake `M10` fixtures shared across targets.

## 14. Exit gate

`M80` is standalone-ready when the shared reducer/API client passes fake-server fixtures for normal, stale, unknown-version, reconnect, pending, denial and terminal states. Web/PWA is accepted only after browser smoke proves one real `M10` journey, accessibility/console/network cleanliness and no direct backend/domain dependency. Desktop/mobile each require separate build, launch, API, event and real-device/host evidence.
