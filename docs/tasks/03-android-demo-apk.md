# M80 bounded Android demo APK

## Task authority

- `Status`: implementation candidate; exact-head CI evidence pending
- `Module`: `M80 Client Core and Interaction Shells`
- `Small module`: bounded pre-Dioxus sub-slice of `platform-android`
- `Base commit`: `a4b08c92697866e9f1e44f37ebedbfd95940a04b`
- `Acceptance`: bounded artifact gate in this taskbook; long-horizon `CLIENT-002` remains planned
- `Integration impact`: presentation adapter over the existing loopback composition

## Goal

A competition judge can install a source-bound Android debug APK, connect it to the already-running loopback Rust MVP through explicit ADB reverse, and complete a real Agent tool journey. The phone remains a thin presentation shell; all Agent, Plugin, source, permission and durable-state authority stays in `ustc-agentd`.

## Inputs and outputs

Inputs:

- validated server origin;
- existing same-origin Web MVP and HTTP API;
- Android WebView runtime;
- explicit ADB reverse transport for the local demo.

Outputs:

- debug-signed APK;
- SHA-256 checksum and build metadata;
- signature/manifest evidence;
- emulator install/launch/CDP journey evidence and screenshot.

## Writable scope

- `apps/ustc-android-demo/**`
- `scripts/build_android_demo.sh`
- `scripts/smoke_android_demo_emulator.sh`
- `scripts/test_android_webview_cdp.py`
- `.gitignore`
- Android-related projections in root README and `docs/{README,plan/modules/80-*,contracts/client-shell,features/06-*,features/07-*,guides/android-demo,tasks/03-android-demo-apk.md}`

## Non-goals

- Dioxus migration or final shared graphical client;
- production signing, Play Store, tag, Release or public deployment;
- remote cleartext service or weakening the server loopback policy;
- production authentication, secure token storage or USTC SSO;
- on-device Agent/tool/domain execution or offline authority;
- claiming complete `CLIENT-002`, real-device, reconnect/version-skew or Custom Tab evidence.

## Acceptance

The bounded artifact is deliverable only when one exact source snapshot proves all of:

1. endpoint validator unit tests and Android lint pass;
2. APK assembles with source SHA embedded and verifies under `apksigner`;
3. manifest exposes only the intended debug application and launchable Activity;
4. API 35 emulator installs and launches the exact APK;
5. `adb reverse tcp:8787 tcp:8787` reaches the real loopback `ustc-agentd`;
6. WebView CDP observes the expected origin and completes an Affairs Chat tool call with a human-readable answer and successful trace;
7. artifact, checksum, build metadata, logs and screenshot upload together;
8. independent review finds no unresolved blocker.

Failure is explicit: invalid endpoint, SSL failure, unavailable server or unsupported navigation produces no hidden local fallback and no success claim.

## Local constraint

The Hermes host has no JDK/Android SDK and less than the repository's 10 GiB Rust-build safety threshold. Local work is limited to source/static contract checks; APK build, install and real-path evidence run on a source-bound remote builder. The normal USTC 107 channel was attempted first and timed out before authentication, so the bounded fallback is an isolated GitHub-hosted builder without changing the repository's governed workflows.
