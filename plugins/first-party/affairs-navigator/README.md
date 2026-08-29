# USTC Affairs Navigator

- Package ID: `ustc.affairs-navigator`
- Package status: Market manifest skeleton; no production executable component wired
- Module evidence: `partial-evidence`
- Product question: What should I do now?

The package-owned Rust crate at `crates/affairs-navigator` contains bounded
query/freshness/conflict behavior and an exact M60 `DemoReviewed` draft →
administrator review → atomic publication kernel. The bounded `ustc-agentd`
composition now admits one fixed administrator republish command through
`M10 → M00 admission/durable evidence → M71`, stores checked canonical recovery
records, survives real process restart with the same revision/receipt, and
serves ordinary reads through the separate Market/Harness/ToolGateway query
spine plus loopback HTTP/Web. This closes bounded `PROC-011` only: production
SSO, live retrieval, generic administrator content management and public
network exposure remain missing.

Canonical documentation:

- product behavior: [`docs/features/01-ustc-affairs-navigator.md`](../../../docs/features/01-ustc-affairs-navigator.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- source authority: [`docs/plan/05-campus-trust-kernel.md`](../../../docs/plan/05-campus-trust-kernel.md)
- module blueprint: [`docs/plan/modules/72-affairs-navigator.md`](../../../docs/plan/modules/72-affairs-navigator.md)

This directory carries only package-owned metadata/resources. It does not own a
crawler, generic source authority, authentication policy, or client shell.
