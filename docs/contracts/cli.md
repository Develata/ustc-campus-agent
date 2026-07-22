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
| `ustc-agentctl market validate` | move final market schema/manifests authority into Rust | read-only |
| `ustc-agentctl acceptance run --gate pr` | run PR acceptance subset | evidence write only when requested |
| `ustc-agentctl source import-snapshot` | import approved source snapshot | operator-only, audited |
| `ustc-agentctl source diff` | compare source revisions | read-only |
| `ustc-agentctl doctor` | expand local configuration and binary sanity | read-only |

Any command that writes durable state must define `--dry-run` semantics before implementation.
