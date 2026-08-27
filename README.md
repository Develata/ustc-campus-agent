# USTC Campus Agent

> 面向科大学生、比赛后仍长期维护使用的插件化校园智能体。项目源于学生竞赛，**不是中国科学技术大学官方服务**。

USTC Campus Agent 的首版目标不是做一个通用聊天机器人，而是做一个有清晰边界的校园 Agent 平台：用户从 Market 安装可信插件，Agent 在来源、权限、版本和审计约束下完成校园任务。

平台有三个正式的 default first-party Plugins：**USTC Affairs Navigator**、**USTC ChangeRadar** 与 **Campus Opportunity Graph**。三者共享 Campus Trust Kernel，但保持独立的 package identity、版本、安装与启停边界。

平台主线已建立 framework-neutral Agent runtime kernel、typed invocation resolver 与 `agent-tool-protocol/v0` 的 executable evidence，并固定 `Agent ↔ ToolGateway ↔ PluginExecutor` 为唯一扩展边界。当前另有一个可直接运行的 **Affairs Navigator** bounded vertical slice：retained source-grounded fixture → M71 typed result → M10 durable ingress → ordinary-user CLI/loopback Web。它证明一个公开办事流程 fixture 的端到端产品路径，不代表管理员审批/发布、完整 M60 自动导入、生产认证、远程部署、Market/runtime 或三插件闭环。

## 可运行的办事导航纵切

准备好 Rust stable toolchain 后，在仓库根目录执行：

```bash
./scripts/run_affairs_web_demo.sh
```

然后访问 <http://127.0.0.1:8787>。页面默认查询 `proc:ustc:undergraduate:transcript-certificate`，展示办理条件、步骤、明确的未知时间边界、入口、联系方式、safe lineage、freshness、conflict 与 uncertainty。当前 fixture 保留了 2026-08-26 获取的[中国科大教务处公开页面](https://www.teach.ustc.edu.cn/service/svc-student/13824.html)及 normalized bytes，并由测试核对两者 SHA-256；这不是管理员审批/发布，也不会替代原始页面。

该命令只绑定 loopback。运行状态默认写入 `$XDG_STATE_HOME/ustc-campus-agent/affairs-demo`（未设置时为 `~/.local/state/ustc-campus-agent/affairs-demo`）；可用 `USTC_AFFAIRS_BIND` 与 `USTC_AFFAIRS_STATE_DIR` 覆盖。当前 slice 没有生产认证、TLS、多用户隔离或自动来源更新，不应直接暴露到公网。

## Current decisions

| Item | Decision |
|---|---|
| Repository | `ustc-campus-agent`，GitHub private，Develata personal account |
| Product name | USTC Campus Agent |
| Default first-party Plugins | `ustc.affairs-navigator`, `ustc.change-radar`, `ustc.opportunity-graph` |
| Implementation order | ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph integration |
| Course Planning | Retained deterministic pack inside the active consent/profile/Market/Web Opportunity Graph composition candidate; production SSO/live-source completion is not claimed |
| Chinese name | TBD；首版使用中文描述“面向科大学生的插件化校园智能体” |
| GitHub organization | Deferred |
| Market repository | Deferred；当前为 monorepo 内 `market/` logical authority boundary |
| Future public release | Possible；public-readiness gate required before changing visibility |
| Runtime strategy | Rust authority core；ADR-0004 reference systems remain references or bounded adapters, not platform authority |
| Agent harness | finite HarnessRun over typed TaskGraph；model proposes, Rust validates；every model call passes context-budget preflight |
| Agent–Plugin boundary | PluginPackage 经 resolver/gateway 编译为 versioned tool protocol；Agent 与 Plugin 不互相依赖实现或状态机 |
| Required delivery targets | Web/PWA + Docker Compose Fullstack server + Android；Windows 为已接纳的 later peer、当前不进入 required gate；iOS/其他 desktop 后续候选 |
| Multi-client shell | `M10` owns framework-neutral versioned operation/client-protocol registry；`M80` owns client core over it；Dioxus Web/Android、`ustc-agent` 与 public-read-first inbound MCP 为 peer adapters；later Windows 复用同一 core；M10 不依赖 client-core，GUI 不 spawn CLI；client/server adapter 不拥有平台 authority |
| CLI privilege split | `ustc-agentctl` 为 operator/developer；`ustc-agent` 已有 bounded ordinary-user/headless Affairs path；`ustc-agentd serve-web` 只提供 loopback-only source-grounded fixture demo，生产 auth/remote HTTP/streaming 仍未实现；MCP 仅暴露 selected least-privilege tools/resources |

## Repository layout

```text
apps/                     # runnable binaries and future interaction-shell source
  ustc-agentd/            # daemon plus bounded source-grounded fixture CLI/Web Affairs composition
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
