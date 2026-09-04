# Android demo guide

The current Android artifact is a debug-signed thin client for the loopback MVP. It is intended for a controlled competition demonstration, not Play Store or public-server deployment.

Verified candidate:

```text
Source: ee8cbc2138184651e32f955efbfec7462a3270e2
APK:    ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk
SHA-256: 83df5784e05bfefd9e16d8b41b05c9ba0f1ba29b589111869fa16475557baf31
Size:   886296 bytes
```

Before installing, keep the APK and its generated `.sha256` file in the same directory and verify the exact bytes:

```bash
sha256sum -c ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk.sha256
```

## 1. Start the backend

From the repository root:

```bash
./scripts/run_three_plugin_mvp.sh
```

Wait until the service reports:

```text
Web:   http://127.0.0.1:8787/
```

## 2. Connect a device

Enable Android developer options and USB debugging, then verify exactly one intended device:

```bash
adb devices
```

Forward the device's TCP 8787 to the host's loopback TCP 8787:

```bash
adb reverse tcp:8787 tcp:8787
adb reverse --list
```

This preserves the Rust server's loopback-only boundary; the service is not opened to the LAN.

## 3. Install and launch

```bash
adb install -r ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk
adb shell am start -n \
  com.develata.ustccampusagent.debug/com.develata.ustccampusagent.MainActivity
```

The app defaults to `http://127.0.0.1:8787/`. If the native offline screen appears, confirm the backend is running, repeat `adb reverse`, and tap **重试连接**.

## 4. Exercise the MVP

In **Agent Chat**, try:

```text
成绩单证明怎么办？
校历最近有什么变更？
记录事项：提交开题报告
列出我的待办事项
```

The WebView uses the same server-owned route and state as the browser demo. The APK does not contain a second local implementation.

## 5. Endpoint policy

The native **服务器** control accepts:

- `http://127.0.0.1:<port>/` or `http://localhost:<port>/` for explicit development forwarding;
- a path-free `https://<host>[:port]/` origin for a future reviewed remote deployment.

It rejects remote HTTP, embedded credentials, paths, query strings, fragments and non-HTTP(S) schemes. The current repository does not claim a production remote HTTPS/auth service.

## 6. Remove

```bash
adb reverse --remove tcp:8787
adb uninstall com.develata.ustccampusagent.debug
```

Removing the app does not delete the host-side Rust state directory. The server launcher prints that directory when it starts.

## Build from source

With JDK 17 and Android SDK 36/build-tools 36.0.0 installed:

```bash
cd apps/ustc-android-demo
./gradlew --no-daemon :app:testDebugUnitTest :app:lintDebug :app:assembleDebug
```

For a source-bound build, run the repository helper in an environment with JDK 17 and the pinned Android SDK packages, then verify the adjacent checksum before installation:

```bash
UCA_SOURCE_COMMIT="$(git rev-parse HEAD)" \
  ./scripts/build_android_demo.sh --output-dir ./dist/android
(cd dist/android && sha256sum -c ustc-campus-agent-android-debug-<SOURCE_SHA>.apk.sha256)
```

The delivered candidate's source-bound build and API 35 emulator smoke are retained in [Actions run 33850505578](https://github.com/Develata/ustc-campus-agent/actions/runs/33850505578). Exact-source repository CI is [run 33851287216](https://github.com/Develata/ustc-campus-agent/actions/runs/33851287216). These receipts prove the bounded debug artifact only; they do not establish production signing, authenticated remote deployment or physical-device acceptance.
