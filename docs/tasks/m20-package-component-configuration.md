# M20 package component configuration task

- Owner: Main; bounded implementation and independent review lanes use disjoint files.
- User direction: install an exact plugin package, configure its MCP/Skills components,
  review permissions, explicitly enable. High cohesion and low coupling are required.
- Authority: [M20 blueprint](../plan/modules/30-market-package-lifecycle.md),
  [market lifecycle](../contracts/market-lifecycle.md) and
  [package contract](../contracts/plugin-package.md).
- Bound acceptance: `MARKET-002` supporting evidence; the complete journey remains planned.

## First cohesive module: M20-CONFIG-001

`crates/platform-core/src/market/configuration_schema.rs` owns pure checked schema
construction, required/closed typed fields, value bounds and a canonical schema
digest. It consumes existing checked installation configuration and never mutates
it. It has no I/O, UI, provider, MCP SDK, grant or installation lifecycle dependency.
The owning contract freezes encoding and errors; tests cover boundary behavior,
input preservation, secret-reference handling and independent digest vectors.

Verification:

```bash
cargo test --locked -p ustc-campus-agent-core --lib market::configuration_schema::tests
cargo test --locked -p ustc-campus-agent-core --test platform_identity
python3 scripts/check_repo_contracts.py
```

The source/import/macro/attribute inventories stay closed and admit only the
reviewed new module. Registering it must not relax the identity or dependency guards.
Status: pure validator implemented with bounded supporting evidence: 8 unit tests,
full Rust baseline, exact identity guard and repository contract checks passed;
independent review found no blocker. Production configuration remains planned.
No runnable smoke applies to this pure module; controlled configuration inputs are
its real feature path. This does not establish durable configuration or Market readiness.

## Second cohesive module: M20-CONFIG-002

`crates/platform-core/src/market/configuration_binding.rs` binds a checked schema
and its expected digest to one component of an exact package pin. It rejects
package/catalog/version/digest drift, full component membership and execution
identity drift, then delegates value validation to CONFIG-001. It adds no parallel
schema digest format and does not extend package manifests.

Status: pure binding implemented; 6 targeted module tests pass, covering schema
substitution, all package/member pin dimensions, canonical multi-component order,
sibling addition/removal/change, error precedence and redacted diagnostics.
Closed Python/Rust inventory registration and independent review have passed; final repository contract/projection checks passed after Main verified the exact changed projections. No admission policy was widened by registering the module.
No runnable service smoke applies; controlled package/configuration inputs are
this module's feature path. This does not prove reviewed provenance, production
schema loading, durable configuration or Market readiness.

```bash
cargo test --locked -p ustc-campus-agent-core --lib market::configuration_binding::tests
```

## Following modules and integration

1. Exact reviewed package/component schema loading into the CONFIG-002 binding.
   The production issuer must obtain and hold the binding from reviewed authority;
   browser-provided schemas cannot become authority. Current manifests are not
   silently extended. Binding consistency alone does not implement this loader.
2. M20 application service for install/configure/grant/enable/disable commands, using
   the existing domain decisions and one atomic persistence boundary for events and
   receipts. Failed persistence leaves the acknowledged in-memory state unchanged.
3. A durable adapter with bounded decoding, exclusive writer, atomic replacement,
   recovery validation and restart/idempotency tests. No direct store access in UI/HTTP.
4. M10 versioned commands/queries and a separate package-management frontend over
   those application ports, retaining expected revisions and explicit consent.
5. Separate bounded Skill context and M51 reviewed MCP discovery/execution modules,
   assembled through application composition. Neither may mint grants or register
   directly into the Agent kernel. Component availability must reflect actual readiness.

These are dependency stages, not completed features. Independent MCP URL or local
Skill import is outside the approved path. No remote publication is authorized.


## Third cohesive module: M20-CONFIG-003

Main owns the [reviewed declaration contract](../contracts/market-component-configuration.md),
fixed application bundle, schema registration and integration. The loader implementer
owns `market/configuration_catalog.rs`; a separate reviewer checks provenance and bounds.
The bundle connects actual checked declarations to the existing catalog read path.
No placeholder install/enable UI is added before the relevant service works.

The following P2 prerequisites were identified against the existing installation API:
read-only receipt lookup in the existing ledger; checked request-equivalence projection
before current-authority reads; full multi-component configuration mapping; admitted
internal enable issuer; and a controlled persistence codec/rebuild boundary. Replays
must return historical receipts without repeating effects when current catalog/schema
services are unavailable. A separate application ledger, single-component restriction
or app-created enable evidence would bypass existing authority and is not adopted.


Status: implemented as bounded declaration/loading support. Eleven loader cases,
five core identity gates, six application catalog/bundle cases, the exact Market HTTP
case and six Market browser cases passed. The real bundle includes one explicitly
empty native Calendar schema; controlled cases cover MCP/Skill declarations and four
field kinds without connecting or executing them. Separate history UI work passed
eight conversation browser cases, including title refresh and stale/failed list reads.
Independent static review reported no blocker. Complete installation management stays
planned; this slice adds neither a persistence adapter nor an enable issuer.
