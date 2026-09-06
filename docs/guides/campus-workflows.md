# 校园任务与多人试用

## 日历

在 Chat 提出带日期的事项，或展开日历手动填写。新增、修改、删除先显示提案，
确认后保存；批量课程提案一次确认全部写入。时间必须带时区，界面显示北京时间。

新确认的定时事项在到点后投递到站内提醒。服务每五秒检查，到期记录与已读状态重启后保留。
修改或删除会取消尚未投递的旧提醒；已经投递的回执保留。这不是手机推送、短信或邮件。

## 官方信息与选课

插件页的官方资料面板可检索已导入观察、查看原链接、抓取时间、审阅状态和前后差异。
管理员按 [来源契约](../contracts/campus-source-workspace.md) 配置精确来源清单；
获取仅限已审阅的官方公开 HTTPS 页面，不使用登录 Cookie，不自动扩大站点范围。
导入、抓取与人工审阅分开；观察记录不等于学校发布的规范化流程。

选课面板接受你提供的课程原文和来源、先修、学分、完整上课时间、兴趣和空闲时间。
勾选仅本次使用后比较方案，核对解释与排除原因，再生成日历批次提案并确认。
课程原文和个人偏好不由此面板持久保存；iCourse 评分采集未获许可，当前不读取。

支持工具的模型也可在 Chat 查询已配置资料、获取最新观察、比较精简课程输入并提出日历批次。
模型不能批准来源或确认日历操作。

## 后台配置账号

只提供后台配置账户与登录／退出，没有注册和邀请入口。学校 SSO 仍需真实接入配置。

```bash
cargo build -p ustc-agentd --example account_password_hash
python3 scripts/configure_local_account.py /home/operator/.config/uca/accounts.json admin --bootstrap-administrator
python3 scripts/configure_local_account.py /home/operator/.config/uca/accounts.json student
export USTC_ACCOUNT_CONFIG=/home/operator/.config/uca/accounts.json
export USTC_ACCOUNT_STATE=/home/operator/.config/uca/sessions.json
# 使用新的运行状态目录，执行原 serve-web 命令
```

配置工具交互读取密码，不把密码放进命令参数。配置和状态目录需 0700，文件需 0600。
新账号模式不会接管旧演示日历。会话、根提示词、日历、插件配置与授权按已认证用户隔离；
公开来源和包目录共享。切换账号后页面清除旧账号待提交操作。

这是本机／WSL 的多账号入口；当前监听仍限回环地址。公网 HTTPS、学校 SSO 和多机运维不在此入口内。
模型配置见 [模型指南](model-selection.md)，账号边界见 [账号契约](../contracts/platform-account-local.md)。

## 2026-09-06 本地集成证据

本批在 WSL Debian 运行 Rust 服务，以 Windows Chrome 验证页面；均使用隔离测试数据。

| 检查 | 实际结果 |
|---|---|
| Rust | 全 workspace/all-targets/all-features 测试、Clippy、fmt、doc tests 通过 |
| 前端 | 完整 UI 回归、14 条 Chat 流程、插件与执行进度专项通过；覆盖 390px 窄屏 |
| 来源与课程 | 13 项 API 检查通过：两次导入与差异、搜索／历史／审阅、约束校验、批量提案确认与重试 |
| 本地账号 | 24 项浏览器／API 检查通过：登录退出、逐请求认证、会话与日历隔离、提醒与已读回执 |
| 真实模型 | gpt-5.6-luna 两次短测完成；日历查询工具调用成功，无写入；约 7.4 秒与 6.7 秒 |

未运行：真实学校 SSO、获许可的校园来源网络获取、公网多用户部署、此次 Android 真机回归。
iCourse 数据许可仍待闭环；来源观测不等于正式发布，站内提醒不等于手机推送。

可复验入口：`cargo test --locked --all-targets --all-features`、
`node scripts/test_agent_chat_browser.mjs target/debug/ustc-agentd`、
`node scripts/test_usable_enhancements_browser.mjs target/debug/ustc-agentd`。
插件和进度专项分别设置 `UCA_BROWSER_SUITE=plugins`、`UCA_BROWSER_SUITE=activity`。
本地账号创建与来源清单配置步骤见上文；禁止将私有配置或测试 Cookie 放进提交包。
