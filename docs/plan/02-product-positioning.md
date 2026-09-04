# Product positioning

## Metadata

- `Layer`: Product foundation
- `Status`: Accepted
- `Version`: `0.4.1`
- `Last Review`: `2026-09-04`
- `Authority Owns`: product scope, formal first-party identities, non-goals and naming
- `Authority Defers To`: chapters 03–08 for engineering mechanisms and typed contracts for exact fields
- `Counterpart Features`: `docs/features/00-market-browse-install.md`, `docs/features/01-ustc-affairs-navigator.md`, `docs/features/02-ustc-change-radar.md`, `docs/features/03-campus-opportunity-graph.md`, `docs/features/05-headless-client-and-agent-integration.md`
- `Counterpart Acceptance`: `FP-*`, `MARKET-*`, `CLIENT-007` through `CLIENT-010` in `docs/acceptance/matrix.tsv`
- `Primary Code Areas`: `market/`, `plugins/first-party/`, `crates/platform-core/`

## 1. Decision

USTC Campus Agent is a campus-scoped Agent platform, not a general-purpose Agent framework. Its product spine is Plugins Market-first: user-visible capabilities are inspected, installed, authorized, disabled, upgraded and audited through `PluginPackage` contracts.

The project is a student competition prototype and is **not an official University of Science and Technology of China service**.

The accepted product access shape includes Dioxus Web/PWA and Android, a separate ordinary-user/headless `ustc-agent` CLI, and selected least-privilege inbound MCP tools/resources for external Agents. These are peer adapters over one framework-neutral typed client core; the graphical client does not spawn the CLI. `ustc-agentctl` remains a separate operator/developer surface. This access shape does not move any product or platform authority into clients.

## 2. Three default first-party Plugins

| Package | Product-facing name | Primary question |
|---|---|---|
| `ustc.affairs-navigator` | USTC Affairs Navigator | What should I do now? |
| `ustc.change-radar` | USTC ChangeRadar | What changed, and does it affect me? |
| `ustc.opportunity-graph` | Campus Opportunity Graph | What fits me, and what should I choose next? |

All three are formal default first-party products. Implementation priority does not remove, subordinate or merge any product identity.

They share the Campus Trust Kernel but keep independent package identity, version, installation, enable/disable and acceptance boundaries.

## 3. Frozen implementation order

```text
ChangeRadar source/revision/diff foundation
→ Affairs Navigator structured procedure entry
→ ChangeRadar per-board semantic feed and RSS/Atom
→ Opportunity Graph consent/profile integration
```

Course Planning is a vertical slice inside `ustc.opportunity-graph`. Its deterministic Rust planner was completed early as a bounded offline spike. It is retained, but it does not establish Opportunity Graph as the sole flagship, change this order or prove Market installation/runtime integration.

## 4. Product contracts

- **Affairs Navigator**: reviewed tree/procedure artifacts first; exact/structured lookup before targeted refresh; bounded retrieval over approved snapshots is a later fallback and never the initial truth path.
- **ChangeRadar**: individually approved public sources, immutable snapshots, normalized semantic diffs, board-scoped candidate maintainers, administrator publication and per-board RSS/Atom.
- **Opportunity Graph**: reviewed opportunities plus a consent-aware, tenant-isolated profile projection; Course Planning is one domain slice rather than the whole product.

## 5. Competition MVP non-goals

- no automatic enrollment or course-selection clicking;
- no storage of raw USTC passwords or CAS sessions as product credentials;
- no arbitrary third-party hosted code execution;
- no generic graph database or universal workflow engine merely for symmetry;
- no full-corpus RAG as the first Affairs truth path;
- no cross-user profile data or silent permission expansion;
- no new tag, GitHub Release, Pages site, download bundle or public runtime claim before its exact artifact/read-back and release gates pass; existing GitHub repository visibility and the MIT source license are not runtime/release evidence;
- no invented Chinese product brand or slice display name.

## 6. Current naming

- Repository slug: `ustc-campus-agent`
- Product name: `USTC Campus Agent`
- Chinese name: TBD
- Chinese descriptor: 面向科大学生的插件化校园智能体
- Approved slice display names: none

The word `Campus` intentionally constrains the product to campus information, opportunity planning, workflows and plugin-governed services.

## 7. Verification

- Exact first-party IDs/versions/status/capabilities/install policy: `FP-006`.
- Rust/catalog identity match: `FP-015`.
- Independent disable/re-enable: `FP-007` (planned).
- Current task order: `docs/tasks/01-execution-roadmap.md`.

## 8. Competition delivery posture

### 8.1 Delivery path

The competition demo follows a narrow vertical slice, not a horizontal platform rollout:

```text
administrator-reviewed immutable snapshot or exact approved public source
→ SourceRevision / evidence
→ one typed product artifact
→ one application query
→ one thin Web result with evidence / freshness / conflict / uncertainty
```

The first graphical product result does not wait for full M10/M80/Android/CLI/MCP/SSE/version-skew/Compose closure. A narrow first-party Web adapter over one public application query is admitted: the Web shell renders typed server-owned state and captures intent only. It performs no canonical product calculation or mutation.

First-party product packs may be statically linked or declarative during the competition slice.

### 8.2 Deferred behind activation triggers

| Deferred item | Activation trigger |
|---|---|
| Full Market storefront / runtime coupling | second independently reviewed package or first third-party plugin admission |
| Durable artifact switching / update / rollback | first production installation or first package version bump |
| Arbitrary hosted executable Plugins | separate security review after the first static-link product proof |
| Generic Harness / TaskGraph expansion | second product whose workflow cannot reuse the finite `HarnessRun` |
| Broad client-core extraction | second non-Web adapter (CLI or MCP) reaching the same application query |
| Inbound MCP productization | first external Agent consumer with an admitted operation allowlist |
| Full client / version-skew matrix | first supported-version bump after an accepted release |

### 8.3 Rule-of-two

Rule-of-two applies only to generic/reusable abstractions: a second independently motivated consumer is required before extracting a shared abstraction. It does not defer authority/security invariants — source revision/provenance, authorization recheck, intent/receipt, idempotency, conflict/uncertainty or irreversible-effect boundaries — merely because only one consumer exists.

### 8.4 Course Planning status

Course Planning now has a bounded deterministic Rust planner plus consent-bound profile and loopback-Web composition over reviewed synthetic fixtures. It remains non-production and must not be presented as live SIS data, automatic registration or a substitute for the first live-source Affairs slice.
