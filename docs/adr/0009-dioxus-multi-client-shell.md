# ADR-0009: Use Dioxus Fullstack for the long-lived Web and Android application

- `Status`: Accepted; amended for mandatory Android, first-party Fullstack ingress and a later admitted Windows peer; complemented by [`ADR-0010`](0010-typed-client-peer-adapters.md)
- `Date`: `2026-07-24`
- `Last Amendment`: `2026-08-12`
- `Depends on`: [`ADR-0004`](0004-runtime-reference-strategy.md), [`ADR-0007`](0007-finite-agent-harness.md), [`ADR-0008`](0008-agent-plugin-tool-boundary.md)

## Context

USTC Campus Agent is intended to remain a useful school-level Agent platform after the competition, not a disposable submission. It must support a Web/PWA client, a server deployment managed through Docker Compose, and an Android client. Windows is admitted as a later desktop peer but is not a current required release gate; iOS and other desktop targets remain later candidates.

Maintaining independent Rust backend and TypeScript/JavaScript frontend stacks would duplicate language/toolchain upkeep, request/response types, error mapping, event reduction and Web/Android presentation logic. Dioxus provides one Rust component model across Web, Desktop and Mobile, while Dioxus Fullstack provides Axum-compatible typed server functions, SSR/hydration, routing, assets, forms, SSE, streams and WebSockets.

The official `0.7` documentation describes Web as the best-supported target and Mobile as a first-class WebView target. Mobile still has separate Android/iOS toolchains and no current native Android widget/animation model. The latest official release observed for this decision is `v0.7.9` (`2026-05-08`). One Rust workspace therefore reduces maintained stacks and duplicated presentation code; it does not make server, Web and Android one artifact or remove target-specific QA.

## Decision

Adopt the Dioxus portion of [`client-shell/v2.1`](../contracts/client-shell.md):

```text
shared Dioxus Fullstack application and presentation model
        ├── Web: SSR/CSR + hydration + PWA
        ├── Android: packaged Dioxus/WebView client
        ├── Windows: admitted later peer target, not a current required gate
        └── iOS/other desktop: later candidates
                 │
                 │ versioned Dioxus server functions / HTTP / typed events
                 ▼
M10 ingress in ustc-agentd
        │ authentication, authorization, bounds,
        │ idempotency, compatibility and audit
        ▼
application command/query ports
        ▼
platform-owned domain/runtime authorities
```

The Dioxus application is Fullstack infrastructure, not merely a renderer. Its first-party Web and Android clients MAY use versioned Dioxus server functions as their canonical typed ingress. A server function is an Axum-compatible HTTP endpoint and MAY call one admitted application command/query port after `M00`/`M10` authentication, authorization, request bounds, idempotency/precondition and audit rules pass.

A server function MUST NOT call concrete repositories, databases, Plugin executors, provider SDKs or domain/runtime internals directly. It cannot own or redefine identity, Market/install/grant state, HarnessRun/AgentRun transitions, Agent tool protocol, Plugin execution, source truth, receipts or audit. Explicit REST/SSE endpoints for the real `ustc-agent` and inbound MCP heterogeneous consumers are peer transport adapters over the same application ports rather than a second implementation of business semantics.

[`ADR-0010`](0010-typed-client-peer-adapters.md) extends the client topology without weakening this Dioxus decision: Dioxus, `ustc-agent` and inbound MCP consume one framework-neutral typed client core as peer adapters; Dioxus Web/Android never spawn or parse the CLI as their production path.

One source/workspace produces separate artifacts:

- a native Linux server build hosted by the Docker Compose profile;
- Web assets/WASM plus optional SSR/hydration served by that server;
- an Android package that points to the deployed HTTPS server;
- a later Windows package only after its promotion gate; iOS/other desktop packages when separately admitted.

Web is the first proof surface because it validates authentication, typed ingress, event reduction and recovery fastest. Android is a mandatory product target and follows immediately after the shared Web contract is executable; it is not an optional later idea. Windows remains later and non-required until installer/signing/update, secure-session/login-callback, sleep/resume/proxy/reconnect and real-host acceptance are active. iOS and other desktop targets remain later candidates.

Independently deployed Android clients create server/client version skew even when source types are shared. Server-function routes, DTOs, errors and events therefore carry explicit compatibility policy, stable versions and unknown-variant behavior. A server upgrade MUST either remain compatible with supported installed Android versions or return a typed minimum-version/upgrade outcome before unsafe dispatch.

No Dioxus dependency or empty app crate is added by this ADR alone. The first implementation batch revalidates and exact-pins the Dioxus/DX release and features, then proves one Fullstack Web journey and one Android build/launch/remote-call path without changing backend domain internals.

## Rejected alternatives

- independent Rust backend plus unrelated Web and Android presentation implementations;
- Leptos plus Tauri as two framework layers when Android and one Rust Fullstack stack are hard requirements;
- make Dioxus signals/hooks/server-function state the platform domain authority;
- force an admitted server function through a redundant loopback HTTP call before it may reach the same application port;
- let server functions reach repositories, executors, provider SDKs or durable journals directly;
- expose arbitrary filesystem/process/WebView eval to shared components;
- create one domain/client implementation per target before target-specific behavior requires it;
- claim Android support from compilation alone;
- assume shared Rust source removes deployed mobile/server compatibility obligations.

## Consequences

Benefits:

- one primary Rust language/toolchain across server, Web and Android;
- shared routes, components, presentation reducer, request/response/event types and typed errors;
- generated client calls for Axum-compatible server-function endpoints instead of a separately maintained TypeScript API client;
- Web SSR/hydration and Android reuse without duplicating platform authority;
- explicit user/integration HTTP adapters serve `ustc-agent` and inbound MCP over the same application ports;
- Dioxus remains confined to the Fullstack application boundary, so domain/runtime evolution stays independent.

Costs and risks:

- Dioxus is pre-1.0 and fast-moving, requiring exact pins, controlled upgrades and rollback evidence;
- WebAssembly and WebView constraints limit reusable dependencies and require browser/device QA;
- Android SDK/NDK/CMake, signing, packaging and real-device lifecycle remain separate obligations;
- mobile authentication/session storage and server URL configuration require narrow target adapters;
- installed Android versions can lag the server and require an explicit compatibility window;
- Dioxus does not supply databases, caches, sessions or mailers; those remain explicit Axum/infrastructure choices;
- shared UI can become lowest-common-denominator design unless target adaptation remains deliberate.

## Verification and rollback

- Web Fullstack proof covers SSR or initial page delivery, hydration/CSR, one typed server-function query/command and one typed event stream.
- Android proof covers a real emulator/device launch, configured HTTPS server URL, authentication/session adapter, the same semantic journey, reconnect and external-service Custom Tab behavior.
- Docker Compose proof covers clean server startup, health/readiness, Web asset/SSR delivery, server-function endpoint access, Android remote access and restart/read-back.
- Compatibility fixtures exercise a supported older Android protocol against the current server plus typed rejection of an unsupported version.
- Dependency checks keep Dioxus out of domain/runtime/Plugin crates and forbid server-function adapters from importing concrete repositories/executors.
- Before implementation, rollback is documentation-only. After implementation, replacement preserves application command/query contracts and persisted domain state; client/server transport migration is explicit rather than hidden.

## Official source baseline

Reviewed on `2026-07-25`:

- Dioxus latest release: <https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.9>
- Fullstack overview: <https://dioxuslabs.com/learn/0.7/essentials/fullstack/>
- Server functions: <https://dioxuslabs.com/learn/0.7/essentials/fullstack/server_functions/>
- Web: <https://dioxuslabs.com/learn/0.7/guides/platforms/web/>
- Mobile: <https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/>
- Getting started/toolchains: <https://dioxuslabs.com/learn/0.7/getting_started/>
- Fullstack examples, including desktop remote server URL and typed SSE: <https://github.com/DioxusLabs/dioxus/tree/v0.7.9/examples/07-fullstack>
