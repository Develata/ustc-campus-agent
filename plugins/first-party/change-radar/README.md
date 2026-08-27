# USTC ChangeRadar

- Package ID: `ustc.change-radar`
- Status: partial executable source-revision/semantic-diff/review/Atom plus fixture-backed M10/Market/Harness/ToolGateway/loopback-Web evidence; durable restart remains planned
- Product question: What changed, and does it affect me?

Canonical documentation:

- product behavior: [`docs/features/02-ustc-change-radar.md`](../../../docs/features/02-ustc-change-radar.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- source authority: [`docs/plan/05-campus-trust-kernel.md`](../../../docs/plan/05-campus-trust-kernel.md)

The package-owned Rust kernel lives at `crates/change-radar`; M60-owned immutable revision values and revision-health type live at `crates/platform-core/src/source_revision.rs`. It currently proves canonical-URL-bound deterministic `DemoReviewed` revisions, exact-source-pinned semantic comparison, complete-policy-bound candidate identity, administrator approve/reject receipts, coherent transaction-current M60 verification, exactly-once in-memory publication and deterministic Atom rendering. `apps/ustc-agentd` now supplies one fixed first-party `change.list` adapter behind current Market projection/recheck, the bounded Harness/ToolGateway sequence and a thin loopback Web/Atom projection. M00-authorized administration, durable restart, M80 peer clients and real source retrieval remain absent; approved semantic changes alone enter Atom.
