# Documentation coverage matrix

This matrix keeps the live documentation layers aligned:

- `plan/` — engineering constitution, cross-system policy and independent module blueprints;
- `features/` — user-visible behavior;
- `contracts/` — exact public/module/machine boundaries;
- `acceptance/` — stable proof cases and gates;
- `tasks/` — work/assembly scheduling only; never behavior authority.

## Foundation and cross-system plans

| Blueprint | Feature projection | Contract / registry | Acceptance binding |
|---|---|---|---|
| `00-engineering-constitution` | — | repository `AGENTS.md`, `tasks/00-module-work-policy.md` | `acceptance/gates.md`, repository checker |
| `01-terminology` | all feature vocabulary | `contracts/module-boundaries.md`, Market schema/capability IDs | matrix/checker vocabulary |
| `02-product-positioning` | `00-market-browse-install`, three first-party features, `05-headless-client-and-agent-integration` | `plugin-package.md`, `client-shell.md`, `cli.md` | `MARKET-*`, `FP-*`, active `CLIENT-007`–`CLIENT-010` |
| `03-platform-authority` | `00-market-browse-install`, `04-bounded-agent-harness`, `05-headless-client-and-agent-integration` | `platform-identity.md`, `platform-session.md`, `module-boundaries.md`, `agent-harness.md`, `agent-plugin-boundary.md`, `client-shell.md`, `interfaces.md`, `permissions.md` | `AUTH-*`, `HARNESS-*`, `AGENT-*`, `MARKET-*`, active `CLIENT-007`–`CLIENT-010`, long-horizon client cases |
| `04-market-and-plugin-lifecycle` | `00-market-browse-install` | `plugin-package.md`, `market-lifecycle.md`, `market-catalog-query.md`, `permissions.md`, `invocation-resolution.md`, `agent-plugin-boundary.md` | `MARKET-*`, `PKG-*`, selected `AGENT-*`, `FP-*` |
| `05-campus-trust-kernel` | all three first-party features | `source-import.md`, `source-retrieval.md`, `data-models.md` | `SRC-*`, `PROC-*`, `RADAR-*`, `COURSE-*` |
| `06-first-party-plugins` | three first-party features | manifests, `data-models.md`, `source-import.md` | `FP-*`, `PROC-*`, `RADAR-*`, `COURSE-*` |
| `07-runtime-and-integration` | `04-bounded-agent-harness` | `agent-harness.md`, `agent-plugin-boundary.md`, `agent-runtime.md`, `agent-chat.md`, `interfaces.md` | active `CHAT-*`, `HARNESS-*`, `AGENT-*`, `RUNTIME-*`; long-horizon `AI-*`/`MCP-*`/`RUN-*` retained in `platform-baseline.md` |
| `08-security-and-delivery` | permission/privacy/publication failure states | `platform-identity.md`, `permissions.md`, `source-import.md`, module-specific security boundaries | `AUTH-*`, `PUBLIC-*`, release/security gates |

## Large-module blueprints

| Module blueprint | Primary public boundary | Feature projection | Acceptance projection |
|---|---|---|---|
| `modules/00-module-map` | `module-boundaries.md` | all | every module must bind before implementation |
| `M00 Platform Control/Identity` | `platform-account.md`, `platform-identity.md`, `platform-session.md`, `platform-session-port.md`, `platform-control-evidence.md`, `platform-request-context.md`, `B-M00-M10-ACTOR` | bounded B4a session read, B4b redacted evidence ports and fixed B5 Affairs/ChangeRadar administrator compositions are implemented; each fixed command persists or verifies admitted-request evidence before its owning product effect, while production evidence service/SSO remain planned | `active:AUTH-011`; `active:AUTH-012`; `active:AUTH-013`; `active:AUTH-014`; `active:AUTH-015`; `active:AUTH-016`; `active:AUTH-017`; `active:AUTH-018`; `active:AUTH-019`; `active:AUTH-020`; `active:AUTH-021`; `active:AUTH-022`; `active:AUTH-023`; `active:AUTH-024` |
| `M10 Application Ingress Host` | `market-catalog-query.md`, `agent-chat.md`, `B-M80-M10-CALL`, `B-M10-M80-RESULT`, `B-M10-M80-EVENT`, `B-M10-APP-CALL`, `B-APP-M10-RESULT`; canonical application-operation/schema and per-adapter allowlist registry in `interfaces.md` | query carriers, fixed typed `affairs.publish`/`change.publish` commands, four static M72 private operations, the closed loopback `agent-chat/v1` route, safe provider-status query and explicit tool-free local-chat profile and loopback HTTP/CLI/Web projections; Dioxus, public-read-first inbound MCP and production auth/streams remain later journeys | `active:CHAT-*`; `active:AUTH-023`; `active:AUTH-024`; `active:PROC-011`; `active:RADAR-001`; `active:CLIENT-007`; `active:CLIENT-009`; `active:CLIENT-010`; `long-horizon:CLIENT-001`; `long-horizon:CLIENT-002`; `long-horizon:CLIENT-003`; `long-horizon:CLIENT-004`; `long-horizon:CLIENT-005`; `long-horizon:CLIENT-006`; `long-horizon:WEB-*` |
| `M20 Market/Package` | `plugin-package.md`, `market-lifecycle.md`, `market-component-configuration.md`, `invocation-resolution.md`, package/install/grant/resolver contracts, `B-M20-M72-AUTH` | `00-market-browse-install`; pure package-owned `M20-CONFIG-001/002` schema validation and exact component binding support configuration; bundled catalog query/browser has bounded `MARKET-008` evidence, while durable configuration remains planned; current bounded M72 composition consumes transaction-current static application authorization without creating a tool projection | `active:MARKET-*`; `active:PKG-*`; `active:AGENT-*`; `active:FP-*` |
| `M30 Agent Harness/Runtime` | `agent-harness.md`, `agent-runtime.md`, `agent-chat.md`, `chat-conversations.md`, `chat-activity.md`, `B-M30-M50-MODEL`, `B-M20-M30-TOOLSET`, `B-M30-M40-CALL`, `B-COMP-M30-EFFECT`, `B-M40-M30-RESULT` | `04-bounded-agent-harness`; a bounded app-private three-turn/four-call Chat coordinator owns M30 provider/tool orchestration for Affairs/ChangeRadar and may invoke only the separately authorized static M72 plan use case; M72 consent/profile/planning semantics are not M30/M40 evidence, and the durable full harness remains planned | `active:CHAT-001`; `active:CHAT-002`; `active:CHAT-004`; `active:CHAT-005`; `active:HARNESS-*`; `active:AGENT-*` |
| `M40 Tool Gateway/Execution` | `agent-plugin-boundary.md`, `invocation-resolution.md`, directional call/result/executor boundaries | tool/review states in harness journey | `active:AGENT-*`; `active:MARKET-*`; `active:PKG-*` |
| `M50 Model Provider` | `agent-chat.md`, `B-M30-M50-MODEL` | deterministic no-network mock and one bounded non-streaming operator-configured OpenAI-compatible adapter are implemented for the Chat MVP; the complete provider platform remains a gap | `active:CHAT-001`; `gap`; `long-horizon:AI-*` |
| `M51 MCP Binding/Executor` | `B-M40-M51-EXEC`, `mcp-execution.md`, `plugin-management.md` | bounded Streamable HTTP adapter and single-component public-read package journey; broader MCP baseline remains later | `active:MCP-019`; `active:PLUGIN-001`; `long-horizon:MCP-001`; `long-horizon:MCP-002`; `long-horizon:MCP-003`; `long-horizon:MCP-004`; `long-horizon:MCP-005`; `long-horizon:MCP-006`; `long-horizon:MCP-007`; `long-horizon:MCP-008`; `long-horizon:MCP-009`; `long-horizon:MCP-010`; `long-horizon:MCP-011`; `long-horizon:MCP-012`; `long-horizon:MCP-013`; `long-horizon:MCP-014`; `long-horizon:MCP-015`; `long-horizon:MCP-016`; `long-horizon:MCP-017`; `long-horizon:MCP-018` |
| `M60 Campus Trust/Source` | `source-import.md`, `source-retrieval.md`, `B-M60-M70/71/72-*`, `B-M60-M90-SOURCE-TRANSPORT` | all first-party features | `active:SRC-*`; `active:PROC-*`; `active:RADAR-*`; `active:COURSE-*`; `active:FP-*` |
| `M70 ChangeRadar` | typed semantic candidate/review/publication/feed plus checked durable recovery | `02-ustc-change-radar` | `active:AUTH-024`; `active:RADAR-001`; `active:RADAR-002`; `active:FP-*` |
| `M71 Affairs Navigator` | procedure draft/artifact/search contracts | `01-ustc-affairs-navigator` | `active:PROC-*`; `active:FP-*` |
| `M72 Opportunity Graph` | `interfaces.md` private-operation registry, `B-M00-M72-PROFILE`, `B-M20-M72-AUTH`, `B-M60-M72-OPPORTUNITY` and typed opportunity/profile/planner values | `03-campus-opportunity-graph`; current four private operations are static M72 application use cases after M00/M10 admission and transaction-current M20 authorization, with no M30/M40/provider/plugin execution spine | `active:COURSE-*`; `active:FP-*` |
| `M80 Client Core and Interaction Shells` | `client-shell.md`, `cli.md`, `interfaces.md`, `permissions.md`, `B-M80-M10-CALL`, `B-M10-M80-RESULT`, `B-M10-M80-EVENT` | current M80 evidence remains the framework-neutral client core, ordinary-user Affairs CLI and operation-specific Affairs presentation proof; the colocated static Chat shell is composition-only evidence credited to M30/M50/M90, not M80; peer Dioxus Web/Android and public-read-first inbound MCP remain later journeys; Windows launchers do not promote Windows GUI target support | `active:CLIENT-007`; `active:CLIENT-008`; `active:CLIENT-009`; `active:CLIENT-010`; `long-horizon:CLIENT-001`; `long-horizon:CLIENT-002`; `long-horizon:CLIENT-003`; `long-horizon:CLIENT-004`; `long-horizon:CLIENT-005`; `long-horizon:CLIENT-006`; `long-horizon:WEB-*` |
| `M90 Infrastructure/Operations` | `agent-chat.md`, module-owned ports plus Docker Compose Fullstack deployment/recovery contracts | loopback deterministic-mock Compose MVP package with stop/restart persistence and reset-only volume deletion; no independent product behavior | `active:CHAT-003`; `active:RUNTIME-*`; `active:PUBLIC-*`; `long-horizon:CFG-*`; `long-horizon:REL-*`; `long-horizon:DEP-*` |

The machine-checked acceptance projection uses only these code-formatted tokens:

- `gap` — dedicated active rows are still missing; this is not a pass;
- `active:<CASE-ID-or-FAMILY-*>` — the exact case or at least one case in the family exists in `matrix.tsv`;
- `long-horizon:<CASE-ID-or-FAMILY-*>` — the exact case or family exists only in `platform-baseline.md` and is not an active gate.

Tokens are checked independently; one `long-horizon:` token cannot mask an invalid `active:` claim for another reference. A module may write contract/fixture scaffolding while it has `gap`, but cannot claim `StandaloneReady` until the required active rows and bindings exist.

## Usable demo and standalone SSO example

The bounded [usable-demo contract](contracts/usable-demo-enhancements.md) and [MVP feature projection](features/06-mvp-core-capabilities.md) bind active `UE-001`, `UE-002`, `UE-003` and `SSO-001` rows in `acceptance/matrix.tsv`. The existing [ci.yml](../.github/workflows/ci.yml) workflow executes their real Python HTTP, JavaScript unit and compiled-binary browser commands on every pull request and main push. Chat-first view navigation, draft and pending-response preservation, mobile focus behavior, bounded Markdown and exact answer copying are presentation evidence under `CHAT-001`/`UE-003`. These cases cover the existing demo composition and the separately executable [SSO sample](../examples/sso-interface/README.zh-CN.md); they do not promote M00 production authentication, M80 client parity or M60 source readiness. All pre-existing matrix rows and module projections retain their prior meanings.

## Non-matrix documents

| Path | Role | Authority rule |
|---|---|---|
| `acceptance/platform-baseline.md` | retained long-horizon case catalog | preserves stable planned cases; only IDs projected into `matrix.tsv` are active gates |
| `overview/architecture.md` | cross-layer/module navigation map | summarizes and links; does not own behavior |
| `tasks/00-module-work-policy.md` | module ownership, commits, review and assembly process | schedules work; defers to constitution/plans/contracts |
| `tasks/01-execution-roadmap.md` | module batches/dependencies/assembly gates | schedules approved contracts; cannot override plans |
| `guides/contributing.md` | collaboration workflow | defers to root/docs `AGENTS.md` and plans |
| `guides/development.md` | local command/runbook | commands must match current CI/contracts |
| `guides/github-pages-brief.md` | future frontend/publication handoff | public-readiness gate owns publication permission |
| `adr/` | decision history | explains why; current plan/contract owns behavior after clarification/amendment |
| `design/` | subordinate presentation design packets (`Proposal`/`Reviewed`/`Superseded`) | presentation proposal only; defers to plans/contracts/acceptance; never a peer authority; coverage claims stay inside the packet and never enter the acceptance matrix |

## Rules

- Every product-visible behavior MUST map to an owning plan, module blueprint, typed contract and active acceptance row before implementation completion.
- Every cross-module call MUST appear in `contracts/module-boundaries.md` or a named more-specific contract.
- A single acceptance case MAY cover several chapters/modules, but its assertion and binding must remain exact.
- `planned`, skipped, unavailable and not-run are non-pass states.
- A task/report/overview MUST NOT introduce a new product identity, authority class or lifecycle transition.
- Existing code evidence does not promote an incomplete large module.
- Add a new docs directory only when a real document has a distinct semantic role; do not create empty architecture theatre.

## Multi-user service construction

[Platform accounts](contracts/platform-account.md) owns the planned SSO/admin-provisioned account
construction contract under M00. [The multi-user delivery task](tasks/multi-user-campus-agent.md)
orders accounts, scoped private state, durable Chat, package lifecycle, MCP/Skills and
campus source integration. Its proposed `ACCOUNT-*` cases are not active/pass evidence.
The existing loopback demo is not a production multi-user deployment. New service state
must preserve legacy ownership and receipts; SSO activation needs actual integration inputs.

`market-catalog-query.md` binds the static composition's public metadata browser to
`MARKET-008`; it does not promote M80 peer-client readiness or the complete `MARKET-001`
journey. `M20-CONFIG-002` contributes only schema/pin consistency evidence to `MARKET-002`.

[Saved conversations](contracts/chat-conversations.md) bind `CHAT-004`: server-owned
owner-scoped history, durable turn reservation, exact submission replay, browser
reload/switch/continuation and explicit interrupted recovery. This local composition
uses the existing demo identity; full Harness, streaming and real multi-user ingress
remain planned. Account entry is SSO or administrator-provisioned users only.

[Chat activity](contracts/chat-activity.md) binds `CHAT-005`: bounded real provider/tool
observations with owner-scoped read-only polling and terminal safe trace reconstruction.
This rebuildable view does not add streaming, cancellation or durable Harness authority.

[Reviewed component configuration](contracts/market-component-configuration.md) binds
`MARKET-009`: complete checked component/schema sidecar loading and a fixed app bundle
whose catalog revision includes configuration sources. Runtime installation, grants,
configuration persistence and MCP/Skill execution are separately bounded by `PLUGIN-001`, not implied by CONFIG003 alone.

`MCP-019`, `SKILL-009`, `PLUGIN-001` and `PLUGIN-002` track the admitted MCP/Skill
protocol, context, lifecycle and operator-preparation slices. The application reuses
original M20 ledgers and M30 journals and keeps frozen projection authority outside
the Agent. Public-read single-component operation does not promote production SSO,
private/write approvals, stdio, executable Skills, updates/rollback or full module exits.

`active:MODEL-001` maps the bounded M50 provider catalog and M30 per-turn model selection
to `contracts/model-selection.md`, `guides/model-selection.md` and the chat feature projection.

`active:CONVERSATION-MANAGE-001` maps the M30 conversation metadata lifecycle to
`contracts/conversation-management.md`, the chat feature, and the history menu UI.

## Calendar proposal extension

[Calendar proposals](contracts/calendar-proposals.md) extends the owner-local
Calendar companion under active:CHAT-003 and active:CHAT-004. Its focused core and
real HTTP/controlled-provider tests run under the existing Rust gate; browser
proposal checks bind through the existing usability runner. Pending proposals do
not mutate items; explicit confirmation atomically saves the effect and receipt.
C1 remains partial for reminders and production tenant storage; no module readiness
or existing acceptance-row status is promoted by this extension.
