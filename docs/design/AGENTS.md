<!-- Parent: ../AGENTS.md -->

# Design layer governance

## Role

`docs/design/` holds UI/presentation design packets. It is a subordinate presentation layer: packets propose how already-defined product and platform semantics are presented and reviewed. `docs/design/` is not a peer of `plan/`, `contracts/`, `features/` or `acceptance/` and never owns product, domain or lifecycle authority.

## Packet status

Each packet carries exactly one status:

- `Proposal` — under review; nothing is accepted; no implementation, readiness or acceptance claim follows from it.
- `Reviewed` — terminal design review completed; still presentation guidance, not product authority.
- `Superseded` — replaced by a newer packet; retained for history only.

## Source binding

Every packet MUST bind to an exact source commit and tree of the repository revision it was designed against, and MUST record source drift when the bound revision moves. Presentation semantics quoted from plans, contracts or code are cited, not redefined.

## Authority deferral

- Plans, contracts and acceptance rows own behavior, state and proof; a design packet MUST NOT change their status or semantics.
- A design packet MAY mark a wire/runtime concept as an unresolved question for the owning module; it MUST NOT answer it as authority.
- `Proposal` packets carry no readiness claim; reviewers decide promotion, not the packet.

## External assets and prototypes

A packet MAY include a disposable static prototype or other review assets. Such assets are review artifacts only: non-product, not a retained frontend, with no backend/API and no readiness evidence. They MUST be marked as such in the packet index and in the asset itself.

## Packet index

`docs/design/README.md` indexes every packet with its status, source binding and scope. A new packet is added only with real content; no empty placeholder trees.

## Design projection coverage

Packet coverage claims (surfaces, artifacts, open questions) are recorded inside the packet and indexed here; they do not enter the acceptance matrix and do not promote any acceptance row.
