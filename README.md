# USTC Campus Agent

> 面向科大学生、比赛后仍长期维护使用的插件化校园智能体。项目源于学生竞赛，**不是中国科学技术大学官方服务**。

USTC Campus Agent 的首版目标不是做一个通用聊天机器人，而是做一个有清晰边界的校园 Agent 平台：用户从 Market 安装可信插件，Agent 在来源、权限、版本和审计约束下完成校园任务。

平台有三个正式的 default first-party Plugins：**USTC Affairs Navigator**、**USTC ChangeRadar** 与 **Campus Opportunity Graph**。三者共享 Campus Trust Kernel，但保持独立的 package identity、版本、安装与启停边界。

平台主线已建立 framework-neutral Agent runtime kernel、typed invocation resolver 与 `agent-tool-protocol/v0` 的 executable evidence，并固定 `Agent ↔ ToolGateway ↔ PluginExecutor` 为唯一扩展边界。当前同一 `ustc-agentd` binary 已组成一个 bounded 三插件 MVP：loopback Web → M10 → bounded Agent/Harness → transaction-current Market authorization → ToolGateway → owning Plugin → source/profile-grounded result → typed projection → Web。它使用显著标记的 DemoReviewed source snapshots 与 synthetic private profile，不代表 USTC 官方服务、实时来源、正式 SSO、自动审批或自动选课。

## 一条命令运行三插件 MVP

准备好 Rust stable toolchain 后，在仓库根目录执行：

```bash
./scripts/run_three_plugin_mvp.sh
```

然后访问 <http://127.0.0.1:8787>。同一页面提供三条展示旅程：

- **Affairs Navigator**：查询 `proc:ustc:undergraduate:transcript-certificate`，展示办理条件、步骤、入口、联系信息、provenance、freshness、conflict 与 uncertainty；
- **ChangeRadar**：读取同一 source identity 的两个 immutable DemoReviewed revisions，展示确定性的 semantic changes，并提供 Atom feed；
- **Opportunity Graph**：明确 consent 后创建 tenant-private synthetic profile，生成 source-grounded 课程计划，随后 revoke/delete；删除后只保留不含 completed courses 或 preference weights 的 typed tombstone。

命令只接受 loopback bind。运行状态默认写入 `$XDG_STATE_HOME/ustc-campus-agent/three-plugin-mvp`（未设置时为 `~/.local/state/ustc-campus-agent/three-plugin-mvp`）；可用 `USTC_AGENTD_BIND` 与 `USTC_AGENTD_STATE_DIR` 覆盖。state directory 必须是当前用户拥有、模式 `0700` 的真实目录；launcher 不会静默 chmod/修复不安全路径。停止并以同一命令、同一 state directory 重启，会读回 `affairs-records.json`、`affairs-idempotency.json`、`m00-sessions.json` 与 `opportunity-profiles.json` 中允许持久化的 Affairs records/idempotency、DemoReviewed 当前 session history 以及 Opportunity active profile/tombstone；Market activation 和 ChangeRadar source baseline 由同一 checkout 中的 reviewed declarative fixtures 重建并重新校验。`m00-sessions.json` 是模式 `0600`、event-history-only 的 B4a read authority，不是正式 SSO 或 durable session lifecycle mutation。

当前 fixture 保留了 2026-08-26 获取的[中国科大教务处公开页面](https://www.teach.ustc.edu.cn/service/svc-student/13824.html)及 normalized bytes，并由 checker/test 核对 SHA-256；ChangeRadar 与 Opportunity source 也显著标记为 DemoReviewed/synthetic。这些 fixture 不会替代原始官方页面。服务没有生产认证、TLS、正式多用户 SSO 或自动来源更新，禁止直接暴露到公网。

只需 Affairs Navigator 的兼容性 demo 入口仍是 `./scripts/run_affairs_web_demo.sh`。

## Current decisions

| Item | Decision |
|---|---|
| Repository | `ustc-campus-agent`，GitHub private，Develata personal account |
| Product name | USTC Campus Agent |
| Default first-party Plugins | `ustc.affairs-navigator`, `ustc.change-radar`, `ustc.opportunity-graph` |
| Implementation order | ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph composition → three-plugin reproducible E2E |
| Course Planning | Retained deterministic pack inside the active consent/profile/Market/Web Opportunity Graph composition; production SSO/live-source completion is not claimed |
| Chinese name | TBD；首版使用中文描述“面向科大学生的插件化校园智能体” |
| GitHub organization | Deferred |
| Market repository | Deferred；当前为 monorepo 内 `market/` logical authority boundary |
| Future public release | Possible；public-readiness gate required before changing visibility |
| Runtime strategy | Rust authority core；ADR-0004 reference systems remain references or bounded adapters, not platform authority |
| Agent harness | finite HarnessRun over typed TaskGraph；model proposes, Rust validates；every model call passes context-budget preflight |
| Agent–Plugin boundary | PluginPackage 经 resolver/gateway 编译为 versioned tool protocol；Agent 与 Plugin 不互相依赖实现或状态机 |
| Required delivery targets | Web/PWA + Docker Compose Fullstack server + Android；Windows 为已接纳的 later peer、当前不进入 required gate；iOS/其他 desktop 后续候选 |
| Multi-client shell | `M10` owns framework-neutral versioned operation/client-protocol registry；`M80` owns client core over it；Dioxus Web/Android、`ustc-agent` 与 public-read-first inbound MCP 为 peer adapters；later Windows 复用同一 core；M10 不依赖 client-core，GUI 不 spawn CLI；client/server adapter 不拥有平台 authority |
| CLI privilege split | `ustc-agentctl` 为 operator/developer；`ustc-agent` 已有 bounded ordinary-user/headless Affairs path；`ustc-agentd serve-web` 提供 loopback-only、三插件 source/profile-grounded MVP，生产 auth/remote HTTP/streaming 仍未实现；MCP 仅暴露 selected least-privilege tools/resources |

## Repository layout

```text
apps/                     # runnable binaries and future interaction-shell source
  ustc-agentd/            # daemon plus bounded three-plugin loopback Web composition
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
  change-radar/            # bounded source-revision semantic diff and baseline/candidate core
market/                   # plugin catalog authority boundary inside this repo
plugins/                  # three first-party plugin implementation/doc boundaries
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
- Typed public/package/data contracts: [`docs/contracts/`](docs/contracts/)
- Cross-module boundary registry: [`docs/contracts/module-boundaries.md`](docs/contracts/module-boundaries.md)
- Acceptance matrix and gates: [`docs/acceptance/`](docs/acceptance/)
- Cross-layer architecture map: [`docs/overview/architecture.md`](docs/overview/architecture.md)
- Module work/commit/assembly policy: [`docs/tasks/00-module-work-policy.md`](docs/tasks/00-module-work-policy.md)
- Module assembly roadmap: [`docs/tasks/01-execution-roadmap.md`](docs/tasks/01-execution-roadmap.md)
- Collaboration, development and publication handoffs: [`docs/guides/`](docs/guides/)
- Architecture decision history: [`docs/adr/`](docs/adr/)

## Security and credentials

Do not commit USTC credentials, CAS cookies, API keys, real student data, generated logs containing private payloads, or source snapshots that contain personal information. `catalog.ustc.edu.cn` data access must use approved read-only snapshot/import paths or future official authorization. iCourse review content remains link-out-only unless explicit permission is obtained.

## License

This private competition repository currently grants no public open-source license. See [`LICENSE.md`](LICENSE.md) and [`docs/acceptance/public-readiness.md`](docs/acceptance/public-readiness.md) before any public visibility change.
