# Agent workflow

This repository is expected to be edited by multiple AI coding agents.

## Before editing

1. Read `AGENTS.md`, `README.md`, and the nearest doc/contract.
2. Check `git status --short --branch`.
3. Declare touched files/directories in the issue or PR.
4. Do not assume legacy docs are current if they conflict with ADRs or current contracts.

## During editing

- Use exact path staging.
- Preserve user work and foreign dirty files.
- Keep generated artifacts out of Git unless the contract says otherwise.
- Add or update acceptance cases for public behavior changes.

## Before handoff

- Run relevant gates.
- Include real output.
- Report not-run gates honestly.
- Do not push without explicit authorization.
