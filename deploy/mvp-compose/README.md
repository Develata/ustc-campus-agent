# USTC Campus Agent · 三插件 MVP 测试包

这是一个**本机、离线 fixture、Docker Compose** 演示包。它包含：

- Affairs Navigator：办事流程与来源证据；
- ChangeRadar：语义变更板与显式 demo publication；
- Opportunity Graph：consent → private profile → 课程规划 → revoke/delete。
- Web Chat：默认 deterministic mock provider；自然语言问题可进入以上三个受限产品路径。

它不是中国科学技术大学官方服务，不连接真实 USTC 账号，也不会抓取实时网页。

默认 `UCA_AGENT_PROVIDER=mock`，不需要密钥，也不会发起 provider 网络请求。真实 OpenAI-compatible provider 与校园 source 权限相互独立：配置模型接口不会启用任何 USTC live retrieval。

## 可选 OpenAI-compatible provider

只有你明确配置后才启用。不要把 key 写入 `.env`、Compose YAML、命令参数或聊天页面：

1. 正式测试包已经生成带精确 `UCA_SOURCE_COMMIT` 的 `.env`：直接编辑它且不得覆盖或改写该字段；只有从源码模板单独组装、目录中尚无 `.env` 时，才复制 `.env.example` 为 `.env`；
2. 将 `.env` 中 `UCA_AGENT_PROVIDER` 改为 `openai-compatible`，填写 HTTPS `UCA_AGENT_BASE_URL`、`UCA_AGENT_MODEL` 与模型真实的 `UCA_AGENT_CONTEXT_TOKENS`；
3. 在本目录创建被 Git 忽略的 `secrets/llm-api-key.txt`，仅写入 key；在 macOS/Linux 上请用 `install -d -m 0700 secrets` 后执行 `umask 077` 再写入，并确认 `chmod 0600 secrets/llm-api-key.txt`；
4. 将 `UCA_AGENT_API_KEY_SOURCE` 改为 `./secrets/llm-api-key.txt`；`.env` 必须是 regular non-symlink file，以上两个 security-critical key 各至多出现一次并使用 column-zero `KEY=value`；值必须是 literal，不支持 Compose `$VAR` / `${VAR:-default}` interpolation（跨平台 launcher 会在 Docker 启动前拒绝），如需动态注入请直接提供规范化后的 process environment literal；
5. 重新运行启动脚本。

Compose 仅在 `openai-compatible` 模式下处理 secret。由于本地 Compose 的 file-backed secret 会保留宿主文件 ownership，Compose 先 drop 全部 capabilities，再仅向 root-only 初始化阶段补回 `CHOWN`、`DAC_OVERRIDE`、`FOWNER`、`SETGID`、`SETPCAP` 与 `SETUID`：entrypoint 因而能读取显式挂载的 owner-only source，把它复制为 `/run/uca-agent-private/uca_agent_api_key` 中由 UID/GID 65532 持有的 mode-0600 ephemeral tmpfs 文件；随后用 `setpriv` 清空 supplementary groups、effective/bounding capability set，设置 no-new-privileges，并以 UID/GID 65532 重新执行自身，daemon 与 loopback proxy 均只在降权后启动。原始 source 不进入镜像或持久 volume。`start.sh` 会在 macOS/Linux 上拒绝 group/world permission bits 非零的 source file，`start.ps1` 会拒绝 symlink/reparse-point source；直接执行 `docker compose` 时 Docker 的 secret projection 无法替容器证明宿主 Unix mode，因此操作者仍须先完成第 3 步的 `chmod 0600`。无论从 launcher 还是直接 Compose 启动，`openai-compatible` 都会由 launcher/container preflight 与权威 Rust key reader 共同拒绝 outer-whitespace-normalized bundled mock placeholder；provider mode 配置不完整或 context limit 不在 `16384..1048576` 时同样 fail closed，不会回退到 mock 或其他模型。每次真实 provider 请求还会在网络 I/O 前执行 conservative UTF-8-byte context preflight，并预留输出与估计误差预算。正常响应、浏览器资源与日志都不应包含 key。`mock-provider-key.txt` 只是为了让默认 Compose 配置可移植启动的非敏感占位文本；mock 不读取它。

本包内置 `linux/amd64` binary；主验收目标是 Windows x64 + Docker Desktop。Intel macOS/Linux 可原生运行，Apple Silicon 由 Docker Desktop 的 amd64 emulation 运行，后者当前属于 best-effort compatibility。

首次 `docker compose up --build` 需要联网拉取 `ubuntu:24.04`，并在 image build 阶段安装 `ca-certificates`、`curl`、`socat`、`util-linux`；这与 MVP application 的 fixture-only runtime 不同。镜像层已缓存后，后续启动不需要 live source retrieval。

## Windows 一键启动

1. 启动 Docker Desktop，并等待状态变为 *Engine running*。
2. 解压本包，双击 `start.cmd`。
3. 健康检查通过后，浏览器会打开 <http://127.0.0.1:8787>。

包内 `.cmd` / `.ps1` 启动脚本刻意只使用 ASCII 字节，兼容仍按系统代码页读取无 BOM 脚本的 Windows PowerShell 5.1。请勿把这些脚本另存为无 BOM 的非 ASCII 文本；需要本地化提示时，应保留这一兼容约束或改用明确的 UTF-8 BOM。

若 Windows 阻止 PowerShell 脚本，可在本目录打开 PowerShell 后执行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\start.ps1
```

## macOS / Linux 启动

```bash
./start.sh
```

也可直接执行：

```bash
docker compose up --build -d
```

随后访问 <http://127.0.0.1:8787>。

## 建议测试顺序

1. **Web Chat / Affairs**：发送“成绩单证明怎么办”；检查工具轨迹和自然语言答案。
2. **Web Chat / ChangeRadar**：发送“校历最近有什么变化”；检查工具轨迹。
3. **Affairs Navigator**：保留默认流程 ID，点击“查看流程”；检查办理条件、步骤、官方入口与 evidence。
4. **Affairs publication**：勾选确认，再发布固定 demo revision；刷新状态并重复提交，检查 receipt 是否稳定。
5. **ChangeRadar**：先读取变更板，再勾选确认并发布固定变更；检查 JSON board 与 Atom 链接。
6. **Opportunity Graph**：先显式同意并创建 profile；随后在 Chat 中再勾选一次“允许本次对话使用当前 synthetic profile”并询问课程规划。Chat 无权创建或删除 profile。
7. **重启恢复**：执行 `docker compose restart`，刷新页面，确认 durable publication/profile 状态仍可读。

## 停止、保留状态与重置

停止服务但保留测试状态：

```bash
docker compose down
```

Windows 也可双击 `stop.cmd`。

彻底删除本 MVP 的 Docker volume 并回到初始状态：

```bash
./reset.sh
```

Windows 可双击 `reset.cmd`，两种入口都会再次确认。

## 端口

默认只监听宿主机 loopback `127.0.0.1:8787`，不会暴露到局域网。若端口冲突，在启动前设置：

容器内的 `socat` 必须监听 container interface，才能把 Docker published port 转发给仍然只监听 `127.0.0.1:8788` 的 Rust application；入站安全边界由 Compose 的 host-loopback port mapping 限定。容器使用默认的 project-scoped bridge network，因为 Docker `internal: true` 会同时阻断此处所需的 published-port NAT；application 本身没有 live source retrieval 路径。

```powershell
$env:UCA_MVP_PORT = "8877"
.\start.ps1
```

然后访问 `http://127.0.0.1:8877`。

## 反馈模板

请把以下信息直接发给 Deve Hermes：

- 操作系统与 Docker Desktop 版本；
- 哪一步失败；
- 页面提示或 `docker compose logs --no-color` 输出；
- 是否为首次启动、重启后或 reset 后；
- 若是 UI 问题，附截图和浏览器窗口大小。

`BUILD-INFO.txt` 与 `SHA256SUMS` 记录本包的源码和文件身份。
