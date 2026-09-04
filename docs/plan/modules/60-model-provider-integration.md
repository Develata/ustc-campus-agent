# M50 — Model Provider Integration

## Metadata

- `Module ID`: `M50`
- `Status`: Accepted blueprint; bounded Chat MVP adapter exists, complete provider platform planned
- `Implementation State`: `partial-evidence`
- `Version`: `m50-model-provider/v0`
- `Last Review`: `2026-09-03`
- `Primary code area`: current bounded adapter in `apps/ustc-agentd/src/chat_provider.rs`; future extraction into replaceable provider modules under `crates/adapters/` or a dedicated crate after two real consumers

## 1. Purpose

`M50` turns one platform-owned, fully assembled model request into normalized provider events and usage. It owns typed provider profiles, profile validation, protocol adaptation, streaming/final parity, token/context estimation and provider-specific error mapping.

Current evidence is deliberately narrower than module completion: the app-private Chat MVP owns a deterministic no-network mock plus one bounded non-streaming operator-configured OpenAI-compatible adapter, complete-request context preflight, typed errors and redacted file-backed secret handling. Generic provider profiles, streaming/final parity, normalized real usage and the full M30↔M50 port/conformance suite remain planned.

It transports a request. It does not own why the request exists or what the run should do next.

## 2. Non-goals

- deciding Agent phases, task graphs, tool grants or completion;
- assembling canonical conversation truth;
- silently selecting another provider/model on failure;
- exposing arbitrary URL/header/body proxying;
- storing literal keys in profiles, logs or events;
- making provider SDK checkpoints authoritative.

## 3. Owned objects and state

```text
ProviderProfile
ProviderMode: OfficialCentral | UserCloud
ProviderKind / fixed origin / model identity
ProviderCapabilitySnapshot
EstimatorIdentity / ContextLimit
ProfileValidationState
ModelRequestEnvelope
ModelStreamEvent / ModelFinalResponse / ModelUsage
ProviderError
```

Reserved later modes such as device/remote relay are not active profile variants until separately reviewed.

## 4. Public inputs and outputs

Input from `M30`:

```text
profile snapshot ID
complete serialized messages/policy/tool definitions/schema/attachments
maximum output and deadline/cancellation
run/turn/correlation IDs
```

Output:

```text
ordered text/tool/usage stream events
one normalized final outcome
exact provider-reported usage when available
stable error and retry classification
```

The adapter cannot add tools, rewrite system policy or drop provider-visible fields to fit context.

## 5. Dependency direction

Allowed dependencies:

- provider-neutral request/event contract declared for `M30`;
- reviewed provider SDK/protocol adapters;
- `M90` HTTP, secret-ref, clock, telemetry and cancellation ports;
- `agent-tool-protocol` provider-visible definitions only.

Forbidden dependencies:

- Market/Plugin/executor internals;
- run transition mutation beyond returned events;
- Dioxus/client state;
- concrete `M30` private checkpoint or graph state;
- generic user-controlled outbound HTTP.

## 6. Lifecycle

```text
profile declared
→ endpoint/model/capability/secret-ref validation
→ estimator/context-limit validation
→ Active | Invalid | Disabled
→ request preflight by M30
→ provider invocation
→ normalized stream/final/usage
→ capability drift revalidation | disable/delete
```

Profile changes produce a new snapshot. An in-flight run retains its pinned profile semantics or fails explicitly.

## 7. Failure and recovery

- Invalid/missing/disabled profile: no provider I/O.
- Secret resolution failure: stable blocked error with no secret leakage.
- DNS/redirect/TLS/origin violation for `UserCloud`: reject validation/call.
- Timeout/rate limit/unavailable: typed retry class under `M30` budget; no silent fallback.
- Malformed stream/protocol response: fail the turn and preserve raw data only under safe bounded diagnostics.
- Provider context-overflow after local pass: record estimator drift and perform only policy-bounded rebuild behavior owned by `M30`.
- Stream disconnect: final state remains unknown/failed until protocol evidence proves completion.

## 8. Configuration and secrets

A profile stores owner, mode, provider kind, fixed normalized origin, model, `SecretRef`, capability snapshot, context limit and estimator identity. The running central service may resolve and use centrally stored secret references; documentation must not claim end-to-end secrecy from that service.

## 9. Observability

Record profile/model/adapter snapshot, request correlation, first-token/final latency, normalized usage, retry class, stream completion and estimator drift. Prompt/tool/private content telemetry is off by default and never part of normal metrics.

## 10. Extension and replacement

Each provider adapter implements one equal contract and conformance suite. Rig or another SDK may be used inside an adapter after official-source and intrusion review; it cannot own run state. New provider kinds are peers, not branches inside Agent logic.

## 11. Performance path

Hot paths are request serialization, token measurement, streaming decode and bounded buffering. The adapter uses backpressure and avoids duplicating large request/content buffers where possible. Context measurement covers the complete provider-visible request.

## 12. Scope boundary

**MVP**

- `OfficialCentral` and `UserCloud` profile contracts;
- one platform-selected provider adapter;
- streaming/final/usage normalization;
- exact context limit + compatible estimator;
- timeout/cancel/rate-limit/error mapping;
- no silent fallback and redacted telemetry.

**Later**

- additional provider peers;
- user device/remote relay after offline/secret/threat contracts;
- richer multimodal accounting after provider evidence.

**Explicit non-goals**

- arbitrary HTTP proxy;
- provider-managed run truth;
- model-name heuristic context limits;
- automatic provider/key/model substitution.

## 13. Small-module decomposition

1. `provider-profile` — typed owner/mode/origin/model/secret-ref state.
2. `profile-validation` — endpoint/capability/estimator checks.
3. `model-port` — provider-neutral request/events/errors.
4. `token-estimator` — exact or conservative complete-request measurement.
5. `stream-normalizer` — ordered events and final convergence.
6. `usage-normalizer` — exact reported usage/cost units.
7. `provider-http-safety` — fixed-origin/redirect/DNS/TLS boundaries.
8. one peer adapter per admitted provider.
9. `provider-conformance` — cassette/fake protocol fixtures.

## 14. Exit gate

`M50` is integration-ready when one adapter and one fake pass equal-contract tests for normal, stream, tool call, malformed, timeout, cancel, rate limit, secret failure and estimator drift. It is accepted when `M30` completes one bounded run through the adapter with stream/non-stream final parity and no fallback.
