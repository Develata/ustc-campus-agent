# Public repository readiness

Develata closed the repository-visibility owner gate on 2026-09-04 and selected the MIT License on 2026-09-05. The GitHub repository is intentionally public, and its project-authored software and documentation are MIT-licensed. These facts are not evidence that every release, deployment, third-party notice or data-use gate has passed. Unchecked items below block stronger readiness claims and new tag, GitHub Release, Pages or stable-download surfaces; they do not describe the current visibility or project-license settings.

- [x] repository visibility owner decision recorded as public;
- [x] MIT License selected and recorded in `LICENSE.md` and workspace package metadata;
- [ ] third-party notices and architecture influence records complete;
- [ ] full reachable Git history secret/private-data audit complete;
- [ ] private or unapproved fixtures replaced with reviewed synthetic/anonymized examples;
- [ ] non-official USTC disclaimer visible in README, Pages and app UI;
- [ ] iCourse and USTC source/data-use permissions documented;
- [ ] no real student personal data in issues, docs, screenshots, logs or artifacts;
- [ ] current implementation/status claims match manifests and acceptance evidence;
- [ ] any stable release/download surface verified from the remote delivery endpoint or kept unpublished; expiring source-bound CI artifacts are evidence, not a stable release;
- [ ] GitHub Pages contains no fabricated metrics, testimonials, affiliations, logos or download links;
- [ ] responsive/accessibility/keyboard/console/link browser checks pass;
- [ ] required release/public acceptance cases pass with no unresolved blocker review.

PR-gate success alone does not satisfy this checklist.

## Recorded data-use gap (2026-09-06)

The [MVP feature](../features/06-mvp-core-capabilities.md) records iCourse aggregate-rating snapshots mapped to synthetic course codes. This differs from pure link-outs: stored aggregate values, copied review text and live collection are separate data-use questions. The owning [security plan](../plan/08-security-and-delivery.md) requires an explicit data-use contract beyond link-outs; the documented permission item above is still unchecked. No such exception is established by this reconciliation. Before stronger publication/reuse claims, an owner-approved resolution must bind permitted data, source, storage/reuse scope and evidence, or select synthetic-only replacement; this page neither grants permission nor changes the already delivered fixture/archive bytes.
