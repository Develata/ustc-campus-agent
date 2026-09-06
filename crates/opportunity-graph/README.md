# M72 Opportunity Graph — consent/profile foundation

Status: `partial-evidence`.

This crate separates one immutable `DemoReviewed` opportunity catalog from an
explicitly consented tenant-private academic profile. It provides:

- exact consent fields and deterministic consent/profile identities;
- a bounded profile repository with one active snapshot per tenant/user;
- fail-closed tenant/user checks before M60 or planning access;
- deterministic course qualification and Course Planning reuse;
- source/profile-pinned planning receipts and stale classification;
- atomic consent revocation + private-payload deletion with tombstones;
- typed stale/unavailable source and repository failures.

The crate's reference adapter is in-memory and fixture-backed. The sibling
`ustc-agentd` composition adds typed profile/plan/delete operations, current grants,
a durable private-profile adapter and Web interaction; see the
[current product scope](../../docs/features/03-campus-opportunity-graph.md).
Live retrieval, production authentication and backup-erasure proof remain unfinished.
Course Planning remains one domain pack rather than the module identity.
