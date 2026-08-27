# USTC ChangeRadar

- Package ID: `ustc.change-radar`
- Status: partial executable source-revision/semantic-diff/review/Atom evidence; durable platform/Web composition remains planned
- Product question: What changed, and does it affect me?

Canonical documentation:

- product behavior: [`docs/features/02-ustc-change-radar.md`](../../../docs/features/02-ustc-change-radar.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- source authority: [`docs/plan/05-campus-trust-kernel.md`](../../../docs/plan/05-campus-trust-kernel.md)

The package-owned Rust kernel lives at `crates/change-radar`; M60-owned immutable revision values and revision-health type live at `crates/platform-core/src/source_revision.rs`. It currently proves canonical-URL-bound deterministic `DemoReviewed` revisions, exact-source-pinned semantic comparison, complete-policy-bound candidate identity, administrator approve/reject receipts, coherent transaction-current M60 verification, exactly-once in-memory publication and deterministic Atom rendering. This directory will carry package resources and adapters as the real journey lands. M00 authorization, durable restart, Market/Agent/ToolGateway and Web projection remain absent; approved semantic changes alone enter Atom.
