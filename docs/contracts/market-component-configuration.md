# Reviewed package component configuration

- Contract: `M20-CONFIG-003 / package-component-configuration/v1`.
- Owner: M20 checked declaration loading; app composition owns the fixed reviewed inputs.
- Scope: exact package configuration authority for the approved install/configure flow.
- Acceptance: `MARKET-009`; supporting evidence only for complete `MARKET-002`.

## Authority and compatibility

Reviewed Git owns package declarations. This package-owned sidecar supplies the exact
component pin and configuration declarations missing from the existing manifest.
It is not an independently installable component or a user configuration document.
The current package manifest schema is unchanged and continues rejecting unknown fields.
The loader validates bytes against a checked manifest and its catalog revision; this
proves consistency, not that arbitrary caller bytes have been reviewed. Production
composition embeds and holds the reviewed sources. Browser-provided schemas, bindings,
digests, URLs and file paths never select configuration authority.

The source remains distinct from runtime configuration, grants and execution readiness.
Declared component artifact digests are not artifact admission or a permission to run
MCP, execute Skill resources or read credentials. No installation mutation, secret
resolution, external I/O or new grant issuer is part of this loader.

## Closed sidecar carrier

One JSON document covers one exact nonempty component package. Its exact fields are:

```text
schemaVersion: "package-component-configuration/v1"
packageId, packageVersion, packageDigest, componentSetDigest, capabilityManifestDigest
components: [
  path, type, mode: string or null (explicit; equal to the manifest declaration)
  componentId, componentVersion, componentDigest
  executionIdentity: string accepted by the existing checked ExecutionIdentity parser
  schemaDigest
  fields: [
    key, required: boolean, kind: "text", maxUtf8Bytes: integer
    OR key, required, kind: "integer", min: i64, max: i64
    OR key, required, kind: "boolean"
    OR key, required, kind: "secret_ref"
  ]
]
```

The loader must match every manifest component exactly once by path, kind and mode;
reject missing/extra members and duplicate paths or IDs. All package digest dimensions
must match the checked manifest. It builds the existing `InstalledComponentPin` and
`InstallationPackagePin`, then delegates checked fields/digest binding to CONFIG001/002.
Zero configuration fields require an explicit empty array and its correct schema digest.
Missing sidecars never mean a package needs no configuration. Packages without declared
components remain browseable and have no configuration authority in this composition.

Input limits: 16 MiB aggregate fixed bundle, 1 MiB per source before decoding, 1..64 components, at most 128 fields per component.
Existing checked identities, field/key/text/integer bounds remain authoritative.
Every JSON object rejects unknown and duplicate fields, wrong/null types and inapplicable
bounds. No defaults, coercion, raw-secret or runtime-value field is accepted. Errors
expose stable categories only: source size, JSON/version rejection, package/member/pin
mismatch, duplicate component, invalid schema and schema digest mismatch. Debug/Display
must not echo rejected input. Output has private fields and no public deserialization.
A partially decoded document is never returned as an accepted authority.

## Application assembly

The fixed bundle includes manifests and explicitly listed configuration sidecars.
It constructs a single catalog revision with domain `market-bundled-sources/v2\0`:
source count as u64 BE; sources sorted by `(role, path)`; for each source, role then
canonical repository-relative path then original bytes, each prefixed by its u64 BE
byte length. Roles are `manifest` and `configuration`. Duplicate role/path entries
are rejected. Source count, byte and path bounds plus manifest decoding precede this bounded
allocation. Sidecar validation uses the derived revision before snapshot publication.
No source scan, environment file selector, request path or network fetch is allowed.

A changed configuration declaration therefore changes the opaque catalog revision.
Sidecars do not contain that derived revision, avoiding self-reference. Package and
component declaration digests retain their existing formats. Public catalog wire
fields stay unchanged. The sidecar set is an explicit allowlist: duplicates, orphan
sidecars or any invalid included source make construction fail; configured components
are not silently skipped. A missing sidecar yields no configuration authority, never
an empty implicit schema. The same immutable assembled object feeds catalog browse
and future exact configuration lookup.

## Failure, recovery and tests

All construction is immutable and all-or-nothing. Rebuilding uses the reviewed bundle;
there is no runtime migration or mutable cache to repair. Reads cannot grant, enable
or execute. Future installation services obtain the loaded object from this internal
application owner and keep runtime owner/revision/receipt checks in their own boundary.

Bound cases cover all four field kinds and explicit empty schema; package/member/pin
changes; duplicate and unknown JSON fields; byte/component/field bounds; wrong schema
digest and malformed execution identity; component ordering; safe error rendering;
sidecar-only revision change; unchanged manifest rejection and fixed bundle lookup.
The real surface is bundled catalog HTTP/browser read plus trusted configuration lookup;
MCP/Skill execution and durable install/configure/grant/enable remain separate work.
