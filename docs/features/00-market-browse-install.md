# Market browse, install and control

- `Status`: Planned user journey; manifest validation exists
- `Owning plan`: `docs/plan/04-market-and-plugin-lifecycle.md`
- `Contracts`: `docs/contracts/plugin-package.md`, `docs/contracts/permissions.md`
- `Acceptance`: `MARKET-*`, `FP-006`, `FP-015`, `FP-007`

## Goal

A user can inspect a Plugin as a concrete package, understand its publisher/version/components/capabilities/source policy, install an exact version, and control whether the Agent may discover it.

## User-visible states

```text
Available
Installed + Enabled
Installed + Disabled
Update requires approval
Revoked / unavailable
Error with recovery action
```

The UI never displays a package as installed or runnable solely because its manifest has a default-install policy.

## Journey

```text
anonymous visitor browses package metadata
→ opens package detail
→ sees publisher, exact version, status, components, permissions and source policy
→ signs in before installation
→ confirms exact package and grants
→ installation becomes enabled
→ Agent can discover approved capability
→ user disables Plugin
→ discovery and invocation are denied
→ user re-enables Plugin
→ discovery is restored under current grants
```

The three default first-party packages appear as independent products and can be disabled/re-enabled independently.

## Failure and recovery copy

- Invalid or unavailable manifest: package is not installable; show validation reason without internal secrets.
- Permission expansion: show exact diff and require reapproval; never auto-enable new access.
- Disabled/revoked: explain that invocation is blocked and identify whether the user or operator can recover it.
- Version/component mismatch: stop invocation and ask the user/operator to repair or reinstall; do not route to a same-name alternative.
- Runtime unavailable: preserve installation state but show availability separately from permission state.

## Non-goals

- anonymous installation or execution;
- hidden default grants;
- arbitrary package code execution;
- treating a package card as proof of backend runtime;
- public download links before verified releases.

## Verification

Current automated evidence validates the exact three manifests and Rust identities. Installation, disable/re-enable and browser journeys remain planned until durable runtime state and a frontend exist.
