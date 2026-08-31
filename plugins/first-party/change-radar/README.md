# USTC ChangeRadar

- Package ID: `ustc.change-radar`
- Status: partial executable durable administrator-publication plus M10/Market/Harness/ToolGateway/loopback-Web JSON/Atom evidence; production SSO/live source/M80 peers remain planned
- Product question: What changed, and does it affect me?

Canonical documentation:

- product behavior: [`docs/features/02-ustc-change-radar.md`](../../../docs/features/02-ustc-change-radar.md)
- engineering contract: [`docs/plan/06-first-party-plugins.md`](../../../docs/plan/06-first-party-plugins.md)
- source authority: [`docs/plan/05-campus-trust-kernel.md`](../../../docs/plan/05-campus-trust-kernel.md)

The package-owned Rust kernel lives at `crates/change-radar`; M60-owned immutable revision values and revision-health type live at `crates/platform-core/src/source_revision.rs`. It proves source-pinned semantic comparison, digest-bound review, coherent M60 verification and deterministic JSON/Atom. `apps/ustc-agentd` adds one fixed `M10 → M00 durable evidence → owning M70` administrator publication path over a strict owner-only canonical repository; exact retry, checked zero-M60 restart recovery, uncertain-rename reconciliation and fail-closed corruption handling preserve one receipt/GUID/item across real process and browser restart. Ordinary `change.list` remains behind current Market/Harness/ToolGateway. Production SSO/admin, M80 peers, maintainer leases and approved live retrieval remain absent; only reviewed semantic changes enter Atom.
