# Reviewed MCP execution

- Owner: M51 protocol/binding adapter; M20 owns installation/grants; M40 owns dispatch/effects.
- Contract: `M51-EXEC-001`; bounded acceptance `MCP-019` (the broader `MCP-001`–`MCP-018` baseline is not promoted).
- Sources: [released MCP transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports), [lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle), [tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools).

## Admitted transport and owned state

The replaceable adapter lives under `crates/adapters/src/mcp/`; no Agent/runtime or
client dependency. It supports released `2025-11-25` Streamable HTTP: initialize,
notifications/initialized, complete paginated tools/list, tools/call, session/version
headers and explicit session close. Each RPC response is correlated with its numeric
request ID and JSON-RPC 2.0; server requests for sampling, roots or elicitation are
not granted. JSON and bounded SSE POST responses are supported. Text content and
structured JSON are returned as untrusted bounded data; unsupported resource/media
content is rejected without dereferencing URLs or paths. Protocol/server errors are
closed redacted categories; external bodies/headers never become diagnostics.

An endpoint uses PublicHttps by default: no userinfo/fragment, no redirect/proxy,
no private/loopback/link-local/multicast/unspecified/reserved DNS results, and DNS
results are checked and pinned for the connection. DNS/IP admission repeats for
requests; TLS hostname/certificate validation stays enabled. An explicit operator-owned
LoopbackDevelopment profile permits only numeric loopback HTTP, for local integration.
Browser input cannot select that profile. Raw credentials are not stored in bindings;
a separate server-owned secret lookup may attach bearer authentication only to the
admitted endpoint. Platform login tokens are never forwarded.

A client instance belongs to one exact owner/installation/component. No global session
cache crosses owners. Session IDs are bounded visible ASCII and treated as credentials
in diagnostics. A 404 invalidates the session; rediscovery may reinitialize, but the
adapter never automatically retries a business call whose result is uncertain. Drop
or cancellation of the local future does not prove external effect cancellation.

## Limits and drift

Default request deadline 15 seconds; response cap 1 MiB; at most 16 pages, 64 tools,
64 KiB per schema and 256 KiB complete tool inventory; bounded cursor/name/description
strings and SSE event count. Repeated cursors/names and incomplete pagination reject
the whole inventory. Tool input schemas are complete objects and must be admitted by
the existing platform schema compiler before projection; MCP annotations are never
permission authority. Namespaced client tool names do not replace original wire names.

Discovery creates a deterministic complete inventory digest. Explicit reviewed
activation binds that digest; changed/removed/new tools quarantine the previous
binding and require review. Invocation requires the exact active digest, known tool,
validated arguments and a current M20 grant checked by the caller before transport.
Testing a connection performs no tools/call. Disabling or revoking blocks new calls;
old receipts remain available. Discovery/transport never writes installation authority.

## Verification

A controlled HTTP peer proves init/notify order, JSON/SSE responses, paging, version
and ID mismatch, session ownership/expiry, drift, redirection/IP denial, timeouts,
oversized/malformed outputs and no implicit call retry. Application integration must
prove current grants and effect ordering separately before reporting Agent readiness.
Arbitrary host stdio commands, OAuth discovery and executable Skill resources are not
part of the admitted central-host profile.

### Admitted schema subset

Tool inputs compile to the existing tool-input-schema/v0 owner: closed objects
(`additionalProperties: false`), strings with optional string enums, integers,
numbers, booleans and arrays of admitted nodes. Objects without the explicit closed
property, references, unions and unsupported validation keywords are rejected, not
silently weakened. This is a bounded MCP interoperability profile, not a claim to
implement every JSON Schema vocabulary used by all MCP servers.

Input numeric matching preserves the existing platform v0 contract: a JSON token
without a decimal point/exponent is an Integer; a decimal/exponent token is a Number.
These tags are distinct and are never coerced. Consequently `type: number` inputs
require decimal/exponent notation (for example `10.0`), while `type: integer` inputs
require integer notation (`10`). This is stricter than JSON Schema numeric membership
and is an explicit input interoperability limitation, not a change to M20's canonical
argument or grant semantics. MCP output validation uses the same admitted schema
structure and value bounds but JSON Schema numeric membership: numbers include
integers, and integer outputs may use an integral decimal representation. Invalid
outputs still fail closed and quarantine the binding.
