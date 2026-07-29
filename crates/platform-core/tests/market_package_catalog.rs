use ustc_campus_agent_core::invocation::{CatalogRevision, PackageId, PackageVersion};
use ustc_campus_agent_core::market::{
    CatalogReadModel, CatalogReadModelError, ImplementationStatus, PackageField, PackageLoadError,
    PackageValidationErrorKind, ValidatedPackageManifest, load_package_manifest,
};

const AFFAIRS: &[u8] =
    include_bytes!("../../../market/packages/ustc.affairs-navigator/package.json");
const CHANGE_RADAR: &[u8] =
    include_bytes!("../../../market/packages/ustc.change-radar/package.json");
const OPPORTUNITY: &[u8] =
    include_bytes!("../../../market/packages/ustc.opportunity-graph/package.json");

fn load(source: &[u8]) -> ValidatedPackageManifest {
    match load_package_manifest(source) {
        Ok(manifest) => manifest,
        Err(error) => panic!("fixture must load: {error}"),
    }
}

fn invalid(source: &[u8], field: PackageField, kind: PackageValidationErrorKind) {
    let result = load_package_manifest(source);
    match result {
        Err(PackageLoadError::InvalidManifest(error)) => {
            assert_eq!(error.field(), field);
            assert_eq!(error.kind(), kind);
        }
        other => panic!("expected semantic rejection, got {other:?}"),
    }
}

fn parsed_package_id(value: &str) -> PackageId {
    match PackageId::parse(value) {
        Ok(value) => value,
        Err(error) => panic!("valid package id fixture: {error}"),
    }
}

fn parsed_version(value: &str) -> PackageVersion {
    match PackageVersion::parse(value) {
        Ok(value) => value,
        Err(error) => panic!("valid package version fixture: {error}"),
    }
}

fn parsed_catalog_revision(value: &str) -> CatalogRevision {
    match CatalogRevision::parse(value) {
        Ok(value) => value,
        Err(error) => panic!("valid catalog revision fixture: {error}"),
    }
}

fn community_manifest_value() -> serde_json::Value {
    match serde_json::from_slice(br#"{
      "id":"community.example","version":"1.0.0","publisher":"community-team",
      "tier":"VerifiedCommunityText","displayName":"Example","description":"Bounded metadata",
      "implementationStatus":"development",
      "installPolicy":{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true},
      "components":[],"capabilities":[],"sourcePolicy":{"policy":"bounded"}
    }"#) {
        Ok(value) => value,
        Err(error) => panic!("valid synthetic manifest JSON: {error}"),
    }
}

fn json_bytes(value: &serde_json::Value) -> Vec<u8> {
    match serde_json::to_vec(value) {
        Ok(value) => value,
        Err(error) => panic!("JSON value must serialize: {error}"),
    }
}

#[test]
fn current_manifests_load_with_exact_literal_digests() {
    let cases = [
        (
            AFFAIRS,
            "ustc.affairs-navigator",
            ImplementationStatus::Planned,
            "sha256:49900ac159dc1a8b381554005ed0cddffa76ebc323d81d6bdbf204f72fd1a018",
            "sha256:4db8de62654354ec7ae17c4cae255b371a6b4681cfc5786cc0622b1607e31e05",
            "sha256:940df79a35cd98c66810e6c73ed046927f277fe814ce81dc1cbc313cf437d242",
        ),
        (
            CHANGE_RADAR,
            "ustc.change-radar",
            ImplementationStatus::Planned,
            "sha256:4d97dce7f74c992d10b9711566eb431f2602df8a389eaa7d861b1edb6f80c49b",
            "sha256:7fc764a879638e239f51959b73d3c6d18ba99529b556b634bd2cf69cafb8874a",
            "sha256:2678a32807b965b811ade09e3d3b8ab7815ab1b58f37105b357bc62f50e42bc2",
        ),
        (
            OPPORTUNITY,
            "ustc.opportunity-graph",
            ImplementationStatus::Development,
            "sha256:90cfaefae8e3e04059b0aaf416bb410e5ddc8ab91f5d7e1083e0c957c8c341a3",
            "sha256:2a51dfa8222213eef541e2adfcc79ffcb893c5fcabf92c946bffa71ea10061b6",
            "sha256:77295dfc45113992601b436b4613cc34fbc5f88f74f11aecebc8b6587852e102",
        ),
    ];

    for (source, id, status, package_digest, capability_digest, source_policy_digest) in cases {
        let manifest = load(source);

        assert_eq!(manifest.package_id().as_str(), id);
        assert_eq!(manifest.package_version().as_str(), "0.1.0");
        assert_eq!(manifest.publisher(), "first-party");
        assert_eq!(manifest.implementation_status(), status);
        assert!(manifest.components().is_empty());
        assert_eq!(manifest.package_digest().as_str(), package_digest);
        assert_eq!(
            manifest.component_declaration_set_digest().as_str(),
            "sha256:ce03a5bf0a3675afa2477fadebce78fb07009f54d440991b6bfcf010ecb1c388"
        );
        assert_eq!(
            manifest.capability_manifest_digest().as_str(),
            capability_digest
        );
        assert_eq!(
            manifest.source_policy_digest().as_str(),
            source_policy_digest
        );
    }
}

#[test]
fn semantic_collection_and_json_order_do_not_change_manifest_identity() {
    let first = br#"{
      "id":"community.example",
      "version":"1.2.3",
      "publisher":"community-team",
      "tier":"VerifiedCommunityText",
      "displayName":"Example",
      "description":"Example metadata",
      "implementationStatus":"development",
      "installPolicy":{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true},
      "components":[
        {"type":"McpServerComponent","path":"plugins/z.json","mode":"remote"},
        {"type":"SkillComponent","path":"plugins/a.md"}
      ],
      "capabilities":["campus.public_rules.read","campus.public_changes.read"],
      "sourcePolicy":{"zPolicy":"z","aPolicy":"a"}
    }"#;
    let second = br#"{
      "sourcePolicy":{"aPolicy":"a","zPolicy":"z"},
      "capabilities":["campus.public_changes.read","campus.public_rules.read"],
      "components":[
        {"path":"plugins/a.md","type":"SkillComponent"},
        {"mode":"remote","path":"plugins/z.json","type":"McpServerComponent"}
      ],
      "installPolicy":{"userDisableAllowed":true,"defaultEnabled":false,"defaultInstalled":false,"class":"UserInstalledPlugin"},
      "implementationStatus":"development",
      "description":"Example metadata",
      "displayName":"Example",
      "tier":"VerifiedCommunityText",
      "publisher":"community-team",
      "version":"1.2.3",
      "id":"community.example"
    }"#;

    assert_eq!(load(first), load(second));
}

#[test]
fn malformed_unknown_and_duplicate_json_members_fail_closed() {
    assert_eq!(
        load_package_manifest(b"{"),
        Err(PackageLoadError::JsonRejected)
    );

    let affairs = String::from_utf8_lossy(AFFAIRS);
    let duplicate_top = affairs.replacen(
        "\"id\": \"ustc.affairs-navigator\",",
        "\"id\": \"ustc.affairs-navigator\",\n  \"id\": \"shadow.example\",",
        1,
    );
    assert_eq!(
        load_package_manifest(duplicate_top.as_bytes()),
        Err(PackageLoadError::JsonRejected)
    );

    let duplicate_nested = affairs.replacen(
        "\"class\": \"FirstPartySystemPlugin\",",
        "\"class\": \"FirstPartySystemPlugin\",\n    \"class\": \"UserInstalledPlugin\",",
        1,
    );
    assert_eq!(
        load_package_manifest(duplicate_nested.as_bytes()),
        Err(PackageLoadError::JsonRejected)
    );

    let duplicate_source_policy = affairs.replacen(
        "\"officialSources\": \"reviewed Source Registry entries and approved revisions only\",",
        "\"officialSources\": \"first\",\n    \"officialSources\": \"second\",",
        1,
    );
    assert_eq!(
        load_package_manifest(duplicate_source_policy.as_bytes()),
        Err(PackageLoadError::JsonRejected)
    );

    let unknown = affairs.replacen('{', "{\n  \"frameworkSession\": \"opaque\",", 1);
    assert_eq!(
        load_package_manifest(unknown.as_bytes()),
        Err(PackageLoadError::JsonRejected)
    );

    let mut explicit_null_description = community_manifest_value();
    explicit_null_description["description"] = serde_json::Value::Null;
    assert_eq!(
        load_package_manifest(&json_bytes(&explicit_null_description)),
        Err(PackageLoadError::JsonRejected)
    );

    let mut null_mode = community_manifest_value();
    null_mode["components"] = serde_json::json!([{
        "type": "SkillComponent",
        "path": "components/a.md",
        "mode": null
    }]);
    assert_eq!(
        load_package_manifest(&json_bytes(&null_mode)),
        Err(PackageLoadError::JsonRejected)
    );
}

#[test]
fn source_and_semantic_bounds_fail_with_stable_categories() {
    let oversized = vec![b' '; 1_048_577];
    assert_eq!(
        load_package_manifest(&oversized),
        Err(PackageLoadError::SourceTooLarge)
    );

    let affairs = String::from_utf8_lossy(AFFAIRS);
    let invalid_id = affairs.replacen("ustc.affairs-navigator", "USTC.affairs", 2);
    invalid(
        invalid_id.as_bytes(),
        PackageField::PackageId,
        PackageValidationErrorKind::InvalidFormat,
    );

    let noncanonical_version = affairs.replacen("\"0.1.0\"", "\"01.1.0\"", 1);
    invalid(
        noncanonical_version.as_bytes(),
        PackageField::PackageVersion,
        PackageValidationErrorKind::InvalidFormat,
    );

    let duplicate_capability = affairs.replacen(
        "\"campus.public_rules.read\"",
        "\"campus.public_rules.read\", \"campus.public_rules.read\"",
        1,
    );
    invalid(
        duplicate_capability.as_bytes(),
        PackageField::Capabilities,
        PackageValidationErrorKind::Duplicate,
    );

    let planned_component = affairs.replacen(
        "\"components\": []",
        "\"components\": [{\"type\":\"SkillComponent\",\"path\":\"../escape.md\"}]",
        1,
    );
    invalid(
        planned_component.as_bytes(),
        PackageField::ComponentPath,
        PackageValidationErrorKind::InvalidFormat,
    );

    let implemented_without_component = affairs.replacen(
        "\"implementationStatus\": \"planned\"",
        "\"implementationStatus\": \"implemented\"",
        1,
    );
    invalid(
        implemented_without_component.as_bytes(),
        PackageField::ImplementationStatus,
        PackageValidationErrorKind::Inconsistent,
    );
}

#[test]
fn semantic_cardinality_and_text_bounds_are_enforced() {
    let mut value = community_manifest_value();
    value["publisher"] = serde_json::json!(".invalid");
    invalid(
        &json_bytes(&value),
        PackageField::Publisher,
        PackageValidationErrorKind::InvalidFormat,
    );

    let mut value = community_manifest_value();
    value["displayName"] = serde_json::json!("");
    invalid(
        &json_bytes(&value),
        PackageField::DisplayName,
        PackageValidationErrorKind::Empty,
    );

    let mut value = community_manifest_value();
    value["description"] = serde_json::json!("control\u{0}character");
    invalid(
        &json_bytes(&value),
        PackageField::Description,
        PackageValidationErrorKind::InvalidFormat,
    );

    let mut value = community_manifest_value();
    value["components"] = serde_json::Value::Array(
        (0..65)
            .map(|index| {
                serde_json::json!({
                    "type": "SkillComponent",
                    "path": format!("components/{index}.md")
                })
            })
            .collect(),
    );
    invalid(
        &json_bytes(&value),
        PackageField::Components,
        PackageValidationErrorKind::TooMany,
    );

    let mut value = community_manifest_value();
    let duplicate = serde_json::json!({"type":"SkillComponent","path":"components/a.md"});
    value["components"] = serde_json::json!([duplicate.clone(), duplicate]);
    invalid(
        &json_bytes(&value),
        PackageField::ComponentPath,
        PackageValidationErrorKind::Duplicate,
    );

    let mut value = community_manifest_value();
    value["components"] = serde_json::json!([{
        "type":"SkillComponent",
        "path":"components/a.md",
        "mode":"m".repeat(65)
    }]);
    invalid(
        &json_bytes(&value),
        PackageField::ComponentMode,
        PackageValidationErrorKind::TooLong,
    );

    let mut value = community_manifest_value();
    value["components"] = serde_json::json!([{
        "type":"SkillComponent",
        "path":"a".repeat(513)
    }]);
    invalid(
        &json_bytes(&value),
        PackageField::ComponentPath,
        PackageValidationErrorKind::TooLong,
    );

    let mut value = community_manifest_value();
    value["capabilities"] = serde_json::Value::Array(
        (0..65)
            .map(|index| serde_json::json!(format!("capability.{index}")))
            .collect(),
    );
    invalid(
        &json_bytes(&value),
        PackageField::Capabilities,
        PackageValidationErrorKind::TooMany,
    );

    let mut value = community_manifest_value();
    value["sourcePolicy"] = serde_json::json!({});
    invalid(
        &json_bytes(&value),
        PackageField::SourcePolicy,
        PackageValidationErrorKind::Empty,
    );

    let mut value = community_manifest_value();
    let entries = (0..33)
        .map(|index| (format!("policy{index}"), serde_json::json!("bounded")))
        .collect();
    value["sourcePolicy"] = serde_json::Value::Object(entries);
    invalid(
        &json_bytes(&value),
        PackageField::SourcePolicy,
        PackageValidationErrorKind::TooMany,
    );

    let mut value = community_manifest_value();
    value["sourcePolicy"] = serde_json::json!({"1invalid":"bounded"});
    invalid(
        &json_bytes(&value),
        PackageField::SourcePolicyKey,
        PackageValidationErrorKind::InvalidFormat,
    );

    let mut value = community_manifest_value();
    value["sourcePolicy"] = serde_json::json!({"policy":"v".repeat(4097)});
    invalid(
        &json_bytes(&value),
        PackageField::SourcePolicyValue,
        PackageValidationErrorKind::TooLong,
    );

    let mut value = community_manifest_value();
    value["implementationStatus"] = serde_json::json!("planned");
    value["components"] = serde_json::json!([{
        "type":"SkillComponent","path":"components/a.md"
    }]);
    invalid(
        &json_bytes(&value),
        PackageField::ImplementationStatus,
        PackageValidationErrorKind::Inconsistent,
    );
}

#[test]
fn install_policy_and_component_declaration_coherence_fail_closed() {
    let first_party = String::from_utf8_lossy(AFFAIRS);
    let false_default = first_party.replacen(
        "\"defaultInstalled\": true",
        "\"defaultInstalled\": false",
        1,
    );
    invalid(
        false_default.as_bytes(),
        PackageField::InstallPolicy,
        PackageValidationErrorKind::Inconsistent,
    );

    let community = br#"{
      "id":"community.example","version":"1.0.0","publisher":"community-team",
      "tier":"VerifiedCommunityText","displayName":"Example","implementationStatus":"development",
      "installPolicy":{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":true,"userDisableAllowed":true},
      "components":[],"capabilities":[],"sourcePolicy":{"policy":"bounded"}
    }"#;
    invalid(
        community,
        PackageField::InstallPolicy,
        PackageValidationErrorKind::Inconsistent,
    );
}

#[test]
fn errors_do_not_echo_rejected_source_fragments() {
    let sentinel = "DO_NOT_ECHO_PRIVATE_FRAGMENT_9381";
    let source = format!(
        r#"{{
          "id":"community.example","version":"1.0.0","publisher":"{sentinel} !",
          "tier":"VerifiedCommunityText","displayName":"Example","implementationStatus":"development",
          "installPolicy":{{"class":"UserInstalledPlugin","defaultInstalled":false,"defaultEnabled":false,"userDisableAllowed":true}},
          "components":[],"capabilities":[],"sourcePolicy":{{"policy":"bounded"}}
        }}"#
    );
    let error = match load_package_manifest(source.as_bytes()) {
        Err(error) => error,
        Ok(_) => panic!("invalid publisher must fail"),
    };
    assert!(!format!("{error}").contains(sentinel));
    assert!(!format!("{error:?}").contains(sentinel));
}

#[test]
fn catalog_read_model_is_exact_ordered_and_duplicate_safe() {
    let manifests = vec![load(OPPORTUNITY), load(AFFAIRS), load(CHANGE_RADAR)];
    let revision = parsed_catalog_revision("catalog:reviewed-v1");
    let model = match CatalogReadModel::new(revision.clone(), manifests) {
        Ok(model) => model,
        Err(error) => panic!("valid catalog read model: {error}"),
    };
    let ids: Vec<&str> = model
        .packages()
        .iter()
        .map(|package| package.package_id().as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "ustc.affairs-navigator",
            "ustc.change-radar",
            "ustc.opportunity-graph"
        ]
    );
    assert_eq!(model.catalog_revision(), &revision);
    assert_eq!(
        model.catalog_digest().as_str(),
        "sha256:a5e660136cdc467a0e75a0cba495706460570e8a4f30b19c5e063c8bec8b24da"
    );

    let id = parsed_package_id("ustc.change-radar");
    let version = parsed_version("0.1.0");
    let found = model.find(&id, &version);
    assert_eq!(
        found.map(|package| package.display_name()),
        Some("USTC ChangeRadar")
    );
    assert!(
        model
            .find(&parsed_package_id("missing.example"), &version)
            .is_none()
    );

    let reversed = vec![load(CHANGE_RADAR), load(AFFAIRS), load(OPPORTUNITY)];
    let second = match CatalogReadModel::new(revision, reversed) {
        Ok(model) => model,
        Err(error) => panic!("permuted catalog read model: {error}"),
    };
    assert_eq!(model, second);

    let duplicate = vec![load(AFFAIRS), load(AFFAIRS)];
    assert_eq!(
        CatalogReadModel::new(parsed_catalog_revision("catalog:duplicate"), duplicate),
        Err(CatalogReadModelError::DuplicatePackageRevision)
    );
}

#[test]
fn catalog_metadata_does_not_claim_runtime_readiness() {
    for manifest in [load(AFFAIRS), load(CHANGE_RADAR), load(OPPORTUNITY)] {
        assert!(manifest.components().is_empty());
        assert!(matches!(
            manifest.implementation_status(),
            ImplementationStatus::Planned | ImplementationStatus::Development
        ));
        assert!(manifest.install_policy().default_installed());
        assert!(manifest.install_policy().default_enabled());
        assert!(manifest.install_policy().user_disable_allowed());
    }
}
