# USTC Campus Agent

> Competition-built, independently maintained, and **not an official University of Science and Technology of China service**.

A loopback-only campus Agent MVP in which the model proposes intent, while Rust validates tools, permissions, sources, state, and side effects before returning a human-checkable result.

模型负责理解与提议，Rust 负责边界与执行；结果附带可核对的来源或受限工具记录。

`“成绩单证明怎么办？”` → bounded ChatRun selects `affairs_navigator_get` → Rust validates the fixed reviewed path → the UI presents a concise Chinese answer plus a redacted tool trace.

The historical `addf6c7f` Android screenshot proves only connection and rendering of that user turn. A completed answer and successful trace must be rerun on the final merged source before they are marked exact-current runtime evidence.

Evidence: [historical planning baseline `addf6c7f`](https://github.com/Develata/ustc-campus-agent/commit/addf6c7f6e58f96cf01bda12c1a812b669015134) · [MVP capability contract](docs/features/06-mvp-core-capabilities.md) · [Docker runbook](deploy/mvp-compose/README.md) · [Android demo boundary](docs/guides/android-demo.md)

## Architecture at a glance

```text
Web browser ───────────────┐
                           ├─ same loopback HTTP ─→ ustc-agentd
Android thin WebView ──────┘                         │
                                                     ├─ bounded ChatRun
mock provider / server-side provider ────────────────┤  model proposes only
                                                     └─ Rust validates a fixed reviewed tool catalogue
                                                        ├─ Affairs Navigator
                                                        ├─ ChangeRadar
                                                        ├─ Opportunity Graph (per-request consent)
                                                        └─ Simple Calendar (owner-local in-process companion)
                                                             ↓
                                           reviewed/synthetic fixtures + local durable state
```

Browser, Android shell, and model own no campus-data or side-effect authority. The current demo is loopback-only; optional provider credentials remain file-backed and server-side.

## Run the bounded MVP

Source-native path, from the repository root:

```bash
./scripts/run_three_plugin_mvp.sh
# open http://127.0.0.1:8787
```

Packaged Docker path, only after assembling the Compose package described by the [runbook](deploy/mvp-compose/README.md):

```bash
# inside the assembled ustc-campus-agent-mvp-compose package
./start.sh          # macOS / Linux
# or start.cmd      # Windows + Docker Desktop
```

The assembled package, tar archive, and ZIP archive must each contain package-root `LICENSE.md` byte-identical to the repository root, mode `0644`, and covered by `SHA256SUMS`. A raw checkout does not contain the package-only binary and copied fixture payload.

## Exact-current evidence checklist

`[x]` is reserved for evidence that binds the same final source identity, an executed action, and a successful outcome. Source presence, README prose, an unexecuted test, prior CI, or historical run `33905840607` is not exact-current runtime proof. Because this documentation changes the source identity, every final-source row remains unchecked until the post-merge acceptance run succeeds.

### Final-source runtime evidence pending

- [ ] Packaged Docker start, `/healthz`, loopback bind, and browser load report the final commit/tree.
- [ ] Web Affairs returns a readable answer, successful redacted `affairs_navigator_get` trace, and reviewed-source evidence.
- [ ] Calendar exact record/list/delete succeeds, denied ambiguous or mismatched mutations perform zero operation, and restart preserves the same stable item.
- [ ] Opportunity profile consent and the separate per-request Chat confirmation produce one source-grounded plan; ChangeRadar returns one semantic change.
- [ ] Android reaches the same loopback service and completes the Affairs answer/trace on the final source.
- [ ] Both deterministic archives pass source, checksum, secret, and package-root `LICENSE.md` byte/mode checks.

### Historical evidence — not exact-current

- The screenshot labelled `addf6c7f` proves only Android connection, source-label rendering, and display of the user turn at that historical source.
- CI run `33905840607` is prior-source package/Android evidence. It neither proves the final source nor repairs its observed missing archive license.

### Planned product scope — not a completed claim

- [ ] Production authentication, USTC SSO, tenant administration, TLS, and an authenticated public runtime.
- [ ] Shared cross-target client parity, production-signed/store-ready Android, and physical-device lifecycle acceptance.
- [ ] Approved automatic campus-source ingestion and broad current campus coverage.
- [ ] Generalized third-party package lifecycle/isolation or a usable inbound MCP adapter/server.
- [ ] A command sandbox, durable chat sessions, streaming, reminders, recurrence, and calendar synchronization.

## Four-dimension evidence map

| Judging dimension | Implemented source evidence | Exact demo proof required | Claim boundary |
|---|---|---|---|
| Innovation | Bounded ChatRun, fixed Rust-owned catalogue, typed results, per-request Opportunity confirmation, and redacted `succeeded / denied / failed` trace. | Ask `成绩单证明怎么办？`, expand the trace, then show Opportunity unavailable without confirmation. | Bounded executable vertical slice; not an unrestricted agent or generalized extension runtime. |
| Usefulness | Fixed paths cover one reviewed Affairs procedure, one semantic ChangeRadar board, synthetic consent-bound planning, and owner-local Calendar record/list/delete. | Show the Affairs answer and evidence, then exact Calendar record/list with its stable ID. | No broad/live/official campus coverage, reminders, enrollment, or approval claim. |
| Technical difficulty | Thin clients submit intent while `ustc-agentd` owns admission, effects, durable state, and file-backed provider configuration; Compose is loopback/read-only with a named state volume. | Show source identity, one denied mutation with zero state change, one consent-gated plan, and restart/read-back if rehearsed. | No public-network hardening, production auth, arbitrary-code isolation, or client-held credential claim. |
| Completeness | Source run path, deterministic Compose package, browser surface, and debug Android thin shell surround one loopback MVP. | Run Docker → Web Affairs → Calendar record/list → Android same-service Affairs, then return to this checklist. | “Competition/demo vertical slice” only after those exact-source actions pass; long-horizon platform and production Android remain incomplete. |

## Five-minute demo order

1. Show Docker ready, loopback URL, and final source identity; state “model proposes, Rust owns validation and effects.”
2. In Web Chat, show the Affairs answer, successful redacted trace, and reviewed-source evidence.
3. Record `记录事项：提交开题报告`, then list it and point to the stable owner-local ID.
4. Create the synthetic Opportunity profile with explicit consent, separately confirm its use for this Chat request, and show the plan.
5. Show one ChangeRadar semantic before → after field; do not use administrator publication in the timed path.
6. Only with exact-current Android evidence, show the same loopback service and Affairs journey in the debug thin shell.
7. End on this exact-source checklist and the explicit non-claims below. A failed or not-run step is omitted or labelled; recorded fallback media is never presented as live.

## Current bounded capability

The fixed reviewed demo catalogue contains Affairs Navigator, ChangeRadar, and Opportunity Graph, plus Simple Calendar as an owner-local in-process companion. It is not a generalized install/disable/revoke-driven provider catalogue or isolated execution platform.

- **Agent Chat** returns bounded Chinese summaries and a redacted trace instead of raw transport JSON.
- **Affairs Navigator** reads one fixed `DemoReviewed` transcript-certificate procedure.
- **ChangeRadar** reads one fixed reviewed academic-calendar board; administrator publication is a separate explicit control and is not model-visible.
- **Opportunity Graph** plans from a synthetic private profile only after profile consent and separate confirmation on that request; community aggregate signals affect soft ranking only.
- **Simple Calendar** accepts exact record intent `记录事项：<nonblank title>` or `记录事项:<nonblank title>`, read-only list, and exact delete intent `删除事项 calendar:item:N`. `scheduled_for` is absent from this slice.

Calendar `record`/`delete` executes only when the final admitted user message has the matching exact grammar and the normalized title or item ID equals the provider call. Absent or mismatched intent yields a bounded denied result/trace and zero executor/store operation; provider prose cannot mint confirmation.

The command accepts loopback bind only. State defaults to `$XDG_STATE_HOME/ustc-campus-agent/three-plugin-mvp` (or `~/.local/state/ustc-campus-agent/three-plugin-mvp`) and may be moved with `USTC_AGENTD_STATE_DIR`; the real directory must be current-user-owned mode `0700`. `calendar-items.json` is a required member of the locked durable state set: fresh bootstrap writes canonical empty mode-`0600` state, non-fresh absence fails `durable_state_set_incomplete`, and rollback treats it with the other members.

`m00-sessions.json` remains the `event-history-only` current-session read authority; the `B4b stable redacted control-event/error` journal remains a `data-only` evidence carrier. Neither is formal SSO or a general administrator API.

All campus facts in this demo are explicitly labelled reviewed or synthetic fixtures. The Affairs fixture retains normalized bytes from the [USTC teaching-affairs public page captured on 2026-08-26](https://www.teach.ustc.edu.cn/service/svc-student/13824.html). Course-planning fixtures retain bounded public iCourse aggregate metadata and link-outs, not review text. They do not establish comprehensive or current official campus data.

The Android artifact is a debug-signed competition bridge over the same Web UI through `adb reverse`; it is not a production Android release. Installation and endpoint details are in the [Android demo guide](docs/guides/android-demo.md).

The current MVP has no command sandbox and rejects arbitrary shell execution. It delivers no generalized dynamic package lifecycle, Skill runtime, usable MCP adapter/server, authenticated public deployment, client-side provider credentials, live campus ingestion, or production Android acceptance.

The Affairs-only compatibility entrypoint remains `./scripts/run_affairs_web_demo.sh`. `ustc-agentctl` is a separate loopback operator/developer surface:

```bash
cargo run -p ustc-agentctl -- affairs publication-status --server 127.0.0.1:8787
cargo run -p ustc-agentctl -- affairs publish-demo --server 127.0.0.1:8787 --confirm
cargo run -p ustc-agentctl -- changes publication-status --server 127.0.0.1:8787
cargo run -p ustc-agentctl -- changes publish-demo --server 127.0.0.1:8787 --confirm
```

## Current decisions

| Item | Decision |
|---|---|
| Repository | [`Develata/ustc-campus-agent`](https://github.com/Develata/ustc-campus-agent), public source under the Develata personal account |
| Product name | USTC Campus Agent |
| Demo catalogue | Three fixed reviewed first-party paths: `ustc.affairs-navigator`, `ustc.change-radar`, `ustc.opportunity-graph` |
| Calendar companion | `ustc.simple-calendar`; optional package declaration, owner-local Rust store, fixed in-process demo composition |
| Course Planning | Deterministic synthetic fixture plus consent/profile composition; no production SSO or live-source completion claim |
| Chinese name | TBD; do not infer a Chinese brand from the descriptive copy |
| Catalog repository | Deferred; `market/` is the current logical declaration boundary inside this monorepo |
| License | Project-authored software/docs use [`MIT`](LICENSE.md); third-party and campus-data rights remain separate |
| Delivery | Public source repository only; no tag, GitHub Release, Pages app, stable download, or authenticated public runtime is claimed |
| Runtime | Rust owns admission/state/effects; reference systems remain references or bounded adapters |

## Repository layout

```text
apps/                     # runnable binaries and future interaction-shell source
  ustc-agentd/            # daemon plus bounded three-plugin loopback Web composition
  ustc-android-demo/       # debug APK thin shell over the loopback Web MVP; not final Dioxus Android
  ustc-agentctl/          # operator/developer CLI skeleton
  ustc-agent/             # bounded ordinary-user/headless Affairs CLI evidence; production transport/auth planned
  ustc-client/            # future shared Dioxus Web/Android Fullstack source
crates/
  client-protocol/        # M10-owned framework-neutral versioned wire DTO/error carrier; bounded Affairs slice exists
  client-core/            # M80-owned client behavior; bounded loopback Affairs slice exists
  platform-core/          # canonical domain invariants and authority decisions
  agent-runtime/          # Plugin-neutral node AgentRun; future finite harness state, graph, context and review kernel
  agent-tool-protocol/    # provider-neutral canonical tool values and sealed view/call/result envelopes
  adapters/               # replaceable provider/tool/executor adapters; no authority ownership
  course-planning/         # typed fixture validation and deterministic planner core
  simple-calendar/         # bounded owner-local calendar item store
  change-radar/            # bounded source-revision semantic diff and baseline/candidate core
market/                   # plugin catalog authority boundary inside this repo
plugins/                  # default first-party and optional bundled plugin boundaries
docs/                     # layered plans, features, contracts, acceptance, tasks, guides and ADRs
  plan/modules/           # 13 independent large-module blueprints and assembly map
scripts/                  # local and CI validation scripts
.github/                  # CI, PR template, issue templates, CODEOWNERS
```

## Local development

See [`docs/guides/development.md`](docs/guides/development.md) for the full local workflow, CodeGraph notes, and cleanup guidance.

Rust builds can consume disk quickly. Check disk first when working locally:

```bash
df -h / /opt/data 2>/dev/null || df -h
```

Then run:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
checker_evidence="$(mktemp -d)"
PYTHONPYCACHEPREFIX="$(mktemp -d)" python3 scripts/run_checker_shards.py \
  --jobs 4 \
  --timeout-seconds 1800 \
  --inventory scripts/checker_test_inventory.json \
  --evidence-dir "$checker_evidence"
python3 scripts/check_repo_contracts.py
```

Useful smoke commands:

```bash
cargo run --locked -p ustc-agentctl -- doctor
cargo run --locked -p ustc-agentctl -- market validate
cargo run --locked -p ustc-agentctl -- course plan \
  --fixture market/fixtures/course-planning/minimal-v0.json \
  --format json
cargo run --locked -p ustc-agentd -- --version
cargo run --locked -p ustc-agent -- --version
```

## Documentation map

- Documentation entry and authority rules: [`docs/README.md`](docs/README.md)
- Engineering blueprint: [`docs/plan/`](docs/plan/)
- Large-module map: [`docs/plan/modules/00-module-map.md`](docs/plan/modules/00-module-map.md)
- User-visible journeys: [`docs/features/`](docs/features/)
- MVP capabilities, architecture and TODO: [`docs/features/06-mvp-core-capabilities.md`](docs/features/06-mvp-core-capabilities.md)
- Android demo artifact and boundary: [`docs/features/07-android-demo-client.md`](docs/features/07-android-demo-client.md), [`docs/guides/android-demo.md`](docs/guides/android-demo.md)
- Typed public/package/data contracts: [`docs/contracts/`](docs/contracts/)
- Cross-module boundary registry: [`docs/contracts/module-boundaries.md`](docs/contracts/module-boundaries.md)
- Acceptance matrix and gates: [`docs/acceptance/`](docs/acceptance/)
- Cross-layer architecture map: [`docs/overview/architecture.md`](docs/overview/architecture.md)
- Module work/commit/assembly policy: [`docs/tasks/00-module-work-policy.md`](docs/tasks/00-module-work-policy.md)
- Module assembly roadmap: [`docs/tasks/01-execution-roadmap.md`](docs/tasks/01-execution-roadmap.md)
- Collaboration, development and publication handoffs: [`docs/guides/`](docs/guides/)
- Architecture decision history: [`docs/adr/`](docs/adr/)

## Security and credentials

Do not commit USTC credentials, CAS cookies, API keys, real student data, generated logs containing private payloads, or source snapshots that contain personal information. `catalog.ustc.edu.cn` data access must use approved read-only snapshot/import paths or future official authorization. iCourse review text remains link-out-only; the MVP stores only bounded public aggregate metadata and URLs.

## License

Project-authored software and documentation in this public repository are licensed under the [`MIT License`](LICENSE.md). The MIT License does not imply USTC endorsement, production readiness, or permission to republish third-party content or campus data; their rights and source permissions remain separate. Any tag, GitHub Release, Pages site, or stable download remains subject to [`docs/acceptance/public-readiness.md`](docs/acceptance/public-readiness.md) and the applicable release gate.
