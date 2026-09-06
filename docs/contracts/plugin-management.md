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

## Mixed components and inert import review (PLUGIN-001)

The local profile also admits `plugin-runtime/v2`: a closed `{schemaVersion,
components}` object whose members contain exact `componentId` and one existing
`plugin-runtime/v1` declaration under `runtime`. Every declared package component
must occur exactly once (at most 16). The current MCP artifact remains `runtime.json`,
so a package admits one MCP member plus declared Skill members; it does not admit
independent component installation. All members share one exact package configuration
schema. This explicit profile preserves full-object validation and rejects conflicting
per-component schemas instead of dropping fields. Every MCP artifact pin hashes the
complete runtime envelope. Skill resources retain exact declared content digests.

Probe validates every member and returns one complete package readiness digest.
Enable requires all package capabilities and exact member readiness. Each model tool
is namespaced by installation and component and freezes the component ID for resolver
admission and execution. A partial failed probe retires every opened member session;
package disable/revoke retires every member. Restart must reproduce the whole package
readiness before either MCP calls or Skill reads become usable. Single-component v1
readiness and tool identities remain compatible.

`POST /api/v1/plugins/import-preview` accepts `plugin-import-preview/v1`: package ID,
version, display name, source description, optional literal Skill text and optional
MCP HTTPS endpoint with explicit tool-to-public-read-capability mapping. Unknown
fields (including executable commands), raw endpoint credentials, endpoint queries,
private/write capabilities and invalid Skill names reject. It returns
`plugin-import-review/v1` with deterministic file-map digest, package/runtime/configuration
files, separate proposed non-secret configuration values, warnings and `admitted:false`.
The tier in a candidate manifest is a proposed post-review classification, never proof
that review has occurred. The response neither persists nor publishes a catalog entry,
contacts an endpoint, reads a filesystem path, grants a capability, installs a package,
nor executes a resource. Text remains untrusted content.

The UI exposes the complete candidate files and JSON download. Operator review and
explicit package-directory admission remain necessary; the resulting package then
uses the existing install/configure/probe/grant/enable path. Automatic catalog promotion,
URLs that fetch arbitrary packages, archive extraction, OAuth/stdio and executable
Skill resources are outside this profile. Mixed-package/import evidence is separate from the B6 update evidence below.

## Exact version update, rollback and recovery (PLUGIN-001)

`POST /api/v1/plugins/updates` accepts a closed `plugin-update/v1` carrier with one
request ID and typed `preview`, `apply`, `review_rollback`, `rollback` or `confirm`
intent. Requests bind the admitted owner, exact installation and expected revision.
Preview accepts one other currently reviewed version of the same package. It runs
bounded member readiness checks for both versions using the existing configuration;
a target requiring an incompatible configuration rejects before a version change.
No automatic configuration migration or permission expansion is inferred.

Preview returns the B6 exact plan digest, change class, old/target versions and both
complete readiness digests. The UI shows the target capability list and requires an
explicit version-change confirmation. Apply rediscovers both versions and rejects a
changed digest before mutation. The public M20 trusted application port constructs
B6 Stage, RecordApproval and Apply under one atomic application transaction, while
reusing the original installation/grant repositories. It never modifies a package
pin directly. Installation identity stays fixed. The result is Disabled and
AppliedPendingConfirmation; every active old grant becomes stale, including grants
already bound to an earlier installation revision after disable. A fresh complete
probe and explicit grants are required before enabling the new version.

Rollback review rechecks the retained exact old package against current configuration.
Rollback requires Disabled state, the reviewed rollback digest and an explicit user
confirmation; it invokes the existing B6 Rollback transition and again invalidates
active grants. Confirm explicitly retains the applied version and closes that B6
rollback window. Missing/changed reviewed package inputs, unavailable artifacts,
conflicting installation revisions, another active B6 update or a separately existing
target-version installation fail closed. Rollback never restores old grants.

Update request identity and the complete typed-intent digest are retained. A same-ID,
same-intent retry returns the original B6 result before consulting current package
availability or revision. A changed intent conflicts. This applies after restart and
after later rollback/confirmation. Readiness probes perform no business-tool call.

The private `uca-plugin-authority/v2` container atomically stores the original
installation/grant ledgers, M30 execution journals and the M20 update application
journal; v1 files remain readable. A successful subsequent write uses v2. Older
servers do not understand v2, so binary downgrade requires a compatible reader or an
operator-controlled restore of a complete pre-update authority snapshot; copying only
one ledger or silently resetting state is forbidden. No automatic destructive file
migration is performed.

The M20 update journal is a bounded replay carrier (`market-update-application/v1`):
at most 64 mutation frames and 16 MiB. Each frame pins its prior installation/grant
ledger, exact archived catalog/configuration declarations, observed readiness and B6
result event digest. Recovery replays B6 commands, verifies original owner/revision
and exact event results, and proves later ordinary-ledger records extend the previous
result. New coupled package/grant update records are rejected unless generated by the
replayed B6 frame. Missing, reordered, substituted or orphaned update frames cannot
restore authority. Empty/legacy journals reject coupled update events. Uncertain disk
commit poisons mutation; known encoding-capacity rejection leaves prior state usable.
The total container ceiling is 56 MiB plus its bounded header, with the original
installation/grant and execution-journal sublimits unchanged.

This is a local, reviewed-directory application profile. It does not claim distributed
rollout, remote artifact download/promotion, automatic configuration migration,
concurrent multi-server operation, security-revoke distribution or production backup
qualification. Bound targeted cases are `plugin_runtime::tests::update_lifecycle`,
`plugin_runtime::tests::mixed_import`, the existing B6 domain suite and PLUGIN-001.
