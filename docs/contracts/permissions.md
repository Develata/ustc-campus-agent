# Permission and capability contract

## Current capabilities

See [`market/capabilities/registry.json`](../../market/capabilities/registry.json).

## Classes

- `public-read`: read public/approved campus facts.
- `public-linkout`: return external URL/title metadata without caching protected content.
- `tenant-private-read`: read the user's own snapshot/preferences.
- `tenant-private-write`: create or update tenant-local drafts only.

## Forbidden in MVP

- raw credential access;
- cross-user academic data;
- automatic enrollment or external mutation;
- silent permission expansion during package upgrade;
- model-generated grants.
