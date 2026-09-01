# M60-B1 `source-import/v1` lifecycle implementation

## Mutable state

- `Stage`: `VERIFIED_READY_FOR_PR`
- `Disposition`: `GO`
- `Bound source commit`: `f29e7625267e62039f27dc79076e92cf6078ef12`
- `Bound source tree`: `98d18a2451596a5d58c78b1ae5e5c985c525a63b`
- `Contract`: accepted `source-import/v1` under `ACCEPT_EXACT_M60_B2_R11_PACKET`
- `B1 status`: bounded pure lifecycle prerequisite implemented
- `M60 status`: `planned`
- `M60-B2 status`: accepted contract plus first bounded offline pure-policy implementation; transport/effects remain separately gated and unimplemented
- `Acceptance`: `SRC-001` remains `implemented`; `SRC-010`, `SRC-011`, `SRC-012` remain `planned`; `SRC-014` remains catalog-only/non-admitted
- `Concrete source`: `ustc-teach-calendar-fall` remains `Proposed`
- `Rust seed source sha256`: `0ab5ad85a3a816ad0146b1f57b6957d1d432738ca46c4c1bf2181441d6fdb7b6`
- `Rust seed integration-test sha256`: `36978060eedecdcb00b348f676262ea2a77c808b6980581f399ec5530fed7e33`
- `Semantic packet`: `sha256:755c6290845a4925d233c4ace4079a461a4bdecc820fbf48bb91d39ec210fa07` over `3547` bytes beginning immediately after the `BEGIN` marker newline and ending immediately before the `END` marker token, including the final packet newline

<!-- M60_B1_V1_LIFECYCLE:BEGIN -->
## 1. Exact bounded implementation

`M60-B1 source-registry` implements the accepted `source-import/v1` lifecycle as a pure in-memory domain kernel:

- `SourceStatus = Proposed | Approved | Suspended | Revoked`;
- non-zero monotone `SourceAuthorityRevision` initialized to `1` by initial `propose`;
- initial `propose(definition)` takes no expected revision because no admitted source revision exists yet;
- every post-proposal lifecycle mutation (`revise`, `approve`, `suspend`, `reinstate`, `revoke`) requires exact-revision CAS and checked increment;
- six-field `SourceRetrievalPolicy`, including the closed one-variant `PublicIpPolicyVersion` inventory;
- evidence-bearing transitions and sealed `RetrievalSubject` projection only from current `Approved` state;
- failed duplicate, stale-revision, illegal-transition, revision-exhaustion and non-retrievable operations leave the registry unchanged.

The exact public v1 type inventory is:

```text
SourceId
SourceOwner
SourceUrl
SourceReviewerId
SourceReviewEvidenceId
SourceStatusEvidenceId
SourceAuthorityRevision
SourceMediaType
SourceRetrievalProtocolVersion
PublicIpPolicyVersion
SourceRetrievalPolicy
SourceReviewReceipt
SourceStatus
SourceStatusKind
SourceTransitionCommand
SourceDefinitionBody
SourceDefinition
RetrievalSubject
SourceValueErrorKind
SourceValueError
SourceRegistryError
SourceRegistry
```

Historical `SourceReviewState`, the two-field retrieval policy, the v0 `approve` signature, `SourceNotApproved`, aggregate `SourceRegistry: Clone`, compatibility aliases and unchecked/mutable construction are absent.

## 2. Projection truth

```text
M60-B1 source-registry implements source-import/v1 as a bounded pure lifecycle prerequisite.
M60 remains planned.
M60-B2 retrieval remains unimplemented and separately gated.
SRC-001 remains implemented under the same exact test command.
SRC-010, SRC-011 and SRC-012 remain planned.
SRC-014 remains catalog-only / non-admitted.
No concrete USTC source is approved and no network path exists.
```

The historical `source-import/v0` §15, the accepted R11 M60-B2 semantic packet and the Roadmap W1 grant block remain immutable historical evidence. The R11 packet phrase “all with `expected_authority_revision` CAS” is represented by its own exact constructor and operation signatures: initial proposal is the no-expected-revision creation exception, while every post-proposal lifecycle mutation uses CAS.

## 3. Non-effects and stop boundary

This slice performs no source retrieval, DNS, socket, TLS or HTTP work; approves no concrete source; adds no dependency, manifest, lockfile, module, adapter, persistence, clock, random ID, source-byte digest, parser, normalizer, snapshot, baseline, publication or product-feed path. It grants no push, PR, merge, tag, release, deployment or publication authority.

`source-retrieval/v0` remains accepted contract authority only. M60-B2 retained implementation, M60-B3 through M60-B8 effects and every real-source action require separate admission.

## 4. Verification boundary

The same active `SRC-001` command remains:

```text
cargo test --locked -p ustc-campus-agent-core --test source_registry
```

The implementation also carries the internal overflow test `revision_overflow_is_revision_exhausted_without_mutation`. This local projection worker runs only the admitted Python checker gates and `git diff --check`; it does not claim local Cargo, rustfmt, Clippy, Rust test or doctest evidence. Authoritative Rust evidence remains parent-owned and bound to the frozen Rust seed.
<!-- M60_B1_V1_LIFECYCLE:END -->
