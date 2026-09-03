# USTC Campus Agent · 三插件 MVP 测试包

这是一个**本机、离线 fixture、Docker Compose** 演示包。它包含：

- Affairs Navigator：办事流程与来源证据；
- ChangeRadar：语义变更板与显式 demo publication；
- Opportunity Graph：consent → private profile → 课程规划 → revoke/delete。

它不是中国科学技术大学官方服务，不连接真实 USTC 账号，也不会抓取实时网页。

本包内置 `linux/amd64` binary；主验收目标是 Windows x64 + Docker Desktop。Intel macOS/Linux 可原生运行，Apple Silicon 由 Docker Desktop 的 amd64 emulation 运行，后者当前属于 best-effort compatibility。

首次 `docker compose up --build` 需要联网拉取 `ubuntu:24.04`，并在 image build 阶段安装 `ca-certificates`、`curl`、`socat`；这与 MVP application 的 fixture-only runtime 不同。镜像层已缓存后，后续启动不需要 live source retrieval。

## Windows 一键启动

1. 启动 Docker Desktop，并等待状态变为 *Engine running*。
2. 解压本包，双击 `start.cmd`。
3. 健康检查通过后，浏览器会打开 <http://127.0.0.1:8787>。

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

1. **Affairs Navigator**：保留默认流程 ID，点击“查看流程”；检查办理条件、步骤、官方入口与 evidence。
2. **Affairs publication**：勾选确认，再发布固定 demo revision；刷新状态并重复提交，检查 receipt 是否稳定。
3. **ChangeRadar**：先读取变更板，再勾选确认并发布固定变更；检查 JSON board 与 Atom 链接。
4. **Opportunity Graph**：同意字段 → 创建 profile → 查看 → 生成计划 → revoke/delete；删除后再次规划应被拒绝。
5. **重启恢复**：执行 `docker compose restart`，刷新页面，确认 durable publication/profile 状态仍可读。

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
