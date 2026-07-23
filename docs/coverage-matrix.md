# Documentation coverage matrix

This matrix keeps the four live documentation layers aligned:

- `plan/` — engineering blueprint and authority;
- `features/` — user-visible behavior;
- `contracts/` — typed public/machine boundaries;
- `acceptance/` — stable proof cases and gates.

| Blueprint | Feature projection | Typed contract / registry | Acceptance binding |
|---|---|---|---|
| `00-engineering-constitution` | — | repository `AGENTS.md` | `docs/acceptance/gates.md`, repository checker |
| `01-terminology` | all feature vocabulary | `market/` schema/capability IDs | matrix/checker vocabulary |
| `02-product-positioning` | `00-market-browse-install`, `01-ustc-affairs-navigator`, `02-ustc-change-radar`, `03-campus-opportunity-graph` | `plugin-package.md` | `MARKET-*`, `FP-*` |
| `03-platform-authority` | `00-market-browse-install`, `04-bounded-agent-harness` | `agent-harness.md`, `invocation-resolution.md`, `agent-runtime.md`, `interfaces.md`, `permissions.md`, Rust domain identities | `HARNESS-*`, `AGENT-*`, `MARKET-*`, `RUNTIME-*` |
| `04-market-and-plugin-lifecycle` | `00-market-browse-install` | `invocation-resolution.md`, `plugin-package.md`, `permissions.md`, Market JSON schema/registry/manifests | `MARKET-*`, `AGENT-002`, `FP-006`, `FP-015`, `FP-007` |
| `05-campus-trust-kernel` | all three first-party feature docs | `source-import.md`, `data-models.md` | `SRC-*`, `PROC-*`, `RADAR-*`, `COURSE-*` |
| `06-first-party-plugins` | three first-party feature docs | package manifests, `data-models.md`, `source-import.md` | `FP-*`, `PROC-*`, `RADAR-*`, `COURSE-*` |
| `07-runtime-and-integration` | `04-bounded-agent-harness`; R0 owned kernel; P0a bounded resolver proof | `agent-harness.md`, `agent-runtime.md`, `invocation-resolution.md`, `interfaces.md`, `permissions.md` | planned `HARNESS-*`; `AGENT-001`, `AGENT-002`; implemented `MARKET-005/006`; planned `MARKET-007`, `RUNTIME-*` |
| `08-security-and-delivery` | publication and permission failure states across features | `permissions.md`, `source-import.md` | `PUBLIC-*`, release-gated security rows |

## Non-matrix documents

| Path | Role | Authority rule |
|---|---|---|
| `acceptance/platform-baseline.md` | retained long-horizon case catalog | preserves stable planned cases; only IDs projected into `matrix.tsv` are active gates |
| `overview/architecture.md` | cross-layer navigation/map | summarizes and links; does not own behavior |
| `tasks/01-execution-roadmap.md` | dependency-aware work order | schedules approved contracts; cannot override plans |
| `guides/contributing.md` | collaboration workflow | defers to root/docs `AGENTS.md` and plans |
| `guides/development.md` | local command/runbook | commands must match current CI/contracts |
| `guides/github-pages-brief.md` | future frontend/publication handoff | public-readiness gate owns publication permission |
| `adr/` | decision history | explains why; amended ADRs are not current behavior authority |

## Rules

- Every product-visible behavior MUST map to an owning plan, typed contract and acceptance row.
- A single acceptance case MAY cover several chapters, but its assertion and binding must remain exact.
- `planned`, skipped, unavailable and not-run are non-pass states.
- A task/report/overview MUST NOT introduce a new product identity, authority class or lifecycle transition.
- Add a new docs directory only when a real document has a distinct semantic role; do not create empty architecture theatre.
