# ADR-0010: Share a typed client core across peer Dioxus, CLI and inbound MCP adapters

- `Status`: Accepted
- `Date`: `2026-07-31`
- `Decision owner`: Develata
- `Depends on`: [`ADR-0009`](0009-dioxus-multi-client-shell.md)
- `Owning contract`: [`client-shell/v2`](../contracts/client-shell.md)

## Context

USTC Campus Agent already requires a long-lived Dioxus Web/PWA and Android client over central server authority. It also needs a pure user/automation CLI that external scripts and Agents can call, plus a native MCP-facing integration surface for mainstream Agent ecosystems.

The existing `ustc-agentctl` is an operator/developer CLI. Extending it into a user/MCP product surface would mix privilege and compatibility contracts. Making every GUI invoke a new user CLI executable would reuse a process rather than reusable client semantics and would add an unjustified local RPC boundary.

The required clients share substantial behavior:

- endpoint/profile validation and authentication ports;
- protocol/build compatibility;
- command/query encoding;
- correlation, idempotency and preconditions;
- typed result/error mapping;
- event reconnect/cursor handling;
- timeout-after-possible-acceptance reconciliation;
- safe bounded projection.

They do not share presentation, command parsing or MCP wire concerns.

## Decision

Adopt one M10-owned framework-neutral typed client protocol carrier, one M80-owned framework-neutral client core, and three M80 peer outer adapters:

```text
Dioxus Web/Android ─┐
ustc-agent CLI ─────┼→ M80 client-core → M10 client-protocol/API → application ports
inbound MCP ────────┘

M10 server adapters ─────────────→ M10 client-protocol/API
```

Rules:

1. `ustc-agent` is a new ordinary-user and automation CLI.
2. `ustc-agentctl` remains operator/admin/developer-only.
3. M10 owns the versioned wire DTO/error/event/compatibility schema; M80 client-core consumes it and owns auth/transport/reconnect/reconciliation behavior.
4. M10 server code never depends on M80 client-core; both sides may consume the M10-owned protocol carrier.
5. Dioxus, `ustc-agent` and inbound MCP share client-core semantics and conformance fixtures.
6. No peer adapter invokes or parses another peer executable as its production path.
7. Web/Android do not spawn CLI processes.
8. The inbound MCP adapter exposes selected least-privilege tools/resources through client-core and M10; it reaches no domain implementation or operator command.
9. M51 remains the opposite protocol direction: platform-to-external-MCP binding/execution.
10. Central server application/domain modules retain every truth-affecting calculation, mutation, authorization and durable transition.
11. Exact MCP package/process placement is chosen by the first accepted implementation slice without changing these boundaries.

M80 remains the existing client-side large module, renamed in authority prose to **Client Core and Interaction Shells**. No M81 is added: the common client behavior and its outer interaction adapters are one cohesive responsibility, while M10 continues to own server ingress.

## Rejected alternatives

### Extend `ustc-agentctl` for ordinary users and external Agents

Rejected because operator and user/MCP surfaces have different grants, credentials, compatibility and mutation risks. A shared binary would make least-privilege confinement harder to prove and encourage accidental operator command exposure.

### Make GUI invoke `ustc-agent`

Rejected as the canonical path because:

- a Web/PWA cannot directly spawn a local CLI;
- server-side shell-out would create redundant `HTTP → process → HTTP` transport;
- Android subprocess/ABI/lifecycle/packaging adds a local RPC system with no local authority benefit;
- stdout framing, concurrency, backpressure, cancellation and crash semantics duplicate typed client transport;
- killing a CLI process does not cancel an accepted server operation;
- GUI/CLI/server become a three-way version matrix;
- CLI business orchestration would become a second application layer, while a strictly thin CLI would add only a redundant process hop.

A future desktop adapter may offer an optional sidecar/debug mode if a real local boundary appears. That exception does not alter Web/Android production paths.

### Duplicate a separate client implementation per shell

Rejected because authentication, compatibility, streams, correlation and recovery would drift. Adapter-specific concerns remain separate over one common semantic core.

### Add a new top-level M81 module

Rejected as unnecessary skeleton expansion. M80 already owns client-side interaction and can remain independently testable with the new real consumers. M10 still owns ingress; M51 still owns outbound MCP execution.

## Consequences

Benefits:

- one maintained client semantic implementation across three real consumers;
- native Agent ecosystem access through MCP plus universal subprocess fallback through CLI;
- clear operator/user privilege split;
- Dioxus remains a thin presentation shell;
- browser/mobile avoid process and local-RPC complexity;
- transport and outer frameworks remain replaceable;
- equivalent fake-M10 fixtures can prove cross-adapter behavior.

Costs:

- a new user CLI artifact and an inbound MCP artifact/entrypoint require independent packaging and compatibility evidence;
- the client-core API must remain framework-neutral without becoming an oversized generic SDK;
- Dioxus generated server-function transport and explicit HTTP/SSE clients need semantic conformance despite different encodings;
- authentication storage remains target-specific;
- MCP tool/resource registry and result bounds add a public compatibility surface.

## Verification and rollback

Acceptance is bound to `CLIENT-007` through `CLIENT-010` in the active matrix. All remain planned until executable evidence exists.

Before implementation, rollback is documentation-only. After implementation, rollback may remove one outer adapter while preserving the client-core/M10 contract and other peer artifacts. Replacing client-core transport must preserve compatibility, correlation, reconnect, cancellation and typed failure fixtures.

This ADR does not introduce a Dioxus dependency, client-core crate, CLI binary or MCP runtime by itself.
