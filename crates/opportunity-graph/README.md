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

The current adapter is in-memory and fixture-backed. It is **not** the complete
Opportunity Graph product path: no M10/M20/M30/M40/M80 composition, no durable
private repository or backup-erasure proof, no M00 consent UI/command, no live
M60 retrieval, and no installed Plugin execution are claimed. The retained
Course Planning pack remains one domain pack rather than the module identity.
