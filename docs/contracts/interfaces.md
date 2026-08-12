# Interface registry

## Metadata

- `Status`: Accepted registry; every operation remains planned until its bound executable evidence passes
- `Version`: `application-interface-registry/v2`
- `Last Review`: `2026-08-12`
- `Owning plan`: [`M10 Application Ingress Host`](../plan/modules/20-application-api-host.md)
- `Client counterpart`: [`client-shell/v2.1`](client-shell.md)
- `Permission counterpart`: [`permissions.md`](permissions.md)

This registry owns the concrete application-operation, HTTP and inbound-MCP projections. [`module-boundaries.md`](module-boundaries.md) owns large-module crossing rules. An adapter registry is an allowlisted projection of this operation registry; no CLI command, MCP tool, Dioxus server function or HTTP route creates a parallel application authority.

The implemented single-node Agent state/event contract is defined in [`agent-runtime.md`](agent-runtime.md). The planned finite user-task lifecycle is defined in [`agent-harness.md`](agent-harness.md). The Agent–Plugin seam is [`agent-plugin-boundary/v0`](agent-plugin-boundary.md). None makes the operations below operational.

## 1. Application-operation registry

Operation IDs name semantics independently of transport or CLI spelling. Each admitted request maps to exactly one owning application operation after M00/M10 admission. `Initial projections` is an allowlist, not a requirement that every adapter expose the same command set.

| Operation ID | Owner | Permission class | Effect class | Initial projections | Status |
|---|---|---|---|---|---|
| `server.info` | M10 | `public-read` | read | CLI, HTTP | planned first protocol slice |
| `capability.list` | M10 | `public-read` | read | CLI, HTTP | planned first protocol slice |
| `market.package.list` | M20 | `public-read` | read | CLI, HTTP, inbound MCP | planned first vertical slice |
| `market.package.get` | M20 | `public-read` | read | CLI, HTTP | planned |
| `affairs.search` | M71 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `affairs.get` | M71 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `change.list` | M70 | `public-read` | read | CLI, HTTP | planned after owning product contract |
| `change.get` | M70 | `public-read` | read | CLI, HTTP | planned after owning product contract |
| `program.list` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `program.get` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `course.search` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `course.get` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `offering.list` | M72 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning product contract |
| `course.review_linkout` | M72 | `public-linkout` | link-out | CLI, HTTP, inbound MCP | planned after owning product contract |
| `source.provenance` | M60 | `public-read` | read | CLI, HTTP, inbound MCP | planned after owning source/product contract |
| `profile.requirement_status` | M72 | `tenant-private-read` | read | CLI, HTTP, later inbound MCP | later private-data slice |
| `planner.draft.list` | M72 | `tenant-private-read` | read | CLI, HTTP | later private-data slice |
| `planner.draft.get` | M72 | `tenant-private-read` | read | CLI, HTTP | later private-data slice |
| `planner.draft.delete` | M72 | `tenant-private-write` | tenant-local mutation | CLI, HTTP | later private-draft slice |
| `planner.generate` | M72 | `tenant-private-write` | tenant-local draft mutation | CLI, HTTP, later inbound MCP | later private-draft slice |
| `planner.explain` | M72 | `tenant-private-read` | read | CLI, HTTP, later inbound MCP | later private-data slice |

`program.*` means an approved cultivation-program projection. `planner.draft.*` means a tenant-local planning draft. Neither is an Agent/Harness plan, and no ambiguous `plan.*` alias is admitted.

The first retained protocol proof is `server.info` plus `capability.list`; the first shared CLI/inbound-MCP vertical slice is `market.package.list`. The first campus-product query follows the existing product order and should be `affairs.search`/`affairs.get` after M60/M71 own the real source and procedure contracts. Existing Course Planning code remains bounded offline evidence and does not make M72 operations operational.

## 2. Schema identity and grant invalidation

Every public operation has a versioned request schema, result/error schema and canonical schema digest in the future M10-owned machine registry. Every CLI/MCP/Dioxus projection references that operation identity; it does not copy and reinterpret the schema.

- A compatible descriptive change that does not alter accepted fields, result meaning, data class or effect class may retain the schema identity.
- Adding or widening input fields, data exposure, permission class, effect class, external target or result authority requires a new schema identity/digest.
- An existing private or delegated grant bound to the old operation/schema digest becomes stale and must be explicitly re-approved before the changed projection is advertised or invoked.
- Public-read operations still pass current server capability and policy admission on every call; public classification is not a client-side bypass.
- Unknown operation or schema identities fail closed. No same-name, prefix or nearby operation fallback is admitted.

## 3. Application HTTP endpoints — draft

Routes are transport projections of §1 operations. An endpoint may be a versioned Dioxus server function, an explicit Axum route, or both when the same wire contract is intentionally admitted.

| Route | Method | Operation/projection | Status |
|---|---:|---|---|
| `/api/health` | GET | `server.info` | planned |
| `/api/client/capabilities` | GET | `capability.list`; server-supported client operation IDs/versions only, no tenant grants or operator registry | planned |
| `/api/market/packages` | GET | `market.package.list` | planned |
| `/api/market/packages/{id}` | GET | `market.package.get` | planned |
| `/api/installations` | POST | future M20 installation operation; outside the initial external-Agent projection | planned by owning M20 contract |
| `/api/installations/{id}:disable` | POST | future M20 installation operation; outside the initial external-Agent projection | planned by owning M20 contract |
| `/api/agent/runs` | POST | future finite HarnessRun operation family | planned by owning harness contract |
| `/api/agent/runs/{id}` | GET | future HarnessRun projection | planned by owning harness contract |
| `/api/agent/runs/{id}/answers` | POST | future bounded clarification operation | planned by owning harness contract |
| `/api/agent/runs/{id}:cancel` | POST | future typed cancellation operation | planned by owning harness contract |
| `/api/agent/runs/{id}/events` | GET/SSE | future HarnessRun event projection | planned by owning harness contract |

Every request carries client build/target/protocol identity. M10 performs version, size, identity, authorization, idempotency/precondition and audit admission before dispatching one application operation. A server function or HTTP/SSE route MUST NOT call concrete repositories, databases, Plugin executors, provider SDKs or journals directly.

## 4. Client adapter projections

| Client adapter | Direction | Admitted transport | Registry constraint |
|---|---|---|---|
| Dioxus Web/Android | user → platform | generated server function / typed events or equivalent M10 transport | presentation only; explicit operation allowlist; no CLI/process bridge |
| `ustc-agent` | user/script → platform | explicit versioned HTTP/JSON and SSE | least-privilege user profile; its command registry projects a subset of §1 |
| inbound MCP | external Agent → platform | reviewed MCP Streamable HTTP surface mapped through client-core to explicit M10 routes | read-only public slice first; exact tool/resource allowlist and schema digests; no operator/domain/M51 reach-through |
| M51 outbound MCP | platform → external MCP server | M51 binding/session/executor contract | opposite direction; never an M80 client adapter |

CLI, MCP and GUI are semantically equivalent only where they project the same operation. Authentication, local configuration, presentation and platform-specific maintenance commands need not appear as MCP tools.

## 5. Inbound MCP registry — phased projection

The inbound MCP surface is a server offered to an external personal Agent acting as MCP client. The first remote profile uses reviewed MCP Streamable HTTP. Local stdio/relay is later and requires a separately accepted deployment/session contract.

**Initial public-read projection**

```text
market.package.list
```

**Campus read projection after owning product contracts exist**

```text
affairs.search
affairs.get
program.list
program.get
course.search
course.get
offering.list
course.review_linkout
source.provenance
```

**Later explicitly delegated private projection**

```text
profile.requirement_status
planner.generate
planner.explain
```

`planner.generate` creates only a tenant-local draft. It does not enroll, register, pay or submit a transaction to any external campus system. MCP discovery advertises an operation only when its exact schema digest, caller profile, tenant scope, current capability/grant and result bounds are admitted.

## 6. Agent tool protocol — H0 values implemented, production execution planned

| Object | Direction | Purpose |
|---|---|---|
| `AgentToolsetView` | resolver/gateway → Agent | immutable per-turn complete tool definitions plus opaque private route references |
| `AgentToolCall` | Agent → ToolGateway | provider-neutral correlated call against the exact frozen projection |
| `PluginExecutionRequest` | ToolGateway → PluginExecutor | authorized bounded execution request after effect intent persistence |
| `PluginExecutionOutcome` | PluginExecutor → ToolGateway | non-authoritative bounded outcome for validation and receipt persistence |
| `AgentToolResult` | ToolGateway → Agent | correlated bounded result/evidence/receipt projection for the next model turn |

This Agent tool protocol and M51 outbound MCP direction are independent from the M80 inbound MCP projection above.
