# USTC Campus Agent

> 面向科大学生的插件化校园智能体。当前为学生竞赛项目，**不是中国科学技术大学官方服务**。

USTC Campus Agent 的首版目标不是做一个通用聊天机器人，而是做一个有清晰边界的校园 Agent 平台：用户从 Market 安装可信插件，Agent 在来源、权限、版本和审计约束下完成校园任务。

平台有三个正式的 default first-party Plugins：**USTC Affairs Navigator**、**USTC ChangeRadar** 与 **Campus Opportunity Graph**。三者共享 Campus Trust Kernel，但保持独立的 package identity、版本、安装与启停边界。

工程主线按 `ChangeRadar source/revision/diff foundation → Affairs Navigator structured procedure entry → ChangeRadar per-board feed → Opportunity Graph consent/profile integration` 推进。已完成的 **Course Planning** 是 Opportunity Graph 内提前落地的 bounded offline spike；它不把 Opportunity Graph 提升为唯一旗舰，也不代表 Market/runtime 闭环已完成。

## Current decisions

| Item | Decision |
|---|---|
| Repository | `ustc-campus-agent`，GitHub private，Develata personal account |
| Product name | USTC Campus Agent |
| Default first-party Plugins | `ustc.affairs-navigator`, `ustc.change-radar`, `ustc.opportunity-graph` |
| Implementation order | ChangeRadar foundation → Affairs Navigator → ChangeRadar feed → Opportunity Graph integration |
| Course Planning | Retained bounded offline spike inside Opportunity Graph; not Market/runtime completion |
| Chinese name | TBD；首版使用中文描述“面向科大学生的插件化校园智能体” |
| Backup | Self-hosted Gitea pull mirror |
| GitHub organization | Deferred |
| Market repository | Deferred；当前为 monorepo 内 `market/` logical authority boundary |
| Future public release | Possible；public-readiness gate required before changing visibility |
| Runtime strategy | Rust authority core；Rig/goose/Pi/LangGraph are references or bounded adapters, not platform authority |

## Repository layout

```text
apps/                     # runnable binaries and future frontend shell
  ustc-agentd/            # service daemon skeleton
  ustc-agentctl/          # operator/developer CLI skeleton
crates/
  platform-core/          # canonical domain invariants and authority decisions
  adapters/               # replaceable external adapters; no authority ownership
  course-planning/         # typed fixture validation and deterministic planner core
market/                   # plugin catalog authority boundary inside this repo
plugins/                  # three first-party plugin implementation/doc boundaries
docs/                     # current contracts, ADRs, plans, acceptance matrix, legacy migration archive
scripts/                  # local and CI validation scripts
.github/                  # CI, PR template, issue templates, CODEOWNERS
```

## Local development

See [`docs/development/local-setup.md`](docs/development/local-setup.md) for the full local workflow, CodeGraph notes, and cleanup guidance.

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

- Product and execution plan: [`docs/plan/`](docs/plan/)
- Architecture contracts: [`docs/architecture/`](docs/architecture/)
- Three first-party Plugin contract: [`docs/architecture/03-three-first-party-plugins.md`](docs/architecture/03-three-first-party-plugins.md)
- Public interfaces and package schema: [`docs/contracts/`](docs/contracts/)
- Acceptance matrix and gates: [`docs/acceptance/`](docs/acceptance/)
- Collaboration rules for multi-human/multi-agent work: [`docs/collaboration/`](docs/collaboration/)
- Future public/GitHub Pages transition: [`docs/public/`](docs/public/)
- Historical source documents migrated from the planning workspace: [`docs/legacy/`](docs/legacy/)

## Security and credentials

Do not commit USTC credentials, CAS cookies, API keys, real student data, generated logs containing private payloads, or source snapshots that contain personal information. `catalog.ustc.edu.cn` data access must use approved read-only snapshot/import paths or future official authorization. iCourse review content remains link-out-only unless explicit permission is obtained.

## License

This private competition repository currently grants no public open-source license. See [`LICENSE.md`](LICENSE.md) and [`docs/public/public-readiness.md`](docs/public/public-readiness.md) before any public visibility change.
