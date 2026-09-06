# Package-owned MCP and Skill application profile

Owner: M20 lifecycle commands, M51 protocol/resource adapters, M30 run journal;
composition and HTTP endpoints live in ustc-agentd. Case: PLUGIN-001.

This local Linux/WSL server profile exposes reviewed package installation, typed
configuration, connection/resource validation, explicit capability approval,
enable, disable and revoke. The current authenticated fixture owner is selected
by the existing application context; HTTP input never selects tenant/user. This
is not production SSO evidence. Remote operations and arbitrary browser-selected
filesystem paths are not part of this profile.

Packages come from the reviewed embedded campus Skill or operator-selected package
directories (UCA_PLUGIN_PACKAGE_DIRS). Each external directory contains package.json,
configuration.json and a closed runtime.json declaration. MCP tool-to-capability
mapping is operator-reviewed package data. Server annotations and model text cannot
declare a call read-only or authorize it. Configuration only accepts fields in the
package binding. Endpoint trust profile and credential file are operator inputs,
not browser settings; credential bytes never enter domain configuration/receipts.
`bearerFile` and operator-only `credentialEndpoint` must appear together in runtime.json.
The latter is a complete HTTP(S) URL without credentials or a fragment. Before opening
that credential file or initializing any connection, the configured endpoint must match
this reviewed URL byte-for-byte, including path and query. A mismatch is denied; changing
browser configuration cannot redirect an operator credential to another target.

The runtime lifecycle query is distinct from anonymous catalog declarations. It
projects current owner installation state and supported configuration fields. Every
package row carries `available`. Historical owner installations whose exact package pin
has disappeared or changed remain visible with `available: false`, historical identity,
and current installation state. Such rows expose no current capability/configuration
schema claims and offer only disable/revoke; they cannot configure, probe, grant or enable. A
probe performs initialization/discovery or verified Skill reads only; it never
calls an MCP business tool. Enable requires the exact complete reviewed readiness
digest returned by that probe plus all required explicit grants. Version/revision
and request identity conflicts fail visibly. Exact historical retries return the
original receipt before consulting current catalog or discovery.

Each future Agent toolset is compiled from enabled owner installations. Skill
metadata becomes a namespaced read-context tool; its body and declared references
are read lazily as untrusted tool data. MCP tools are namespaced and validated
against the platform schema before the complete model batch executes. Current
authority is checked again immediately before execution through the adopted
resolver. Disabled/revoked installations cannot load context or start MCP calls.
Each model request freezes an opaque gateway session containing the owner, installation revision,
readiness, tool/schema and grant identity/version. A later change rejects the old proposal before
adapter I/O or effect intent; a new request must receive a fresh projection.
Persisted review identity must match rediscovery after restart; tool/schema drift
requires another review. No MCP business call is automatically retried.

The admitted application profile currently dispatches public-read capabilities;
private/side-effect capabilities remain denied until the concrete per-call approval
flow is implemented. This restriction is visible in the UI and does not reinterpret
MCP annotations as permission. Central stdio execution and executable Skill scripts
remain outside the admitted server profile.

Persistence is one exclusive private owner with atomic installation/grant snapshots;
M30 run specifications/events are separately owned journal data. A persistence
failure prevents acknowledgement and poisons the writer when the commit outcome
is uncertain. Recovery never re-executes an unresolved effect.

Bound acceptance: owner isolation, exact retries/conflicts, configure/probe/grant/
enable, actual MCP call and lazy Skill read, disable/revoke, restart/reconnect,
schema drift, server failure and persistence failure; browser assertions exercise
actual supported lifecycle controls. This bounded profile is implemented with the PLUGIN-001 evidence; the broader module exit gates remain unchanged.

## Operator preparation command

`ustc-agentctl market prepare-component-config --package-dir PATH` reads one local
checked single-component package declaration and its declared Skill or runtime.json
artifact, then creates configuration.json with the existing M20 canonical declaration
and schema owners. It does not contact a service, install, grant, enable, execute a
Skill, or overwrite an existing sidecar. MCP preparation reads the declared endpointKey
from runtime.json and emits one required text field (2048-byte ceiling); Skill
preparation emits an empty configuration schema. Full application loading still
validates runtime policy, resources and capability mapping separately. Symlinks,
traversal, unsupported component kinds and out-of-bounds source input reject.

Preparation walks the absolute package directory chain through directory handles and
rejects symlink roots/ancestors. Relative CLI paths resolve from the current directory;
parent (`..`) traversal rejects. Reads use descriptor-relative NOFOLLOW/NONBLOCK and
regular-file checks. Exclusive sidecar creation, file sync and directory sync use the
same opened package directory, even if its pathname is concurrently replaced. Native
Windows and other non-Unix preparation hosts explicitly return unsupported; other
operator commands retain their existing platform support.

## Capacity and recovery (PLUGIN-001)

This bounded local profile retains at most 1,024 execution journals and 8 MiB of
serialized journal data per runtime. Reaching either bound rejects further writes;
it does not evict acknowledged receipts. A full journal count rejects a new call
before adapter execution or a new effect intent. Tool results report
`plugin_capacity_exceeded`, so capacity cannot masquerade as a transient transport
failure. Restart, disable and revoke do not release retained evidence. Durable
archival, retention administration and production per-tenant quotas remain planned;
this profile must not be presented as an unlimited multi-user service. Operators
must preserve evidence rather than clear state to conceal exhaustion.

The application reserves at most 28 dynamic tools per owner (four Chat tools retain
their existing slots). Enable checks the sum of current exact-package enabled
installations and the candidate before committing; Skill counts as one tool and MCP
uses its complete reviewed tool mapping. It does not double-count the candidate or
borrow slots from another owner, and does not perform discovery to count declarations.
A rejected excess enable leaves existing installations and grants intact. A legacy
over-budget session must produce an explicit composition failure instead of silently
removing all plugin tools from a Chat request.

Encoding/capacity rejection before disk I/O is known non-commit and does not poison
the runtime. Uncertain persistence failure still poisons mutation. The UI recognizes
only the exact 429 `plugin-error/v1` / `plugin_capacity_exceeded` as known rejection,
clears that pending request and refreshes state. Generic 429, timeouts, incomplete
success and network loss retain the original request for explicit same-ID retry.
The plugin page also projects when the selected model has no tool-calling capability;
plugin enablement alone cannot give a text-only model tool support. The deterministic
mock is separately labelled as supporting only its built-in campus tools; its tool
capability flag does not imply dispatch of installed MCP or Skill tools.

## Recoverable invocation arguments (PLUGIN-001)

A current, authorized plugin invocation with invalid arguments returns a failed
untrusted tool result with code `plugin_invalid_arguments`, allowing the model to
correct its proposal within the existing turn/tool budget. This does not request
new user permission or automatically retry a tool. Missing/revoked authority remains
`plugin_permission_denied`; changed or unverifiable readiness remains
`plugin_review_required`. Raw adapter errors, credentials and host paths are never
projected. MCP invocation errors use the adapter-owned binding state: loss of active
readiness requires review, while a transport or business failure on an active binding
remains execution-unavailable. Each admitted failure still records its M30 receipt
without automatically retrying the business call. Explicit undeclared Skill resource paths are argument errors; they do not
invalidate a previously reviewed installation or permit a filesystem fallback.
