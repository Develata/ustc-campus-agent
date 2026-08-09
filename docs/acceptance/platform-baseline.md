# Long-horizon platform acceptance catalog

## Metadata

- `Layer`: `Acceptance / Coverage / Evidence`
- `Status`: **Planning baseline; only `matrix.tsv` is the active competition gate registry**
- `Version`: `0.6.0`
- `Last Review`: `2026-07-26`
- `Authority Owns`: stable long-horizon case IDs, planned assertions and evidence-binding classes
- `Authority Defers To`: `docs/plan/`, `docs/contracts/` and `docs/acceptance/matrix.tsv` for current scope/status

## 1. Current evidence state

This document preserves the accepted long-horizon verification surface while keeping the active competition matrix small and honest. It is a catalog of planned contracts, not a completion report and not evidence that every domain is in current MVP scope.

```text
Catalog cases: defined below
Active/current cases: docs/acceptance/matrix.tsv
Pass authority: exact current binding + reviewed evidence only
Release readiness: false
```

A catalog case becomes an active required case only when its owning feature enters scope and is projected into `matrix.tsv`. `NotRun`, `Unavailable`, `Skipped`, `planned` and `deferred` are all non-pass states. Cases are retained rather than silently deleted when work is de-scoped.

## 2. Binding types

| Binding | Meaning |
| --- | --- |
| `rust-unit` | pure/domain Rust unit or property test |
| `rust-integration` | isolated Rust integration test with real dependency adapter |
| `rust-doc-test` | compile-fail or executable Rust doctest run by `cargo test --doc` |
| `rust-cli-smoke` | exact `ustc-agentctl` command path |
| `rust-cli-real-host` | CLI against explicit test namespace on target host |
| `browser-automation` | real browser journey with screenshot/console/network evidence |
| `manual-security` | named security review with evidence file and reviewer |
| `manual-doc` | source/contract/governance review |
| `external-conformance` | external SDK/client/protocol/remote artifact evidence |

The active projection is [`matrix.tsv`](matrix.tsv), which records current competition case IDs, exact bindings, gates, status and owner:

```text
case_id\tdomain\tassertion\tbinding\tgate\tstatus\towner
```

A manual or automated case cannot pass without a named owner, exact evidence and current source/config/target identity. Missing bindings or evidence remain non-pass; never invent evidence to make the catalog look complete.

## 3. Contract coverage map

| Contract domain | Current authority | Case suite | Primary verification |
| --- | --- | --- | --- |
| engineering/docs projection | [`../plan/00-engineering-constitution.md`](../plan/00-engineering-constitution.md), [`../coverage-matrix.md`](../coverage-matrix.md) | `DOC-*` | repository checker + manual doc review |
| typed config and CLI smoke | [`../contracts/cli.md`](../contracts/cli.md), [`../plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md) | `CFG-*` | `ustc-agentctl` smoke + typed loader tests |
| catalog authority and Plugin lifecycle | [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md), [`../contracts/agent-plugin-boundary.md`](../contracts/agent-plugin-boundary.md) | `CAT-*`, `PKG-*` | catalog validator + domain/integration tests |
| identity/session/RBAC and capability isolation | [`../plan/03-platform-authority.md`](../plan/03-platform-authority.md), [`../plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md) | `AUTH-*`, `SEC-*` | policy/integration/browser/security evidence |
| community component validation | [`../plan/04-market-and-plugin-lifecycle.md`](../plan/04-market-and-plugin-lifecycle.md) | `SKILL-*` | deterministic validator + security review |
| finite Agent harness, Agent–Plugin tool protocol, MCP binding, hosted runtime, node AgentRun state and model providers | [`../plan/07-runtime-and-integration.md`](../plan/07-runtime-and-integration.md), [`../contracts/agent-plugin-boundary.md`](../contracts/agent-plugin-boundary.md) | `HARNESS-*`, `MCP-*`, `RUN-*`, `AGENT-*`, `AI-*` | owned state-machine/context/graph tests + protocol/real-host conformance |
| Market Web, i18n and clients | [`../features/00-market-browse-install.md`](../features/00-market-browse-install.md), [`../guides/github-pages-brief.md`](../guides/github-pages-brief.md) | `WEB-*`, `I18N-*`, `CLIENT-*` | browser/client conformance evidence |
| Campus Trust Kernel, procedures, graph and evaluation | [`../plan/05-campus-trust-kernel.md`](../plan/05-campus-trust-kernel.md), [`../plan/06-first-party-plugins.md`](../plan/06-first-party-plugins.md) | `SRC-*`, `PROC-*`, `GRAPH-*`, `EVAL-*`, `FP-*` | source/procedure/graph invariants + evaluated journeys |
| reliability, deployment and restore | [`../plan/08-security-and-delivery.md`](../plan/08-security-and-delivery.md) | `REL-*`, `DEP-*` | doctor/preflight/restore/remote read-back gates |

Long-horizon suites remain planned until projected into `matrix.tsv`. New current plan chapters or product-visible features require a current coverage row and an active acceptance binding; catalog-only presence is insufficient.

### 3.1 ADR traceability baseline

| ADR | Projection | Long-horizon suites |
| --- | --- | --- |
| [`ADR-0001`](../adr/0001-canonical-repository.md) | one canonical collaboration/release surface | `DOC-*`, `CAT-*`, `DEP-*` |
| [`ADR-0002`](../adr/0002-market-boundary-in-monorepo.md) | logical Catalog Authority inside monorepo | `CAT-*`, `PKG-*` |
| [`ADR-0003`](../adr/0003-campus-opportunity-graph.md) | Course Planning inside Opportunity Graph | `GRAPH-*`, `EVAL-*`, `FP-*` |
| [`ADR-0004`](../adr/0004-runtime-reference-strategy.md) | owned Rust runtime semantics; bounded adapters | `MCP-*`, `RUN-*`, `AGENT-*`, `AI-*` |
| [`ADR-0005`](../adr/0005-public-transition-deferred.md) | publication only after explicit gate | `WEB-*`, `SEC-*`, `REL-*`, `DEP-*` |
| [`ADR-0006`](../adr/0006-three-default-first-party-plugins.md) | three equal first-party Plugins over one kernel | `SRC-*`, `PROC-*`, `GRAPH-*`, `EVAL-*`, `FP-*` |
| [`ADR-0007`](../adr/0007-finite-agent-harness.md) | finite HarnessRun, typed TaskGraph, bounded review and pre-send context budget | `HARNESS-*`, `AGENT-*`, `AI-*` |
| [`ADR-0008`](../adr/0008-agent-plugin-tool-boundary.md) | Agent and Plugin evolve independently through a versioned ToolGateway protocol | `PKG-*`, `MCP-*`, `AGENT-*`, `HARNESS-*` |
| [`ADR-0009`](../adr/0009-dioxus-multi-client-shell.md) | one Dioxus Fullstack source serves required Web/PWA and Android through admitted typed ingress; Compose hosts the server; iOS/desktop are later | `WEB-*`, `I18N-*`, `CLIENT-*`, `DEP-*` |

[`../coverage-matrix.md`](../coverage-matrix.md) owns current plan/feature/contract/acceptance projection. This catalog preserves candidate and long-horizon cases without creating a second current authority map.

## 4. Documentation and governance — `DOC-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `DOC-001` | every authority/plan chapter has a coverage row | rust-cli-smoke | PR |
| `DOC-002` | every acceptance case exists in Rust case registry | rust-cli-smoke | PR |
| `DOC-003` | every manual case has owner/evidence binding | rust-cli-smoke | PR |
| `DOC-004` | every public CLI command/config key matches generated registry | rust-cli-smoke | PR |
| `DOC-005` | Markdown links/fences/privacy scan pass | rust-cli-smoke | PR |
| `DOC-006` | ADR/report/task docs cannot override current plan authority | manual-doc | release |
| `DOC-007` | current decision IDs map to plan/case/evidence or explicit deferral | rust-cli-smoke | PR |
| `DOC-008` | no required case is silently skipped or counted as pass | rust-unit | PR |

Primary command：

```bash
ustc-agentctl acceptance matrix-check --strict --format json
```

## 5. Configuration smoke — `CFG-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `CFG-001` | minimal valid config parses deterministically | rust-cli-smoke | PR |
| `CFG-002` | unknown key fails closed | rust-unit | PR |
| `CFG-003` | wrong type/range fails closed | rust-unit | PR |
| `CFG-004` | missing required key/reference fails closed | rust-unit | PR |
| `CFG-005` | profile merge precedence is deterministic and evidenced | rust-unit | PR |
| `CFG-006` | duplicate/conflicting declarations fail closed | rust-unit | PR |
| `CFG-007` | literal secret in config is rejected | rust-unit | PR |
| `CFG-008` | effective config output redacts all secret values | rust-cli-smoke | PR |
| `CFG-009` | unsafe URL/path/listen surface fails closed | rust-unit | PR |
| `CFG-010` | production profile rejects DevelopmentIdentityProvider | rust-cli-smoke | PR |
| `CFG-011` | Rust schema and config-key registry have zero drift | rust-cli-smoke | PR |
| `CFG-012` | server/worker/CLI use same checked loader | rust-integration | PR |
| `CFG-013` | missing secret/env reference reports identity, not value | rust-cli-smoke | integration |
| `CFG-014` | unsafe owner/mode/path is rejected | rust-cli-smoke | integration |
| `CFG-015` | missing exact catalog/schema/artifact revision fails | rust-cli-smoke | integration |
| `CFG-016` | PostgreSQL live probe is read-only and migration-free | rust-cli-real-host | demo |
| `CFG-017` | catalog/artifact probes resolve exact revision/digest | rust-cli-real-host | demo |
| `CFG-018` | IdP validation profile is protocol-complete | rust-cli-real-host | demo |
| `CFG-019` | optional Redis loss does not change durable truth | rust-integration | demo |
| `CFG-020` | static/resolved/live-readonly smoke leaves durable state unchanged | rust-integration | release |

## 6. Public catalog authority — `CAT-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `CAT-001` | valid reviewed Git revision imports deterministically | rust-integration | integration |
| `CAT-002` | malformed/schema-incompatible manifest is rejected atomically | rust-integration | PR |
| `CAT-003` | public manifest cannot contain secrets/private endpoints | rust-cli-smoke | PR |
| `CAT-004` | manual PostgreSQL catalog drift is overwritten/quarantined, never published | rust-integration | integration |
| `CAT-005` | Git catalog revoke blocks new invocation/runtime start | rust-integration | demo |
| `CAT-006` | PostgreSQL emergency block can deny but never override Git revoke | rust-integration | integration |
| `CAT-007` | private PluginPackage never appears on anonymous/public catalog | rust-integration | integration |
| `CAT-008` | artifact store presence does not imply reviewed/allowed state | rust-unit | PR |
| `CAT-009` | importer records exact Git revision/schema version | rust-integration | integration |
| `CAT-010` | GitHub ownership/ruleset/review/status-check substrate is verified | manual-security | release |

## 7. PluginPackage lifecycle — `PKG-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `PKG-001` | PluginPackage resolves exact component graph/version/digest | rust-unit | PR |
| `PKG-002` | pure MCP/Skill is wrapped as single-component PluginPackage | rust-unit | PR |
| `PKG-003` | private remote/upload creates PrivatePluginPackage + installation | rust-integration | integration |
| `PKG-004` | McpBinding cannot invoke without enabled owning installation | rust-integration | integration |
| `PKG-005` | install pins exact package/components/execution identities | rust-integration | integration |
| `PKG-006` | disable immediately blocks new Agent/tool invocation | rust-integration | demo |
| `PKG-007` | re-enable restores only still-valid grants/version | rust-integration | demo |
| `PKG-008` | uninstall/revoke retires component bindings and secret refs safely | rust-integration | integration |
| `PKG-009` | new account gets exact default FirstPartySystemPlugin versions | rust-integration | demo |
| `PKG-010` | user can disable default first-party Plugin | browser-automation | demo |
| `PKG-011` | verified patch without permission expansion can canary | rust-integration | demo |
| `PKG-012` | minor/major or permission/risk expansion requires reapproval | rust-integration | integration |
| `PKG-013` | canary changes exact cohort version, never random-routes pinned old digest | rust-integration | release |
| `PKG-014` | rollback restores exact prior version and audit resolution | rust-integration | release |
| `PKG-015` | package config schema and secret refs are checked before enable | rust-integration | integration |
| `PKG-016` | shared service binding remains tenant-scoped and installation-owned | rust-integration | integration |
| `PKG-017` | DeclarativeResourcePack resolves exact ID/version/digest/provenance | rust-unit | PR |
| `PKG-018` | declarative resource cannot register executable/grant authority; duplicate/conflicting IDs fail closed | rust-integration | integration |
| `PKG-019` | admitted executable components compile into namespaced tool definitions and private routes without Agent linkage | rust-integration | PR |
| `PKG-020` | package update disable or revoke changes only new projections while in-flight Agent toolsets remain pinned | rust-integration | integration |

## 8. Identity, session and RBAC — `AUTH-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `AUTH-001` | anonymous visitor can browse but cannot install/test/deploy | rust-integration | integration |
| `AUTH-002` | USTC provider validates issuer/audience/client/redirect/protocol controls | external-conformance | demo |
| `AUTH-003` | state/nonce/PKCE or CAS replay controls fail closed as applicable | external-conformance | demo |
| `AUTH-004` | role mapping is deny-by-default; Publisher cannot self-grant Reviewer/Admin | rust-integration | integration |
| `AUTH-005` | break-glass subject namespace cannot merge with normal USTC subject | rust-integration | release |
| `AUTH-006` | break-glass has isolated entry, rate limit, audit and rotation evidence | manual-security | release |
| `AUTH-007` | development identity provider fails closed in production | rust-cli-smoke | PR |
| `AUTH-008` | session idle/absolute expiry and logout invalidation work | rust-integration | demo |
| `AUTH-009` | CSRF/cookie/origin protections hold on Market mutation routes | manual-security | release |
| `AUTH-010` | platform never stores or logs raw USTC password/token | manual-security | release |
| `AUTH-011` | each of the six M00 ID kinds enforces the exact bounded grammar deterministic error precedence and validating Serde path through that kind's inherent parse constructor | rust-unit | PR |
| `AUTH-012` | M00 identity values are byte exact in string Serde ordering and hashing while compile-fail API checks reject private-field construction Default unchecked construction mutable backing access cross-kind conversion and identifier-shape parsing | rust-unit + rust-doc-test | PR |
| `AUTH-013` | the request-context batch defines a tenant-scoped actor key that exposes exactly tenant plus user remains distinct across tenants for equal user text and rejects incomplete or unknown-field Serde input | rust-integration | PR |
| `AUTH-014` | identity construction errors expose only the value kind failure class fixed bound and byte index while Display and Debug contain no complete rejected input input-derived fragment or offending-byte rendering | rust-unit | PR |
| `AUTH-015` | the M00 identity module mints no identifier and declares no clock random transport database framework or auth-adapter dependency | rust-unit | PR |
| `AUTH-016` | Market invocation authority consumes the M00 owned tenant and user types so no duplicate public tenant or user identity remains in platform-core and invocation policy snapshot identity stays M20 owned | rust-unit | PR |
| `AUTH-017` | session open pins immutable tenant user session adapter evidence and resolved policy scope while checked policy credential and idle deadline algebra rejects stale evidence invalid time and overflow | rust-unit | PR |
| `AUTH-018` | refresh extends only idle expiry while equality expires credential absolute and idle cause ties have deterministic precedence late observation preserves effective expiry revoke blocks immediately and terminal sessions cannot mutate or resurrect | rust-unit | PR |
| `AUTH-019` | session expected revisions and ordered events fail closed on exhaustion and reject gap duplicate reorder wrapped-zero cross-session and forged-derived-field input while replay reconstructs an equal snapshot without adapter I/O | rust-unit | PR |
| `AUTH-020` | session-domain commands events snapshots and B2-owned errors retain no raw credential or arbitrary adapter payload expose no mutable unchecked or raw-credential conversion authority fields never represent structural evidence decoding as authentication and declare no clock RNG transport database framework or auth-adapter dependency while deserializer-generated syntax and type diagnostics are excluded as untrusted boundary diagnostics owned by the later control-evidence redaction boundary | rust-unit + rust-doc-test | PR |

## 9. Capability registry and ControlledCLI — `SEC-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `SEC-001` | unknown capability/risk/data class fails closed | rust-unit | PR |
| `SEC-002` | default manifest accepts only AutoGrantEligible exact read capabilities | rust-unit | PR |
| `SEC-003` | broad profile/memory/cross-user/internal diagnostic reads cannot auto-grant | rust-unit | PR |
| `SEC-004` | capability/risk/data-class change is permission expansion | rust-unit | PR |
| `SEC-005` | ControlledCLI rejects arbitrary command/path/URL/object escape | rust-integration | integration |
| `SEC-006` | worker runs outside public authority process with scrubbed environment | rust-cli-real-host | demo |
| `SEC-007` | worker has no DB admin/master key/broad secret/Docker socket | manual-security | release |
| `SEC-008` | egress/fs profile is deny-by-default and subcommand-scoped | rust-cli-real-host | demo |
| `SEC-009` | mutation rechecks user/tenant/capability/object ownership | rust-integration | integration |
| `SEC-010` | preview/apply binds plan hash, confirmation and idempotency key | rust-integration | integration |
| `SEC-011` | dry-run creates no durable baseline/cache/lock/state | rust-integration | release |
| `SEC-012` | structured errors/output are bounded and secret-redacted | rust-integration | PR |

## 10. Community Skill validation — `SKILL-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `SKILL-001` | strict Skill manifest/schema rejects unknown or ambiguous declarations | rust-unit | PR |
| `SKILL-002` | text-only policy rejects binary, executable script and forbidden file type | rust-unit | PR |
| `SKILL-003` | archive traversal, absolute path, symlink and duplicate-normalized path fail closed | rust-unit | PR |
| `SKILL-004` | size, file-count, depth and reference expansion fuses are enforced | rust-unit | PR |
| `SKILL-005` | Unicode control/bidi/zero-width/hidden-content hazards are detected and reviewed | rust-unit | PR |
| `SKILL-006` | declared tools/capabilities exactly cover referenced tools; undeclared use fails | rust-integration | integration |
| `SKILL-007` | external URLs, source, license and provenance are explicit and validated | rust-integration | integration |
| `SKILL-008` | prompt-injection, secret-exfiltration and authority-escalation review evidence exists | manual-security | release |

## 11. MCP binding and gateway — `MCP-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `MCP-001` | released MCP version initialization succeeds | external-conformance | demo |
| `MCP-002` | UserRemote supports approved Streamable HTTP only | rust-integration | integration |
| `MCP-003` | endpoint validation rejects loopback/private/link-local/metadata | rust-integration | integration |
| `MCP-004` | redirects/auth metadata repeat the same SSRF validation | rust-integration | release |
| `MCP-005` | connection test performs discovery only, no business tool call | rust-integration | integration |
| `MCP-006` | paginated tools/list normalizes and hashes schemas | rust-integration | integration |
| `MCP-007` | new/changed tool or schema blocks old grant and requires reapproval | rust-integration | integration |
| `MCP-008` | typed gateway rejects unknown/ungranted tool before outbound call | rust-integration | PR |
| `MCP-009` | arguments validate against exact approved schema | rust-integration | PR |
| `MCP-010` | installation/component/execution/grant resolution is exact | rust-integration | release |
| `MCP-011` | session is isolated by installation/component and reauthorizes each call | rust-integration | integration |
| `MCP-012` | one user's credential/session cannot be used by another | rust-integration | release |
| `MCP-013` | USTC login token is never passed to remote MCP | manual-security | release |
| `MCP-014` | Write/Destructive/Unknown requires required confirmation policy | rust-integration | demo |
| `MCP-015` | scheduled task blocks interactive-only or changed grants | rust-integration | integration |
| `MCP-016` | untrusted tool output is bounded, labeled and instruction-isolated | rust-integration | release |
| `MCP-017` | no silent fallback to same-name alternative MCP | rust-integration | integration |
| `MCP-018` | audit records complete resolved execution identity without secret payload | rust-integration | release |

## 12. Hosted runtime — `RUN-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `RUN-001` | only admitted exact OCI digest can start | rust-cli-real-host | demo-hosted |
| `RUN-002` | concurrent first requests cause one cold start | rust-cli-real-host | demo-hosted |
| `RUN-003` | bounded queue/readiness/timeout/backoff behave deterministically | rust-cli-real-host | demo-hosted |
| `RUN-004` | idle scale-down drains in-flight invocation before stop | rust-cli-real-host | demo-hosted |
| `RUN-005` | user A cannot access user B deployment/session/volume/secret | rust-cli-real-host | release |
| `RUN-006` | workload is non-root/read-only/cap-drop/no host mount/device/socket | manual-security | release |
| `RUN-007` | workload cannot reach DB/secret master/metadata/runtime admin API | rust-cli-real-host | release |
| `RUN-008` | runtime controller accepts only typed digest-pinned specs | rust-integration | integration |
| `RUN-009` | public API has no Docker/orchestrator admin capability | manual-security | release |
| `RUN-010` | runtime resource/egress profiles and quota are enforced | rust-cli-real-host | demo-hosted |
| `RUN-011` | revoke/emergency block prevents new session and drains old replica | rust-cli-real-host | release |
| `RUN-012` | market listing alone does not grant SharedSafe/Warm | rust-unit | PR |

## 13. Agent run state and orchestration — `AGENT-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `AGENT-001` | owned run state machine permits only legal, evidenced transitions | rust-unit | PR |
| `AGENT-002` | immutable run spec pins user/provider/MCP/grant/schema versions | rust-unit | PR |
| `AGENT-003` | tool side effect persists intent and receipt before advancing state | rust-integration | integration |
| `AGENT-004` | crash/resume cannot duplicate a committed tool side effect | rust-integration | release |
| `AGENT-005` | queued versus in-flight cancellation has explicit deterministic semantics | rust-integration | integration |
| `AGENT-006` | token/tool/time/retry budgets persist across resume and fail closed | rust-integration | integration |
| `AGENT-007` | streaming order, backpressure, timeout and cancellation remain coherent | rust-integration | integration |
| `AGENT-008` | provider/tool failures are typed; no silent model/tool/runtime fallback | rust-unit | PR |
| `AGENT-009` | policy and grant checks execute before every tool side effect | rust-integration | integration |
| `AGENT-010` | dynamic MCP tools use the exact approved schema snapshot in the run spec | rust-integration | integration |
| `AGENT-011` | prompt/tool payload telemetry is off by default and all diagnostics redact secrets | rust-integration | release |
| `AGENT-012` | Rig/rmcp remain replaceable behind owned ports without changing run semantics | external-conformance | release |
| `AGENT-013` | Observer/Transformer/Gate/Registry event semantics and order are typed and deterministic | rust-unit | PR |
| `AGENT-014` | every transformed tool/procedure input is re-schema-validated and re-authorized before effect | rust-integration | integration |
| `AGENT-015` | security/publish Gate error fails closed; Observer failure policy is explicit and evidenced | rust-integration | release |
| `AGENT-016` | restore validates runtime dependency IDs/versions and never auto-retries non-idempotent unfinished effects | rust-integration | release |
| `AGENT-017` | Agent runtime compilation inputs and all direct Cargo dependency classes remain confined and allowlisted, free of Market Plugin component and adapter implementations; composition owns cross-boundary proof | rust-cli-smoke | PR |
| `AGENT-018` | Agent framework/harness replacement passes the same tool-protocol fixtures without Plugin package or executor changes | external-conformance | integration |
| `AGENT-019` | frozen provider definitions bind one private route; unknown tools, malformed arguments, route mismatch, current denial and projection mismatch reach no executor; successful fake execution returns one correlated protocol result | rust-integration | PR |

### Finite Agent harness — `HARNESS-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `HARNESS-001` | every HarnessRun follows legal finite evidenced transitions and reaches an explicit terminal phase | rust-unit | PR |
| `HARNESS-002` | clarification asks only material blocking uncertainty under bounded rounds and deadline | rust-unit | PR |
| `HARNESS-003` | accepted TaskGraph is finite acyclic authority-valid and resource-compatible | rust-unit | PR |
| `HARNESS-004` | child planning cannot rewrite parent goal prohibitions deliverables or acceptance | rust-unit | PR |
| `HARNESS-005` | every complete model request including each compression-plan call satisfies the pinned context inequality before provider I/O | rust-integration | PR |
| `HARNESS-006` | compaction/compression preserves canonical history and emits provenance-bearing summary artifacts | rust-integration | PR |
| `HARNESS-007` | anchors and whole tool schemas survive rebuild; compression is finite/non-recursive and failure to fit fails closed | rust-integration | release |
| `HARNESS-008` | worker context may resume while each reviewer is fresh read-only and remediation bounded | rust-integration | integration |
| `HARNESS-009` | final rejection appends a validated remediation graph revision without restarting the session/task | rust-integration | integration |
| `HARNESS-010` | hooks or process exit cannot complete a node without structured outcome artifacts evidence and required review | rust-integration | release |

## 14. Market Web and i18n — `WEB-*`, `I18N-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `WEB-001` | `/market` anonymous browse/detail works on same HTTPS origin | browser-automation | demo |
| `WEB-002` | install redirects unauthenticated user to configured IdP | browser-automation | demo |
| `WEB-003` | install shows publisher/version/components/permissions/source/license | browser-automation | demo |
| `WEB-004` | install/disable/enable state matches Agent capability availability | browser-automation | demo |
| `WEB-005` | permission/update diff is explicit and fail-closed | browser-automation | release |
| `WEB-006` | admin/review/runtime diagnostics are absent from visitor/user surface | browser-automation | release |
| `WEB-007` | keyboard/focus/error/recovery paths satisfy accessibility contract | browser-automation | release |
| `WEB-008` | browser evidence records screenshot/console/network/viewport/locale | rust-cli-smoke | release |
| `I18N-001` | en-US/zh-CN locale key and placeholder parity | rust-cli-smoke | PR |
| `I18N-002` | no user-facing hardcoded strings outside approved fixtures | rust-cli-smoke | PR |
| `I18N-003` | Market metadata contains required locales and fallback is deterministic | rust-integration | integration |
| `I18N-004` | backend protocol uses stable error code, not localized prose contract | rust-unit | PR |
| `I18N-005` | localized Skill artifacts share logical ID/version and review independently | rust-cli-smoke | integration |
| `I18N-006` | security-sensitive prompt is not runtime machine-translated | manual-security | release |

## 15. Campus Trust Kernel — `SRC-*`, `PROC-*`, `GRAPH-*`, `EVAL-*`

### Source Registry — `SRC-*`

Current projection: only `SRC-001` is implemented, and only as a bounded `M60-B1 source-registry` review candidate (exact active binding `cargo test --locked -p ustc-campus-agent-core --test source_registry` in [`matrix.tsv`](matrix.tsv)); `M60` overall and `M60-B2` through `M60-B8` remain planned, so `SRC-010`, `SRC-011`, `SRC-012` and every catalog-only `SRC-*` row below remain non-pass.

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `SRC-001` | every source has stable identity, authority class, owner and retrieval policy | rust-unit | PR |
| `SRC-002` | crawl permission, rate policy and operator ownership have reviewed evidence | manual-security | release |
| `SRC-003` | retrieval records source revision/URL, retrieved-at and provenance chain | rust-integration | integration |
| `SRC-004` | normalized records retain source content hash and effective/retrieval time | rust-integration | integration |
| `SRC-005` | authority priority and conflict rules are explicit, deterministic and tested | rust-integration | integration |
| `SRC-006` | stale/conflicting sources yield uncertainty instead of silent overwrite | rust-integration | release |
| `SRC-007` | retry/restart cannot duplicate or prematurely advance source baseline | rust-integration | release |
| `SRC-008` | Source Registry schema/docs/code drift fails closed | rust-cli-smoke | PR |
| `SRC-009` | one normalized URL may resolve multiple immutable revisions; `--at/--digest` disambiguates deterministically | rust-integration | integration |
| `SRC-010` | approved host/path, redirect, DNS/IP and content-type/size/time policy prevents arbitrary/SSRF fetch | rust-integration | release |
| `SRC-011` | raw/normalized snapshots and digests are immutable and exact-revision-bound | rust-integration | integration |
| `SRC-012` | baseline advances only after snapshot/parse/normalize/diff/candidate/evidence durable success | rust-integration | release |
| `SRC-013` | model-proposed URL outside approved Source Registry enters review, never immediate fetch | rust-integration | integration |
| `SRC-014` | suspended/revoked source blocks new fetch while preserving historical evidence | rust-integration | release |

### Structured procedures — `PROC-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `PROC-001` | stable node/procedure/artifact IDs remain valid when navigation paths move | rust-unit | PR |
| `PROC-002` | typed artifact lifecycle rejects contradictory state/path/digest combinations | rust-unit | PR |
| `PROC-003` | Full/Partial/Clarification/Duplicate supersession uses direct edges and deterministic current view | rust-integration | integration |
| `PROC-004` | Full replacement requires authority/scope/field/effective-time/evidence coverage matrix | rust-integration | release |
| `PROC-005` | Agent output is typed candidate only; it cannot invoke canonical Git publish | rust-integration | release |
| `PROC-006` | ProcedureDraft schema/policy/citation validation and deterministic Markdown render agree across CLI/server/CI | rust-integration | demo |
| `PROC-007` | Git canonical, PostgreSQL projection and object evidence rebuild without authority inversion | rust-integration | release |
| `PROC-008` | exact/structured retrieval precedes targeted refresh; bounded RAG cannot override reviewed current procedure | rust-integration | demo |
| `PROC-009` | archive preserves procedure/source history; hard delete is narrow, audited and policy-bound | rust-integration | release |

### Campus Graph — `GRAPH-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `GRAPH-001` | node/edge/identifier schema and graph invariants validate | rust-unit | PR |
| `GRAPH-002` | qualification/prerequisite relationships preserve conditions and scope | rust-integration | integration |
| `GRAPH-003` | temporal validity and supersession produce deterministic current views | rust-integration | integration |
| `GRAPH-004` | every material graph fact retains source/provenance references | rust-integration | integration |
| `GRAPH-005` | tenant-private preferences/derived edges cannot enter public graph projection | rust-integration | release |
| `GRAPH-006` | graph schema migration/rebuild is versioned, idempotent and reversible | rust-integration | release |

### Evaluation Harness — `EVAL-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `EVAL-001` | fact correctness is scored against reviewed fixture/oracle | rust-integration | demo |
| `EVAL-002` | citation correctness and source entailment meet declared threshold | rust-integration | demo |
| `EVAL-003` | timeliness/freshness evaluation detects stale answers | rust-integration | demo |
| `EVAL-004` | qualification filtering includes/excludes the right user fixtures | rust-integration | demo |
| `EVAL-005` | missing/conflicting evidence triggers refusal or calibrated uncertainty | rust-integration | release |
| `EVAL-006` | source change produces expected diff and qualified-user impact set | rust-integration | demo |
| `EVAL-007` | evaluation fixtures and seeds are deterministic and leakage-checked | rust-unit | PR |
| `EVAL-008` | threshold, source revision, binary/config identity and result evidence are recorded | rust-cli-smoke | release |

## 16. First-party campus value — `FP-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `FP-001` | Affairs Navigator answer includes conditions/steps/source/time/uncertainty | rust-integration | demo |
| `FP-002` | ChangeRadar produces source-bound normalized diff | rust-integration | demo |
| `FP-003` | crawler write authority is operator-only, user Plugin remains read-only | rust-integration | release |
| `FP-004` | Opportunity Graph explains match/prerequisite/time conflict/next action | rust-integration | demo |
| `FP-005` | private opportunity preferences are exact tenant-scoped projection | rust-integration | release |
| `FP-006` | all three default Plugins bootstrap exact approved versions | rust-integration | demo |
| `FP-007` | user can disable/re-enable each default Plugin independently | browser-automation | demo |
| `FP-008` | every material answer carries provenance and retrieval/effective time | rust-integration | demo |
| `FP-009` | conflicting/stale sources produce explicit uncertainty, not silent merge | rust-integration | release |
| `FP-010` | source change affects only qualified users/scopes | rust-integration | release |
| `FP-011` | RSS/Atom emits only approved semantic changes with stable event GUID and provenance | rust-integration | demo |
| `FP-012` | feed subscription follows stable node ID across path/slug movement | rust-integration | integration |
| `FP-013` | board maintainer is node/source/policy scoped, lease/idempotency safe and cannot publish canonical content | rust-integration | release |
| `FP-014` | Affairs Navigator and ChangeRadar share one source/revision/change ledger and baseline | rust-integration | demo |
| `FP-015` | Rust first-party identity authority matches exact catalog IDs and versions | rust-unit | PR |

## 17. Model provider and clients — `AI-*`, `CLIENT-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `AI-001` | OfficialCentral provider works without user credential | rust-integration | demo |
| `AI-002` | UserCloud profile stores encrypted secret ref and performs one call | rust-integration | demo |
| `AI-003` | provider URL/credential validation prevents SSRF/token leakage | rust-integration | release |
| `AI-004` | provider failure is structured; no silent identity/model fallback | rust-integration | release |
| `CLIENT-001` | Dioxus Web/PWA completes login/chat/tool-status/Market launch through admitted typed server-function calls/events with the selected SSR-plus-hydration or explicit CSR path | browser-automation | demo |
| `CLIENT-002` | Dioxus Android launches on emulator and real device, uses the shared ingress/reducer contract against the HTTPS server, survives lifecycle/reconnect and opens external service in Custom Tab | external-conformance | demo |
| `CLIENT-003` | raw transcript local archive and central durable memory remain distinct | rust-integration | integration |
| `CLIENT-004` | offline/relay unavailable state is explicit, no hidden execution switch | browser-automation | release |
| `CLIENT-005` | current server remains compatible with the declared supported older Android protocol and returns typed `UpgradeRequired` before dispatch for an unsupported client | rust-integration | release |
| `CLIENT-006` | every Dioxus server function passes M00/M10 admission and one application port; dependency/conformance checks prevent repository, executor, provider SDK or journal reach-through | rust-integration | release |
| `CLIENT-007` | M10 owns one framework-neutral versioned client-protocol carrier and never depends on M80 client-core; Dioxus, `ustc-agent` and inbound MCP consume one M80 framework-neutral typed client core over that carrier; equivalent fake-M10 request/result/error/event, reconnect, cancellation and compatibility cases reduce to equivalent semantic client state | rust-integration | PR |
| `CLIENT-008` | no GUI/service production path spawns or parses a peer CLI, and Dioxus/user-CLI/inbound-MCP dependency plus credential/command registries cannot reach `ustc-agentctl`, backend domain implementations, M51, repositories or executors | rust-cli-smoke | PR |
| `CLIENT-009` | real `ustc-agent` read and stream paths preserve versioned JSON/NDJSON stdout, redacted stderr, stable exit classes, least-privilege auth, compatibility, cursor reconnect, cancellation distinction and timeout reconciliation | rust-cli-real-host | integration |
| `CLIENT-010` | inbound MCP discovery/invocation maps only reviewed bounded tools/resources through client-core and M10, binds delegated caller/tenant/grant state, instruction-isolates results and reaches no operator, domain or outbound-M51 authority | external-conformance | integration |

## 18. Reliability, deployment and recovery — `REL-*`, `DEP-*`

| Case ID | Assertion | Binding | Required gate |
| --- | --- | --- | --- |
| `REL-001` | audit/evidence write failure prevents success acknowledgement | rust-integration | release |
| `REL-002` | retryable mutations are idempotent and do not duplicate effects | rust-integration | release |
| `REL-003` | catalog projection rebuild from pinned Git revision is deterministic | rust-cli-real-host | demo |
| `REL-004` | PostgreSQL backup/restore preserves users/install/grants/audit references | rust-cli-real-host | release |
| `REL-005` | Redis flush causes cache miss/retry only, no durable loss | rust-integration | demo |
| `REL-006` | desired/observed deployment divergence is diagnosed and fail-closed | rust-integration | release |
| `REL-007` | partial first-party bootstrap rolls back or resumes deterministically | rust-integration | release |
| `REL-008` | expired/missing evidence cannot satisfy current source/config gate | rust-cli-smoke | release |
| `DEP-001` | host preflight records real OS/CPU/RAM/disk/runtime/network facts | rust-cli-real-host | demo |
| `DEP-002` | config static/resolved/live-readonly smokes pass on target | rust-cli-real-host | demo |
| `DEP-003` | doctor required scopes pass without secret disclosure | rust-cli-real-host | demo |
| `DEP-004` | only reverse proxy HTTPS surface is user-exposed | rust-cli-real-host | release |
| `DEP-005` | dev/staging/production worktrees/users/credentials remain separated | manual-security | release |
| `DEP-006` | Slurm build evidence binds exact source revision/artifact | external-conformance | release |
| `DEP-007` | clean-host restore/redeploy uses reviewed commit/artifact only | rust-cli-real-host | release |
| `DEP-008` | remote release/download surface read-back verifies checksum/version/smoke | external-conformance | release |
| `DEP-009` | self-hosted stack applies same authority/config/acceptance contracts | rust-cli-real-host | release |
| `DEP-010` | disk/cache/log/image/backup retention prevents uncontrolled exhaustion | rust-cli-real-host | release |
| `DEP-011` | clean Docker Compose deployment serves Dioxus Web assets/SSR and admitted Fullstack endpoints, accepts Android remote access, then passes restart/restore/read-back | external-conformance | release |

## 19. Long-horizon suite profiles

The catalog preserves five planning profiles:

- `pr`: deterministic checks suitable for every change;
- `integration`: `pr` plus dependency-adapter and isolated service checks;
- `demo`: integration plus complete browser/IdP/PostgreSQL/three-Plugin/Campus Trust Kernel journeys;
- `demo-hosted`: conditional hosted-private runtime feasibility profile, activated only by an explicit GO decision;
- `release`: every active case for the included release scope.

The active `matrix.tsv` uses the smaller gate vocabulary `pr|core-demo|release|public`. When a catalog case enters current scope, its owner MUST map the planning profile explicitly; no catalog profile or missing environment counts as pass. A de-scoped case retains its ID with a decision, owner, reason and review condition rather than being silently deleted.

## 20. Validator target contract

The current repository checker validates the active matrix shape, unique IDs, gate/status vocabulary, links, docs topology and implemented package/fixture contracts. A future complete acceptance registry/CLI SHOULD additionally validate:

1. catalog case ID syntax and uniqueness;
2. active matrix ↔ implementation registry bidirectional coverage;
3. current authority docs ↔ active suite coverage;
4. named binding/owner/evidence for manual cases;
5. evidence paths exist, remain repository-safe and bind exact source/config/target identity;
6. required profiles reference existing cases only;
7. public CLI/config/capability/error registries have active acceptance coverage;
8. ADRs have acceptance or explicit deferral;
9. no skipped/unavailable/not-run/planned/deferred state is counted as pass;
10. generated reports identify exact source, binary, config and target.

## 21. Evidence closure rule

Case closure requires：

```text
case contract
+ implementation binding
+ exact runner/manual evidence
+ review of evidence
+ current source/config/target identity
```

Dated reports may summarize closure but never override this matrix or the owning plan. If implementation behavior changes, stale evidence remains historical and the current case returns to NotRun until rerun.
