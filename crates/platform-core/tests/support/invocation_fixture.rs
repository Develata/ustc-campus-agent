use serde::{Deserialize, Deserializer};

macro_rules! closed_string_enum {
    ($(#[$meta:meta])* $visibility:vis enum $name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        $(#[$meta])*
        $visibility enum $name {
            $($variant),+
        }

        impl $name {
            pub fn parse(value: &str) -> Result<Self, String> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    other => Err(format!("unknown {} value {other:?}", stringify!($name))),
                }
            }

            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::parse(&value).map_err(serde::de::Error::custom)
            }
        }
    };
}

closed_string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FixtureApi {
        SchemaConstructor => "schema_constructor",
        ArgumentConstructor => "argument_constructor",
        ResolveProjection => "resolve_projection",
        AuthorizeCall => "authorize_call",
        RunSpecMapping => "run_spec_mapping",
    }
}

closed_string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FixtureRecipe {
        GoldenSchema => "golden_schema",
        GoldenSchemaReversed => "golden_schema_reversed",
        SchemaDialectOther => "dialect=other",
        SchemaMalformedMatrix => "duplicate_property|required_not_subset",
        SchemaLimitMatrix => "depth=9|nodes=257|properties=65|enum=65|bytes>65536",
        GoldenArguments => "golden_arguments",
        GoldenArgumentsReversed => "golden_arguments_reversed",
        ArgumentDuplicateKey => "object_duplicate_key",
        ArgumentBadKey => "object_key=bad key",
        ArgumentLimitMatrix => "depth=9|nodes=257|members=65|array=256|string=4097|bytes>65536",
        ArgumentNumberOverflow => "integer=9223372036854775808|number=1e999",
        ArgumentNumberNormalization => "-0.0=0.0|subnormal=5e-324",
        ArgumentTypeDistinction => "integer=2|number=2.0",
        ValidAuthority => "valid_authority",
        TurnTwo => "turn_id=turn:2",
        ProjectionValidAuthority => "projection=valid_authority",
        CatalogMissing => "catalog=null",
        CatalogNotRunnable => "catalog.runnable=false",
        CatalogPackageVersionNine => "catalog.package_version=9.0.0",
        InstallationPackageDigestNine => "installation.package_digest=sha256:9999",
        CatalogComponentMissing => "catalog.component=null",
        InstallationComponentDigestNine => "installation.component.digest=sha256:9999",
        ExecutionIdentityUnknown => "policy.admitted_execution_identity=null",
        ExecutionIdentityOther => "policy.admitted_execution_identity=native:other",
        ToolMissing => "catalog.component.tool=null",
        ToolIdentityOther => "catalog.component.tool.id=tool:other",
        InstallationTenantOther => "installation.tenant_id=tenant:other",
        GrantUserOtherAndStale => "grant.user_id=user:other|grant.state=stale",
        GrantTenantOtherAndStale => "grant.tenant_id=tenant:other|grant.state=stale",
        CapabilityUnknown => "policy.capability_class=null",
        CapabilitiesEmpty => "component.declared_capabilities=empty",
        GrantManifestDigestNine => "grant.capability_manifest_digest=sha256:9999",
        GrantMissing => "grant=null",
        SourcePolicyMissing => "catalog.source_policy=null",
        AdmittedSourcePolicyMissing => "policy.admitted_source_policy=null",
        InstallationMissing => "installation=null",
        InstallationDisabled => "installation.state=disabled",
        InstallationRevoked => "installation.state=revoked",
        InstallationIdOther => "installation.id=installation:other",
        CatalogRevoked => "catalog.revoked=true",
        EmergencyAndCatalogMissing => "policy.emergency_blocked=true|catalog=null",
        TwoTargetInstallationConflict => "two-target installation.id conflict",
        OptionalLayerTransitivity => "three-target first-missing catalog|component|installation|grant;later-present-conflict",
        OptionalLayerAbsence => "sole-or-uniform-missing catalog|component|installation|grant",
        GrantStale => "grant.state=stale",
        GrantExpired => "grant.state=expired",
        GrantRevoked => "grant.state=revoked",
        GrantInstallationOther => "grant.installation_id=installation:other",
        GrantScopeOther => "grant.object_scope=scope:other",
        DefinitionNameChanged => "tool.model_visible_name=campus_search_changed",
        DefinitionDescriptionChanged => "tool.description+=Changed",
        DefinitionSchemaBoolean => "tool.input_schema=boolean-enabled",
        DefinitionNameCollision => "two-target same-visible-name",
        CallNameNotProjected => "model_visible_name=not_projected",
        CallDispatchWrongLiteral => "dispatch_key=dispatch:sha256:wrong",
        CallNoFallback => "known-name-wrong-dispatch-with-second-entry",
        CompleteProjectionErrorMatrix => "execute-complete-projection-error-matrix",
        ProjectionGroupMajorForward => "a.installation.state=disabled|z.catalog=null",
        ProjectionGroupMajorReverse => "a.catalog=null|z.installation.state=disabled",
        ProjectionGroupMajorThree => "a.grant.state=stale|m.installation.state=disabled|z.catalog=null",
        CompleteCallErrorMatrix => "execute-complete-call-error-matrix",
        CallNameBeforeDeny => "model_visible_name=not_projected|policy.emergency_blocked=true",
        CallDispatchBeforeDeny => "dispatch_key=wrong|policy.emergency_blocked=true",
        CallDispatchBeforeArguments => "dispatch_key=wrong|argument_digest=wrong",
        PostEmergency => "policy.emergency_blocked=true",
        PostCatalogRevoked => "catalog_revoked=true",

        FrozenDisabledThenEnabled => "frozen-installation-disabled-then-current-enabled",
    }
}

closed_string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FixturePrecedence {
        NotApplicable => "not-applicable",
        ConstructorOnly => "constructor-only",
        ProjectionGroup2BeforeGroup4 => "group-2-before-group-4",
        ProjectionGroup2BeforeTargetLocal => "group-2-before-target-local",
        ProjectionGroup3CanonicalLeftmost => "group-3/canonical-target/leftmost",
        ProjectionGroup3BeforeGroup9 => "group-3-before-group-9",
        ProjectionGroup4CanonicalLeftmost => "group-4/canonical-target/leftmost",
        ProjectionGroup5CanonicalLeftmost => "group-5/canonical-target/leftmost",
        ProjectionGroup6CanonicalLeftmost => "group-6/canonical-target/leftmost",
        ProjectionGroup7CanonicalLeftmost => "group-7/canonical-target/leftmost",
        ProjectionGroup8CanonicalLeftmost => "group-8/canonical-target/leftmost",
        ProjectionGroup9CanonicalLeftmost => "group-9/canonical-target/leftmost",
        ProjectionGroup10CanonicalLeftmost => "group-10/canonical-target/leftmost",
        ProjectionGroup12 => "group-12",
        ProjectionTargetLocalByGroup => "target-local-by-group",
        ProjectionGroups1Through12 => "groups-1-through-12",
        ProjectionGroup4BeforeGroup5 => "group-4-before-group-5",
        ProjectionGroup4BeforeGroup5BeforeGroup9 => "group-4-before-group-5-before-group-9",
        CallGroup2BeforeDenyAndArguments => "group-2-before-deny-and-arguments",
        CallGroup3BeforeDenyAndArguments => "group-3-before-deny-and-arguments",
        ExactFrozenEntryOnly => "exact-frozen-entry-only",
        CallGroups1Through10 => "groups-1-through-10",
        CallGroup5BeforeGroup8 => "group-5-before-group-8",
        CallGroup3BeforeGroup4 => "group-3-before-group-4",
        CallGroup4 => "group-4",
        CallGroup6 => "group-6",
        CallGroup7 => "group-7",
        CallGroup8 => "group-8",
        ProjectionDenialIsTerminal => "projection-denial-is-terminal",
        DenialProducesNoRun => "denial-produces-no-run",
    }
}

closed_string_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FixtureExpectedName {
        SchemaDialectUnsupported => "SchemaDialectUnsupported",
        SchemaMalformed => "SchemaMalformed",
        SchemaLimitExceeded => "SchemaLimitExceeded",
        ArgumentDuplicateKey => "ArgumentDuplicateKey",
        ArgumentInvalidName => "ArgumentInvalidName",
        ArgumentLimitExceeded => "ArgumentLimitExceeded",
        ArgumentNumberOutOfRange => "ArgumentNumberOutOfRange",
        CanonicalSuccess => "canonical-success",
        DistinctCanonicalValues => "distinct-canonical-values",
        PackageMissing => "PackageMissing",
        PackageNotRunnable => "PackageNotRunnable",
        PackageVersionMismatch => "PackageVersionMismatch",
        PackageDigestMismatch => "PackageDigestMismatch",
        ComponentMissing => "ComponentMissing",
        ComponentIdentityMismatch => "ComponentIdentityMismatch",
        ExecutionIdentityUnknown => "ExecutionIdentityUnknown",
        ExecutionIdentityMismatch => "ExecutionIdentityMismatch",
        ToolMissing => "ToolMissing",
        ToolIdentityMismatch => "ToolIdentityMismatch",
        TenantOrUserScopeMismatch => "TenantOrUserScopeMismatch",
        CapabilityUnknown => "CapabilityUnknown",
        CapabilityNotDeclared => "CapabilityNotDeclared",
        CapabilityManifestMismatch => "CapabilityManifestMismatch",
        CapabilityNotGranted => "CapabilityNotGranted",
        SourcePolicyMissing => "SourcePolicyMissing",
        SourcePolicyMismatch => "SourcePolicyMismatch",
        InstallationMissing => "InstallationMissing",
        InstallationDisabled => "InstallationDisabled",
        InstallationRevoked => "InstallationRevoked",
        InstallationRevisionMismatch => "InstallationRevisionMismatch",
        CatalogRevoked => "CatalogRevoked",
        EmergencyBlocked => "EmergencyBlocked",
        AuthorityConflict => "AuthorityConflict",
        GrantStale => "GrantStale",
        GrantExpired => "GrantExpired",
        GrantRevoked => "GrantRevoked",
        GrantVersionMismatch => "GrantVersionMismatch",
        GrantScopeMismatch => "GrantScopeMismatch",
        ProviderDefinitionAndSchemaSetDigestsChange => "provider-definition-and-schema-set-digests-change",
        ToolNameCollision => "ToolNameCollision",
        ToolNotProjected => "ToolNotProjected",
        DispatchIdentityMismatch => "DispatchIdentityMismatch",
        AllProjectionErrors => "all-ProjectionResolutionError-variants",
        AllAuthorizationErrors => "all-InvocationAuthorizationError-variants",
        NoProjectedEntry => "no-projected-entry",
        SuccessOnly => "success-only",
        SameSchemaSetDifferentSnapshot => "same-schema-set;different-snapshot",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureExpected {
    Named(FixtureExpectedName),
    CanonicalGolden {
        bytes: String,
        digest: String,
    },
    EqualTo {
        case: String,
    },
    ResolvedIdentities([String; 7]),
    Dispatch(String),
    ProjectionGoldens {
        definition: String,
        schema_set: String,
        authority_entry: String,
        authority_set: String,
        snapshot: String,
    },
    ProjectionErrors(Vec<FixtureExpectedName>),
}

impl FixtureExpected {
    pub fn parse(value: &str) -> Result<Self, String> {
        if let Some(rest) = value.strip_prefix("bytes=") {
            let Some((bytes, digest)) = rest.split_once(";digest=") else {
                return Err("canonical golden must contain ;digest=".to_owned());
            };
            if bytes.is_empty() || digest.is_empty() {
                return Err("canonical golden fields must be non-empty".to_owned());
            }
            return Ok(Self::CanonicalGolden {
                bytes: bytes.to_owned(),
                digest: digest.to_owned(),
            });
        }
        if let Some(case) = value.strip_prefix("equal-to=") {
            if case.is_empty() {
                return Err("equal-to case must be non-empty".to_owned());
            }
            return Ok(Self::EqualTo {
                case: case.to_owned(),
            });
        }
        if let Some(dispatch) = value.strip_prefix("dispatch:") {
            if dispatch.is_empty() {
                return Err("dispatch golden must be non-empty".to_owned());
            }
            return Ok(Self::Dispatch(format!("dispatch:{dispatch}")));
        }
        if let Some(rest) = value.strip_prefix("definition=") {
            let fields = rest.split(';').collect::<Vec<_>>();
            if fields.len() != 5 {
                return Err("projection golden must contain five fields".to_owned());
            }
            let take = |field: &str, prefix: &str| {
                field
                    .strip_prefix(prefix)
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned)
                    .ok_or_else(|| format!("projection golden missing {prefix}"))
            };
            return Ok(Self::ProjectionGoldens {
                definition: fields[0].to_owned(),
                schema_set: take(fields[1], "schema_set=")?,
                authority_entry: take(fields[2], "authority_entry=")?,
                authority_set: take(fields[3], "authority_set=")?,
                snapshot: take(fields[4], "snapshot=")?,
            });
        }
        if value.contains('|') {
            let parts = value.split('|').collect::<Vec<_>>();
            if let Ok(errors) = parts
                .iter()
                .map(|part| FixtureExpectedName::parse(part))
                .collect::<Result<Vec<_>, _>>()
            {
                return Ok(Self::ProjectionErrors(errors));
            }
            let identities: [String; 7] = parts
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| "resolved identity golden must contain seven fields".to_owned())?;
            if identities.iter().any(String::is_empty) {
                return Err("resolved identity golden fields must be non-empty".to_owned());
            }
            return Ok(Self::ResolvedIdentities(identities));
        }
        FixtureExpectedName::parse(value).map(Self::Named)
    }
}

impl<'de> Deserialize<'de> for FixtureExpected {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationFixture {
    pub schema_version: String,
    pub synthetic: bool,
    pub fixture: String,
    pub cases: Vec<InvocationFixtureCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationFixtureCase {
    pub name: String,
    pub api: FixtureApi,
    pub recipe: FixtureRecipe,
    pub expected: FixtureExpected,
    pub precedence: FixturePrecedence,
}
