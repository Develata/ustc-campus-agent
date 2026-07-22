# Gitea mirror policy

The self-hosted Gitea repository is a backup/vault, not the collaboration authority.

## Current authority split

```text
GitHub private repository
  ├── canonical source
  ├── PRs / Issues / reviews
  ├── GitHub Actions CI
  └── future Releases / Pages

Self-hosted Gitea
  └── scheduled pull mirror and disaster-recovery source copy
```

## Rules

- Do not push feature branches to Gitea as a second primary.
- Do not treat Gitea CI, if later enabled, as replacing GitHub Actions unless a new ADR says so.
- Git mirrors preserve Git objects. Issues, PRs, Actions artifacts, packages, Pages, and Release assets require separate export/backup if they become important.
- Mirror credentials must be fine-scoped and stored outside this repository.

## Future work

After the first GitHub commit lands, create or update the Gitea pull mirror using a fine-scoped GitHub token. Record the mirror URL and schedule in private operations notes, not in public docs if it exposes private infrastructure.
