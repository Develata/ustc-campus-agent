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
| `02-product-positioning` | `00-market-browse-install`, three first-party features | `plugin-package.md` | `MARKET-*`, `FP-*` |
| `03-platform-authority` | `00-market-browse-install`, `04-bounded-agent-harness` | `module-boundaries.md`, `agent-harness.md`, `agent-plugin-boundary.md`, `client-shell.md`, `interfaces.md`, `permissions.md` | `HARNESS-*`, `AGENT-*`, `MARKET-*`, long-horizon client cases |
| `04-market-and-plugin-lifecycle` | `00-market-browse-install` | `plugin-package.md`, `permissions.md`, `invocation-resolution.md`, `agent-plugin-boundary.md` | `MARKET-*`, `PKG-*`, selected `AGENT-*`, `FP-*` |
| `05-campus-trust-kernel` | all three first-party features | `source-import.md`, `data-models.md` | `SRC-*`, `PROC-*`, `RADAR-*`, `COURSE-*` |
| `06-first-party-plugins` | three first-party features | manifests, `data-models.md`, `source-import.md` | `FP-*`, `PROC-*`, `RADAR-*`, `COURSE-*` |
| `07-runtime-and-integration` | `04-bounded-agent-harness` | `agent-harness.md`, `agent-plugin-boundary.md`, `agent-runtime.md`, `interfaces.md` | `HARNESS-*`, `AGENT-*`, `RUNTIME-*`; long-horizon `AI-*`/`MCP-*`/`RUN-*` retained in `platform-baseline.md` |
| `08-security-and-delivery` | permission/privacy/publication failure states | `permissions.md`, `source-import.md`, module-specific security boundaries | `PUBLIC-*`, release/security gates |

## Large-module blueprints

| Module blueprint | Primary public boundary | Feature projection | Current acceptance binding |
|---|---|---|---|
| `modules/00-module-map` | `module-boundaries.md` | all | every module must bind before implementation |
| `M00 Platform Control/Identity` | `B-M00-M10-ACTOR`, request/session/causation values | cross-feature admission only | dedicated active rows must be added before implementation; current policies are cross-covered by security/harness cases only |
| `M10 Application Ingress Host` | `B-M80-M10-CALL`, `B-M10-M80-RESULT`, `B-M10-M80-EVENT`, `B-M10-APP-CALL`, `B-APP-M10-RESULT`; server-function/public endpoint registry in `interfaces.md` | all real Web/Android/integration journeys | dedicated Fullstack/API/compatibility rows must be active before retained implementation; long-horizon client/web cases remain non-active |
| `M20 Market/Package` | package/install/grant/resolver contracts | `00-market-browse-install` | `MARKET-*`, `PKG-*`, selected `AGENT-*`, `FP-*` |
| `M30 Agent Harness/Runtime` | `agent-harness.md`, `agent-runtime.md`, `B-M30-M50-MODEL`, `B-M20-M30-TOOLSET`, `B-M30-M40-CALL`, `B-COMP-M30-EFFECT`, `B-M40-M30-RESULT` | `04-bounded-agent-harness` | planned `HARNESS-*`; implemented/remaining `AGENT-*` as declared in matrix |
| `M40 Tool Gateway/Execution` | `agent-plugin-boundary.md`, `invocation-resolution.md`, directional call/result/executor boundaries | tool/review states in harness journey | selected `AGENT-*`, `MARKET-*`, `PKG-*` |
| `M50 Model Provider` | `B-M30-M50-MODEL` | model-turn states inside harness journey | active provider rows must be added before implementation; long-horizon `AI-*` retained in `platform-baseline.md` |
| `M51 MCP Binding/Executor` | `B-M40-M51-EXEC` | future MCP-backed tool journey | active MCP rows must be added before implementation; long-horizon `MCP-*` retained in `platform-baseline.md` |
| `M60 Campus Trust/Source` | `source-import.md`, `B-M60-M70/71/72-*` | all first-party features | `SRC-*` plus product rows |
| `M70 ChangeRadar` | typed semantic candidate/event/feed | `02-ustc-change-radar` | `RADAR-*`, relevant `FP-*` |
| `M71 Affairs Navigator` | procedure draft/artifact/search contracts | `01-ustc-affairs-navigator` | `PROC-*`, relevant `FP-*` |
| `M72 Opportunity Graph` | opportunity/profile/planner values | `03-campus-opportunity-graph` | `COURSE-*`, relevant `FP-*` |
| `M80 Dioxus Fullstack Multi-client` | `client-shell.md`, `B-M80-M10-CALL`, `B-M10-M80-RESULT`, `B-M10-M80-EVENT` | required Web/PWA and Android presentation; later iOS/desktop | active client/web/Android compatibility rows must be added before retained implementation; long-horizon `CLIENT-*`/`WEB-*` remain in `platform-baseline.md` |
| `M90 Infrastructure/Operations` | module-owned ports plus Docker Compose Fullstack deployment/recovery contracts | no independent product behavior | long-horizon `CFG-*`, `REL-*`, `DEP-*` include required Compose Web/Android read-back and must be activated with exact bindings as M90 scope enters implementation; active `RUNTIME-*` applies only to shared restart/receipt behavior and `PUBLIC-*` to public delivery |

“Must be added before implementation” is an explicit gap, not a pass. A module may write contract/fixture scaffolding first, but cannot claim `StandaloneReady` until its active rows and bindings exist.

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
