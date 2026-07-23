# USTC Campus Agent

> 面向科大学生的插件化校园智能体。当前为学生竞赛项目，**不是中国科学技术大学官方服务**。

USTC Campus Agent 的首版目标不是做一个通用聊天机器人，而是做一个有清晰边界的校园 Agent 平台：用户从 Market 安装可信插件，Agent 在来源、权限、版本和审计约束下完成校园任务。

平台有三个正式的 default first-party Plugins：**USTC Affairs Navigator**、**USTC ChangeRadar** 与 **Campus Opportunity Graph**。三者共享 Campus Trust Kernel，但保持独立的 package identity、版本、安装与启停边界。

平台主线已建立 framework-neutral Agent runtime kernel 与 typed invocation resolver；下一步并行补齐 `finite HarnessRun/TaskGraph kernel` 与 `read-only Market projection → durable installation/grant state`，再收敛为有 clarification、context budget、verification 与 bounded review 的用户 Agent journey。first-party 产品仍按 `ChangeRadar source/revision/diff foundation → Affairs Navigator structured procedure entry → ChangeRadar per-board feed → Opportunity Graph consent/profile integration` 推进。已完成的 **Course Planning** 是 Opportunity Graph 内提前落地的 bounded offline spike；它不把 Opportunity Graph 提升为唯一旗舰，也不代表 Market/runtime 闭环已完成。

## Current decisions

| Item | Decision |
|---|---|
| Repository | `ustc-campus-agent`，GitHub private，Develata personal account |
| Product name | USTC Campus Agent |
| Default first-party Plugins | `ustc.affairs-navigator`, `ustc.change-radar`, `ustc.opportunity-graph` |
| Implementation order | ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph integration |
| Course Planning | Retained bounded offline spike inside Opportunity Graph; not Market/runtime completion |
| Chinese name | TBD；首版使用中文描述“面向科大学生的插件化校园智能体” |
| GitHub organization | Deferred |
| Market repository | Deferred；当前为 monorepo 内 `market/` logical authority boundary |
| Future public release | Possible；public-readiness gate required before changing visibility |
| Runtime strategy | Rust authority core；ADR-0004 reference systems remain references or bounded adapters, not platform authority |
| Agent harness | finite HarnessRun over typed TaskGraph；model proposes, Rust validates；every model call passes context-budget preflight |

## Repository layout

```text
apps/                     # runnable binaries and future frontend shell
  ustc-agentd/            # service daemon skeleton
  ustc-agentctl/          # operator/developer CLI skeleton
crates/
  platform-core/          # canonical domain invariants and authority decisions
  agent-runtime/          # node AgentRun today; future finite harness state, graph, context and review kernel
  adapters/               # replaceable external adapters; no authority ownership
  course-planning/         # typed fixture validation and deterministic planner core
market/                   # plugin catalog authority boundary inside this repo
plugins/                  # three first-party plugin implementation/doc boundaries
docs/                     # layered plans, features, contracts, acceptance, tasks, guides and ADRs
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
python3 -m unittest discover -s scripts/tests -p 'test_*.py'
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
```

## Documentation map

- Documentation entry and authority rules: [`docs/README.md`](docs/README.md)
- Engineering blueprint: [`docs/plan/`](docs/plan/)
- User-visible journeys: [`docs/features/`](docs/features/)
- Typed public/package/data contracts: [`docs/contracts/`](docs/contracts/)
- Acceptance matrix and gates: [`docs/acceptance/`](docs/acceptance/)
- Cross-layer architecture map: [`docs/overview/architecture.md`](docs/overview/architecture.md)
- Dependency-aware execution roadmap: [`docs/tasks/01-execution-roadmap.md`](docs/tasks/01-execution-roadmap.md)
- Collaboration, development and publication handoffs: [`docs/guides/`](docs/guides/)
- Architecture decision history: [`docs/adr/`](docs/adr/)

## Security and credentials

Do not commit USTC credentials, CAS cookies, API keys, real student data, generated logs containing private payloads, or source snapshots that contain personal information. `catalog.ustc.edu.cn` data access must use approved read-only snapshot/import paths or future official authorization. iCourse review content remains link-out-only unless explicit permission is obtained.

## License

This private competition repository currently grants no public open-source license. See [`LICENSE.md`](LICENSE.md) and [`docs/acceptance/public-readiness.md`](docs/acceptance/public-readiness.md) before any public visibility change.
