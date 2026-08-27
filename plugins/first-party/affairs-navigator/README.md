# USTC Affairs Navigator

- Package ID: `ustc.affairs-navigator`
- Package status: Market manifest skeleton; no production executable component wired
- Module evidence: `partial-evidence`
- Product question: What should I do now?

The package-owned Rust crate at `crates/affairs-navigator` now contains bounded
query/freshness/conflict behavior and an exact M60 `DemoReviewed` draft →
administrator review → atomic in-memory publication foundation. The bounded
`ustc-agentd` fixture now publishes and queries that same repository through
M10/CLI/loopback Web, but this does **not** form the production package journey:
M00 administrator authorization, Market-controlled activation, durable
persistence/restart and a production operator publication API remain missing.
`PROC-011` is therefore still non-pass.

Canonical documentation:

- product behavior: [`docs/features/01-ustc-affairs-navigator.md`](../../../docs/features/01-ustc-affairs-navigator.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- source authority: [`docs/plan/05-campus-trust-kernel.md`](../../../docs/plan/05-campus-trust-kernel.md)
- module blueprint: [`docs/plan/modules/72-affairs-navigator.md`](../../../docs/plan/modules/72-affairs-navigator.md)

This directory carries only package-owned metadata/resources. It does not own a
crawler, generic source authority, authentication policy, or client shell.
