# M51 — MCP Binding and Executor

## Metadata

- `Module ID`: `M51`
- `Status`: Accepted blueprint; implementation planned
- `Implementation State`: `planned`
- `Version`: `m51-mcp-binding/v0`
- `Last Review`: `2026-07-25`
- `Primary code area`: replaceable MCP modules under `crates/adapters/` or a dedicated crate after real consumers justify it

## 1. Purpose

`M51` owns the lifecycle of one reviewed MCP binding and executes bounded MCP tool calls for `M40`. It validates endpoints, negotiates a released protocol version, discovers all tools under limits, snapshots schemas, detects drift and maps MCP results/errors into the generic executor outcome.

A Market component declaration is not an active binding. A discovered tool is not automatically granted.

## 2. Non-goals

- publishing a package or deciding installation/grants;
- registering tools directly into Agent state;
- accepting arbitrary `stdio` commands on the central host;
- trusting server annotations/output as platform policy;
- forwarding platform login tokens or USTC credentials;
- becoming a generic network proxy.

## 3. Owned objects and state

```text
McpBindingId
BindingOwner and exact installation/component identity
Transport/endpoint identity
ProtocolSessionIdentity
DiscoveredServerIdentity
ToolInventory / schema digests
BindingState:
  Declared | EndpointValidated | ProtocolInitialized |
  ToolsDiscovered | SchemaReviewed | Approved | Active |
  Quarantined | Retired
McpExecutorError
```

`M20` owns package/install/grant truth. `M40` owns call correlation and effect ordering. `M51` owns only MCP binding/protocol execution state.

## 4. Public inputs and outputs

Administrative inputs:

```text
DeclareBinding
ValidateEndpoint
InitializeAndDiscover
ReviewSchemaSnapshot
Activate/Quarantine/Retire
```

Execution input/output:

```text
PluginExecutionRequest from M40
→ exact active binding + tool/schema lookup
→ MCP protocol call
→ PluginExecutionOutcome with bounded content/artifact claims, usage and redacted diagnostics
```

Connection testing initializes/discovers only; it never invokes a business tool.

## 5. Dependency direction

Allowed dependencies:

- generic executor request/outcome contract from `M40`;
- exact installation/component/binding identities supplied by composition;
- released MCP protocol adapter/SDK;
- `M90` safe HTTP, secret-ref, binding repository, clock and telemetry ports.

Forbidden dependencies:

- Agent run/graph internals;
- Dioxus/client types;
- direct grant mutation;
- public Docker/process administration;
- arbitrary host `command/args/env/cwd` from unreviewed user/package data.

## 6. Lifecycle

```text
Declared
→ EndpointValidated
→ ProtocolInitialized
→ complete bounded ToolsDiscovered
→ SchemaReviewed
→ Approved
→ Active
→ re-review on tool/schema/protocol drift
→ Quarantined | Retired
```

New/removed tools or changed schemas invalidate the old active snapshot/grant relationship. They are not enabled automatically.

## 7. Failure and recovery

- Invalid scheme/host/IP/redirect/auth metadata: reject before session creation.
- Protocol/version/identity mismatch: quarantine binding.
- Pagination/size/tool-count limit exceeded: discovery incomplete, not active.
- Schema drift: block old projection and require review.
- Session expiry: reinitialize under the same binding policy; never use session ID as auth.
- Timeout/crash/malformed output: typed executor outcome; no same-name fallback.
- Output schema/size/URI violation: reject/bound as untrusted.
- Credential failure: redacted blocked result; never forward platform identity token.

## 8. Configuration and secrets

MVP user-remote MCP uses reviewed Streamable HTTP only. Binding config stores normalized fixed endpoint, transport, owner, exact component, limits and `SecretRef`s. SSRF checks repeat for DNS results, redirects and nested auth-discovery endpoints. Internal-network exceptions require operator-owned profiles.

## 9. Observability

Record binding/server/protocol/tool/schema snapshots, lifecycle transition, discovery counts, drift, call latency, output size class and redacted error. Never log tokens, auth headers or private tool payloads by default.

## 10. Extension and replacement

MCP protocol-version behavior lives behind a transport adapter. Remote HTTP and future admitted local/package-hosted execution are separate peers with separate threat contracts. Changing MCP SDK does not change `M40` executor or Agent protocols.

## 11. Performance path

Discovery is bounded by page/tool/schema size and time. Execution is exact binding/tool lookup plus one protocol call with cancellation and output limits. Session pooling is owner/binding-scoped and cannot mix tenant credentials.

## 12. Scope boundary

**MVP**

- reviewed remote Streamable HTTP binding;
- complete bounded discovery and schema snapshot;
- explicit review/activation;
- schema drift quarantine;
- one read-only tool through `M40`;
- SSRF, secret, timeout and output boundaries.

**Later**

- admitted package-hosted MCP execution;
- additional released transport/protocol versions;
- scheduled execution after interaction/grant policy.

**Explicit non-goals**

- arbitrary central stdio commands;
- automatic trust from registry listing;
- shared credentials/sessions across users;
- tools enabled solely because server announced them.

## 13. Small-module decomposition

1. `binding-domain` — identity, owner and legal lifecycle.
2. `endpoint-policy` — URL/DNS/IP/redirect/auth-discovery safety.
3. `mcp-transport` — released protocol initialization/session/call.
4. `tool-discovery` — bounded pagination and inventory.
5. `schema-snapshot` — canonical schema digest and drift.
6. `binding-review` — approve/active/quarantine transitions.
7. `mcp-executor` — generic execution request/outcome mapping.
8. `session-pool` — owner/binding isolation and expiry.
9. `output-validation` — schemas, URIs, MIME and size limits.
10. `mcp-conformance` — fake server and hostile fixture suite.

## 14. Exit gate

`M51` is standalone-ready when a fake MCP server proves lifecycle, pagination, drift, SSRF rejection, session isolation, timeout, malformed and oversized output. It is accepted when one reviewed binding executes a read-only tool through `M40` with exact grant/schema/correlation/receipt evidence and no direct Agent dependency.
