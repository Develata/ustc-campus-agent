# User context profile contract

## Metadata

- `Status`: Accepted semantic contract; implementation planned
- `Version`: `user-context-profile/v0`
- `Last Review`: `2026-08-02`
- `Owning Blueprint`: [`M00 Platform Control and Identity`](../plan/modules/10-platform-control-identity.md)
- `Depends On`: [`platform-account/v0`](platform-account.md), [`module-boundaries.md`](module-boundaries.md)
- `Infrastructure`: [`storage-profiles/v0`](storage-profiles.md)
- `Consumers`: M30 prompt projection and M72 Opportunity Graph through purpose-bound read projections
- `Acceptance`: catalog-only `PROFILE-001` through `PROFILE-012`; none is active or implemented

## 1. Scope and authority

M00 owns one platform-level, tenant/user-scoped context profile. It is separate from runtime account, authentication, membership and authorization state.

```text
ProfileFieldDefinition   reviewed registry entry and policy for one extensible field key
ProfileFact              one provenance-bearing candidate or accepted field value
ProfileProposal          AI/system suggestion without direct acceptance authority
CurrentProfileProjection deterministic current read view at one source revision
ProfileAccessGrant       purpose/consumer/field/sensitivity consent boundary
ProfileAuditReceipt      durable mutation/access/deletion evidence
```

The profile supports personalization and context. It cannot authenticate a user, link accounts, issue a session, create a tenant membership, assign a role or mint a grant.

## 2. Field registry

Every admitted field key has one reviewed `ProfileFieldDefinition`:

```text
field_key
value_schema_id
cardinality: Single | Multiple
allowed_sources[]
verification_policy
sensitivity: Public | Internal | Sensitive | HighlySensitive
visibility_policy
prompt_exposure_policy
retention_policy
conflict_resolution_policy
registry_revision
```

Unknown field keys, unknown value/schema variants and policy/registry revision mismatch fail closed. An extension adds a registry entry and compatibility behavior; it does not require a database column or reinterpret historical bytes.

Initial general-purpose keys include:

| Field key | Value shape | Cardinality | Default sensitivity |
|---|---|---:|---|
| `profile.name.display` | bounded text | single | internal |
| `profile.institution.person_number` | typed number/issuer pair | multiple | sensitive |
| `profile.institution.student_number` | typed number/issuer pair | multiple | sensitive |
| `profile.institution.staff_number` | typed number/issuer pair | multiple | sensitive |
| `profile.academic.status` | `Undergraduate | Master | Doctoral | Faculty | Staff | Other | Unknown` | single/current | internal |
| `profile.institution.campus_card_uid` | bounded opaque text | multiple | highly sensitive |
| `profile.academic.department` | bounded organization reference/text | multiple/temporal | internal |
| `profile.academic.major` | bounded reference/text | multiple/temporal | internal |
| `profile.academic.enrollment_year` | bounded year | single/temporal | internal |
| `profile.academic.class` | bounded text | multiple/temporal | sensitive |
| `profile.residence.dormitory` | bounded text | multiple/temporal | highly sensitive |
| `profile.contact.email` | normalized address | multiple | sensitive |
| `profile.contact.phone` | normalized phone value | multiple | highly sensitive |

Absence is allowed for every optional field. Values in fixtures are synthetic and must not contain real personal identifiers.

## 3. Fact model

One `ProfileFact` contains at least:

```text
tenant_id
user_id
fact_id
field_key
value_json
value_digest
source_kind
source_ref?
observed_at
valid_from?
valid_until?
confidence?
verification_level
status
sensitivity
created_at
updated_at
revision
```

Controlled source kinds begin with:

```text
UserAsserted
InstitutionClaim
SystemDerived
AiProposed
AdminSet
```

Controlled verification levels begin with:

```text
Unverified
UserConfirmed
InstitutionVerified
AdminVerified
```

Controlled fact statuses begin with:

```text
Proposed
Confirmed
Superseded
Rejected
Deleted
```

`value_json` is validated by the registered schema before a fact is admitted. Confidence is bounded and optional; it never substitutes for verification. Source payloads are referenced or reduced to approved fields/digests rather than copied wholesale.

## 4. Current projection and missing-value semantics

The source of truth is the fact history plus registry/policy revision. A `CurrentProfileProjection` is a deterministic, rebuildable convenience view for a declared purpose and consumer.

The projection distinguishes:

```text
Known(value, supporting_fact_ids)
Unknown
Withheld
NotApplicable
Conflicted(candidate_fact_ids)
```

A UI may render several of these as an empty field, but storage and policy must not collapse them into one ambiguous SQL `NULL`.

Resolution uses only the field's registered source/verification/time/conflict policy. A newer AI proposal cannot override an institution-verified or user-confirmed current fact merely by having a larger confidence score. Conflicts remain explicit unless a registered deterministic rule resolves them.

Projection identity binds at least tenant, user, purpose, consumer class, registry revision, source fact revision and projection-policy revision. A source revision change makes prior projections stale.

## 5. AI and system derivation

An AI may invoke only a typed proposal operation:

```text
ProposeProfileFact
```

It cannot invoke `ConfirmProfileFact`, mutate an accepted fact in place, change field policy, or write account/membership/grant state. A proposal records model/provider reference, bounded evidence reference, confidence and exact field/value candidate without copying a complete private conversation into the profile store.

User or administrator flows decide:

```text
AcceptProfileProposal
RejectProfileProposal
SupersedeProfileFact
DeleteProfileFact
```

A field policy may permit explicit user-configured auto-accept only for low-sensitivity personalization fields. The initial institutional, identity, contact and residence keys above do not auto-accept AI proposals.

A school-number classifier is a versioned `SystemDerived` rule. Prefix/year interpretation produces a derived profile fact with rule identity and confidence; it neither authenticates the number nor creates membership/role/grant authority. Unknown or contradictory formats remain `Unknown`/`Conflicted`.

## 6. Purpose-bound reads

Profile reads require current M00 actor admission plus one exact purpose, consumer and field allowlist. The caller receives only a safe `CurrentProfileProjection`, never repository access or unrestricted fact history.

Initial consumer boundaries are:

- M30 receives a prompt-safe projection containing only fields permitted for the exact task/run purpose. Campus-card UID, telephone and dormitory are excluded by default;
- M72 receives the minimum profile fields needed for an exact qualification/planning request. It owns product-specific opportunity preferences and derived matches, not the general profile facts;
- M10/client surfaces receive user-view/edit projections and typed missing/conflict/sensitivity state, not hidden provenance payloads or other users' data.

A model/tool/plugin cannot request `all profile fields`. Access is deny-by-default and purpose-specific.

## 7. Lifecycle and concurrency

```text
user input, approved institutional claim, documented system derivation or AI proposal
→ registry/schema/source/scope validation
→ proposed or confirmed fact append under expected revision
→ deterministic current projection rebuild
→ purpose-bound read
→ accept/reject/supersede/delete
→ invalidate dependent projections and retain bounded audit receipt
```

Updates append a new fact/status transition rather than mutating provenance in place. Concurrent writes use expected revision and deterministic conflict behavior. A projection is published only after the fact mutation and required audit evidence commit successfully.

Deletion covers canonical payload, rebuildable projections, caches and policy-covered recoverable copies. Policy-retained audit keeps identifiers/digests and disposition only where required; it does not retain a supposedly deleted sensitive value.

## 8. Persistence and portability

M00 declares profile registry/fact/proposal/projection/access/audit ports. M90 implements them under [`storage-profiles/v0`](storage-profiles.md).

A query-optimized fixed projection may materialize common nullable fields, but it is never the only authority. The extensible fact layer remains canonical. SQLite stores validated canonical JSON text; PostgreSQL may store equivalent `jsonb`. Domain canonicalization and value digests must agree across both adapters.

## 9. Security and observability

- Tenant/user scope is included in every key, query and cache identity.
- Sensitive/highly-sensitive fields are redacted from normal logs, errors, metrics and prompt projections.
- Access receipts record purpose, consumer, field-key set, projection revision and decision without copying values.
- Export exposes provenance and verification state to the owning user under policy.
- Bulk/cross-user profile reads require a separately reviewed administrative contract and are not implied here.

## 10. Verification

Before implementation admission, active planned acceptance rows and exact bindings must cover:

- field-registry closure and unknown-field denial;
- fixed common-field projection plus additive registry extension;
- provenance, temporal validity, confidence and verification precedence;
- `Unknown | Withheld | NotApplicable | Conflicted` preservation;
- deterministic SQLite/PostgreSQL projections;
- AI proposal-only authority and explicit confirmation;
- school-number derivation as non-authoritative versioned evidence;
- sensitive-field prompt/log exclusion;
- tenant/user/purpose isolation;
- optimistic conflict and stale projection behavior;
- delete/recovery semantics;
- M72/M30 consumption without general-profile ownership.

Catalog-only `PROFILE-001`–`PROFILE-012` preserve these obligations. They remain non-pass until promoted to `docs/acceptance/matrix.tsv` with exact evidence.
