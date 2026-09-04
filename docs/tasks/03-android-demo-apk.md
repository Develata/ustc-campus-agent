# M80 bounded Android demo APK

## Task authority

- `Status`: bounded artifact delivered; exact-source CI and source-bound emulator evidence complete
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

## Completion receipt

- Source commit: `ee8cbc2138184651e32f955efbfec7462a3270e2`.
- APK: `ustc-campus-agent-android-debug-ee8cbc2138184651e32f955efbfec7462a3270e2.apk`.
- APK SHA-256: `83df5784e05bfefd9e16d8b41b05c9ba0f1ba29b589111869fa16475557baf31`; size: 886296 bytes.
- Source-bound build/emulator gate: [Actions run 33850505578](https://github.com/Develata/ustc-campus-agent/actions/runs/33850505578), success. The isolated builder workflow commit is not the source identity; `build-info.json`, the artifact filename and embedded `BuildConfig` bind the output to the source commit above.
- Product-branch exact-source CI: [Actions run 33851287216](https://github.com/Develata/ustc-campus-agent/actions/runs/33851287216), success for the source commit above.
- Governance controller: [Actions run 33853176792](https://github.com/Develata/ustc-campus-agent/actions/runs/33853176792), success for the SHA-bound approval event.
- Retained emulator evidence: endpoint unit tests, `No issues found.` Android lint, signature/manifest verification, `adb install` success, Activity launch, process presence, screenshot and `android-webview-smoke: PASS` for the Affairs Chat journey.
- Independent exact-candidate review: `PASS`; no unresolved blocker. This receipt does not authorize merge, tag, Release or deployment.

The long-horizon `CLIENT-002` acceptance remains planned. Physical-device evidence, production signing, authenticated remote HTTPS deployment, Dioxus parity, lifecycle/reconnect conformance and store publication are not implied by this receipt.

## Build-environment boundary

The delivery host did not provide the complete Android SDK/emulator path required for authoritative APK evidence. Build, install and real-path verification therefore ran on a source-bound GitHub-hosted builder without adding its one-shot workflow to the product candidate. This is remote emulator evidence, not a claim of local or physical-device validation.
