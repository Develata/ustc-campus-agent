# Bundled Market catalog query contract

- Owner: M20 catalog semantics; M10 wire carrier and HTTP admission; `ustc-agentd::market_catalog` application composition.
- Version: `market-catalog/v1` and `market-package/v1`.
- Parent authority: [Market lifecycle](market-lifecycle.md), [Plugin package](plugin-package.md), [module boundaries](module-boundaries.md).
- Acceptance: bounded supporting evidence for `MARKET-001`; full multi-client Market delivery remains planned.

## Responsibility and flow

The read-only application query projects the four package manifests bundled into the
binary through M20 `load_package_manifest` and `CatalogReadModel`. It performs no
network, user-state access, installation, configuration, grants or execution. All
manifests must validate before any catalog is returned; failure is atomic and closed.
Reviewed repository declarations remain catalog authority. The bundled catalog is
an immutable deployment snapshot, not a separately editable catalog or evidence of
publication, installation, enablement, artifact readiness or source activation.

The transport calls exactly one public application query. Domain construction and
projection stay in `market_bundle` / `market_catalog`; framework-free DTOs stay in M10's
`client-protocol::market`. No domain type crosses the wire.

## Routes and values

`GET /api/v1/market/packages` returns `MarketCatalogDto`:
`schema`, `catalog_revision`, `catalog_digest`, `management_available`, `packages`.
Each package summary carries `package_id`, `version`, `publisher`, `tier`,
`display_name`, nullable `description`, `implementation_status`, `package_digest`,
`component_count`, and `requested_capabilities`.

`GET /api/v1/market/packages/{package_id}/{version}` returns `MarketPackageDto`:
`schema`, `catalog_revision`, `catalog_digest`, `management_available`, `package`
(the same summary), `components` (`kind`, `path`, nullable `mode`), `source_policy`
(ordered `key`/`value` entries), and `install_policy` (`class`, `default_installed`,
`default_enabled`, `user_disable_allowed`). Component kind and manifest enum strings
retain their existing schema spellings. They are declarations, never execution handles.
`management_available` is false in this bounded query slice and is independent of
manifest status and install-policy defaults. No install/configure/enable POST is exposed.

Both routes require client protocol major 1 and inherit existing loopback Host and
same-origin checks. Missing/newer/invalid majors return the existing typed 409
compatibility outcome; older majors return its typed 426 outcome before the query.
Malformed package/version identity returns 400 `invalid_package_reference`, missing
exact version returns 404 `package_not_found`, invalid bundled catalog returns 503
`market_catalog_unavailable`. Errors contain stable codes, never rejected input.
Query matching is exact; no latest-version or same-name fallback. Responses use the
existing no-store/security headers. GET is safe to repeat and never creates state.

## Snapshot identity and bounds

At most 64 embedded manifests are admitted, each under the existing M20 1 MiB
manifest bound. Production embeds exactly the current four package manifests.
The snapshot revision is `catalog:` plus SHA-256 hex of the role/path/bytes encoding
in [M20-CONFIG-003](market-component-configuration.md). Its v2 domain includes the
fixed reviewed configuration sidecars as well as manifests, so schema replacement
changes catalog identity. The entire fixed bundle is bounded to 16 MiB. The application
query retains one immutable `market_bundle` snapshot; configuration lookup is internal
and does not expose an install/configure command or infer execution readiness.
The existing M20 catalog digest additionally binds the revision and canonical
package declarations. Repository or binary replacement rebuilds the entire read
model; there is no live mutation, migration, partial refresh or cache write.

## Client behavior and failures

Plugins retains its capability-to-Chat entry points under the default capabilities tab.
An accessible capabilities/catalog tab switch shows one panel at a time; the catalog
loads only on first selection. Known bundled names and short introductions have
Chinese presentation text, with exact original declarations retained in details.
A separate catalog panel
supports search, exact package detail, declared MCP/Skill/native/resource components,
requested permissions and source policy. Requests time out after 15 seconds, and refresh clears the obsolete projection.
Loading, empty search, unavailable catalog,
unavailable detail and retry are explicit. The client never fabricates an installation
or grants from metadata, never sends writes while browsing, and renders package text
as text rather than HTML. Detail requests must bind the selected package version and
catalog digest; stale or out-of-order results cannot replace a newer selection.

## Verification

Targeted application query tests cover real manifests, exact lookup, invalid/missing
identity, atomic invalid-source refusal, deterministic snapshot identity and no inferred
installation. HTTP tests cover compatibility, Host/Origin and readonly success/errors.
Browser checks cover search/detail/return, error/retry, mobile layout, light/dark,
keyboard access and zero mutation requests. This slice does not prove installation,
multi-user authentication, reminders, MCP execution, or Skill activation.

Bounded verification commands:

```bash
cargo test --locked -p ustc-agentd --lib market_catalog::tests
cargo test --locked -p ustc-agentd --test affairs_web market_catalog_http_reads_exact_bundled_metadata_and_gates_protocol -- --exact
# Dedicated browser loop over an isolated compiled-binary server:
UCA_BROWSER_SUITE=market node scripts/test_usable_enhancements_browser.mjs --base http://127.0.0.1:18852
```

The port in the last example belongs to an explicitly started local test server;
it is not a product default. Omitting `UCA_BROWSER_SUITE` retains the full existing
UI integration suite. Windows Chromium may test the WSL server using the same
`--base` mode. This is browser evidence against an external compiled-binary listener,
not evidence that the Windows runner can itself launch a Linux binary.
