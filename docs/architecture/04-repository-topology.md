# Repository topology

## Current topology

```text
GitHub private: Develata/ustc-campus-agent
  └── canonical source, PRs, Issues, Actions, future Releases

Self-hosted Gitea
  └── scheduled pull mirror / code vault / disaster backup
```

No GitHub organization is created initially. The personal private repository uses Develata's GitHub Pro features for private-branch protection and review workflow.

## Future public transition

This repository may become public after the public-readiness gate. The transition is not a refactor; it is a security/release decision involving license, secret scan, fixture scrub, data-source permission, non-official disclaimers, Pages, and release artifact policy.
