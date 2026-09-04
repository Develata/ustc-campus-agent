# Bounded Web Chat MVP delivery taskbook

## Metadata

- `Status`: implementation and delivery tracking only
- `Date`: 2026-09-03
- `Scope`: one loopback Web Chat vertical slice over the three existing DemoReviewed/synthetic Plugin journeys
- `Normative contract`: [`../contracts/agent-chat.md`](../contracts/agent-chat.md)
- `Acceptance`: active `CHAT-001`, `CHAT-002`, `CHAT-003`
- `Decision provenance`: Develata selected “Web Chat + OpenAI-compatible provider + three-Plugin tool calling”, then requested a ChatGPT-like chat-first shell and account login while allowing administration to remain CLI-only; real campus-source activation remains forbidden and real-provider smoke requires separately supplied runtime configuration

This taskbook records implementation boundaries and fan-in order. It owns no wire schema, authority rule, provider behavior, budget, tool mapping, error code or delivery lifecycle; those are governed by `agent-chat/v1` and the active acceptance matrix.

## 1. Selected delivery slice

```text
Web Chat
→ one deployment-local login (no registration/SSO)
→ POST /api/v1/agent/chat
→ deterministic mock by default or operator-configured OpenAI-compatible provider
→ finite sequential tool loop
→ Affairs / ChangeRadar / Opportunity Graph existing product compositions
→ final answer + safe tool trace
→ loopback Docker Compose ZIP
```

The work retains reviewed/synthetic fixtures and introduces no campus retrieval, USTC source activation, CAS/SSO, production/multi-user identity, production database, generic Plugin runtime, streaming, RAG, durable chat history or multi-agent graph. The chat-first sidebar therefore exposes only New chat, the current page-lifetime conversation, a secondary campus-tools view, safe runtime/provider status and local-account logout; it never fabricates saved conversations or projects.

## 2. Implementation ownership

- `chat_provider.rs`: provider profile parsing, key-file loading, HTTP mapping and provider DTOs only.
- `agent_chat.rs`: closed request validation, finite loop budgets, provider/tool ordering and safe response projection.
- `chat_tools.rs`: exact model-visible names, schemas and mapping to pre-existing product operations; no product truth or grant ownership.
- `web.rs`: HTTP composition and stable error/status projection.
- `src/web/`: thin presentation and explicit Opportunity per-request confirmation.
- `deploy/mvp-compose/` and the package script: loopback deployment, persistent state, reset, cross-platform launchers and deterministic archives.
- `ustc-agentd` remains the single shared composition/fan-in owner.

`Cargo.lock`, shared module declarations, routes, Compose, acceptance/status projections and candidate identity are serialized through the integration owner. Partial worker output is not accepted without exact diff, compilation/tests and review.

## 3. Delivery gates

A candidate is deliverable only after all applicable `CHAT-*` bindings pass on one exact semantic head:

1. Rust format, clippy, unit/integration tests and doc tests;
2. deterministic mock direct answer plus all three sequential product tool paths;
3. Opportunity absent/unconfirmed/current/missing-profile cases;
4. OpenAI-compatible protocol/error/role/limit cassette tests without real-provider egress;
5. browser request/DOM/static behavior checks;
6. isolated Compose clean start, restart/stop persistence and explicit reset deletion;
7. relative-output packaging, archive/checksum/provenance/secret scans;
8. Windows PowerShell 5.1 parser and launcher byte/exit-code checks;
9. exact-head independent code/security review;
10. GitHub CI/governance and remote-head read-back.

A separately approved real-provider smoke may be reported in addition. Without that approval it remains `not-run` and does not block the deterministic MVP.

## 4. Fan-in sequence

1. freeze or amend the normative contract and acceptance rows;
2. implement provider, loop/tool bridge and Web projections behind one fan-in owner;
3. run local static and no-network mock tests;
4. use exact-head hosted CI for Rust, Docker Compose and Windows evidence unavailable locally;
5. remediate review findings on the product branch, invalidating prior receipts after every semantic edit;
6. package only the final reviewed head and verify the downloaded artifact independently;
7. merge or release only under separate authority.
