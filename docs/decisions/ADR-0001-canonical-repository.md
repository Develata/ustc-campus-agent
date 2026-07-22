# ADR-0001: Canonical repository

- Status: Accepted
- Date: 2026-07-22

## Decision

Use `Develata/ustc-campus-agent` as the canonical private GitHub repository, with self-hosted Gitea as pull mirror / backup.

## Rationale

GitHub gives the best CI, review, Actions, release, and AI-coding integration for the competition timeline. Gitea mitigates platform dependency as a code vault. USTC GitLab is useful for school-local collaboration but has access and SSH limitations that complicate multi-device and external tooling.

## Consequences

- Do not create a GitHub organization initially.
- Do not create a second market repository initially.
- Do not push or change visibility without explicit Develata approval.
