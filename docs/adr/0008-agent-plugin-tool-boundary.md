# ADR-0008: Decouple Agent and Plugin through the tool protocol

- `Status`: Accepted
- `Date`: `2026-07-24`
- `Depends on`: [`ADR-0004`](0004-runtime-reference-strategy.md), [`ADR-0007`](0007-finite-agent-harness.md)

## Context

The platform has two independent change families:

- Agent orchestration: harness phases, graph scheduling, model loop, context budget, review and replay;
- Plugin delivery: package/component versioning, installation, grants, execution artifacts and product capabilities.

If Agent code imports package manifests, component implementations or framework extension APIs, an Agent-loop change can force Plugin rebuilds. If Plugins can register callbacks directly inside the Agent state machine, they can also bypass grants, effect ordering and replay. Conversely, hard-coding first-party tools into the Agent would make every Plugin addition a framework change.

Pi demonstrates a useful minimal-core pattern: tools are registered into the Agent loop, while extensions and Pi Packages distribute extra behavior without forking the core. Claude Code demonstrates self-contained, namespaced, versioned Plugins whose MCP servers appear through the common tool surface. Both systems also permit extension code with broad process/runtime access; that authority model is unsuitable for a multi-tenant campus platform.

The current Rust production dependency is already clean, but one cross-crate proof test was owned by `agent-runtime`, and the protocol/packaging boundary was not yet normative.

## Decision

Adopt [`agent-plugin-boundary/v0`](../contracts/agent-plugin-boundary.md):

```text
Agent kernel/harness
        │ versioned AgentToolDefinition / AgentToolCall / AgentToolResult
        ▼
    ToolGateway
        │ authorized PluginExecutionRequest / non-authoritative outcome
        ▼
Plugin executor selected from exact installed package/component identity
```

`InvocationResolver` compiles package, installation, grant and policy state into one immutable per-turn `ToolProjectionSnapshot`. The Agent receives only a plugin-neutral `AgentToolsetView`; private route/authority data remains in `ToolGateway`. The composition root alone may depend on both Agent and Plugin/resolver modules.

Plugins extend capability primarily by packaging tool providers plus optional bounded skills/resources. They do not inject arbitrary code or lifecycle hooks into the Agent kernel. `NativeRustComponent` execution must cross a separately admitted out-of-process executor boundary; direct dynamic linkage into `agent-runtime` is forbidden.

A concrete protocol crate is deferred until H0 supplies two real consumers—the Agent side and a fake gateway. This preserves modules-before-crates while reserving a high-value dependency boundary.

## Compatibility rule

An internal Agent/framework update requires no Plugin change while the major tool protocol remains compatible. A Plugin update requires no Agent change while the executor protocol remains compatible. Breaking protocol changes require an explicit major version and migration/dual-adapter plan; in-flight runs keep their pinned version and projection.

`agent-run/v0` retains existing package/install/component identity strings for replay/audit compatibility. They are opaque provenance bindings: Agent code cannot import Plugin types, parse component kinds or branch on package identity. Removing/replacing these fields requires a separately versioned run contract, not an incidental refactor.

## Rejected alternatives

- compile first-party Plugin logic directly into the Agent runtime;
- let each Plugin own an Agent/subclass or model loop;
- permit arbitrary in-process Plugin hooks to mutate prompts, tools, phases or approval state;
- make a framework registry or hot-reload state the installation/grant authority;
- dispatch by model-visible tool name without a frozen route identity;
- create one crate/trait per conceptual object before a second consumer exists;
- require Plugin rebuilds whenever the Agent harness, provider adapter or reviewer policy changes.

## Consequences

Benefits:

- Agent and Plugin modules can be built, tested, versioned and replaced independently;
- new capabilities enter through one uniform tool/evidence/effect path rather than Agent special cases;
- package updates cannot mutate in-flight toolsets;
- framework adoption/removal remains a bounded Agent-side change;
- authority, audit and failure ordering stay platform-owned.

Costs and risks:

- the gateway adds explicit mapping and conformance code;
- protocol versions carry compatibility/migration responsibility;
- broad Pi/Claude-style extension callbacks are unavailable;
- runnable native first-party Plugins need a tool-host/executor artifact instead of direct linking;
- one extra IPC/protocol boundary may add latency, which is accepted unless measurement later justifies a safe optimized path.

## Migration and verification

- Move the P0a→`RunSpec` cross-boundary proof to `ustc-agentd`, the existing composition root.
- Mechanically reject Market/Plugin/adapter dependencies in `agent-runtime`.
- H0 introduces fake Agent/tool/executor conformance without claiming production Plugin execution.
- Later real Plugin execution must satisfy package, projection, authorization, intent/receipt and replacement acceptance cases.
- Rollback is contract-preserving: the current R0/P0a semantics remain valid; no durable wire/state migration is performed by this ADR.
