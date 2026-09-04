# Android demo client

This directory builds the bounded Android demonstration APK for USTC Campus Agent. It is a thin `WebView` shell around the existing server-owned Web MVP; it does not duplicate Agent, Plugin, source, permission, Calendar, or persistence authority on-device.

## Build

Prerequisites: JDK 17, Android SDK platform 36 and build tools 36.0.0. From the repository root:

```bash
UCA_SOURCE_COMMIT="$(git rev-parse HEAD)" \
  ./scripts/build_android_demo.sh --output-dir ./dist/android
```

The helper runs unit tests, Android lint, assembly, `apksigner`, manifest checks and SHA-256 packaging. The debug APK is written under `dist/android/`; it is not a production release.

The source-bound remote gate additionally verifies the debug signature and manifest, installs the APK on an API 35 emulator, connects it to the real loopback Rust MVP through `adb reverse`, and completes one Affairs Chat journey through the WebView CDP surface.

## Verified demo artifact

The bounded candidate built from source commit
`ee8cbc2138184651e32f955efbfec7462a3270e2` is:

```text
ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk
SHA-256: 83df5784e05bfefd9e16d8b41b05c9ba0f1ba29b589111869fa16475557baf31
Size: 886296 bytes
```

The source-bound build/emulator gate is [Actions run 33850505578](https://github.com/Develata/ustc-campus-agent/actions/runs/33850505578); the product branch's exact-source CI gate is [run 33851287216](https://github.com/Develata/ustc-campus-agent/actions/runs/33851287216). Both completed successfully. The Actions workflow itself ran from an isolated builder commit; `build-info.json`, the artifact filename and the embedded `BuildConfig` source identity bind the APK to the source commit above.

## Run against the local MVP

Start the Rust service from the repository root:

```bash
./scripts/run_three_plugin_mvp.sh
```

With an Android 8.0+ device or emulator visible to `adb`:

```bash
adb reverse tcp:8787 tcp:8787
sha256sum -c ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk.sha256
adb install -r ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk
adb shell am start -n \
  com.develata.ustccampusagent.debug/com.develata.ustccampusagent.MainActivity
```

The app defaults to `http://127.0.0.1:8787/`. Its native **服务器** control also accepts a path-free HTTPS origin. Remote cleartext HTTP, credentials embedded in URLs, query strings, fragments, and non-Web schemes are rejected.

## Honest boundary

This APK is a usable, debug-signed competition/demo bridge. It provides:

- installable Android package;
- native loading, offline, retry, reload, server-origin, back-navigation and lifecycle handling;
- same-origin WebView confinement and external-browser handoff;
- the complete existing Web MVP when connected to `ustc-agentd`.

It does **not** establish final `CLIENT-002` acceptance. Production signing, Dioxus shared presentation source, secure authenticated remote sessions, real-device evidence, lifecycle/reconnect conformance, Custom Tabs and public HTTPS deployment remain planned.
