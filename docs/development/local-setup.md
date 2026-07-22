# Development guide

## Prerequisites

- Rust stable 1.97.1 with `rustfmt` and `clippy`.
- Python 3 for repository contract checks.
- CodeGraph CLI for local code navigation.
- GitHub CLI for repository operations when authorized.

## Rust gates

```bash
df -h / /opt/data 2>/dev/null || df -h
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
```

## Repository contract gate

```bash
python3 scripts/check_repo_contracts.py
```

The checker validates:

- internal Markdown links;
- obvious secret patterns;
- market manifest/publisher/capability consistency;
- acceptance matrix shape and duplicate IDs.

## CodeGraph

`codegraph init /opt/gitclone/ustc-campus-agent` has been run once for the local clone. The generated `.codegraph/` index is intentionally ignored by Git.

Use:

```bash
codegraph status /opt/gitclone/ustc-campus-agent
codegraph sync /opt/gitclone/ustc-campus-agent
codegraph explore --project /opt/gitclone/ustc-campus-agent "market plugin authority"
```

For Agent tool usage, pass `projectPath=/opt/gitclone/ustc-campus-agent` to CodeGraph MCP calls.

## Local cleanup

After local Rust work, `target/` may be removed when no current review/debugging step depends on it:

```bash
cargo clean
```
