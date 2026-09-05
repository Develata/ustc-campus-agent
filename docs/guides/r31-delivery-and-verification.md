# R3.1 交付身份与复验

> 类型：交付快照与操作指南，不是新的行为权威。记录日期：2026-09-06 CST。来源基线：`2f03a13c3ec03eab791f1728e2c2973ed82c3e50`。后续 main 前进不自动改变本页记录的归档字节。

## 1. 交付物是什么

R3.1 是 R3 的 **Windows launcher 修正版完整竞赛材料包**，不是新功能版，也不是 GitHub Release。该包已通过会话附件交付给接收者；本仓库不托管该 ZIP，也不提供未经验证的稳定下载 URL。未持有附件者应取得原始 ZIP 与同名 sidecar，不要从当前 main 自行重建后沿用本页 hash。

- 外层文件：`ustc-campus-agent-submission-v1-staging-r3.1-a26c55b1.zip`
- 文件大小：`10801359` bytes。
- 外层 SHA-256：`de2bf78992b82a6eb8be63e12dcf6a2ace942074b6baac1f9c0817dcf4396cfe`
- sidecar：上述文件名加 `.sha256`。
- 外层 46 个 regular files，其中 45 个由 `SUBMISSION-MANIFEST.txt` 覆盖；manifest 不校验自身。
- 内层程序 ZIP：`03_Program/ustc-campus-agent-mvp.zip`。
- 内层 ZIP SHA-256：`1ecb7326335a4d962f33c570fa9015b696e43c69dd787e3fb59e92f42257027c`；其内 `SHA256SUMS` 覆盖 28 项。

### 不可混同的源码身份

| 对象 | 固定身份 | 解释 |
|---|---|---|
| runtime、APK、页面、fixtures、SSO 样例 | `a26c55b1c5d2ec094a492505deee1a983466586d` | 沿用 R3 原字节，不重编 |
| launcher 受验 head | `11c546b1c7d1fa7182660a36a97bb9203a10e1ed` | PR #77 的修正与回归测试 |
| launcher 合并 commit | `2f03a13c3ec03eab791f1728e2c2973ed82c3e50` | 与受验 head 树相同 |
| R3 artifact-only builder | `9178e4d4e5f09b496558f6cb549f5de6377ab98e` | 构建入口身份，不是产品源码版本 |

R3.1 内层仅变更 `start.ps1`、`README.md`、`BUILD-INFO.txt`、`SHA256SUMS`，其余条目逐字节保持。`06_Source/` 保留原 runtime 源码快照，另附 launcher 修正及回归测试的源码 ZIP。不得把 launcher commit 写成 APK/runtime 的重新构建版本。

原 R3 归档保持不变，但最终 Windows launcher 验收使用 R3.1。

## 2. Windows PowerShell 5.1 修正

用户在 R3 实机测试中报告：容器已健康，但 `docker compose port ... | Select-Object -First 1` 后退出码为 `-1`，直接接收 native stdout 时为 `0`。

已合并的 [start.ps1](../../deploy/mvp-compose/start.ps1) 完整接收 native stdout，立即保存退出码，再选择内存结果的首行；显式 string 转换使空输出被地址检查拒绝。真正非零退出仍失败，没有忽略 `-1` 或强行改成成功。行为归属见 [Chat contract](../contracts/agent-chat.md) §7。

[原生回归测试](../../scripts/check_windows_native_port.ps1) 由 [launcher suite](../../scripts/check_windows_launchers.ps1) 调用，在托管 Windows PowerShell 5.1 上使用原生进程 fixture 检查 single、multiple、failure、empty、wildcard、range、malformed。**它不等于真实 Docker Desktop 全链路。**

## 3. 证据与尚未完成事项

可公开回读的源码/CI：

- R3 功能与 SSO：[PR #75](https://github.com/Develata/ustc-campus-agent/pull/75)。
- R3 exact-source CI：[33968865184](https://github.com/Develata/ustc-campus-agent/actions/runs/33968865184)，success。
- R3 artifact builder：[33968933195](https://github.com/Develata/ustc-campus-agent/actions/runs/33968933195)，success；该 workflow run 不是 R3.1 外层材料包下载入口。
- launcher 修正：[PR #77](https://github.com/Develata/ustc-campus-agent/pull/77)，merged。
- exact-head CI：[33973676031](https://github.com/Develata/ustc-campus-agent/actions/runs/33973676031)，success；其中 [Windows job](https://github.com/Develata/ustc-campus-agent/actions/runs/33973676031/job/101326562143) 含七条 native PASS marker。
- 合并后 CI：[33974451181](https://github.com/Develata/ustc-campus-agent/actions/runs/33974451181)，success。

随包证据：`05_Evidence/R31-DELTA.json`、`R31-SHIP.json`、`R31-QA.md`。集成/独立 QA 记录为 PASS；封包时 ZIP CRC、fresh 解压逐字节、清单及两层 checksum、提取 binary `--version` 已通过。**这些是该附件的生产者回执，不是托管 CI 对整个竞赛 ZIP 的认证；接收者仍应执行下面的独立校验。**随包 `R31-SHIP.json` 保留封包时的 post-merge CI 快照；上面的终态 URL 是后续 read-back，不为更新文字而修改固定包。

| 项目 | 已有证据 | 仍需完成 |
|---|---|---|
| 旧 R3 Windows | 用户报告外层 39 项、内层 28 项、SSO 15 tests、真实 Docker build、健康检查、四工具组合通过；旧 launcher FAIL | 不将旧结论升级为 R3.1 全链路 PASS |
| R3.1 launcher | 托管 Windows PowerShell 5.1 七类 native 回归 PASS | 用户机器启动退出 0、ready URL、Docker Desktop 复验 |
| 浏览器 | retained browser 自动化通过 | 用户完整浏览器交互 |
| Calendar | 源码/Compose 自动化有持久化证据 | 用户机器重启持久化 |
| Android | debug APK / emulator 证据 | 物理 Android |
| Provider | mock 与受限 adapter 测试 | R3.1 对应环境真实 provider E2E |
| 竞赛材料 | 设计/介绍 PDF 与视频脚本随包 | 最终 MP4、完整播放/隐私、门户上传/read-back |

用户本机报告没有在当前 Linux 环境读取；上表仅按用户提供的结果登记，不发布私人报告路径或数据卷标识。

另有非测试型缺口：[public-readiness](../acceptance/public-readiness.md) 的 iCourse/USTC 数据使用许可仍未关闭；真实 aggregate snapshot 不能被“只外链”措辞或 mock 测试自动授权。该问题需要 owner 的许可依据/范围或另行替代决定，本指南不授予更多收集、发布或复用权限。

## 4. 附件接收与 WSL 校验

在附件目录执行；创建新的临时解压目录，不覆盖旧包或测试报告：

```bash
set -euo pipefail
printf '%s  %s\n' \
  'de2bf78992b82a6eb8be63e12dcf6a2ace942074b6baac1f9c0817dcf4396cfe' \
  'ustc-campus-agent-submission-v1-staging-r3.1-a26c55b1.zip' | sha256sum -c -
sha256sum -c ustc-campus-agent-submission-v1-staging-r3.1-a26c55b1.zip.sha256
fresh="$(mktemp -d ./uca-r31-fresh.XXXXXX)"
unzip -q ustc-campus-agent-submission-v1-staging-r3.1-a26c55b1.zip -d "$fresh"
cd "$fresh/ustc-campus-agent-submission-v1-staging-r3.1-a26c55b1"
```

外层 `SUBMISSION-MANIFEST.txt` 是三列 `SHA256  bytes  relative_path`，**不能直接交给 `sha256sum -c`**。在外层解压根目录运行：

```bash
python3 - <<'PY'
from pathlib import Path
import hashlib
root = Path('.')
seen = set()
for line in (root / 'SUBMISSION-MANIFEST.txt').read_text().splitlines():
    digest, size, name = line.split('  ', 2)
    rel = Path(name)
    assert not rel.is_absolute() and '..' not in rel.parts and name not in seen
    seen.add(name)
    data = (root / rel).read_bytes()
    assert len(data) == int(size) and hashlib.sha256(data).hexdigest() == digest, name
actual = {p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file()}
assert seen == actual - {'SUBMISSION-MANIFEST.txt'}
assert len(seen) == 45
print('MANIFEST=PASS', len(seen))
PY
(cd 03_Program && sha256sum -c SHA256SUMS)
```

预期分别为外层 ZIP `OK`、`MANIFEST=PASS 45` 和程序目录 checksum 全部 `OK`。内层解包/校验继续按随包 `03_Program/WINDOWS-EXACT-ARCHIVE-ACCEPTANCE.zh-CN.md` 执行；它绑定新内层 ZIP hash。

## 5. 实机复验与交付顺序

1. **Windows 11 + Windows PowerShell 5.1 + Docker Desktop**：执行随包验收清单，以独立 Compose project 运行 `start.ps1`/`start.cmd`；保存环境、archive hash、退出码 0、ready URL、`/healthz`、四工具、浏览器与 Calendar restart 结果。WSL Docker、Linux PowerShell 和 hosted runner 不能替代这一 gate。
2. **保留旧数据**：不要用 reset 掩盖启动脚本失败。若专门验证旧卷，先只读核对旧 run directory 和 `COMPOSE_PROJECT_NAME`，备份并沿用确切配置；不猜卷名、不 broad cleanup。正常停止不带 `--volumes`；显式 reset 的破坏性步骤只能在已确认的独立测试项目上执行。
3. **最终 MP4**：按随包 `02_Demo/VIDEO-SHOOTING-PACKAGE.zh-CN.md` 录制。285 秒只是分镜目标；对最终文件运行 `ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 demo.mp4`，要求不超过 300 秒，并完整播放和检查隐私；记录 `sha256sum demo.mp4`。
4. **门户**：上传实际要求的 ZIP/MP4/材料后，回读项目/赛道、附件名/大小/完成状态、最终提交状态；可下载时重新校验。read-back 前不能称“已提交”。不要为嵌入视频擅自修改固定 ZIP。
5. **物理 Android 与真实 provider**：在不挤压前三项时并行或随后执行。[Android guide](android-demo.md) 不等于真机 PASS。provider 凭据只用服务端 secret 边界，不进入报告/聊天/归档。未执行则 `NOT_RUN`。

SSO 口径见 [SSO 样例](../../examples/sso-interface/README.zh-CN.md)：当前只提供禁用的接口预留，没有学校 SSO、普通注册或密码登录的实现声明。未来授权仍须协议与会话集成；不以“时间来不及”替代真实授权/配置边界。

## 6. 接收者回执

```text
R31_ARCHIVE_SHA256=
OUTER_MANIFEST=PASS|FAIL
INNER_CHECKSUMS=PASS|FAIL
WINDOWS_LAUNCHER=PASS|FAIL|NOT_RUN
WINDOWS_LAUNCHER_EXIT_CODE=
WINDOWS_DOCKER=PASS|FAIL|NOT_RUN
BROWSER_FULL_INTERACTION=PASS|FAIL|NOT_RUN
CALENDAR_RESTART_PERSISTENCE=PASS|FAIL|NOT_RUN
PHYSICAL_ANDROID=PASS|FAIL|NOT_RUN
REAL_PROVIDER_E2E=PASS|FAIL|NOT_RUN
MP4_SHA256=
MP4_DURATION_SECONDS=
FULL_PLAYBACK_PRIVACY=PASS|FAIL|NOT_RUN
PORTAL_UPLOAD_READBACK=PASS|FAIL|NOT_RUN
BLOCKERS=
```

本指南只补仓库内可复用入口。私人主机位置、运行中任务、凭据和原始聊天保持在独立交接渠道，不作为公开产品文档。
