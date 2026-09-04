# USTC Campus Agent

> 面向科大学生、比赛后仍长期维护使用的插件化校园智能体。项目源于学生竞赛，**不是中国科学技术大学官方服务**。

USTC Campus Agent 的目标不是包装一个通用聊天框，而是建立有清晰 authority boundary 的校园 Agent 平台：模型负责理解与提议，Rust 负责验证工具、权限、来源、状态和副作用。

平台保留三个 default first-party Plugins：**USTC Affairs Navigator**、**USTC ChangeRadar** 与 **Campus Opportunity Graph**；另提供一个非默认安装的 Rust **Simple Calendar** companion plugin。当前 `ustc-agentd` 已把普通问答、bounded tool loop、三条校园数据路径与本地事项记录组合成可运行的 loopback MVP。详细能力、架构、数据边界与 TODO 见 [`docs/features/06-mvp-core-capabilities.md`](docs/features/06-mvp-core-capabilities.md)。

Android 8.0+ 现有一个可安装、debug-signed 的薄客户端 APK：它通过 `adb reverse` 复用同一 loopback Rust 服务与完整 Web MVP，不在手机上复制 Agent 或校园数据 authority。安装命令、endpoint 约束与尚未完成的 production/Dioxus 边界见 [`docs/guides/android-demo.md`](docs/guides/android-demo.md)。

## 一条命令运行 MVP

准备好 Rust stable toolchain 后，在仓库根目录执行：

```bash
./scripts/run_three_plugin_mvp.sh
```

然后访问 <http://127.0.0.1:8787>。同一页面提供五类最小闭环：

- **Agent Chat**：正常问答；按需调用流程、变更、课程规划与 Calendar tools；默认离线模式把 typed results 整理为有界中文摘要，而不是向用户倾倒 transport JSON，并返回 redacted tool trace；
- **Affairs Navigator**：查询固定的 `DemoReviewed` 成绩单证明流程；
- **ChangeRadar**：读取固定的校历变更 board，并保留显式管理员发布演示；
- **Opportunity Graph**：在逐次 consent 后使用 synthetic private profile 生成课程计划；community signal 只参与 soft ranking，并返回 iCourse aggregate-rating link-outs；
- **Simple Calendar**：通过 Agent 记录、列出或删除最多 128 条 owner-local 事项。

命令只接受 loopback bind。运行状态默认写入 `$XDG_STATE_HOME/ustc-campus-agent/three-plugin-mvp`（未设置时为 `~/.local/state/ustc-campus-agent/three-plugin-mvp`）；可用 `USTC_AGENTD_BIND` 与 `USTC_AGENTD_STATE_DIR` 覆盖。state directory 必须是当前用户拥有、模式 `0700` 的真实目录。以同一目录重启会恢复 product state、session/read authority、publication/control evidence、Opportunity profile/tombstone 与 Simple Calendar items。所有校园来源均为显著标记的 reviewed/synthetic fixtures；服务没有生产认证、TLS、正式多用户 SSO 或自动来源更新，禁止直接暴露到公网。

Android 本机演示保持这一边界：先启动上述服务，再执行 `adb reverse tcp:8787 tcp:8787` 并安装 CI 产出的 APK。该 artifact 是 competition/demo bridge，不是 Play Store release，也不宣称完成 authenticated remote deployment。

`m00-sessions.json` 仍是 `event-history-only` 的 current-session read authority；`B4b stable redacted control-event/error` journal 仍为 `data-only` evidence carrier，不是正式 SSO 或通用管理员 API。

当前 Affairs fixture 保留了 2026-08-26 获取的[中国科大教务处公开页面](https://www.teach.ustc.edu.cn/service/svc-student/13824.html)及 normalized bytes，并校验 SHA-256。课程规划 fixture 仅保存 iCourse 公开 aggregate-rating metadata 与 link-out，不缓存点评正文；它不能替代官方课程目录或用户对具体教师与学期的复核。

只需 Affairs Navigator 的兼容性 demo 入口仍是 `./scripts/run_affairs_web_demo.sh`。

`ustc-agentctl` 可从另一个本机进程读取或触发同一固定 demo 命令；非 loopback 地址、发布时缺少 `--confirm` 或 HTTP 侧缺少自定义确认请求头都会 fail closed：

```bash
cargo run -p ustc-agentctl -- affairs publication-status --server 127.0.0.1:8787
cargo run -p ustc-agentctl -- affairs publish-demo --server 127.0.0.1:8787 --confirm
cargo run -p ustc-agentctl -- changes publication-status --server 127.0.0.1:8787
cargo run -p ustc-agentctl -- changes publish-demo --server 127.0.0.1:8787 --confirm
```

## Current decisions

| Item | Decision |
|---|---|
| Repository | `ustc-campus-agent`，GitHub private，Develata personal account |
| Product name | USTC Campus Agent |
| Default first-party Plugins | `ustc.affairs-navigator`, `ustc.change-radar`, `ustc.opportunity-graph` |
| Optional bundled Plugin | `ustc.simple-calendar`；Rust owner-local item store，非默认安装 |
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
| Current Android evidence | `apps/ustc-android-demo` debug APK：API 35 emulator 安装/启动、ADB reverse、真实 Affairs Chat journey；最终 Dioxus/HTTPS/session/真机 `CLIENT-002` 仍未完成 |
| Multi-client shell | `M10` owns framework-neutral versioned operation/client-protocol registry；`M80` owns client core over it；Dioxus Web/Android、`ustc-agent` 与 public-read-first inbound MCP 为 peer adapters；later Windows 复用同一 core；M10 不依赖 client-core，GUI 不 spawn CLI；client/server adapter 不拥有平台 authority |
| CLI privilege split | `ustc-agentctl` 为 operator/developer；`ustc-agent` 已有 bounded ordinary-user/headless Affairs path；`ustc-agentd serve-web` 提供 loopback-only、三插件 source/profile-grounded MVP，生产 auth/remote HTTP/streaming 仍未实现；MCP 仅暴露 selected least-privilege tools/resources |

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

This private competition repository currently grants no public open-source license. See [`LICENSE.md`](LICENSE.md) and [`docs/acceptance/public-readiness.md`](docs/acceptance/public-readiness.md) before any public visibility change.
