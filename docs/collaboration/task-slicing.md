# Task slicing and ownership

## Core owner lanes

| Lane | Owns |
|---|---|
| Product / Source | Course Planning journey, source authority, iCourse/USTC communication, fixture oracle |
| Backend / Runtime / Security | Rust core, market/install/grants, Agent gateway, privacy/security |
| Frontend / Demo | Market + Agent Web/PWA, Chinese UX, browser evidence, demo narration |
| Evaluation / Release | fixtures, acceptance runner, CI/release gates, public-readiness evidence |

## Slice shape

Good slice:

```text
contract update
→ minimal implementation
→ test/evidence
→ docs projection
→ review
```

Bad slice:

```text
random frontend polish + schema change + runtime refactor + source parser + CI cleanup
```
