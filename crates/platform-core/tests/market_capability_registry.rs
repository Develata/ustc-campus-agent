use ustc_campus_agent_core::invocation::{CapabilityClass, CapabilityId, ConfirmationPolicy};
use ustc_campus_agent_core::market::capability::{
    AutoGrantDisposition, CapabilityPolicyChange, CapabilityRegistryLoadError, CapabilityStatus,
    DataClass, EffectClass, RiskClass, ScopeKind, compare_capability_definitions,
    compare_capability_policy, load_capability_registry,
};

const REGISTRY: &[u8] = include_bytes!("../../../market/capabilities/registry.json");

const SINGLE: &str = r#"{"schemaVersion":"capability-registry/v1","registryRevision":"capability-registry:2026-07-29-01","capabilities":[{"id":"campus.public_rules.read","effectClass":"Read","dataClass":"PublicCampusFact","scopeKind":"CampusPublic","autoGrant":"FirstPartyDefaultOnly","confirmationDefault":"Allow","status":"Active"}]}"#;

const REVISION: &str = "capability-registry:2026-07-29-01";

fn load(source: &[u8]) -> ustc_campus_agent_core::market::capability::CapabilityRegistry {
    match load_capability_registry(source) {
        Ok(registry) => registry,
        Err(error) => panic!("fixture must load: {error}"),
    }
}

fn load_err(source: &[u8]) -> CapabilityRegistryLoadError {
    match load_capability_registry(source) {
        Ok(_) => panic!("expected rejection, got a registry"),
        Err(error) => error,
    }
}

fn json_bytes(value: &serde_json::Value) -> Vec<u8> {
    match serde_json::to_vec(value) {
        Ok(bytes) => bytes,
        Err(error) => panic!("JSON value must serialize: {error}"),
    }
}

fn parsed_capability_id(value: &str) -> CapabilityId {
    match CapabilityId::parse(value) {
        Ok(id) => id,
        Err(error) => panic!("valid capability id fixture: {error}"),
    }
}

fn registry_from_values(
    capabilities: &[serde_json::Value],
) -> ustc_campus_agent_core::market::capability::CapabilityRegistry {
    let value = serde_json::json!({
        "schemaVersion": "capability-registry/v1",
        "registryRevision": REVISION,
        "capabilities": capabilities,
    });
    load(&json_bytes(&value))
}

fn registry_from_value(
    capability: &serde_json::Value,
) -> ustc_campus_agent_core::market::capability::CapabilityRegistry {
    registry_from_values(std::slice::from_ref(capability))
}

fn registry_err_from_values(capabilities: &[serde_json::Value]) -> CapabilityRegistryLoadError {
    let value = serde_json::json!({
        "schemaVersion": "capability-registry/v1",
        "registryRevision": REVISION,
        "capabilities": capabilities,
    });
    load_err(&json_bytes(&value))
}

fn registry_err_from_value(capability: serde_json::Value) -> CapabilityRegistryLoadError {
    registry_err_from_values(&[capability])
}

fn public_read_value() -> serde_json::Value {
    serde_json::json!({
        "id": "campus.public_rules.read",
        "effectClass": "Read",
        "dataClass": "PublicCampusFact",
        "scopeKind": "CampusPublic",
        "autoGrant": "FirstPartyDefaultOnly",
        "confirmationDefault": "Allow",
        "status": "Active"
    })
}

fn capability_value(
    id: &str,
    effect: &str,
    data: &str,
    scope: &str,
    auto_grant: &str,
    confirmation: &str,
    status: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "effectClass": effect,
        "dataClass": data,
        "scopeKind": scope,
        "autoGrant": auto_grant,
        "confirmationDefault": confirmation,
        "status": status
    })
}

fn admitted_capability_value(id: &str, effect: EffectClass, data: DataClass) -> serde_json::Value {
    let (scope, confirmation, auto_grant) = match data {
        DataClass::PublicCampusFact => ("CampusPublic", "Allow", "FirstPartyDefaultOnly"),
        DataClass::TenantPrivateFact | DataClass::UserProfile => {
            ("TenantPrivateUser", "Ask", "Never")
        }
        _ => ("CampusPublic", "Allow", "FirstPartyDefaultOnly"),
    };
    capability_value(
        id,
        effect_name(effect),
        data_name(data),
        scope,
        auto_grant,
        confirmation,
        "Active",
    )
}

fn effect_name(effect: EffectClass) -> &'static str {
    match effect {
        EffectClass::Read => "Read",
        EffectClass::Write => "Write",
        EffectClass::Destructive => "Destructive",
        EffectClass::Linkout => "Linkout",
        EffectClass::Diagnostic => "Diagnostic",
    }
}

fn data_name(data: DataClass) -> &'static str {
    match data {
        DataClass::PublicCampusFact => "PublicCampusFact",
        DataClass::TenantPrivateFact => "TenantPrivateFact",
        DataClass::UserProfile => "UserProfile",
        DataClass::Credential => "Credential",
        DataClass::Administrative => "Administrative",
    }
}

#[test]
fn current_registry_loads_with_exact_eight_definitions() {
    let registry = load(REGISTRY);
    assert_eq!(
        registry.registry_revision().as_str(),
        "capability-registry:2026-07-29-01"
    );
    assert_eq!(registry.definitions().len(), 8);

    let sorted_ids: Vec<&str> = registry
        .definitions()
        .iter()
        .map(|definition| definition.id().as_str())
        .collect();
    assert_eq!(
        sorted_ids,
        [
            "campus.community_review.linkout",
            "campus.public_changes.read",
            "campus.public_course.read",
            "campus.public_plan.read",
            "campus.public_rules.read",
            "user.own_academic_snapshot.read",
            "user.own_course_preferences.read",
            "user.own_plan_draft.write",
        ]
    );

    for definition in registry.definitions() {
        match definition.id().as_str() {
            "campus.community_review.linkout" => {
                assert_eq!(definition.effect_class(), EffectClass::Linkout);
                assert_eq!(definition.data_class(), DataClass::PublicCampusFact);
                assert_eq!(definition.scope_kind(), ScopeKind::CampusPublic);
                assert_eq!(
                    definition.auto_grant(),
                    AutoGrantDisposition::FirstPartyDefaultOnly
                );
                assert_eq!(definition.confirmation_default(), ConfirmationPolicy::Allow);
                assert_eq!(definition.status(), CapabilityStatus::Active);
                assert!(definition.is_first_party_default_auto_grant_candidate());
                assert_eq!(
                    definition.compatibility_class(),
                    Some(CapabilityClass::PublicLinkout)
                );
                assert_eq!(definition.risk_class(), RiskClass::Low);
            }
            "campus.public_changes.read"
            | "campus.public_plan.read"
            | "campus.public_rules.read"
            | "campus.public_course.read" => {
                assert_eq!(definition.effect_class(), EffectClass::Read);
                assert_eq!(definition.data_class(), DataClass::PublicCampusFact);
                assert_eq!(definition.scope_kind(), ScopeKind::CampusPublic);
                assert_eq!(
                    definition.auto_grant(),
                    AutoGrantDisposition::FirstPartyDefaultOnly
                );
                assert_eq!(definition.confirmation_default(), ConfirmationPolicy::Allow);
                assert_eq!(definition.status(), CapabilityStatus::Active);
                assert!(definition.is_first_party_default_auto_grant_candidate());
                assert_eq!(
                    definition.compatibility_class(),
                    Some(CapabilityClass::PublicRead)
                );
                assert_eq!(definition.risk_class(), RiskClass::Low);
            }
            "user.own_academic_snapshot.read" | "user.own_course_preferences.read" => {
                assert_eq!(definition.effect_class(), EffectClass::Read);
                assert_eq!(definition.data_class(), DataClass::UserProfile);
                assert_eq!(definition.scope_kind(), ScopeKind::TenantPrivateUser);
                assert_eq!(definition.auto_grant(), AutoGrantDisposition::Never);
                assert_eq!(definition.confirmation_default(), ConfirmationPolicy::Ask);
                assert_eq!(definition.status(), CapabilityStatus::Active);
                assert!(!definition.is_first_party_default_auto_grant_candidate());
                assert_eq!(
                    definition.compatibility_class(),
                    Some(CapabilityClass::TenantPrivateRead)
                );
                assert_eq!(definition.risk_class(), RiskClass::High);
            }
            "user.own_plan_draft.write" => {
                assert_eq!(definition.effect_class(), EffectClass::Write);
                assert_eq!(definition.data_class(), DataClass::UserProfile);
                assert_eq!(definition.scope_kind(), ScopeKind::TenantPrivateUser);
                assert_eq!(definition.auto_grant(), AutoGrantDisposition::Never);
                assert_eq!(definition.confirmation_default(), ConfirmationPolicy::Ask);
                assert_eq!(definition.status(), CapabilityStatus::Active);
                assert!(!definition.is_first_party_default_auto_grant_candidate());
                assert_eq!(
                    definition.compatibility_class(),
                    Some(CapabilityClass::TenantPrivateWrite)
                );
                assert_eq!(definition.risk_class(), RiskClass::High);
            }
            other => panic!("unexpected id {other}"),
        }
    }
}

#[test]
fn enum_risk_and_compatibility_mappings_are_exact() {
    let cases: [(EffectClass, DataClass, RiskClass, Option<CapabilityClass>); 6] = [
        (
            EffectClass::Read,
            DataClass::PublicCampusFact,
            RiskClass::Low,
            Some(CapabilityClass::PublicRead),
        ),
        (
            EffectClass::Linkout,
            DataClass::PublicCampusFact,
            RiskClass::Low,
            Some(CapabilityClass::PublicLinkout),
        ),
        (
            EffectClass::Read,
            DataClass::TenantPrivateFact,
            RiskClass::Medium,
            Some(CapabilityClass::TenantPrivateRead),
        ),
        (
            EffectClass::Read,
            DataClass::UserProfile,
            RiskClass::High,
            Some(CapabilityClass::TenantPrivateRead),
        ),
        (
            EffectClass::Write,
            DataClass::TenantPrivateFact,
            RiskClass::High,
            Some(CapabilityClass::TenantPrivateWrite),
        ),
        (
            EffectClass::Write,
            DataClass::UserProfile,
            RiskClass::High,
            Some(CapabilityClass::TenantPrivateWrite),
        ),
    ];
    let mut capabilities = Vec::new();
    for (index, (effect, data, _, _)) in cases.iter().enumerate() {
        capabilities.push(admitted_capability_value(
            &format!("campus.case_{index}.read"),
            *effect,
            *data,
        ));
    }
    let registry = registry_from_values(&capabilities);
    for (definition, (effect, data, risk, compatibility)) in
        registry.definitions().iter().zip(cases.iter())
    {
        assert_eq!(definition.effect_class(), *effect);
        assert_eq!(definition.data_class(), *data);
        assert_eq!(definition.risk_class(), *risk);
        assert_eq!(definition.compatibility_class(), *compatibility);
    }
}

#[test]
fn source_size_and_malformed_json_fail_closed() {
    let oversized = vec![b' '; 1_048_577];
    assert_eq!(
        load_err(&oversized),
        CapabilityRegistryLoadError::SourceTooLarge
    );
    assert_eq!(load_err(b"{"), CapabilityRegistryLoadError::JsonRejected);
    assert_eq!(load_err(b"[]"), CapabilityRegistryLoadError::JsonRejected);
    assert_eq!(
        load_err(b"{\"capabilities\":[]"),
        CapabilityRegistryLoadError::JsonRejected
    );
}

#[test]
fn duplicate_json_keys_fail_closed() {
    let duplicate_top = SINGLE.replacen(
        r#""schemaVersion":"capability-registry/v1","#,
        r#""schemaVersion":"capability-registry/v1","schemaVersion":"capability-registry/v2","#,
        1,
    );
    assert_eq!(
        load_err(duplicate_top.as_bytes()),
        CapabilityRegistryLoadError::JsonRejected
    );

    let duplicate_capability_field = SINGLE.replacen(
        r#""effectClass":"Read","#,
        r#""effectClass":"Read","effectClass":"Write","#,
        1,
    );
    assert_eq!(
        load_err(duplicate_capability_field.as_bytes()),
        CapabilityRegistryLoadError::JsonRejected
    );
}

#[test]
fn duplicate_capability_ids_fail_closed() {
    assert_eq!(
        registry_err_from_values(&[public_read_value(), public_read_value()]),
        CapabilityRegistryLoadError::DuplicateCapabilityId
    );
}

#[test]
fn invalid_capability_id_grammar_fail_closed() {
    for id in [
        "campus",
        "Campus.public.read",
        "campus..read",
        ".campus.read",
        "campus.public.",
        "campus .public.read",
        "campus.Public.read",
    ] {
        let mut value = public_read_value();
        value["id"] = serde_json::json!(id);
        assert_eq!(
            registry_err_from_value(value),
            CapabilityRegistryLoadError::InvalidCapabilityId,
            "id {id} must be rejected"
        );
    }
}

#[test]
fn missing_extra_and_unknown_fields_fail_closed() {
    let mut missing = public_read_value();
    missing["status"].take();
    assert_eq!(
        registry_err_from_value(missing),
        CapabilityRegistryLoadError::JsonRejected
    );

    let mut extra = public_read_value();
    extra["autoGrantEligible"] = serde_json::json!(true);
    assert_eq!(
        registry_err_from_value(extra),
        CapabilityRegistryLoadError::JsonRejected
    );

    let mut unknown_enum = public_read_value();
    unknown_enum["effectClass"] = serde_json::json!("ReadWrite");
    assert_eq!(
        registry_err_from_value(unknown_enum),
        CapabilityRegistryLoadError::JsonRejected
    );
}

#[test]
fn invalid_schema_version_and_registry_revision_fail_closed() {
    let mut value = serde_json::json!({
        "schemaVersion": "capability-registry/v2",
        "registryRevision": REVISION,
        "capabilities": [public_read_value()],
    });
    assert_eq!(
        load_err(&json_bytes(&value)),
        CapabilityRegistryLoadError::InvalidSchemaVersion
    );

    value["schemaVersion"] = serde_json::json!("capability-registry/v1");
    value["registryRevision"] = serde_json::json!("catalog:bad");
    assert_eq!(
        load_err(&json_bytes(&value)),
        CapabilityRegistryLoadError::InvalidRegistryRevision
    );

    value["registryRevision"] = serde_json::json!("capability-registry:");
    assert_eq!(
        load_err(&json_bytes(&value)),
        CapabilityRegistryLoadError::InvalidRegistryRevision
    );
}

#[test]
fn forbidden_and_incoherent_combinations_fail_closed() {
    let mut destructive = public_read_value();
    destructive["effectClass"] = serde_json::json!("Destructive");
    assert_eq!(
        registry_err_from_value(destructive),
        CapabilityRegistryLoadError::ForbiddenCombination
    );

    let mut credential = public_read_value();
    credential["dataClass"] = serde_json::json!("Credential");
    assert_eq!(
        registry_err_from_value(credential),
        CapabilityRegistryLoadError::ForbiddenCombination
    );

    let mut administrative = public_read_value();
    administrative["dataClass"] = serde_json::json!("Administrative");
    assert_eq!(
        registry_err_from_value(administrative),
        CapabilityRegistryLoadError::ForbiddenCombination
    );

    let mut operator_scope = public_read_value();
    operator_scope["scopeKind"] = serde_json::json!("OperatorAdministrative");
    assert_eq!(
        registry_err_from_value(operator_scope),
        CapabilityRegistryLoadError::ForbiddenCombination
    );

    let mut wrong_scope = public_read_value();
    wrong_scope["scopeKind"] = serde_json::json!("TenantPrivateUser");
    assert_eq!(
        registry_err_from_value(wrong_scope),
        CapabilityRegistryLoadError::IncoherentDefinition
    );

    let mut wrong_confirmation = public_read_value();
    wrong_confirmation["confirmationDefault"] = serde_json::json!("Ask");
    assert_eq!(
        registry_err_from_value(wrong_confirmation),
        CapabilityRegistryLoadError::IncoherentDefinition
    );

    let mut wrong_auto_public = public_read_value();
    wrong_auto_public["autoGrant"] = serde_json::json!("Never");
    assert_eq!(
        registry_err_from_value(wrong_auto_public),
        CapabilityRegistryLoadError::IncoherentDefinition
    );

    let wrong_auto_tenant = capability_value(
        "user.own_academic_snapshot.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "FirstPartyDefaultOnly",
        "Ask",
        "Active",
    );
    assert_eq!(
        registry_err_from_value(wrong_auto_tenant),
        CapabilityRegistryLoadError::IncoherentDefinition
    );

    let mut write_public = public_read_value();
    write_public["effectClass"] = serde_json::json!("Write");
    assert_eq!(
        registry_err_from_value(write_public),
        CapabilityRegistryLoadError::ForbiddenCombination
    );
}

#[test]
fn auto_grant_candidacy_and_deprecated_revoked_exclusions() {
    let active_public = public_read_value();
    let deprecated_public = capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Deprecated",
    );
    let revoked_public = capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Revoked",
    );
    let tenant = capability_value(
        "user.own_academic_snapshot.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "Never",
        "Ask",
        "Active",
    );

    let public_rules = parsed_capability_id("campus.public_rules.read");
    let academic = parsed_capability_id("user.own_academic_snapshot.read");

    let active_registry = registry_from_values(&[active_public]);
    let active_def = match active_registry.find(&public_rules) {
        Some(definition) => definition,
        None => panic!("active definition must be present"),
    };
    assert!(active_def.is_first_party_default_auto_grant_candidate());
    assert_eq!(active_def.status(), CapabilityStatus::Active);
    assert_eq!(
        active_def.auto_grant(),
        AutoGrantDisposition::FirstPartyDefaultOnly
    );

    let tenant_registry = registry_from_values(&[tenant]);
    let tenant_def = match tenant_registry.find(&academic) {
        Some(definition) => definition,
        None => panic!("tenant definition must be present"),
    };
    assert!(!tenant_def.is_first_party_default_auto_grant_candidate());
    assert_eq!(tenant_def.status(), CapabilityStatus::Active);
    assert_eq!(tenant_def.auto_grant(), AutoGrantDisposition::Never);

    let deprecated_registry = registry_from_values(&[deprecated_public]);
    let dep_def = match deprecated_registry.find(&public_rules) {
        Some(definition) => definition,
        None => panic!("deprecated definition must be present"),
    };
    assert_eq!(dep_def.status(), CapabilityStatus::Deprecated);
    assert_eq!(dep_def.auto_grant(), AutoGrantDisposition::Never);
    assert!(!dep_def.is_first_party_default_auto_grant_candidate());

    let revoked_registry = registry_from_values(&[revoked_public]);
    let rev_def = match revoked_registry.find(&public_rules) {
        Some(definition) => definition,
        None => panic!("revoked definition must be present"),
    };
    assert_eq!(rev_def.status(), CapabilityStatus::Revoked);
    assert!(!rev_def.is_first_party_default_auto_grant_candidate());
}

#[test]
fn deterministic_ordering_and_permutation_independent_digest() {
    let registry = load(REGISTRY);
    let mut permuted = match serde_json::from_slice::<serde_json::Value>(REGISTRY) {
        Ok(value) => value,
        Err(error) => panic!("fixture must parse: {error}"),
    };
    if let Some(capabilities) = permuted["capabilities"].as_array_mut() {
        capabilities.reverse();
    }
    let permuted_registry = load(&json_bytes(&permuted));
    assert_eq!(registry, permuted_registry);
    assert_eq!(
        registry.registry_digest().as_str(),
        permuted_registry.registry_digest().as_str()
    );
    assert_eq!(registry.definitions().len(), 8);
}

#[test]
fn fixed_definition_and_registry_digest_vectors() {
    let registry = load(REGISTRY);
    assert_eq!(
        registry.registry_digest().as_str(),
        "sha256:428dc176278de88565478b61733606cebfc6e1da5bd6ffa4be0b2afa4694e92a"
    );
    let linkout_def = match registry.find(&parsed_capability_id("campus.community_review.linkout"))
    {
        Some(definition) => definition,
        None => panic!("linkout definition must be present"),
    };
    assert_eq!(
        linkout_def.definition_digest().as_str(),
        "sha256:0f25515d529a49c634ebaeb4073900e2223b46bb730f621ab081663efea08c72"
    );
    let public_rules = match registry.find(&parsed_capability_id("campus.public_rules.read")) {
        Some(definition) => definition,
        None => panic!("public_rules definition must be present"),
    };
    assert_eq!(
        public_rules.definition_digest().as_str(),
        "sha256:bc371fdcfe32e81e59652b81c57bff64af49827894ed232b964fd4730ea7610b"
    );
    let plan_draft = match registry.find(&parsed_capability_id("user.own_plan_draft.write")) {
        Some(definition) => definition,
        None => panic!("plan_draft definition must be present"),
    };
    assert_eq!(
        plan_draft.definition_digest().as_str(),
        "sha256:a5daebb9965ec0eaca1b1cd2d7778a9acb2ea4ece75196f5cb07c9957fb0ab57"
    );
}

#[test]
fn one_field_change_alters_definition_digest() {
    let read_registry = registry_from_values(&[capability_value(
        "campus.dual.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "FirstPartyDefaultOnly",
        "Allow",
        "Active",
    )]);
    let linkout_registry = registry_from_values(&[capability_value(
        "campus.dual.read",
        "Linkout",
        "PublicCampusFact",
        "CampusPublic",
        "FirstPartyDefaultOnly",
        "Allow",
        "Active",
    )]);
    let id = parsed_capability_id("campus.dual.read");
    let read_digest = read_registry
        .find(&id)
        .map(|definition| definition.definition_digest().as_str().to_owned())
        .unwrap_or_else(|| panic!("read definition present"));
    let linkout_digest = linkout_registry
        .find(&id)
        .map(|definition| definition.definition_digest().as_str().to_owned())
        .unwrap_or_else(|| panic!("linkout definition present"));
    assert_ne!(read_digest, linkout_digest);
}

#[test]
fn registry_revision_does_not_change_definition_digests() {
    let first = load(REGISTRY);
    let other = match serde_json::from_slice::<serde_json::Value>(REGISTRY) {
        Ok(mut value) => {
            value["registryRevision"] = serde_json::json!("capability-registry:other-revision-02");
            load(&json_bytes(&value))
        }
        Err(error) => panic!("fixture must parse: {error}"),
    };
    assert_ne!(
        first.registry_digest().as_str(),
        other.registry_digest().as_str()
    );
    for definition in first.definitions() {
        let other_definition = match other.find(definition.id()) {
            Some(definition) => definition,
            None => panic!("definition must be present in both registries"),
        };
        assert_eq!(
            definition.definition_digest().as_str(),
            other_definition.definition_digest().as_str()
        );
    }
}

#[test]
fn policy_change_comparator_branches_and_precedence() {
    let public_rules = parsed_capability_id("campus.public_rules.read");
    let academic = parsed_capability_id("user.own_academic_snapshot.read");

    let active_public = public_read_value();
    let deprecated_public = capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Deprecated",
    );
    let revoked_public = capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Revoked",
    );
    let linkout_public = capability_value(
        "campus.public_rules.read",
        "Linkout",
        "PublicCampusFact",
        "CampusPublic",
        "FirstPartyDefaultOnly",
        "Allow",
        "Active",
    );
    let active_tenant = capability_value(
        "campus.public_rules.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "Never",
        "Ask",
        "Active",
    );
    let deprecated_tenant = capability_value(
        "campus.public_rules.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "Never",
        "Ask",
        "Deprecated",
    );

    let empty: &[serde_json::Value] = &[];

    // Unchanged: identical definition despite revision change.
    let old = registry_from_value(&active_public);
    let new = registry_from_value(&active_public);
    assert_eq!(
        compare_capability_policy(&old, &new, &public_rules),
        CapabilityPolicyChange::Unchanged
    );

    // Unchanged: absent from both registries.
    assert_eq!(
        compare_capability_policy(&old, &new, &academic),
        CapabilityPolicyChange::Unchanged
    );

    // RemovedOrRevoked: old present, new absent.
    let new_empty = registry_from_values(empty);
    assert_eq!(
        compare_capability_policy(&old, &new_empty, &public_rules),
        CapabilityPolicyChange::RemovedOrRevoked
    );

    // RemovedOrRevoked: old active, new revoked.
    let new_revoked = registry_from_value(&revoked_public);
    assert_eq!(
        compare_capability_policy(&old, &new_revoked, &public_rules),
        CapabilityPolicyChange::RemovedOrRevoked
    );

    // ExpansionRequiresReapproval: old absent, new active present.
    assert_eq!(
        compare_capability_policy(&new_empty, &old, &public_rules),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );

    // ExpansionRequiresReapproval: deprecated -> active (rule 3).
    let old_deprecated = registry_from_value(&deprecated_public);
    assert_eq!(
        compare_capability_policy(&old_deprecated, &new, &public_rules),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );

    // ExpansionRequiresReapproval: effect/data/scope change (rule 5).
    let new_linkout = registry_from_value(&linkout_public);
    assert_eq!(
        compare_capability_policy(&old, &new_linkout, &public_rules),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );

    // Narrowed: active -> deprecated (rule 4).
    assert_eq!(
        compare_capability_policy(&old, &old_deprecated, &public_rules),
        CapabilityPolicyChange::Narrowed
    );

    // Combined-fault precedence: active -> deprecated AND data change; rule 4 beats rule 5.
    let new_deprecated_tenant = registry_from_value(&deprecated_tenant);
    let old_active_tenant = registry_from_value(&active_tenant);
    assert_eq!(
        compare_capability_policy(&old_active_tenant, &new_deprecated_tenant, &public_rules),
        CapabilityPolicyChange::Narrowed
    );

    // Combined-fault precedence: deprecated -> active AND data change; rule 3 beats rule 5.
    let old_deprecated_tenant = registry_from_value(&deprecated_tenant);
    let new_active_public = registry_from_value(&active_public);
    assert_eq!(
        compare_capability_policy(&old_deprecated_tenant, &new_active_public, &public_rules),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );

    // ExpansionRequiresReapproval: active -> active with data change (rule 5).
    let new_active_tenant = registry_from_value(&active_tenant);
    assert_eq!(
        compare_capability_policy(&old, &new_active_tenant, &public_rules),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );
}

#[test]
fn errors_do_not_leak_rejected_source_fragments() {
    let sentinel = "DO_NOT_ECHO_PRIVATE_FRAGMENT_9381";
    let invalid_id = format!(
        r#"{{"schemaVersion":"capability-registry/v1","registryRevision":"capability-registry:2026-07-29-01","capabilities":[{{"id":"{sentinel}.bad","effectClass":"Read","dataClass":"PublicCampusFact","scopeKind":"CampusPublic","autoGrant":"FirstPartyDefaultOnly","confirmationDefault":"Allow","status":"Active"}}]}}"#
    );
    let error = load_err(invalid_id.as_bytes());
    assert_eq!(error, CapabilityRegistryLoadError::InvalidCapabilityId);
    assert!(!format!("{error}").contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));

    let unknown_enum = format!(
        r#"{{"schemaVersion":"capability-registry/v1","registryRevision":"capability-registry:2026-07-29-01","capabilities":[{{"id":"campus.public_rules.read","effectClass":"{sentinel}","dataClass":"PublicCampusFact","scopeKind":"CampusPublic","autoGrant":"FirstPartyDefaultOnly","confirmationDefault":"Allow","status":"Active"}}]}}"#
    );
    let json_error = load_err(unknown_enum.as_bytes());
    assert_eq!(json_error, CapabilityRegistryLoadError::JsonRejected);
    assert!(!format!("{json_error}").contains(sentinel));
    assert!(!format!("{json_error:?}").contains(sentinel));
}

#[test]
fn empty_registry_loads_with_zero_definitions() {
    let value = serde_json::json!({
        "schemaVersion": "capability-registry/v1",
        "registryRevision": REVISION,
        "capabilities": [],
    });
    let registry = load(&json_bytes(&value));
    assert_eq!(registry.definitions().len(), 0);
    assert!(
        registry
            .find(&parsed_capability_id("campus.public_rules.read"))
            .is_none()
    );
}

#[test]
fn definition_classifier_preserves_existing_policy_matrix() {
    let id = parsed_capability_id("campus.public_rules.read");

    let active_public = public_read_value();
    let deprecated_public = capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Deprecated",
    );
    let revoked_public = capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Revoked",
    );
    let linkout_public = capability_value(
        "campus.public_rules.read",
        "Linkout",
        "PublicCampusFact",
        "CampusPublic",
        "FirstPartyDefaultOnly",
        "Allow",
        "Active",
    );
    let active_tenant = capability_value(
        "campus.public_rules.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "Never",
        "Ask",
        "Active",
    );
    let deprecated_tenant = capability_value(
        "campus.public_rules.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "Never",
        "Ask",
        "Deprecated",
    );

    let cases: &[(serde_json::Value, serde_json::Value)] = &[
        (active_public.clone(), active_public.clone()),
        (active_public.clone(), deprecated_public.clone()),
        (deprecated_public.clone(), active_public.clone()),
        (active_public.clone(), revoked_public.clone()),
        (active_public.clone(), linkout_public.clone()),
        (active_public.clone(), active_tenant.clone()),
        (active_tenant.clone(), deprecated_tenant.clone()),
        (deprecated_tenant.clone(), active_public.clone()),
    ];

    for (old_value, new_value) in cases {
        let old_reg = registry_from_value(old_value);
        let new_reg = registry_from_value(new_value);
        let old_def = old_reg.find(&id);
        let new_def = new_reg.find(&id);
        let registry_result = compare_capability_policy(&old_reg, &new_reg, &id);
        let definition_result = compare_capability_definitions(old_def, new_def);
        assert_eq!(
            registry_result, definition_result,
            "definition classifier must agree with registry comparator"
        );
    }
}

#[test]
fn definition_classifier_handles_none_added_removed_revoked_and_all_axes() {
    let id = parsed_capability_id("campus.public_rules.read");

    let active = registry_from_value(&public_read_value());
    let active_def = active.find(&id).expect("active definition exists");

    let deprecated = registry_from_value(&capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Deprecated",
    ));
    let deprecated_def = deprecated.find(&id).expect("deprecated definition exists");

    let revoked = registry_from_value(&capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Revoked",
    ));
    let revoked_def = revoked.find(&id).expect("revoked definition exists");

    let policy_axes_changed = registry_from_value(&capability_value(
        "campus.public_rules.read",
        "Read",
        "UserProfile",
        "TenantPrivateUser",
        "Never",
        "Ask",
        "Active",
    ));
    let policy_axes_changed_def = policy_axes_changed
        .find(&id)
        .expect("policy-axes-changed definition exists");

    let axis_changed = registry_from_value(&capability_value(
        "campus.public_rules.read",
        "Linkout",
        "PublicCampusFact",
        "CampusPublic",
        "FirstPartyDefaultOnly",
        "Allow",
        "Active",
    ));
    let axis_changed_def = axis_changed
        .find(&id)
        .expect("axis-changed definition exists");

    assert_eq!(
        compare_capability_definitions(None, None),
        CapabilityPolicyChange::Unchanged
    );
    assert_eq!(
        compare_capability_definitions(None, Some(active_def)),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), None),
        CapabilityPolicyChange::RemovedOrRevoked
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(revoked_def)),
        CapabilityPolicyChange::RemovedOrRevoked
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(active_def)),
        CapabilityPolicyChange::Unchanged
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(deprecated_def)),
        CapabilityPolicyChange::Narrowed
    );
    assert_eq!(
        compare_capability_definitions(Some(deprecated_def), Some(active_def)),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(policy_axes_changed_def)),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(axis_changed_def)),
        CapabilityPolicyChange::ExpansionRequiresReapproval
    );
}

#[test]
fn definition_classifier_uses_complete_definition_not_digest_or_caller_hint() {
    let id = parsed_capability_id("campus.public_rules.read");

    let active = registry_from_value(&public_read_value());
    let active_def = active.find(&id).expect("active definition exists");

    let deprecated = registry_from_value(&capability_value(
        "campus.public_rules.read",
        "Read",
        "PublicCampusFact",
        "CampusPublic",
        "Never",
        "Allow",
        "Deprecated",
    ));
    let deprecated_def = deprecated.find(&id).expect("deprecated definition exists");

    let axis_changed = registry_from_value(&capability_value(
        "campus.public_rules.read",
        "Linkout",
        "PublicCampusFact",
        "CampusPublic",
        "FirstPartyDefaultOnly",
        "Allow",
        "Active",
    ));
    let axis_changed_def = axis_changed
        .find(&id)
        .expect("axis-changed definition exists");

    assert_ne!(
        active_def.definition_digest(),
        deprecated_def.definition_digest(),
        "fixture must produce distinct digests"
    );
    assert_ne!(
        active_def.definition_digest(),
        axis_changed_def.definition_digest(),
        "fixture must produce distinct digests"
    );

    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(deprecated_def)),
        CapabilityPolicyChange::Narrowed,
        "complete definition axes drive classification, not digest inequality alone"
    );
    assert_eq!(
        compare_capability_definitions(Some(active_def), Some(axis_changed_def)),
        CapabilityPolicyChange::ExpansionRequiresReapproval,
        "axis change must be classified as expansion, not narrowed"
    );
    assert_eq!(
        compare_capability_definitions(Some(deprecated_def), Some(active_def)),
        CapabilityPolicyChange::ExpansionRequiresReapproval,
        "status reactivation is an expansion regardless of other axis agreement"
    );
}
