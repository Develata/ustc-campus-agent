# USTC ChangeRadar

- Package ID: `ustc.change-radar`
- Status: partial executable source-revision/semantic-diff evidence; review/feed/platform composition remains planned
- Product question: What changed, and does it affect me?

Canonical documentation:

- product behavior: [`docs/features/02-ustc-change-radar.md`](../../../docs/features/02-ustc-change-radar.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- source authority: [`docs/plan/05-campus-trust-kernel.md`](../../../docs/plan/05-campus-trust-kernel.md)

The package-owned Rust foundation lives at `crates/change-radar`; M60-owned immutable revision values and revision-health type live at `crates/platform-core/src/source_revision.rs`. It currently proves only canonical-URL-bound deterministic `DemoReviewed` revisions, exact-source-pinned semantic comparison, complete-policy-bound candidate identity and service-minted atomic updates to an explicitly bounded in-memory baseline/candidate repository. This directory will carry package resources and adapters as the real journey lands. Maintainer work remains candidate-only; approved semantic changes alone enter RSS/Atom.
