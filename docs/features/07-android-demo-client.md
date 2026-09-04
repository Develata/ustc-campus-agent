# Android demo APK

## Metadata

- `Status`: bounded implementation candidate; source-bound remote build and emulator evidence required
- `Owning module`: `M80 Client Core and Interaction Shells`
- `Contract`: [`client-shell/v2.3`](../contracts/client-shell.md)
- `A...[truncated]
- `Artifact`: debug-signed APK built from `apps/ustc-android-demo/`

## User-visible result

An Android 8.0+ user can install the APK, connect it to the existing USTC Campus Agent Web MVP, and use the same Agent Chat, Affairs, ChangeRadar, Course Planning and Simple Calendar journeys inside an Android `WebView`.

For the bounded local demonstration, the phone or emulator reaches the host's loopback-only Rust service through:

```bash
adb reverse tcp:8787 tcp:8787
```

The app shows explicit loading, offline and retry states. The **服务器** control accepts the default loopback HTTP origin or a path-free HTTPS origin. It rejects remote cleartext HTTP, URL credentials, paths, queries, fragments and non-Web schemes.

## Authority and security

The Android app is a presentation and navigation adapter only:

```text
Android Activity / WebView
→ same-origin HTTP page and API requests
→ ustc-agentd M10/Agent/Market composition
→ Rust-owned authority and durable state
```

It contains no JavaScript bridge and no campus, permission, tool-routing or persistence implementation. File/content access is disabled, mixed content is denied, third-party cookies are disabled, SSL errors fail closed, and only the configured origin stays in the WebView. Other HTTP(S) links are handed to an external browser.

Cleartext traffic is admitted only for `127.0.0.1` and `localhost`, enabling the explicit ADB reverse demo without weakening arbitrary remote origins.

## Verification

`scripts/build_android_demo.sh` binds the exact source SHA into `BuildConfig`, and `scripts/smoke_android_demo_emulator.sh` exercises the resulting bytes. The source-bound remote builder then:

1. runs endpoint validation unit tests and Android lint;
2. assembles the debug APK with Gradle 9.6.0, AGP 9.4.0 and JDK 17;
3. verifies signature, package and launchable activity;
4. starts the real Rust Web MVP;
5. boots an API 35 emulator and installs the APK;
6. uses `adb reverse` and launches `MainActivity`;
7. connects to the WebView CDP socket and completes an Affairs Chat tool journey;
8. uploads the APK, checksum, build metadata, logs and emulator screenshot as a source-bound Actions artifact.

## Deferred

This bounded bridge does not complete long-horizon `CLIENT-002`. The following remain separate work:

- Dioxus shared Web/Android presentation source and typed client-core parity;
- production signing and store-ready release packaging;
- authenticated public HTTPS deployment and secure session storage;
- real-device lifecycle, process recreation, reconnect and version-skew evidence;
- Android Custom Tabs for admitted external navigation;
- standalone/offline backend execution, which is explicitly not implied.
