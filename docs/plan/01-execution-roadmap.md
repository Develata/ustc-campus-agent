# Execution roadmap

This roadmap is the implementation projection of the planning brief in `docs/plan/initial-plan-v2-2026-07-22.md`.

## P0 — repository and source gates

- initialize Rust monorepo, market boundary, CI, contracts, and acceptance matrix;
- assign Product/Source, Backend/Runtime/Security, Frontend/Demo, Evaluation/Release owners;
- complete read-only catalog structure probing through approved MFA/manual path;
- contact iCourse maintainers for data/API/AI-use permission;
- freeze one Course Planning fixture path and fallback.

Gate: if only user-import offering data is available, the MVP and demo must explicitly state “user-imported opening list + plan-aware planning”, not real-time official integration.

## P1 — risk-first spikes

- Rig provider/adapter spike;
- LangGraph/PydanticAI durable baseline spike under the same contract;
- catalog snapshot/import parser spike;
- deterministic Course Planning hard-constraint planner for 20–30 candidate courses.

Gate: choose one execution path for MVP and prove hard-constraint violation count is zero on curated fixture.

## P2 — Market read path + Course contracts

- PluginPackage schema and capability registry;
- `ustc.opportunity-graph` manifest;
- minimal typed Opportunity Graph ontology;
- Course domain model, source contract, and tool schema;
- market browse/detail UI shell;
- Web/PWA + SSE minimal connectivity.

Gate: pinned repository revision can deterministically validate the catalog; malformed or secret-bearing manifest fails.

## P3 — install/grant/Agent lifecycle

- development identity boundary;
- install/enable/disable/grant/audit;
- tool gateway and schema/grant resolution;
- bounded conversation stream;
- PlatformOperator role and snapshot import audit;
- planner spike connected to Rust hard-constraint validation.

Gate: disabling the package removes Agent tool discovery/invocation immediately.

## P4 — Course Planning real journey

- integrate plan/offering/link-out/review adapters;
- source revisions, provenance, conflict records;
- user academic snapshot and preferences;
- multi-candidate planner explanation;
- stale/conflict/low-confidence UX;
- planner/LLM consistency gate.

Gate: a non-developer can reproduce one complete journey with source evidence.

## P5 — productization and adversarial testing

- polish Market/Agent/plugin detail;
- browser desktop/mobile, keyboard, focus, console/network checks;
- tenant isolation, redaction, disable/revoke, stale-source tests;
- fixture oracle and small user trial;
- deployment/restore/evidence bundle.

Extra ecosystem fixture is stretch only after P4 passes.

## P6 — freeze and submission

- fix blockers only;
- record demo and failure/recovery cut;
- prepare architecture, framework influence, source/license, and evidence documents;
- clean-host restore/read-back where applicable;
- submit.
