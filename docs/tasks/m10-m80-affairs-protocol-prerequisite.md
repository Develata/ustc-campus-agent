# M10-B1/B7 Affairs-first protocol prerequisite for formal M80 Web

## Task authority

- `Status`: retained bounded implementation prerequisite; supporting partial evidence only, with no production or acceptance-status promotion
- `Owning modules`: `M10 Application Ingress Host` (`B1 ingress-registry`, `B7 client-contract`) and consuming `M80 Client Core and Interaction Shells`
- `Source commit`: `ac9a38f6a979f03d88676cdb512e1103519b7bd4`
- `Source tree`: `d28ebfc3dbfb843f987fdeab6c3ba1f673e8b84e`
- `Branch`: `feat/m10m80-affairs-protocol-v1`
- `Approved direction`: Direction A — M10 owns the framework-neutral operation/protocol carrier and compatibility decision; M80 owns only client-side semantic reduction over it
- `Acceptance posture`: supporting partial evidence for planned `CLIENT-007` and `CLIENT-009`; neither row is promoted by this slice

This taskbook schedules an approved public protocol prerequisite. It does not redefine product/domain authority, and it grants no remote operation, release, deployment or publication authority.

## 1. Goal / ISA

A formal M80 Web or CLI client can bootstrap one M10 server without already knowing a version header, learn protocol major `1` and the exact Web/CLI Affairs-first operation subset, and call `affairs.get` only after M10 accepts its declared major. An older major receives typed `upgrade_required`; a newer, absent or unparseable major receives typed `incompatible_protocol`; both outcomes occur before application dispatch. M80 preserves those server-owned outcomes in the existing `ustc-client-result/v1` outer result and does not infer compatibility or domain authority itself.

## 2. Frozen public protocol

### 2.1 Version and header

- Current protocol major: `1`.
- Supported major set: exactly `[1]`.
- Minimum supported client major: `1`.
- Header on version-gated HTTP operations: `X-USTC-Client-Protocol-Major`.
- `GET /api/v1/server/info` is the sole bootstrap exception and MUST NOT require the header.
- Every other route in this slice requires one base-10 `u16` header value.

Admission is exact:

| Presented major | Typed outcome | HTTP projection | Application dispatch |
|---|---|---:|---|
| `1` | admitted | operation-owned status | allowed |
| `< 1` | `upgrade_required` | `426 Upgrade Required` | zero |
| `> 1` | `incompatible_protocol` | `409 Conflict` | zero |
| absent, repeated-invalid or unparseable | `incompatible_protocol` with unknown presented major | `409 Conflict` | zero |

HTTP status is an adapter projection of the typed compatibility or domain terminal. It is not the authority value consumed by M80.

### 2.2 Operation registry

The retained registry is a closed, deterministic projection containing exactly:

| Operation | Permission | Effect | Method and route | Header | Adapters |
|---|---|---|---|---|---|
| `server.info` | `public_read` | `read` | `GET /api/v1/server/info` | bootstrap-exempt | Web, CLI |
| `capability.list` | `public_read` | `read` | `GET /api/v1/client/capabilities` | required | Web, CLI |
| `affairs.get` | `public_read` | `read` | `GET /api/v1/affairs/{procedure_id}?as_of=<unix-ms>` | required | Web, CLI |

The projection carries only closed operation/schema/permission/effect/route/adapter values and the registry revision. It exposes no tenant grants, operator commands, bearer capabilities, concrete handlers, executor routes or arbitrary operation dispatch. Unknown operation/schema/adapter variants fail Serde decoding.

### 2.3 Result and error carrier

- M10 continues to produce the existing typed `ClientResponseDto` domain responses; the Affairs terminal remains `M71TerminalDto` and is not flattened into HTTP status or reclassified by M80.
- `server.info`, `capability.list` and protocol compatibility are added as closed `ClientResponseDto` variants in the M10-owned `client-protocol` carrier.
- M80 reduces those variants into typed bootstrap/capability/upgrade/incompatible client states.
- M80 machine rendering retains the existing outer schema string `ustc-client-result/v1`; this slice creates no `v2` or transport-specific result envelope.
- `upgrade_required` includes the presented, minimum and server major.
- `incompatible_protocol` includes the optional presented major plus the exact supported-major set.

## 3. Ownership and call order

```text
HTTP bootstrap
  GET server.info
  → M10 typed server info (no application/domain dispatch)

Version-gated operation
  parse protocol-major header
  → M10 compatibility admission
  → reject typed compatibility outcome with zero dispatch
  | admit exact major 1
  → select one closed operation route
  → affairs.get recomputes the existing payload digest
  → existing M00/M10/application/domain path
  → existing typed M71 terminal

M80
  consumes M10 response
  → exhaustively reduces the server-owned variant
  → renders ustc-client-result/v1
```

`M10` and `application-ingress` MUST NOT depend on `client-core`. `client-core` may depend only on the M10 protocol carrier and its existing narrow data dependencies.

## 4. Writable scope

Expected paths:

- `docs/tasks/m10-m80-affairs-protocol-prerequisite.md`
- `docs/plan/modules/20-application-api-host.md`
- `docs/plan/modules/80-dioxus-multi-client.md`
- `docs/contracts/interfaces.md`
- `docs/contracts/client-shell.md`
- `docs/contracts/module-boundaries.md`
- `docs/acceptance/matrix.tsv`
- `scripts/check_repo_contracts.py`
- `crates/client-protocol/src/` and focused `crates/client-protocol/tests/`
- `crates/application-ingress/src/` and focused `crates/application-ingress/tests/`
- `crates/client-core/src/` and focused `crates/client-core/tests/`
- `apps/ustc-agentd/src/web.rs`, the retained thin-shell header in `apps/ustc-agentd/src/web/app.js`, and focused `apps/ustc-agentd/tests/affairs_web.rs`

No shared M40 or Opportunity implementation path may be modified.

## 5. Non-goals

- Dioxus dependency, components, routes, visual system or frontend design;
- Android, inbound MCP, events, streams, reconnect, cancellation or production authentication;
- ChangeRadar or Opportunity operation admission into this registry;
- generic arbitrary-operation, URL, database, process, executor or operator endpoint;
- changing Affairs/M71 domain terminal meaning, Market/ToolGateway authority, public capability semantics or durable publication state;
- production TLS, remote exposure, Docker Compose, release, status promotion, staging, commit, push or PR.

## 6. Required evidence

1. Golden JSON pins `server.info`, the three-entry capability registry and both compatibility outcomes.
2. Serde tests reject unknown variants/fields and incoherent compatibility or registry values.
3. Application-ingress tests prove old/new/unknown majors invoke a counting dispatch closure zero times and major `1` invokes it once.
4. M80 reducer tests prove server-owned info/capability/upgrade/incompatible variants reduce exhaustively and render under `ustc-client-result/v1` without recalculating major relations.
5. Web tests prove bootstrap without a header; exact capability projection; `426` for old major; `409` for newer/missing/malformed major; no successful Affairs path without major `1`; and `as_of` reaches the existing typed M71 cutoff behavior.
6. Dependency confinement remains green; no M10→M80 dependency or forbidden domain/server dependency enters `client-protocol`/`client-core`.
7. Run formatter, repository contract checker, `git diff --check` and focused tests when host capacity permits. Full workspace tests are explicitly out of scope for this isolated slice.

## 7. Honest completion and Dioxus handoff

This slice may add bounded M10/M80 partial evidence only. `CLIENT-007` remains planned because Dioxus/inbound-MCP peer fixtures, events, reconnect and cancellation are absent. `CLIENT-009` remains planned because the real CLI HTTP/stream/host matrix is absent.

After this seam is green, the approved frontend lane may implement formal Dioxus Web against only these frozen facts:

- bootstrap from `GET /api/v1/server/info` without a major header;
- send major `1` on capability and Affairs calls;
- derive visible operation availability only from the safe capability projection;
- render typed M71 outcomes and M10 compatibility states without calculating campus or compatibility authority;
- keep all presentation/interaction design in the assigned Kimi K3 + Claude Opus 5 frontend lane.
