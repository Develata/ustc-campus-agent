# USTC ChangeRadar

- Package ID: `ustc.change-radar`
- Current status: planned manifest skeleton; no executable component is claimed
- Product question: What changed, and does it affect me?

ChangeRadar is engineering-first because its source/revision/diff foundation is shared by Affairs Navigator and Opportunity Graph.

## First implementation contract

- one individually reviewed public USTC source pair;
- stable source and revision identities;
- conditional retrieval and immutable raw/normalized snapshots;
- deterministic parser fixtures and semantic diff;
- fail-closed baseline advancement;
- idempotent repeated processing;
- no arbitrary URL fetch.

After the shared foundation is stable, board-scoped maintainer Agents may propose change candidates but cannot publish. Administrator-approved semantic changes materialize into per-board RSS/Atom with stable GUID, effective time, affected scope, provenance, and before/after summary. Raw HTML/hash noise, parser failure, and unreviewed inference never enter the feed.
