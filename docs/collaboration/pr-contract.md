# PR and branch contract

## Branch names

- `feat/<short-topic>`
- `fix/<short-topic>`
- `docs/<short-topic>`
- `chore/<short-topic>`
- `spike/<short-topic>` for disposable evidence branches

## PR requirements

Every nontrivial PR must include:

- owner and reviewer;
- linked issue/task/case ID;
- touched directories;
- non-goals;
- validation output;
- screenshots/browser evidence when UI changed;
- rollback notes for stateful changes.

## Multi-agent rules

- One agent owns one slice at a time.
- Avoid broad formatting.
- Do not edit files outside the declared slice.
- Do not trust another agent's claim of test success without real output.
- New untracked files must be explicitly listed in review packets.
