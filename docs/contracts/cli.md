# CLI contract

## Metadata

- `Status`: current operator CLI contract plus accepted phased user/automation CLI boundary
- `Version`: `cli/v2.1`
- `Last Review`: `2026-08-12`
- `Owning plans`: [`M80 Client Core and Interaction Shells`](../plan/modules/80-dioxus-multi-client.md), [`M90 Infrastructure and Operations`](../plan/modules/90-infrastructure-operations.md)
- `Counterpart contract`: [`client-shell/v2.1`](client-shell.md)
- `Acceptance`: implemented command-specific evidence below; planned `CLIENT-008` and `CLIENT-009`

## 1. Binary and privilege split

```text
ustc-agentctl
  operator / administrator / developer / repository verification

ustc-agent
  ordinary user / headless automation / least-privilege remote client
```

The two binaries are separate public and privilege surfaces.

`ustc-agent` MUST NOT expose, import or dispatch an `ustc-agentctl` command handler, operator credential profile, direct repository/database operation, source-apply operation, package-administrator mutation or local backend domain implementation.

Production services and Dioxus/MCP clients call shared Rust application/client interfaces. They do not shell out to either CLI for normal business paths.

## 2. Implemented `ustc-agentctl` commands

```bash
ustc-agentd --version
ustc-agentctl --version
ustc-agentctl doctor
ustc-agentctl market validate
ustc-agentctl course plan \
  --fixture market/fixtures/course-planning/minimal-v0.json \
  --format json
```

### `course plan`

Input:

- required `--fixture <path>`;
- optional `--format json`; JSON is the only accepted v0 format.

Output:

- `course-plan-result/v0` JSON on stdout;
- deterministic candidate ordering;
- zero exit status only when the fixture validates and at least one feasible plan exists;
- diagnostic on stderr and exit code `2` for invalid options, unreadable/invalid fixtures or no feasible plan.

The command is a bounded offline spike. It is read-only: it does not access the network, store credentials, modify external systems or enroll a user in courses. It is not evidence for `ustc-agent`, M10 or client-core.

## 3. Planned `ustc-agentctl` commands

| Command | Purpose | Side-effect policy |
|---|---|---|
| `ustc-agentctl acceptance run --gate pr` | run a named acceptance subset | evidence write only when requested |
| `ustc-agentctl source registry-check --strict` | validate reviewed source declarations | read-only |
| `ustc-agentctl source crawl-plan <source-id>` | produce a bounded retrieval plan | read-only; no cursor/baseline mutation |
| `ustc-agentctl source crawl-apply --plan <path> --plan-digest <digest>` | apply an approved retrieval plan | operator-only, idempotent, audited |
| `ustc-agentctl source diff` | compare immutable source revisions | read-only |
| `ustc-agentctl procedure validate --candidate <path>` | validate typed procedure candidate | read-only |
| `ustc-agentctl procedure publish-plan --candidate <path>` | produce deterministic publish plan | read-only |
| `ustc-agentctl procedure publish-apply --plan <path> --plan-digest <digest>` | publish an approved artifact | administrator-only, audited |

Any durable mutation must define dry-run/plan semantics, idempotency identity, authorization, receipt and recovery before implementation.

## 4. Planned `ustc-agent` initial command families

The first retained slice freezes and implements only an admitted read-only subset of:

```bash
ustc-agent --version
ustc-agent server info --format json
ustc-agent capabilities list --format json
ustc-agent market packages list --format json
```

These commands project `server.info`, `capability.list` and `market.package.list` from [`interfaces.md`](interfaces.md). Exact options, DTO schema versions and route bindings are fixed in the first accepted command-registry slice before implementation. Product-specific query/command/event families are added only after their owning M10/application contracts and active acceptance rows exist. Command spelling never creates an operation absent from that registry.

The initial CLI does not include install/grant/source-publish/operator actions, an embedded model loop or a generic arbitrary HTTP/tool invocation command.

After real campus-product contracts exist, the CLI lane may add read-only projections in the existing product order: `affairs search/get` first, then approved ChangeRadar and Opportunity Graph operations. Cultivation programs use `program`; tenant-local planning drafts use `planner draft`. An ambiguous `plan` namespace is not admitted.

The inbound MCP adapter's executable/subcommand packaging is intentionally not fixed by this command table. Its first accepted slice may choose a dedicated binary or `ustc-agent mcp serve`; either choice must preserve the same client-core and privilege contract and cannot change the user CLI command semantics by implication.

## 5. Common `ustc-agent` request path

```text
parse bounded command input
→ load validated server/profile config
→ obtain least-privilege user session through ClientAuthPort
→ client-core compatibility/capability preflight
→ send one versioned M10 query/command
→ preserve correlation/idempotency/precondition identity
→ map typed result/error/event
→ render human or machine output
```

The CLI never imports backend domain implementations to “optimize” a remote operation. It may use local deterministic formatting/validation, but the server recomputes every truth-affecting decision.

## 6. Machine output

Machine mode is selected explicitly by `--format json` or `--format ndjson` where streaming is supported.

### JSON result

- stdout contains exactly one versioned `ustc-client-result/v1` JSON value and a trailing newline;
- stderr contains only redacted human diagnostics;
- field ordering is deterministic where the owning DTO contract specifies ordering;
- no progress spinner, prompt, ANSI escape or prose is mixed into stdout;
- a non-success typed result is represented by the stable envelope and matching nonzero exit class.

### NDJSON stream

- each complete stdout line is one versioned `ustc-client-event/v1` value;
- every line carries correlation identity, event kind and monotone cursor when the server supplied one;
- partial/malformed final frames make the command non-success;
- transport closure is not emitted as task completion;
- resume requires an explicit last accepted cursor and follows server resync semantics.

Human rendering consumes the same typed values. It does not become a second parser or semantic path.

## 7. Exit classes

The initial user CLI reserves these stable classes:

| Exit | Class | Meaning |
|---:|---|---|
| `0` | success | complete typed result or cleanly completed stream |
| `2` | usage/input | invalid command, option, local bounded input or config shape |
| `3` | authentication | missing, expired or invalid user authentication |
| `4` | policy denial | authenticated but forbidden/ungranted operation |
| `5` | compatibility | unsupported client/server protocol or required upgrade |
| `6` | unavailable | endpoint, DNS/TLS or transport unavailable before a known accepted operation |
| `7` | conflict | stale precondition, idempotency conflict or current-state conflict |
| `8` | outcome unknown | timeout/disconnect after possible acceptance; reconciliation required |
| `9` | protocol/internal | malformed/unknown server value or non-recoverable client invariant failure |

A server domain denial keeps its typed stable code inside the result envelope and maps to the closest class above. The CLI MUST NOT map an unknown error to success or choose a same-name fallback operation.

## 8. Noninteractive and cancellation semantics

`--non-interactive` MUST:

- never prompt or open a browser/terminal UI;
- fail with a typed prerequisite class when authentication/confirmation is unavailable;
- keep stdout machine-clean when a machine format is selected;
- avoid reading hidden operator environment/config by fallback.

Interactive authentication is not frozen by this revision. The preferred next contract candidate is server-mediated browser pairing: the CLI obtains a short-lived one-time pairing identity, opens the system browser only in interactive mode, and exchanges successful server admission for a least-privilege client session reference. `--non-interactive` never opens a browser and requires a pre-admitted profile. No flow accepts or forwards a raw USTC password, CAS ticket or complete CAS session.

Process termination or broken stdout means only that the observer stopped. It does not prove server cancellation. A cancellable accepted operation uses an explicit typed cancellation command and, after timeout, reconciliation by correlation/idempotency identity.

## 9. Authentication and secrets

`ustc-agent` accepts a validated profile/reference, not a raw secret in argv. Secret/session material:

- is resolved through the user client auth adapter;
- is stored with restrictive target-appropriate permissions;
- is never printed in stdout/stderr, process arguments, evidence or telemetry;
- is not shared with `ustc-agentctl` operator credentials by default;
- cannot be forwarded to a Plugin, external MCP server or model provider except through a separately owned admitted credential contract.

Production config rejects loopback/default-server ambiguity where the target profile requires a remote HTTPS origin.

## 10. Conformance and current status

Before `ustc-agent` is claimed implemented:

- `CLIENT-008` proves no GUI/service shell-out path and no operator command/credential reachability;
- `CLIENT-009` proves real M10 read-only invocation, JSON/NDJSON framing, stderr separation, exit classes, auth isolation, version mismatch, reconnect/cancellation distinction and timeout reconciliation;
- dependency checks prove no backend domain, repository, executor, provider, M51 or `ustc-agentctl` implementation dependency;
- the real binary is exercised, not only a parser/unit test.

Current status: `ustc-agent` does not exist. Only the `ustc-agentctl` commands in §2 and `ustc-agentd --version` are implemented. All other command rows remain planned and non-operational.
