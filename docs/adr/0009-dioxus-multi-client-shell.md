# ADR-0009: Use Dioxus for a thin multi-client shell

- `Status`: Accepted; full-stack Rust deployment shape clarified
- `Date`: `2026-07-24`
- `Last Clarification`: `2026-07-25`
- `Depends on`: [`ADR-0004`](0004-runtime-reference-strategy.md), [`ADR-0007`](0007-finite-agent-harness.md), [`ADR-0008`](0008-agent-plugin-tool-boundary.md)

## Context

The product needs a Web/PWA client and may later add desktop and mobile shells. Separate frontend stacks would duplicate routes, event reduction, Agent status semantics, accessibility fixes and typed API mappings. Conversely, sharing server/domain crates directly with a client renderer would leak authority into UI code and make every backend change a client framework concern.

Dioxus provides a Rust component model across Web, Desktop and Mobile. The official `0.7` documentation describes Web as the best-supported target, Desktop as a system-WebView renderer with native Rust execution, and Mobile as a first-class WebView target. The latest official release observed for this decision is `v0.7.9` (`2026-05-08`). Platform support still has different toolchains and runtime capabilities; “one framework” does not make targets behaviorally identical.

Successful Agent products also separate interaction surfaces from the core loop. Claude Code exposes the same harness through terminal, desktop, IDE, web and remote interfaces. Pi emits ordered message/tool lifecycle events for UI projection. goose presents tool permission choices without making the UI the tool authority.

## Decision

Adopt [`client-shell/v0`](../contracts/client-shell.md):

```text
one Dioxus app and shared reducer
      │ optional SSR/page host
      │ typed ClientApi
      ▼
versioned HTTP JSON + SSE
      ▼
ustc-agentd authority plane
```

Roll out Web/PWA first. The Dioxus application MAY own SSR and page delivery so the presentation stack remains full-stack Rust. Desktop follows as packaging plus narrow native ports; mobile follows after the Web lifecycle is proven. Start with one app crate and internal modules. Do not create separate web/desktop/mobile domain crates or add Dioxus to backend/domain crates.

Dioxus is presentation infrastructure only. Dioxus signals, hooks, router types, server functions and WebView handles cannot own or redefine identity, Market/install/grant state, HarnessRun/AgentRun transitions, Agent tool protocol, Plugin execution, source truth, receipts or audit.

The canonical client/server seam remains explicit versioned HTTP JSON and event streaming owned by `M10`/`ustc-agentd`. Dioxus fullstack server functions MAY assist SSR, page bootstrap or deployment, but every business read or mutation—including SSR data loading—MUST use the same explicit `M10` API through `ClientApi`. They MUST NOT call application services, repositories or executors directly, become an alternate business API, or bypass `M00`/`M10` admission. Browser, desktop and mobile behavior remains testable against the same public API/event contract.

No Dioxus dependency or empty client crate is added by this ADR. The first `M80` implementation batches will revalidate and pin the exact release/features, create `apps/ustc-client`, and prove one real API/event consumer without changing another module's private implementation.

## Rejected alternatives

- independent React/desktop/mobile implementations with duplicated product semantics;
- make Dioxus Fullstack or server functions the platform domain boundary or a direct repository/executor path;
- import `platform-core`, `agent-runtime` or Plugin implementation types directly into UI components;
- expose arbitrary filesystem/process/WebView eval to shared components;
- create one crate per target before any target has a real consumer;
- claim Android/iOS support from compilation alone;
- implement desktop/mobile before Web/PWA auth, event and recovery semantics are proven.

## Consequences

Benefits:

- Rust language/tooling and component reuse across target shells;
- one view-model/reducer interpretation of server events;
- Dioxus replacement remains confined to the client application;
- target-specific privileges are visible behind narrow ports;
- Agent/Plugin/backend evolution stays independent of the renderer.

Costs and risks:

- WebAssembly compatibility constrains reusable client dependencies;
- Desktop and Mobile WebView behavior still requires target-specific QA;
- Android/iOS toolchains and packaging remain separate work;
- Dioxus is fast-moving, so exact versions/features must be pinned and periodically revalidated;
- shared UI can become a lowest-common-denominator design unless platform adaptation remains explicit.

## Verification and rollback

- Web/PWA is the first implementation and acceptance surface.
- Every later target must replay common API/event fixtures and pass target-specific launch/navigation/security checks.
- Repository dependency checks must keep Dioxus out of authority/domain crates.
- Rollback before implementation is documentation-only. After implementation, replacing Dioxus is an app-shell migration that preserves `client-shell/v0` and server contracts.

## Official source baseline

Reviewed on `2026-07-24`:

- Dioxus latest release: <https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.9>
- Getting started/toolchain: <https://dioxuslabs.com/learn/0.7/getting_started/>
- Web: <https://dioxuslabs.com/learn/0.7/guides/platforms/web/>
- Desktop: <https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/>
- Mobile: <https://dioxuslabs.com/learn/0.7/guides/platforms/mobile/>
- Claude Code loop/interfaces: <https://code.claude.com/docs/en/how-claude-code-works>
- Pi Agent core events/tools: <https://github.com/badlogic/pi-mono/tree/main/packages/agent>
- goose tool permissions: <https://goose-docs.ai/docs/guides/managing-tools/tool-permissions>
