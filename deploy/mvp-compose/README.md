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

1. 复制 `.env.example` 为 `.env`；
2. 将 `.env` 中 `UCA_AGENT_PROVIDER` 改为 `openai-compatible`，填写 HTTPS `UCA_AGENT_BASE_URL` 与 `UCA_AGENT_MODEL`；
3. 在本目录创建被 Git 忽略的 `secrets/llm-api-key.txt`，仅写入 key；
4. 将 `UCA_AGENT_API_KEY_SOURCE` 改为 `./secrets/llm-api-key.txt`；
5. 重新运行启动脚本。

服务只在容器启动时读取 `/run/secrets/uca_agent_api_key`。provider mode 配置不完整会 fail closed，不会回退到 mock 或其他模型。正常响应、浏览器资源与日志都不应包含 key。`mock-provider-key.txt` 只是为了让默认 Compose 配置可移植启动的非敏感占位文本；mock 不读取它。

本包内置 `linux/amd64` binary；主验收目标是 Windows x64 + Docker Desktop。Intel macOS/Linux 可原生运行，Apple Silicon 由 Docker Desktop 的 amd64 emulation 运行，后者当前属于 best-effort compatibility。

首次 `docker compose up --build` 需要联网拉取 `ubuntu:24.04`，并在 image build 阶段安装 `ca-certificates`、`curl`、`socat`；这与 MVP application 的 fixture-only runtime 不同。镜像层已缓存后，后续启动不需要 live source retrieval。

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
docker compose down --volumes
```

Windows 可双击 `reset.cmd`，脚本会再次确认。

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
