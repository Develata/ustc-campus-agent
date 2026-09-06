# Market browse, install and control

- `Status`: Partial user journey: bundled browsing and durable single-component public-read MCP/Skill lifecycle are implemented in the loopback application; authenticated multi-client delivery, package updates and artifact switching remain planned
- `Owning plan`: `docs/plan/04-market-and-plugin-lifecycle.md`
- `Contracts`: `docs/contracts/market-catalog-query.md`, `docs/contracts/plugin-package.md`, `docs/contracts/market-lifecycle.md`, `docs/contracts/agent-plugin-boundary.md`, `docs/contracts/permissions.md`, `docs/contracts/invocation-resolution.md`
- `Acceptance`: `MARKET-*`, `PKG-019/020`, `AGENT-002`, `AGENT-017/018`, `FP-006`, `FP-015`, `FP-007`

## Goal

A user can inspect a Plugin as a concrete package, understand its publisher/version/components/capabilities/source policy, install an exact version, and control whether the Agent may discover it.

## User-visible states

```text
Available
Installed + Enabled
Installed + Disabled
Update requires approval
Revoked / unavailable
Error with recovery action
```

The UI never displays a package as installed or runnable solely because its manifest has a default-install policy.

## Target journey

The authenticated journey below is the product target. The current loopback flow
uses a controlled demo owner; its supported package scope is described below.

```text
anonymous visitor browses package metadata
→ opens package detail
→ sees publisher, exact version, status, components, permissions and source policy
→ signs in before installation
→ confirms exact package and grants
→ installation becomes enabled
→ resolver/gateway projects approved Plugin-neutral tools
→ Agent can discover approved capability without loading Plugin code
→ user disables Plugin
→ discovery and invocation are denied
→ user re-enables Plugin
→ discovery is restored under current grants
```

The three default first-party packages appear as independent products and can be disabled/re-enabled independently.

Package installation or update never updates the Agent framework. The Agent consumes the same versioned tool definitions/calls/results while package/component/executor identities remain gateway-private. In-flight runs keep their frozen projection; a new package version appears only in a later accepted projection.

## Failure and recovery copy

- Invalid or unavailable manifest: package is not installable; show validation reason without internal secrets.
- Permission expansion: show exact diff and require reapproval; never auto-enable new access.
- Disabled/revoked: explain that invocation is blocked and identify whether the user or operator can recover it.
- Version/component mismatch: stop invocation and ask the user/operator to repair or reinstall; do not route to a same-name alternative.
- Tool/schema/grant mismatch: expose a stable denial/recovery class; never dispatch a same-name alternative or continue with a stale projection.
- Runtime unavailable: preserve installation state but show availability separately from permission state.

## Non-goals

- anonymous installation or execution;
- hidden default grants;
- arbitrary package code execution;
- direct Plugin linkage or arbitrary extension hooks inside the Agent kernel;
- treating a package card as proof of backend runtime;
- public download links before verified releases.

## Verification

Core evidence covers package/catalog identities, capability and installation/grant
rules, bounded transaction-current authority-assembly evidence and update/rollback
decisions. Update
and rollback retain domain/semantic-fake evidence; they do not switch live artifacts.

The [application profile](../contracts/plugin-management.md) adds durable
install/configure/grant/enable/disable/revoke commands, original-receipt replay,
restart recovery, frozen tool projections and M30 call journals for single-component
public-read MCP/Skill packages. `PLUGIN-001` binds the bounded application/browser
path; it does not complete the full Market module.

For the full authenticated target, `MARKET-001` through `MARKET-004`, `MARKET-007`, `PKG-020` and the user journey remain planned.
Production authentication, general package composition, durable update/rollback
and artifact switching remain outside this implemented profile.

## Package component configuration

The approved direction is install an exact package, configure its declared MCP or
Skill components, review permissions, then explicitly enable. Independent component
import is outside this delivery scope. Configuration alone does not make a component
available to the Agent.

`M20-CONFIG-001` adds pure typed schema validation as supporting evidence: required
fields, closed keys, text/integer bounds and secret-reference types. This validator
alone grants no runtime authority. The supported application flow below adds the
configuration API, persistence and component execution within its narrower profile.

## Bundled directory delivery

The static Plugins page also reads the server's immutable bundled package catalog.
Users can search names/identities/permissions, open an exact revision, inspect its
publisher/components/requested permissions/source policy and return to the list.
Failures have retry actions; stale detail responses cannot replace another selection.
This read-only surface has no installation or grant side effect. Current metadata
can describe planned packages even while the separate fixed demo capability works.
`market-catalog-query/v1` is specified in [the query contract](../contracts/market-catalog-query.md);
`MARKET-008` binds bounded application/HTTP/browser evidence. Full multi-client
`MARKET-001` delivery and authenticated lifecycle `MARKET-002` remain planned.


The source build loads the reviewed Simple Calendar configuration declaration together
with the bundled catalog, under [M20-CONFIG-003](../contracts/market-component-configuration.md).
It explicitly declares an empty configuration schema for the existing native demo
component. This declaration lookup is separate from the application lifecycle;
loading a sidecar alone neither installs nor authorizes a component.

## Supported MCP / Skill package flow

The plugin page now links to **管理 MCP 与 Skills**. The supported single-component
profile follows install → typed configuration → check components → review individual
permissions → review the discovered tools/resources and enable. Disabling stops new
Agent calls. Removed package sources retain historical disable/revoke controls.
The embedded campus guide is an optional Skill; operator-reviewed MCP packages can
join the same flow. Broader package classes and private/write approval remain planned.
Use [the setup guide](../guides/mcp-skills.md) for exact supported formats and limits.
