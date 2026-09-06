# 一〇七杯：演示与提交指南

使用当前源码构建、独立状态目录和同一份配置完成排练、录像与打包。
[功能与评分证据](../features/06-mvp-core-capabilities.md#一〇七杯评分与证据)说明可展示范围；
本页只列操作。提交时间和材料要求以[学校通知及提交入口](https://www.ustc.edu.cn/info/1360/25272.htm)为准。

## 启动独立演示

在 Linux/WSL 的仓库根目录执行：

```bash
cargo build --locked -p ustc-agentd
umask 077
uca_demo_state=$(mktemp -d)
printf '演示状态目录：%s\n' "$uca_demo_state"
export UCA_AGENT_PROVIDER=mock
unset UCA_AGENT_MODELS_FILE UCA_PLUGIN_PACKAGE_DIRS
target/debug/ustc-agentd serve-web --bind 127.0.0.1:18874 \
  --fixture fixtures/affairs/proc-011-reviewed.json \
  --change-fixture fixtures/change-radar/academic-calendar-demo-reviewed.json \
  --opportunity-fixture fixtures/opportunity-graph/course-planning-demo-reviewed.json \
  --opportunity-catalog market/fixtures/course-planning/minimal-v0.json \
  --opportunity-profile-store "$uca_demo_state/profiles.json" \
  --store "$uca_demo_state/affairs.json" \
  --idempotency "$uca_demo_state/idempotency.json" \
  --session-store "$uca_demo_state/sessions.json"
```

打开 `http://127.0.0.1:18874/#chat`。端口被占用时换一个空闲端口，
不要停止其他预览。按 Ctrl+C 结束本次进程；复查持久化时保留该终端变量，
只重跑 daemon 命令，不重新创建状态目录。

mock 无需密钥，适合演示四项内置工具。录像要展示安装的 Skill/MCP 实际执行时，
先停本次进程，配置支持工具调用的真实模型：

```bash
export UCA_AGENT_PROVIDER=openai-compatible
export UCA_AGENT_BASE_URL=https://your-provider.example/v1
export UCA_AGENT_MODEL=your-tool-capable-model
export UCA_AGENT_API_KEY_FILE=/absolute/private/path/model.key
export UCA_AGENT_TIMEOUT_MS=15000
export UCA_AGENT_CONTEXT_TOKENS=32768
```

将占位值替换为已授权配置，再执行上面的 daemon 命令；不要再次设置 mock。
密钥只放私有文件，不写进页面、命令参数或录像。上下文预算不得超过模型能力；
`local-chat` 只验证文本连接，不能用于插件执行演示。详见[模型配置](model-selection.md)。

## 真实模型预检

录像前可先运行一次有限预检：

```bash
python3 scripts/smoke_model_plugins.py \
  --provider-base-url https://your-provider.example/v1 \
  --model your-tool-capable-model \
  --key-file /absolute/private/path/model.key
```

脚本使用独立临时服务与 synthetic 状态，完成校园 Skill 的安装、检查、授权、启用，
然后发出一条只读请求，要求 Skill 和 Calendar 均实际成功。结果只输出耗时、用量和
工具状态；失败非零退出，不自动重试。结束后清理自己的进程和临时状态，不改日常预览。

默认上下文预算为 32768，可用 `--context-tokens` 调整。若出现
`context_budget_exceeded`，先核对模型容量与配置。模型最多调用三轮，每轮超时
15 秒。预检成功不替代外部 MCP 服务兼容性或手机验收。

## 录像顺序

以下约四分钟，使用支持工具调用的真实模型。只录 mock 时跳过第 6 步，
并在画面中标注“离线演示”。每次展示工具轨迹，不把模型口头确认当作成功。

| 步骤 | 操作或可复制输入 | 镜头应保留的结果 |
|---|---|---|
| 1 · 20 秒 | 展示 Chat 首页、模型模式和插件入口 | 当前运行版本与模型 |
| 2 · 35 秒 | `成绩单证明怎么办？` | 办理步骤、官方链接、来源与成功轨迹 |
| 3 · 35 秒 | `记录事项：准备成绩单申请材料`；再问 `列出我的待办事项` | 真实事项 ID，刷新或重启后回读 |
| 4 · 25 秒 | `校历最近有什么变化？` | 固定演示资料的版本差异与来源 |
| 5 · 35 秒 | 创建 synthetic 演示档案，确认仅本次对话使用，再问选课建议 | 约束、候选与依据；指出聚合评分不等于官方课程事实 |
| 6 · 60 秒 | 插件页安装校园指南→检查→授权→启用，再发送下方请求 | Skill 和 Calendar 各自的成功轨迹 |
| 7 · 30 秒 | 停用校园指南；重命名对话并刷新 | 不再提供该插件调用，历史和标题仍可读取 |

第 6 步提示：

> 请实际调用已启用的校园使用指南 Skill，省略 resource 读取入口，并列出我的个人日历事项。两项都调用工具，最后用一句话概括。不要创建或删除事项。

录制中只创建演示档案，不录真实学生记录。选课样例中的 iCourse 聚合评分快照
须说明来源及尚未闭环的数据使用许可；不要称其为纯虚构数据或已授权实时来源。
课程建议不等于已写入日历。日期提案需在日历面板核对并确认；保存日期不等于设置了提醒。
失败时保留真实状态说明，重新排练与正式录像分开保存。

Android 如需补拍，按[Android 指南](android-demo.md)操作。本轮仅本地构建、
4 项端点测试、lint 与签名检查通过；小米安装返回 `INSTALL_FAILED_USER_RESTRICTED`，
尚无本轮真机功能视频或键盘、返回键、旋转、离线恢复的通过证据。

## 提交清单

- [ ] 设计文档围绕四个评分维度组织，引用[功能与证据表](../features/06-mvp-core-capabilities.md#一〇七杯评分与证据)；架构说明引用[现有图](../overview/architecture.md)，不重复堆叠模块说明。
- [ ] 视频显示版本、模型模式、实际操作和工具状态；未实现项不剪成成功演示。
- [ ] 程序包、运行说明和测试结果对应同一候选；从解压后的包重新启动验证。
- [ ] 保留源码身份、配置模式、校验和与测试输出；正式打包按[部署说明](../../deploy/mvp-compose/README.md)执行，不绕过干净候选检查。
- [ ] 材料不含密钥或真实学生数据，并保留非官方说明及数据来源/许可状态。
- [ ] 最后检查提交入口已接收文件，分别确认设计文档、视频和可部署程序。

旧 R3.1 冻结包不包含后来全部功能，不能证明当前源码能力。含未提交改动的目录
只能标为开发预览。已有一次本地真实模型 Skill＋Calendar 成功收据位于生成文件
`dist/competition-20260906/verification.md`；它不属于冻结包，也不是投稿回执。
