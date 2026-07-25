# Multi-client Dioxus shell contract

## Metadata

- `Status`: Accepted target architecture; no client crate or Dioxus dependency implemented yet
- `Version`: `client-shell/v0`
- `Last Review`: `2026-07-25`
- `Owning Plan`: [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md)
- `Large-module Blueprint`: [`../plan/modules/80-dioxus-multi-client.md`](../plan/modules/80-dioxus-multi-client.md)
- `Decision`: [`ADR-0009`](../adr/0009-dioxus-multi-client-shell.md)
- `Counterpart Interfaces`: [`interfaces.md`](interfaces.md)
- `Acceptance`: planned `WEB-*`, `I18N-*`, `CLIENT-*`

## 1. Purpose

USTC Campus Agent uses one Rust/Dioxus presentation shell for Web/PWA first and later desktop/mobile targets. The shell MAY own SSR and page hosting so the presentation path remains full-stack Rust. It renders server-owned projections and submits typed user intent. It does not own Agent, Market, grant, Plugin, source or audit authority.

```text
shared Dioxus app / view models
          │ SSR/page hosting
          │
          ▼
      typed ClientApi port
          │ versioned HTTP JSON + SSE
          ▼
  ustc-agentd authority plane

platform adapters: web | desktop | mobile
```

Changing the client renderer must not change platform domain contracts. Changing Agent or Plugin internals must not require client rewrites while the client API remains compatible.

## 2. Why Dioxus

The accepted baseline is Dioxus `0.7`; the latest official release observed on `2026-07-24` is `v0.7.9`. Official documentation identifies Web as the best-supported target, Desktop as a system-WebView renderer with native Rust code, and Mobile as a first-class WebView target with Android/iOS toolchain requirements.

Dioxus is selected for presentation reuse, Rust type safety, SSR/page delivery and one component model across targets. It is not selected as a backend authority framework. Before the first client implementation, the exact Dioxus release/features and `dx` toolchain are revalidated and pinned.

## 3. Ownership

| Owner | Owns | Must not own |
|---|---|---|
| shared Dioxus app | routes, layout, SSR/page delivery, accessible components, view-model reduction, form drafts and intent capture | domain transitions, grant decisions, Plugin routing, run completion or audit truth |
| typed `ClientApi` port | versioned requests, responses, event decoding, cancellation and retry classification | server policy, hidden fallback or local authority |
| target adapter | browser/native transport, external navigation, notifications, secure local integration and packaging | reusable product semantics or direct database/Plugin access |
| `ustc-agentd` | identity, Market/install/grant, HarnessRun/AgentRun, tool gateway, source and audit decisions | target-specific UI state |
| durable platform state | canonical run/session/package/source/evidence history | transient navigation, animations or form focus |

Client-side cached data is always a projection. Optimistic UI may display `pending`, but it cannot claim an authoritative transition before the server event/response confirms it.

## 4. Module topology

The first real client starts as one application crate, not one crate per platform or architectural noun:

```text
apps/ustc-client/                 # created only with the first Web/PWA slice
  src/app/                        # target-neutral routes/components/view models
  src/api/                        # explicit ClientApi implementation and DTO mapping
  src/platform/                   # web first; desktop/mobile adapters later
  assets/
```

Start with modules. Extract a shared crate only after a second independent consumer, privilege boundary or measured build/feature isolation justifies it.

Dioxus types, hooks, signals, router objects and renderer handles terminate inside `apps/ustc-client`. `platform-core`, `agent-runtime`, `agent-tool-protocol`, Plugin crates and HTTP contracts must not depend on Dioxus.

## 5. Client API boundary

The client consumes explicit versioned HTTP JSON and server event streams declared in [`interfaces.md`](interfaces.md). Dioxus fullstack server functions or SSR actions may assist page rendering/bootstrap, but every business read or mutation—including SSR data loading—must use the same `ClientApi` over the explicit `M10` API. They must not call application services, domain repositories or executors directly, become an alternate canonical API, or bypass authentication, authorization, idempotency and audit.

A client intent contains only the user action and current API preconditions. The server returns one typed accepted/denied result and authoritative projection. Stable error codes cross the boundary; localized prose does not define protocol behavior.

The shared reducer handles at least:

- initial/loading/ready/empty/error/offline states;
- monotone server event sequence and reconnect cursor;
- explicit stale-projection and reauthentication outcomes;
- pending intent correlation without inventing success;
- terminal run outcomes distinct from transport disconnection.

Unknown schema versions, unknown event variants or non-monotone event sequences fail closed and request refresh; they are not silently ignored into a plausible state.

## 6. Platform ports

Target-specific capability is narrow and explicit:

- `ExternalNavigation`: browser tab or platform Custom Tab for USTC/iCourse link-out;
- `NotificationPort`: opt-in local notification projection, never source of task completion;
- `SecureSessionPort`: target-appropriate credential/session integration under a later auth contract;
- `LocalArchivePort`: optional user-controlled transcript export distinct from durable server memory;
- `PlatformInfo`: non-authoritative capability/diagnostic facts.

Shared components cannot call filesystem, WebView JavaScript, keychain/keystore, notifications or process APIs directly. Unsupported capability returns a typed unavailable state; it never switches execution location or transport implicitly.

## 7. Target order

1. **Web/PWA primary**: prove login, Market, finite Agent journey, tool/review status, responsive/accessibility and reconnect behavior against the explicit API.
2. **Desktop packaging**: reuse the same app/reducer and API; add only admitted window, external-navigation and secure-session adapters.
3. **Mobile shell**: reuse the same API and app semantics; external campus services remain Custom Tab/link-out. Android/iOS full experience follows Web/PWA lifecycle proof.

A target is not supported merely because Dioxus can compile it. Each target needs build, launch, API, event, accessibility and platform-adapter evidence.

## 8. Agent UI projection

The client borrows successful Agent interface separation rather than embedding an Agent loop:

- Claude Code demonstrates one underlying loop exposed through terminal, desktop, IDE, web and remote interfaces;
- Pi exposes ordered lifecycle/tool events suitable for responsive UI projection;
- goose exposes per-tool allow/ask/deny UX;
- Hermes separates gateway interfaces from its central tool runtime.

USTC adaptation is stricter: UI controls emit typed intents; they do not mutate tool registries, grant state, run phases or effect receipts. Tool output is rendered as bounded untrusted content. Approval UI names the exact tool/capability/scope and correlates one server-owned intent.

## 9. Failure and security

- no raw secret, USTC credential, grant snapshot payload or executor configuration enters normal client logs;
- external URLs are server-projected or locally validated against the relevant link-out contract;
- HTML/Markdown/tool output is rendered under an explicit sanitization policy;
- desktop/mobile native bridges expose narrow commands rather than arbitrary eval/process access;
- reconnect never duplicates an accepted mutation; client retries require server idempotency identity;
- client cache loss affects usability only, not canonical platform state;
- Dioxus/WebView failure must not corrupt server state or installed package artifacts.

## 10. Replacement and acceptance

The boundary is accepted when:

- Web/PWA completes `CLIENT-001` and applicable `WEB-*` through the explicit API;
- Android/mobile reuses the same API contract and passes `CLIENT-002` without a second authority path;
- local archive and central memory remain distinct under `CLIENT-003`;
- unavailable/offline/relay state is explicit under `CLIENT-004`;
- equivalent server fixtures reduce to equivalent product state across supported targets;
- replacing Dioxus inside the client app would not change platform domain crates, Agent/Plugin protocol or persisted authority schemas.

## 11. Current status

Implemented now: none of the application client, Dioxus dependency, HTTP/SSE service, auth flow or browser journey.

Accepted now: Dioxus as the multi-target presentation and SSR/page-hosting shell; Web/PWA-first rollout; explicit `M10` API/event boundary; target-specific ports; authority and dependency constraints.

This contract does not claim a runnable client and does not move P5 productization ahead of H0/P0b/P0c.
