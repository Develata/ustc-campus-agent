# Interface registry

This registry names draft public surfaces before implementation. Implementation PRs must update this document or create a more specific contract before changing surfaces.

## HTTP routes — draft

| Route | Method | Purpose | Status |
|---|---:|---|---|
| `/api/health` | GET | service health and version | planned |
| `/api/market/packages` | GET | list visible packages | planned |
| `/api/market/packages/{id}` | GET | package details | planned |
| `/api/installations` | POST | install exact package version with grants | planned |
| `/api/installations/{id}:disable` | POST | disable installed package | planned |
| `/api/agent/runs` | POST | create bounded Agent run | planned |
| `/api/agent/runs/{id}/events` | GET/SSE | stream model/tool/state events | planned |

## MCP/tool surface — Course Planning draft

| Tool | Purpose | Mutates external systems |
|---|---|---:|
| `plan.list` | list available plans | no |
| `plan.get` | get plan revision | no |
| `course.search` | search normalized courses | no |
| `course.get` | course detail/provenance | no |
| `review.linkout` | return iCourse link-out metadata | no |
| `offering.list` | list imported/approved offerings | no |
| `profile.requirement_status` | compute progress against a plan | no |
| `planner.generate` | create tenant-local plan candidates | tenant draft only |
| `planner.explain` | explain candidate rationale | no |
| `source.provenance` | show evidence chain | no |
