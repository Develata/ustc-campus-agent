# Android 演示指南

APK 是连接现有 Rust 服务的 debug 签名 WebView 客户端。手机展示同一套 Chat 和插件页面，业务与数据留在服务端；它不能离线运行后端，也不是生产发布包。

## 连接与演示

在主机仓库根目录启动服务：

```bash
./scripts/run_three_plugin_mvp.sh
```

等候服务输出：

```text
Web:   http://127.0.0.1:8787/
```

手机开启开发者选项和 USB 调试，确认仅连接预期设备：

```bash
adb devices
```

建立设备到主机回环地址的转发；连接多台设备时，每条命令加 `-s DEVICE_SERIAL` 指定目标：

```bash
adb reverse tcp:8787 tcp:8787
adb reverse --list
```

这不会把 Rust 服务开放到局域网。将 APK 和配套 `.sha256` 放在同一目录，替换下面的实际文件名，再校验、安装和启动：

```bash
sha256sum -c your-android-debug.apk.sha256
adb install -r your-android-debug.apk
adb shell am start -n \
  com.develata.ustccampusagent.debug/com.develata.ustccampusagent.MainActivity
```

应用默认连接 `http://127.0.0.1:8787/`。出现离线页时检查服务和转发，再点“重试连接”。在 Chat 中依次试用：

```text
成绩单证明怎么办？
校历最近有什么变更？
记录事项：提交开题报告
列出我的待办事项
```

应看到回答和工具状态。官方信息来自受审阅演示资料；日历记录在主机持久化，新确认的定时事项支持站内提醒，尚无手机系统推送。模型能力见[模型指南](model-selection.md)。

## 从源码构建

本机复现环境为 JDK 21、Android SDK 36、build-tools 36.0.0；Java 编译目标仍为 17。在仓库根目录执行：

```bash
cd apps/ustc-android-demo
./gradlew --no-daemon :app:testDebugUnitTest :app:lintDebug :app:assembleDebug
```

需要带校验文件、签名检查和构建信息的本地包时，回到仓库根目录：

```bash
./scripts/build_android_demo.sh --source-commit local --output-dir ./dist/android-local
```

`local` 表示本地构建。只有从已核对的干净提交构建时，才用 `--source-commit <SOURCE_SHA>` 绑定来源；不能给含未提交修改的 APK 标上旧提交身份。

## 当前验证结果

截至 2026-09-06：

| 检查 | 结果 |
| --- | --- |
| 本机 JDK 21 / SDK 36 | 4 项单元测试、lint、APK 构建和签名校验通过 |
| 小米 API 35 真机安装 | 机主调整安装许可后，同源调试 APK 安装成功 |
| 当前真机检查 | Activity 启动、Chat 首页和 WebView 日历查询通过，应用崩溃缓冲未见 fatal |

本次候选来自 `a1988e8`，独立导出 Android 源码新构建；APK 为 886356 字节，debug 签名，DEX 内来源标识已核对。手机端日历请求返回 HTTP 200 与 `calendar-proposals/v1`，临时 ADB/CDP 转发在检查后移除。原始记录随提交包 `Android/android-check.json` 与 `webview-query.json` 保留。未执行完整模型回合、键盘／返回键／旋转／进程重建回归；本地提交包不等于公开 APK Release。

<details>
<summary>历史来源绑定候选与模拟器证据</summary>

以下身份仅对应旧候选，不代表当前修改已打包：

```text
Source: ee8cbc2138184651e32f955efbfec7462a3270e2
APK:    ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk
SHA-256: 83df5784e05bfefd9e16d8b41b05c9ba0f1ba29b589111869fa16475557baf31
Size:   886296 bytes
```

构建及 API 35 模拟器 Chat 证据见 [Actions 33850505578](https://github.com/Develata/ustc-campus-agent/actions/runs/33850505578)，对应源码 CI 见 [33851287216](https://github.com/Develata/ustc-campus-agent/actions/runs/33851287216)。

</details>

## 地址与退出

“服务器”接受回环 HTTP 地址或不含路径的 HTTPS origin，拒绝远程 HTTP、凭据、路径、查询和片段。当前没有可据此使用的生产 HTTPS／认证服务。

```bash
adb reverse --remove tcp:8787
adb uninstall com.develata.ustccampusagent.debug
```

卸载不删除主机数据。客户端范围、Dioxus、生产签名、远程认证和真机验收要求见[客户端契约](../contracts/client-shell.md)及 [Android 功能说明](../features/07-android-demo-client.md)。
