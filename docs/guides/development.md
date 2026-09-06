# 本地开发与验证

## 环境与启动

- Rust 版本以 [`rust-toolchain.toml`](../../rust-toolchain.toml) 为准，安装 rustfmt 和 clippy。
- Python 3 用于契约检查；Node.js 和 Chromium 用于浏览器测试。
- 当前完整演示建议使用 Linux/WSL。Android 构建环境见 [Android 指南](android-demo.md)。

在仓库根目录启动：

```bash
bash scripts/run_three_plugin_mvp.sh
```

默认地址为 `http://127.0.0.1:8787`，程序打印持久化目录；停止后使用同一目录重启可回读数据。
隔离测试可指定新的 `USTC_AGENTD_STATE_DIR` 和 loopback `USTC_AGENTD_BIND`，不要复用正在操作的用户状态。
模型与密钥配置统一见[模型指南](model-selection.md)，MCP/Skill 包配置见[插件指南](mcp-skills.md)。

## 按改动范围检查

| 改动 | 定向检查示例 |
|---|---|
| 文档 | `python3 scripts/check_repo_contracts.py`、`git diff --check` |
| 插件生命周期与 Skill 应用 | `cargo test --locked -p ustc-agentd --lib plugin_runtime` |
| MCP 协议适配 | `cargo test --locked -p ustc-campus-agent-adapters mcp::` |
| 日历与模型调用约束 | `cargo test --locked -p ustc-agentd --lib calendar` |
| 对话存储 | `cargo test --locked -p ustc-agentd --lib chat_conversations` |
| Android WebView 测试驱动 | `python3 -B -m unittest discover -s scripts/android_tests -p test_webview_cdp.py -v` |

浏览器测试使用当前构建并创建隔离后端。示例：

```bash
cargo build --locked -p ustc-agentd
UCA_BROWSER_SUITE=plugins node scripts/test_usable_enhancements_browser.mjs target/debug/ustc-agentd
```

`UCA_BROWSER_SUITE` 可选择 `plugins`、`models`、`conversations`、`management` 等已有套件。
非默认安装位置通过 `CHROME_BIN` 指定 Chromium。运行前查看脚本参数；
`--base` 模式使用指定的现有服务，执行写入测试时只能连接独立测试状态。
真实模型的短预检见[演示指南](competition-demo.md)，不与离线结果混记。

## 集成基线

Rust 改动在集成检查点运行：

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
python3 scripts/check_repo_contracts.py
git diff --check
```

增加文件时同时检查未跟踪文件。最终检查范围以[验收矩阵](../acceptance/matrix.tsv)与所属任务为准；
没有执行的检查记为未运行，不因其他测试通过而推定成功。
修改契约检查器时另运行其分片测试：

```bash
checker_evidence=$(mktemp -d)
PYTHONPYCACHEPREFIX=$(mktemp -d) python3 scripts/run_checker_shards.py \
  --jobs 4 --timeout-seconds 1800 \
  --inventory scripts/checker_test_inventory.json \
  --evidence-dir "$checker_evidence"
```

## 用户与管理员操作

普通用户 CLI 的现有范围见[客户端说明](../features/05-headless-client-and-agent-integration.md)。
以下是独立的开发者/管理员检查，不属于 Agent 的用户工具权限：

```bash
cargo run --locked -p ustc-agentctl -- doctor
cargo run --locked -p ustc-agentctl -- market validate
cargo run --locked -p ustc-agentctl -- course plan \
  --fixture market/fixtures/course-planning/minimal-v0.json --format json
```

课程命令的合规结果为 `course-plan-result/v0`，固定样例至少两个候选且
`hard_constraint_violations: 0`；这只验证离线规划，不证明真实课程来源获准使用。
固定示例发布需要显式 `--confirm`，对应流程见[演示指南](competition-demo.md)。

## 工作区约定

遵循根目录 [AGENTS.md](../../AGENTS.md)：保留他人改动，只暂存本次精确文件；
远程操作遵循当前授权。CodeGraph 为可选导航工具，已有索引时可使用。
不提交构建目录、索引、模型密钥、真实学生数据或包含私有载荷的证据。
