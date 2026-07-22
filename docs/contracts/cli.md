# CLI contract

Binaries:

- `ustc-agentd` — future service daemon.
- `ustc-agentctl` — operator/developer CLI.

Current implemented smoke surface:

```bash
ustc-agentd --version
ustc-agentctl --version
ustc-agentctl doctor
ustc-agentctl market validate
```

Future planned commands:

| Command | Purpose | Side effect policy |
|---|---|---|
| `ustc-agentctl market validate` | validate market schema/manifests | read-only |
| `ustc-agentctl acceptance run --gate pr` | run PR acceptance subset | evidence write only when requested |
| `ustc-agentctl source import-snapshot` | import approved source snapshot | operator-only, audited |
| `ustc-agentctl source diff` | compare source revisions | read-only |
| `ustc-agentctl doctor` | local configuration and binary sanity | read-only |

Any command that writes durable state must define `--dry-run` semantics before implementation.
