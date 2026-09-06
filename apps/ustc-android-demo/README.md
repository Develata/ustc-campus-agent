# Android 演示客户端

调试版 WebView 客户端，通过 `adb reverse` 连接 `ustc-agentd`。
Agent、插件、权限和持久化仍由 Rust 服务端负责。

## 本地构建

准备 JDK 17+、Android SDK 36 与 build-tools 36.0.0。在仓库根目录执行：

```bash
bash scripts/build_android_demo.sh --source-commit local --output-dir dist/android-local
```

脚本运行端点单元测试、lint、组装、签名和 manifest 检查。
`local` 标识开发预览；正式候选应使用已确认的源码身份，不能把未提交改动标成历史候选。
本轮使用 JDK 21 构建，4 项端点测试、lint 与签名检查通过；小米安装被系统权限阻断，真机功能尚未验证。

## 安装与连接

先启动后端，再指定唯一测试设备：

```bash
adb devices -l
adb -s <SERIAL> reverse tcp:8787 tcp:8787
adb -s <SERIAL> install -r dist/android-local/ustc-campus-agent-android-debug-local.apk
adb -s <SERIAL> shell am start -n \
  com.develata.ustccampusagent.debug/com.develata.ustccampusagent.MainActivity
```

`install -r` 保留已有应用数据。默认连接 `http://127.0.0.1:8787/`；原生“服务器”控件
接受本机 HTTP 或无路径 HTTPS origin，拒绝远程明文地址、内嵌凭据和任意跳转。
连接失败时先核对后端与 reverse，再点“重试连接”。

[完整操作与历史候选收据](../../docs/guides/android-demo.md) ·
[客户端范围](../../docs/features/07-android-demo-client.md) ·
[当前功能](../../docs/features/06-mvp-core-capabilities.md)

历史 API 35 模拟器通过记录只适用于原 APK，不证明当前新增界面已在所有 Android
设备通过。生产签名、学校 SSO、最终 Dioxus 客户端和完整 `CLIENT-002` 仍待完成。
