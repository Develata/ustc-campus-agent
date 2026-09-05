# Headless client and external Agent integration

- `Status`: Accepted phased journey with retained bounded client-core, fixture-loopback ordinary-user Affairs CLI and Affairs-first M10 protocol/compatibility evidence; production transport/auth/streams, inbound MCP and Dioxus peer journeys remain planned
- `Owning plan`: [`M80 Client Core and Interaction Shells`](../plan/modules/80-dioxus-multi-client.md)
- `Contracts`: [`client-shell/v2.3`](../contracts/client-shell.md), [`cli/v2.2`](../contracts/cli.md), [`application-interface-registry/v2.1`](../contracts/interfaces.md), [`permissions/v2`](../contracts/permissions.md)
- `Decision`: [`ADR-0010`](../adr/0010-typed-client-peer-adapters.md)
- `Acceptance`: `CLIENT-007`, `CLIENT-008`, `CLIENT-009`, `CLIENT-010`

## Goal

A student can use USTC Campus Agent without a graphical client, and an explicitly delegated external Agent can consume selected campus tools/resources without receiving operator privileges or bypassing platform authority.

The headless surface is a first-class client, not a debug wrapper around backend crates. It uses the same typed client semantics as Dioxus while keeping CLI and MCP protocol concerns at their outer adapters.

### Current bounded evidence, not full client readiness

The retained `crates/client-core` and `apps/ustc-agent` implement public `affairs get` and capability-stdin `affairs lookup` over numeric loopback, typed response reduction, canonical `ustc-client-result/v1` JSON and stable exit classes. They do not import backend/operator implementations or accept raw session/operator authority. The accepted Affairs-first prerequisite additionally retains the M10-owned major-1 `server.info`, `capability.list`, `affairs.get` protocol and Web/client-core compatibility rejection before application dispatch. See the current [client contract](../contracts/client-shell.md), [CLI contract](../contracts/cli.md) and [operation registry](../contracts/interfaces.md).

These are supporting partial proofs, not a production user profile, remote HTTP/TLS transport, streaming/reconnect/cancellation matrix, complete Dioxus peer, inbound MCP server or full cross-platform CLI acceptance. The Android debug WebView and composition-owned Chat page are separate demo evidence; neither promotes the full client contract.

## Planned production user and automation journey

```text
user configures an admitted server and least-privilege profile
→ runs one ustc-agent read-only command
→ client performs compatibility preflight and preserves M10's typed compatibility decision
→ M10 admits identity, bounds and authorization
→ owning application module returns a typed projection
→ CLI emits deterministic human text or versioned JSON
```

For a long-lived operation or event stream:

```text
client submits correlated intent
→ reports Pending only after typed acceptance
→ follows events by monotone cursor
→ reconnects or reconciles an unknown timeout outcome
→ reports terminal state only from server-owned projection
```

Stopping the CLI does not claim server cancellation or success.

## Planned external Agent journey

```text
external Agent connects to reviewed inbound MCP surface
→ discovers only selected bounded tools/resources
→ call binds external session and delegated platform profile
→ adapter invokes the corresponding admitted M10 client operation
→ result preserves source provenance/freshness/uncertainty and typed denial
→ untrusted content is returned as bounded data, not instruction authority
```

The initial inbound MCP surface is reviewed Streamable HTTP and exposes only an exact public-read operation/schema allowlist. It begins with `market.package.list`; campus queries arrive after their owning source/product contracts, and private reads/drafts require a later explicit delegated-profile slice. The MCP adapter does not expose `ustc-agentctl`, arbitrary HTTP, direct Plugin execution, operator credentials or a second Agent/model loop.

CLI, MCP and Dioxus preserve equal permission/result/error/provenance/audit semantics where they expose the same application operation. They do not need identical registries: login and target-local maintenance are not MCP business tools.

## User-visible states

```text
Ready
Authentication required
Forbidden / capability unavailable
Upgrade required
Incompatible protocol
Offline / transport unavailable
Pending
Conflict / stale precondition
Outcome unknown; reconciliation required
Complete server projection
```

Human CLI output may explain recovery. Machine output retains a stable versioned result/event envelope and exit class.

## Privilege separation

| Surface | Intended caller | Allowed initial authority |
|---|---|---|
| `ustc-agentctl` | operator/developer | separately contracted verification and future audited operator actions |
| `ustc-agent` | ordinary user/script | admitted user queries and explicitly contracted commands |
| inbound MCP | delegated external Agent | selected least-privilege tools/resources only |
| Dioxus | ordinary user | admitted presentation queries/commands through the same semantic client core |

No surface inherits another surface's credential profile or command registry by fallback.

The paired Skill is documentation only: it may explain operation choice, typed inputs/results and stop/confirmation rules, but it contains no credential and cannot grant or widen authority.

## Failure and recovery copy

- Server-typed `upgrade_required`: “The server requires a newer client protocol; no operation was submitted.”
- Server-typed `incompatible_protocol`: “The client protocol is unsupported, missing or malformed; no operation was submitted.”
- Authentication required: “Choose or refresh a user profile; operator credentials are not used automatically.”
- Policy denial: report the stable capability/policy class without suggesting a same-name fallback.
- Timeout after possible acceptance: report the correlation identity and require reconciliation before retry.
- Cursor too old: reload the current projection explicitly; do not invent missed events.
- Partial machine output: mark the command failed; consumers must not accept a truncated JSON/NDJSON value.
- MCP result rejected or oversized: return a bounded typed error; do not bypass through raw backend access.

## Non-goals

- replacing the graphical client;
- making GUI call a CLI executable;
- embedding backend domain logic or an Agent loop in the CLI/MCP adapter;
- exposing arbitrary campus URLs, filesystem/process operations or database queries;
- granting external Agents operator/admin capability;
- treating Market package metadata as campus source truth;
- claiming source/product data is available before its owning module is implemented.
- automatic enrollment, registration, payment or external campus-system submission;
- arbitrary shell, URL, filesystem, database, container or third-party MCP access.

## Delivery sequence

The retained Affairs-first protocol/compatibility prerequisite and fixture-loopback CLI above precede the following still-planned production access lane. Its Market-first public-read order remains the accepted target; listing it does not deny the already retained Affairs evidence or promote later routes:

```text
M10 operation/schema/permission registry
→ M10 client-protocol + M80 client-core against fake M10
→ server.info / capability.list protocol proof
→ ustc-agent + market.package.list real read path
→ inbound MCP public-read market.package.list
→ affairs.search / affairs.get after M60/M71 authority exists
→ delegated private reads and tenant-local planner drafts
→ later admitted Windows GUI after a separate promotion gate
```

This is the M10/M80 client-access lane order. It does not replace the product implementation order of ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph integration.

## Verification

- `CLIENT-007`: M10 owns the framework-neutral protocol carrier and never depends on M80 core; equivalent fake-M10 cases reduce equivalently through Dioxus, CLI and MCP adapters over one M80 typed core.
- `CLIENT-008`: GUI/service paths never shell out to CLI, and user/MCP surfaces cannot reach operator command or credential paths.
- `CLIENT-009`: real `ustc-agent` read path proves JSON/NDJSON framing, typed exit/error, auth isolation, compatibility and reconnect/cancellation distinction.
- `CLIENT-010`: external MCP conformance proves bounded discovery/invocation, tenant/grant isolation, instruction-isolated results and no M51/domain/operator reach-through.

All four rows remain `planned` and non-pass in the [active matrix](../acceptance/matrix.tsv). CLIENT-007 and CLIENT-009 retain supporting Affairs-first protocol/client-core evidence, and the fixture-loopback CLI is executable, but the full peer/host/stream/authentication assertions are not satisfied. CLIENT-008's complete confinement manifest and CLIENT-010's external MCP conformance remain future bindings. No MCP endpoint, live campus-source activation, production client or Dioxus readiness is claimed.
