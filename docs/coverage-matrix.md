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
| `01-terminology` | all feature vocabulary | `contracts/platform-account.md`, `contracts/user-context-profile.md`, `contracts/storage-profiles.md`, `contracts/module-boundaries.md`, Market schema/capability IDs | matrix/checker vocabulary |
| `02-product-positioning` | `00-market-browse-install`, three first-party features, `05-headless-client-and-agent-integration` | `plugin-package.md`, `client-shell.md`, `cli.md` | `MARKET-*`, `FP-*`, active `CLIENT-007`–`CLIENT-010` |
| `03-platform-authority` | `00-market-browse-install`, `03-campus-opportunity-graph`, `04-bounded-agent-harness`, `05-headless-client-and-agent-integration` | `platform-identity.md`, `platform-session.md`, `platform-account.md`, `user-context-profile.md`, `storage-profiles.md`, `module-boundaries.md`, `agent-harness.md`, `agent-plugin-boundary.md`, `client-shell.md`, `interfaces.md`, `permissions.md` | `AUTH-*`, `PROFILE-*`, `STORAGE-*`, `HARNESS-*`, `AGENT-*`, `MARKET-*`, active `CLIENT-007`–`CLIENT-010`, long-horizon client cases |
| `04-market-and-plugin-lifecycle` | `00-market-browse-install` | `plugin-package.md`, `market-lifecycle.md` including accepted A1, `interfaces.md`, `module-boundaries.md`, `permissions.md`, `invocation-resolution.md`, `agent-plugin-boundary.md` including accepted B7-B | `MARKET-*`, `PKG-*`, selected `AGENT-*`, `FP-*`; contract acceptance does not change case status |
| `05-campus-trust-kernel` | all three first-party features | `source-import.md`, `data-models.md`, `user-context-profile.md` for the separate M00 private-context boundary | `SRC-*`, `PROC-*`, `RADAR-*`, `COURSE-*`, `PROFILE-*` |
| `06-first-party-plugins` | three first-party features | manifests, `data-models.md`, `source-import.md`, `user-context-profile.md`, `B-M00-M72-PROFILE` | `FP-*`, `PROC-*`, `RADAR-*`, `COURSE-*`, `PROFILE-*` |
| `07-runtime-and-integration` | `04-bounded-agent-harness` | `agent-harness.md`, `agent-plugin-boundary.md`, `agent-runtime.md`, `interfaces.md` | `HARNESS-*`, `AGENT-*`, `RUNTIME-*`; long-horizon `AI-*`/`MCP-*`/`RUN-*` retained in `platform-baseline.md` |
| `08-security-and-delivery` | permission/privacy/publication failure states | `platform-identity.md`, `platform-account.md`, `user-context-profile.md`, `storage-profiles.md`, `permissions.md`, `source-import.md`, module-specific security boundaries | `AUTH-*`, `PROFILE-*`, `STORAGE-*`, `PUBLIC-*`, release/security gates |

## Large-module blueprints

| Module blueprint | Primary public boundary | Feature projection | Acceptance projection |
|---|---|---|---|
| `modules/00-module-map` | `module-boundaries.md` | all | every module must bind before implementation |
| `M00 Platform Control/Identity` | `platform-identity.md`, `platform-session.md`, `platform-account.md`, `user-context-profile.md`, `B-M10-M00-AUTH`, `B-M00-M10-ACTOR`, `B-M00-M30-PROFILE`, `B-M30-M00-PROFILE-PROPOSAL`, `B-M00-M72-PROFILE` | account/auth/session admission, user profile management and purpose-bound context across features | `active:AUTH-011`; `active:AUTH-012`; `active:AUTH-014`; `active:AUTH-015`; `active:AUTH-016`; `active:AUTH-017`; `active:AUTH-018`; `active:AUTH-019`; `active:AUTH-020`; `long-horizon:AUTH-013`; `long-horizon:AUTH-021`; `long-horizon:AUTH-022`; `long-horizon:AUTH-023`; `long-horizon:AUTH-024`; `long-horizon:AUTH-025`; `long-horizon:AUTH-026`; `long-horizon:AUTH-027`; `long-horizon:AUTH-028`; `long-horizon:AUTH-029`; `long-horizon:AUTH-030`; `long-horizon:PROFILE-*` |
| `M10 Application Ingress Host` | `B-M80-M10-CALL`, `B-M10-M80-RESULT`, `B-M10-M80-EVENT`, `B-M10-APP-CALL`, `B-APP-M10-RESULT`; server-function/public endpoint registry in `interfaces.md` | Dioxus, `ustc-agent`, inbound MCP and other admitted integration journeys | `active:CLIENT-007`; `active:CLIENT-009`; `active:CLIENT-010`; `long-horizon:CLIENT-001`; `long-horizon:CLIENT-002`; `long-horizon:CLIENT-003`; `long-horizon:CLIENT-004`; `long-horizon:CLIENT-005`; `long-horizon:CLIENT-006`; `long-horizon:WEB-*` |
| `M20 Market/Package` | `plugin-package.md`, `market-lifecycle.md` exact A1 surface, `interfaces.md` operation registry, `B-M10-M20-MARKET-A1`, `B-M20-M10-MARKET-A1-RESULT`, `invocation-resolution.md`, package/install/grant/update contracts | `00-market-browse-install`; A1/B7-B are accepted unimplemented contracts with no live wire/client projection | `active:MARKET-*`; `active:PKG-*`; `active:AGENT-*`; `active:FP-*` |
| `M30 Agent Harness/Runtime` | `agent-harness.md`, `agent-runtime.md`, `B-M30-M50-MODEL`, `B-M20-M30-TOOLSET`, `B-M30-M40-CALL`, `B-COMP-M30-EFFECT`, `B-M40-M30-RESULT` | `04-bounded-agent-harness` | `active:HARNESS-*`; `active:AGENT-*` |
| `M40 Tool Gateway/Execution` | `agent-plugin-boundary.md` §7.1 accepted B7-B staged test contract, `invocation-resolution.md`, directional call/intent/receipt/result/executor boundaries | tool/review states in harness journey; B7-B unimplemented and no production gateway added | `active:AGENT-*`; `active:MARKET-*`; `active:PKG-*` |
| `M50 Model Provider` | `B-M30-M50-MODEL` | model-turn states inside harness journey | `gap`; `long-horizon:AI-*` |
| `M51 MCP Binding/Executor` | `B-M40-M51-EXEC` | future MCP-backed tool journey | `gap`; `long-horizon:MCP-*` |
| `M60 Campus Trust/Source` | `source-import.md`, `B-M60-M70/71/72-*` | all first-party features | `active:SRC-*`; `active:PROC-*`; `active:RADAR-*`; `active:COURSE-*`; `active:FP-*` |
| `M70 ChangeRadar` | typed semantic candidate/event/feed | `02-ustc-change-radar` | `active:RADAR-*`; `active:FP-*` |
| `M71 Affairs Navigator` | procedure draft/artifact/search contracts | `01-ustc-affairs-navigator` | `active:PROC-*`; `active:FP-*` |
| `M72 Opportunity Graph` | opportunity/preference/planner values plus consumer side of `B-M00-M72-PROFILE`; general profile remains M00-owned | `03-campus-opportunity-graph` | `active:COURSE-*`; `active:FP-*`; `long-horizon:PROFILE-*` |
| `M80 Client Core and Interaction Shells` | `client-shell.md`, `cli.md`, `B-M80-M10-CALL`, `B-M10-M80-RESULT`, `B-M10-M80-EVENT` | framework-neutral client core; peer Dioxus Web/Android, `ustc-agent` and inbound MCP; later iOS/desktop | `active:CLIENT-007`; `active:CLIENT-008`; `active:CLIENT-009`; `active:CLIENT-010`; `long-horizon:CLIENT-001`; `long-horizon:CLIENT-002`; `long-horizon:CLIENT-003`; `long-horizon:CLIENT-004`; `long-horizon:CLIENT-005`; `long-horizon:CLIENT-006`; `long-horizon:WEB-*` |
| `M90 Infrastructure/Operations` | `storage-profiles.md`, module-owned ports plus Docker Compose Fullstack deployment/recovery contracts | no independent product behavior; SQLite local-demo and PostgreSQL hosted/production adapters | `active:RUNTIME-*`; `active:PUBLIC-*`; `long-horizon:STORAGE-*`; `long-horizon:CFG-*`; `long-horizon:REL-*`; `long-horizon:DEP-*` |

The machine-checked acceptance projection uses only these code-formatted tokens:

- `gap` — dedicated active rows are still missing; this is not a pass;
- `active:<CASE-ID-or-FAMILY-*>` — the exact case or at least one case in the family exists in `matrix.tsv`;
- `long-horizon:<CASE-ID-or-FAMILY-*>` — the exact case or family exists only in `platform-baseline.md` and is not an active gate.

Tokens are checked independently; one `long-horizon:` token cannot mask an invalid `active:` claim for another reference. A module may write contract/fixture scaffolding while it has `gap`, but cannot claim `StandaloneReady` until the required active rows and bindings exist.

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

## Rules

- Every product-visible behavior MUST map to an owning plan, module blueprint, typed contract and active acceptance row before implementation completion.
- Every cross-module call MUST appear in `contracts/module-boundaries.md` or a named more-specific contract.
- A single acceptance case MAY cover several chapters/modules, but its assertion and binding must remain exact.
- `planned`, skipped, unavailable and not-run are non-pass states.
- A task/report/overview MUST NOT introduce a new product identity, authority class or lifecycle transition.
- Existing code evidence does not promote an incomplete large module.
- Add a new docs directory only when a real document has a distinct semantic role; do not create empty architecture theatre.
