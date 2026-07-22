# CLI contract

Binaries:

- `ustc-agentd` — future service daemon.
- `ustc-agentctl` — operator/developer CLI.

## Implemented commands

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
- diagnostic on stderr and exit code `2` for invalid options, unreadable/invalid fixtures, or no feasible plan.

The command is read-only: it does not access the network, store credentials, modify external systems, or enroll a user in courses.

## Planned commands

| Command | Purpose | Side effect policy |
|---|---|---|
| `ustc-agentctl acceptance run --gate pr` | run a named acceptance subset | evidence write only when requested |
| `ustc-agentctl source registry-check --strict` | validate reviewed source declarations | read-only |
| `ustc-agentctl source crawl-plan <source-id>` | produce a bounded retrieval plan | read-only; no cursor/baseline mutation |
| `ustc-agentctl source crawl-apply --plan <path> --plan-digest <digest>` | apply an approved retrieval plan | operator-only, idempotent, audited |
| `ustc-agentctl source diff` | compare immutable source revisions | read-only |
| `ustc-agentctl procedure validate --candidate <path>` | validate typed procedure candidate | read-only |
| `ustc-agentctl procedure publish-plan --candidate <path>` | produce deterministic publish plan | read-only |
| `ustc-agentctl procedure publish-apply --plan <path> --plan-digest <digest>` | publish an approved artifact | administrator-only, audited |

Any durable mutation must define dry-run/plan semantics, idempotency identity, authorization, receipt and recovery before implementation. Server/runtime code calls shared Rust domain services directly; it does not shell out to this CLI for normal business paths.
