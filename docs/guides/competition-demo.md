# 一〇七杯：演示与提交指南

本页区分本次固定提交包与从源码启动的排练环境；两者不能直接互换。
[功能与评分证据](../features/06-mvp-core-capabilities.md#一〇七杯评分与证据)说明可展示范围；
本页列出五分钟录像顺序和四项提交材料。本次补充通知要求 **9 月 6 日 23:59 前**完成提交；
报名背景见[学校通知](https://www.ustc.edu.cn/info/1360/25272.htm)，实际提交步骤和接收状态以比赛附件及入口为准。

## 本次固定提交包

设计文档与作品简介描述的是单独组装的比赛交付包，其身份如下：

| 对象 | 固定来源与差异 |
|---|---|
| 程序二进制 | `a1988e892a6bad42a56032dd9de96701d64fbcf1`，release 构建；后续文档提交不会改变这个已构建程序 |
| 课程目录 | 包内 `market/fixtures/course-planning/minimal-v0.json` 清空全部 10 条 `community_signals`，删除 `icourse-public-aggregate-2026-09-03` 与 `icourse-linkout` 来源，保留 21 门合成课程及其约束 |
| 课程观察 | 包内 `fixtures/opportunity-graph/course-planning-demo-reviewed.json` 同步更新目录摘要、snapshot/evidence 标识及 `synthetic-course-planning-competition-20260906` 版本标记 |
| 启动与证据 | 随包 `START-HERE.md`、`BUILD-INFO.txt`、`FIXTURE-NOTICE.md` 和 `SMOKE-RESULTS.json`；执行随包 `bash start-local.sh` |

这两份资料的修改仅在交付 artifact 中，**本仓库历史 fixture 没有随文档提交被改写**。
比赛包已从解压目录验证四工具调用与三个可行课程方案，且没有社区评分信号；
这项排除声明不适用于原样运行源码 fixture 或原有 Compose 打包脚本生成的包。

如需从源码重建该二进制，应先创建固定版本的独立工作树：

```bash
git worktree add --detach ../uca-competition-runtime a1988e892a6bad42a56032dd9de96701d64fbcf1
cd ../uca-competition-runtime
cargo build --release --locked -p ustc-agentd --bin ustc-agentd
```

原有 `scripts/package_three_plugin_mvp_compose.sh` 只打包它所在检出的 HEAD，
不会自动生成上述资料变体。重建提交 artifact 还必须在独立输出目录应用表中两份资料变更、
更新包内说明与文件清单并重新启动复验；仅在当前 `main` 执行原脚本，不能复现本次交付包。

## 从源码启动独立排练

下面命令使用当前检出的源码及**原始历史资料**，含数据许可尚未闭环的 iCourse 聚合评分快照。
它用于开发排练，不是上述无聚合评分比赛包的启动或重建步骤。比赛接收者请直接使用随包
`START-HERE.md`。开发者在 Linux/WSL 的仓库根目录执行：

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

以下共约五分钟，使用同一候选与独立演示数据。真实模型镜头显示实际配置；
使用 mock 的段落在画面中标注“离线确定性演示”，不能剪成真实模型执行证据。
工具状态与结果一同入镜，不把模型口头确认当作成功。

| 时段 | 操作或可复制输入 | 镜头应保留的结果 |
|---|---|---|
| 0:00–0:20 | 说明学生核对办事资料、安排课程和记录事项的需求；展示 Chat 首页 | 项目名、校园场景、非官方说明、实际模型 |
| 0:20–1:00 | `成绩单证明怎么办？`；打开来源和办理清单 | 办理步骤、原链接、固定审阅资料标识与实际工具状态 |
| 1:00–2:00 | 在月历选日，新建“准备成绩单申请材料”；核对完整日期／时区，确认后刷新；再修改或取消一个提案 | 提案与已保存事项分开，确认结果和回读一致；不要把未确认草稿说成已保存 |
| 2:00–2:25 | `校历最近有什么变化？` | 固定样例的前后差异与来源；说明不代表实时学校通知 |
| 2:25–3:10 | 选课面板填入两门标注为自建演示材料的课程及个人空闲时间，确认仅本次使用，比较后预览课程日历批次 | 先修／学分／冲突检查、推荐依据与待确认批次；不混用真实评分许可未闭环的旧样例 |
| 3:10–4:10 | 插件页展示 MCP／Skill 配置；安装校园指南→检查→授权→启用，再发送下方只读请求 | 配置与授权分开，Skill 和 Calendar 各自的实际成功轨迹 |
| 4:10–4:35 | 停用校园指南，展示模型按钮与对话重命名后刷新 | 插件不再可调用，完整模型名称和能力，`YYMMDD|话题` 标题保留 |
| 4:35–5:00 | 展示简明架构与部署文件，说明已验证和待完成范围 | Rust 中心服务、可运行入口；SSO、真实来源许可和手机推送的实际边界 |

插件调用提示：

> 请实际调用已启用的校园使用指南 Skill，省略 resource 读取入口，并列出我的个人日历事项。两项都调用工具，最后用一句话概括。不要创建或删除事项。

只创建自建演示课程和演示账号，不录真实学生记录。若使用历史选课样例，需说明其中
包含 iCourse 聚合评分快照，且数据使用许可尚未闭环；不能称为纯虚构或已授权实时数据。
课程建议不等于写入日历。新确认的定时事项支持站内提醒；站内投递回执不代表手机系统推送。
失败镜头不能伪装成成功，排练与正式录像分别保存。可补字幕或讲解说明等待和操作目的，
但不得编造回答、工具结果或与录像候选不一致的成功状态。

Android 如需补拍，按[Android 指南](android-demo.md)操作。本轮仅本地构建、
4 项端点测试、lint 与签名检查通过；小米安装返回 `INSTALL_FAILED_USER_RESTRICTED`，
尚无本轮真机功能视频或键盘、返回键、旋转、离线恢复的通过证据。

## 四项提交材料

| 材料 | 格式与应含内容 | 最后检查 |
|---|---|---|
| 设计文档 | 格式不限，建议 PDF；[设计源稿](competition-design.md)包含设计思路、架构、模块、执行流程、技术难点和真实边界 | 图表能阅读，文字与当前程序一致 |
| 演示视频 | MP4 等常见格式，约五分钟；说明具体校园场景、操作过程、主要功能和实际结果 | 独立播放器可打开，画面与字幕清楚，无密钥或真实学生数据 |
| 智能体／可部署程序文件 | 提供可运行程序、必要资料、依赖条件、启动入口及模型占位配置 | 从解压后的包实际启动通过，说明操作系统／架构，不能只放源码截图 |
| 作品简介 | **PDF 或 Word**；介绍背景、解决问题、核心功能、使用模型与 API 调用方式、创新点 | 文件实际存在且可打开，不能用 README 或 Markdown 代替这一项 |

设计文档可以保留 Markdown 源稿方便修改，最终作品简介必须另外导出 PDF 或 Word。
模型一栏写实际演示使用的服务配置标识与调用协议；密钥和私有模型地址不随材料交付。
源码提前单独保留，主办方后续需要时再按通知提交。程序、视频和测试说明应对应同一候选。

<a id="提交清单"></a>

## 单 ZIP 命名与提交清单

同一队伍在同一赛道只能提交一个作品。四项材料放入**一个 ZIP 压缩文件**，由队长提交，
其他队员提交无效；不要直接上传文件夹。本项目按智能体赛道准备，算力平台赛道要求的
Slurm 集群运行数据不是本赛道四项材料之一。

最终文件名按通知填写：

```text
队长学号+队长手机号+智能体赛道+本科生队伍.zip
```

研究生队伍将末段替换为“研究生队伍”；学号与手机号由队长填真实信息。
上式是模板，不能原样作为提交文件名，也不要把个人号码写进公开仓库。

- [ ] 设计文档、约五分钟视频、已测试程序与 PDF／Word 简介四项齐全。
- [ ] 所有材料的模型模式、版本与实现范围一致；源码与必要运行配置示例已留存。
- [ ] 从最终解压目录重新启动，按包内说明完成一项查询与一项确认后回读操作。
- [ ] 包内没有 API 密钥、登录凭据、私人配置、用户历史或真实学生资料。
- [ ] 保留非官方说明、开源许可证及第三方来源说明，未验证项没有写成通过。
- [ ] 队长填写真实文件名，在规定时间前上传单个 ZIP，并核对入口的文件名和接收状态。

本地生成压缩包不等于比赛入口已接收。旧 R3.1 冻结包不包含后来全部功能，不能证明
当前源码能力，也不要把新材料塞入旧包的冻结清单。新候选运行、录像与打包证据随本次材料保存。
