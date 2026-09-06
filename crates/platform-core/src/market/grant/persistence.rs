//! Versioned bounded snapshot of the existing domain command ledger.
//! Storage bytes are private authority data, never a client import surface.
use super::*;
use serde::{Deserialize, Serialize};

const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_COMMANDS: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotCodecError {
    TooLarge,
    InvalidJson,
    UnsupportedVersion,
    InvalidValue,
    CorruptLedger,
    UnsupportedAction,
}
impl fmt::Display for SnapshotCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("lifecycle snapshot rejected")
    }
}
impl Error for SnapshotCodecError {}
fn checked<T, E>(value: Result<T, E>) -> Result<T, SnapshotCodecError> {
    value.map_err(|_| SnapshotCodecError::InvalidValue)
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RawPre {
    Absent {},
    Present { revision: String },
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RawOutcome {
    Accepted {
        event_digest: String,
        revision: String,
    },
    Rejected {
        category: String,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSnapshot {
    version: String,
    record_count: usize,
    history_digest: String,
    records: Vec<RawRecord>,
}

use crate::market::capability::load_capability_registry;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecord {
    command_id: String,
    snapshot_id: String,
    action: RawAction,
    pre: RawPre,
    outcome: RawOutcome,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RawAction {
    Issue {
        evidence: RawEvidence,
    },
    Replace {
        version: String,
        evidence: RawEvidence,
    },
    MarkStale {
        version: String,
        reason: u8,
    },
    Expire {
        version: String,
    },
    Revoke {
        version: String,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RawScope {
    CampusPublic {},
    TenantPrivateUser { tenant: String, user: String },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvidence {
    snapshot_id: String,
    approval_id: String,
    tenant_id: String,
    user_id: String,
    installation_id: String,
    expected_installation_revision: String,
    catalog_revision: String,
    package_id: String,
    package_version: String,
    package_digest: String,
    capability_id: String,
    scope: RawScope,
    confirmation_policy: u8,
    capability_manifest_digest: String,
    capability_registry_revision: String,
    capability_definition: RawDefinition,
    capability_definition_digest: String,
    evidence_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawDefinition {
    id: String,
    effect_class: String,
    data_class: String,
    scope_kind: String,
    auto_grant: String,
    confirmation_default: String,
    status: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegistryCarrier {
    schema_version: String,
    registry_revision: String,
    capabilities: Vec<RawDefinition>,
}
fn raw_evidence(e: &GrantAdmissionEvidence) -> Result<RawEvidence, SnapshotCodecError> {
    let scope = match e.scope.scope_kind() {
        ScopeKind::CampusPublic => RawScope::CampusPublic {},
        ScopeKind::TenantPrivateUser => RawScope::TenantPrivateUser {
            tenant: e
                .scope
                .tenant_id()
                .ok_or(SnapshotCodecError::InvalidValue)?
                .as_str()
                .to_owned(),
            user: e
                .scope
                .user_id()
                .ok_or(SnapshotCodecError::InvalidValue)?
                .as_str()
                .to_owned(),
        },
        ScopeKind::OperatorAdministrative => return Err(SnapshotCodecError::InvalidValue),
    };
    let d = &e.capability_definition;
    Ok(RawEvidence {
        snapshot_id: e.snapshot_id.as_str().to_owned(),
        approval_id: e.approval_id.as_str().to_owned(),
        tenant_id: e.tenant_id.as_str().to_owned(),
        user_id: e.user_id.as_str().to_owned(),
        installation_id: e.installation_id.as_str().to_owned(),
        expected_installation_revision: e.expected_installation_revision.as_str().to_owned(),
        catalog_revision: e.catalog_revision.as_str().to_owned(),
        package_id: e.package_id.as_str().to_owned(),
        package_version: e.package_version.as_str().to_owned(),
        package_digest: e.package_digest.as_str().to_owned(),
        capability_id: e.capability_id.as_str().to_owned(),
        capability_manifest_digest: e.capability_manifest_digest.as_str().to_owned(),
        capability_registry_revision: e.capability_registry_revision.as_str().to_owned(),
        capability_definition_digest: e.capability_definition_digest.as_str().to_owned(),
        evidence_digest: e.evidence_digest.as_str().to_owned(),
        scope,
        confirmation_policy: match e.confirmation_policy {
            ConfirmationPolicy::Allow => 0,
            ConfirmationPolicy::Ask => 1,
        },
        capability_definition: RawDefinition {
            id: d.id().as_str().to_owned(),
            effect_class: format!("{:?}", d.effect_class()),
            data_class: format!("{:?}", d.data_class()),
            scope_kind: format!("{:?}", d.scope_kind()),
            auto_grant: format!("{:?}", d.auto_grant()),
            confirmation_default: format!("{:?}", d.confirmation_default()),
            status: format!("{:?}", d.status()),
        },
    })
}
fn evidence(raw: RawEvidence) -> Result<GrantAdmissionEvidence, SnapshotCodecError> {
    let registry = RegistryCarrier {
        schema_version: "capability-registry/v1".to_owned(),
        registry_revision: raw.capability_registry_revision.clone(),
        capabilities: vec![raw.capability_definition],
    };
    let registry = checked(load_capability_registry(&checked(serde_json::to_vec(
        &registry,
    ))?))?;
    let capability = checked(CapabilityId::parse(raw.capability_id.clone()))?;
    let definition = registry
        .find(&capability)
        .ok_or(SnapshotCodecError::InvalidValue)?
        .clone();
    let scope = match raw.scope {
        RawScope::CampusPublic {} => checked(GrantScope::campus_public())?,
        RawScope::TenantPrivateUser { tenant, user } => checked(GrantScope::tenant_private_user(
            checked(TenantId::parse(tenant))?,
            checked(UserId::parse(user))?,
        ))?,
    };
    let value = GrantAdmissionEvidence {
        snapshot_id: checked(GrantSnapshotId::parse(raw.snapshot_id))?,
        approval_id: checked(GrantApprovalId::parse(raw.approval_id))?,
        tenant_id: checked(TenantId::parse(raw.tenant_id))?,
        user_id: checked(UserId::parse(raw.user_id))?,
        installation_id: checked(InstallationId::parse(raw.installation_id))?,
        expected_installation_revision: checked(InstallationRevision::parse(
            raw.expected_installation_revision,
        ))?,
        catalog_revision: checked(CatalogRevision::parse(raw.catalog_revision))?,
        package_id: checked(PackageId::parse(raw.package_id))?,
        package_version: checked(PackageVersion::parse(&raw.package_version))?,
        package_digest: checked(Sha256Digest::parse(raw.package_digest))?,
        capability_id: checked(CapabilityId::parse(raw.capability_id))?,
        capability_manifest_digest: checked(Sha256Digest::parse(raw.capability_manifest_digest))?,
        capability_registry_revision: checked(CapabilityRegistryRevision::parse(
            raw.capability_registry_revision,
        ))?,
        capability_definition_digest: checked(Sha256Digest::parse(
            raw.capability_definition_digest,
        ))?,
        evidence_digest: checked(Sha256Digest::parse(raw.evidence_digest))?,
        scope,
        confirmation_policy: match raw.confirmation_policy {
            0 => ConfirmationPolicy::Allow,
            1 => ConfirmationPolicy::Ask,
            _ => return Err(SnapshotCodecError::InvalidValue),
        },
        capability_definition: definition,
    };
    checked(verify_evidence(&value))?;
    Ok(value)
}
fn raw_action(command: &GrantCommand) -> Result<RawAction, SnapshotCodecError> {
    Ok(match &command.action {
        GrantCommandAction::Issue(e) => RawAction::Issue {
            evidence: raw_evidence(e)?,
        },
        GrantCommandAction::Replace {
            expected_version,
            evidence,
        } => RawAction::Replace {
            version: expected_version.as_str().to_owned(),
            evidence: raw_evidence(evidence)?,
        },
        GrantCommandAction::MarkStale {
            expected_version,
            reason,
        } => RawAction::MarkStale {
            version: expected_version.as_str().to_owned(),
            reason: match reason {
                GrantInvalidationReason::CapabilityManifestChanged => 0,
                GrantInvalidationReason::CapabilityDefinitionChanged => 1,
                GrantInvalidationReason::InstallationChanged => 2,
                GrantInvalidationReason::PolicyChanged => 3,
            },
        },
        GrantCommandAction::Expire { expected_version } => RawAction::Expire {
            version: expected_version.as_str().to_owned(),
        },
        GrantCommandAction::Revoke { expected_version } => RawAction::Revoke {
            version: expected_version.as_str().to_owned(),
        },
    })
}
fn command(
    id: String,
    snapshot: String,
    action: RawAction,
) -> Result<GrantCommand, SnapshotCodecError> {
    let id = checked(GrantCommandId::parse(id))?;
    let snapshot = checked(GrantSnapshotId::parse(snapshot))?;
    let result = checked(match action {
        RawAction::Issue { evidence: e } => GrantCommand::issue(id, evidence(e)?),
        RawAction::Replace {
            version,
            evidence: e,
        } => GrantCommand::replace(id, checked(GrantVersion::parse(version))?, evidence(e)?),
        RawAction::MarkStale { version, reason } => GrantCommand::mark_stale(
            id,
            snapshot.clone(),
            checked(GrantVersion::parse(version))?,
            match reason {
                0 => GrantInvalidationReason::CapabilityManifestChanged,
                1 => GrantInvalidationReason::CapabilityDefinitionChanged,
                2 => GrantInvalidationReason::InstallationChanged,
                3 => GrantInvalidationReason::PolicyChanged,
                _ => return Err(SnapshotCodecError::InvalidValue),
            },
        ),
        RawAction::Expire { version } => {
            GrantCommand::expire(id, snapshot.clone(), checked(GrantVersion::parse(version))?)
        }
        RawAction::Revoke { version } => {
            GrantCommand::revoke(id, snapshot.clone(), checked(GrantVersion::parse(version))?)
        }
    })?;
    if result.snapshot_id() != &snapshot {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    Ok(result)
}
fn raw_pre(pre: Option<&GrantSnapshot>) -> RawPre {
    pre.map_or(RawPre::Absent {}, |s| RawPre::Present {
        revision: s.version().as_str().to_owned(),
    })
}
fn raw_outcome(receipt: &GrantCommandReceipt) -> RawOutcome {
    match receipt.outcome() {
        GrantCommandOutcome::Accepted { event, snapshot } => RawOutcome::Accepted {
            event_digest: event.canonical_coupling_digest().as_str().to_owned(),
            revision: snapshot.version().as_str().to_owned(),
        },
        GrantCommandOutcome::Rejected { error } => RawOutcome::Rejected {
            category: format!("{error:?}"),
        },
    }
}

fn history_digest(repository: &InMemoryGrantRepository) -> String {
    let mut bytes = b"market-grant-snapshot-history/v1\0".to_vec();
    encode_u64(repository.events.len() as u64, &mut bytes);
    for (id, events) in &repository.events {
        encode_string(id.as_str(), &mut bytes);
        encode_u64(events.len() as u64, &mut bytes);
        for event in events {
            encode_string(event.canonical_coupling_digest().as_str(), &mut bytes);
        }
    }
    Sha256Digest::from_bytes(&bytes).as_str().to_owned()
}

/// Encode the original ordered grant ledger, including accepted and rejected commands.
pub fn encode_snapshot(
    repository: &InMemoryGrantRepository,
) -> Result<Vec<u8>, SnapshotCodecError> {
    if repository.command_ledger.len() > MAX_COMMANDS {
        return Err(SnapshotCodecError::TooLarge);
    }
    let mut entries: Vec<_> = repository.command_ledger.values().collect();
    entries.sort_by_key(|entry| entry.commit_ordinal);
    let mut records = Vec::with_capacity(entries.len());
    let mut total = 128usize;
    for (ordinal, entry) in entries.into_iter().enumerate() {
        if entry.commit_ordinal != ordinal {
            return Err(SnapshotCodecError::CorruptLedger);
        }
        let record = RawRecord {
            command_id: entry.command.command_id().as_str().to_owned(),
            snapshot_id: entry.command.snapshot_id().as_str().to_owned(),
            action: raw_action(&entry.command)?,
            pre: raw_pre(entry.observed_pre_snapshot.as_ref()),
            outcome: raw_outcome(&entry.receipt),
        };
        total = total
            .checked_add(checked(serde_json::to_vec(&record))?.len() + 1)
            .ok_or(SnapshotCodecError::TooLarge)?;
        if total > MAX_BYTES {
            return Err(SnapshotCodecError::TooLarge);
        }
        records.push(record);
    }
    let bytes = checked(serde_json::to_vec(&RawSnapshot {
        version: "market-grant-ledger/v1".to_owned(),
        record_count: records.len(),
        history_digest: history_digest(repository),
        records,
    }))?;
    if bytes.len() > MAX_BYTES {
        return Err(SnapshotCodecError::TooLarge);
    }
    Ok(bytes)
}
/// Recover private storage into a fresh repository; never use this as a grant import API.
pub fn decode_snapshot(bytes: &[u8]) -> Result<InMemoryGrantRepository, SnapshotCodecError> {
    if bytes.len() > MAX_BYTES {
        return Err(SnapshotCodecError::TooLarge);
    }
    let raw: RawSnapshot =
        serde_json::from_slice(bytes).map_err(|_| SnapshotCodecError::InvalidJson)?;
    if raw.version != "market-grant-ledger/v1" {
        return Err(SnapshotCodecError::UnsupportedVersion);
    }
    if raw.record_count != raw.records.len() {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    if raw.records.len() > MAX_COMMANDS {
        return Err(SnapshotCodecError::TooLarge);
    }
    let mut repository = InMemoryGrantRepository::new();
    let mut receipts = Vec::with_capacity(raw.records.len());
    for record in raw.records {
        let command = command(record.command_id, record.snapshot_id, record.action)?;
        if repository.command_ledger.contains_key(command.command_id()) {
            return Err(SnapshotCodecError::CorruptLedger);
        }
        let pre = repository.aggregates.get(command.snapshot_id()).cloned();
        if raw_pre(pre.as_ref()) != record.pre {
            return Err(SnapshotCodecError::CorruptLedger);
        }
        let receipt = repository
            .execute(command)
            .map_err(|_| SnapshotCodecError::CorruptLedger)?;
        if raw_outcome(&receipt) != record.outcome {
            return Err(SnapshotCodecError::CorruptLedger);
        }
        receipts.push((receipt, pre));
    }
    if history_digest(&repository) != raw.history_digest {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    InMemoryGrantRepository::try_from_histories_and_receipts(
        repository.events.into_iter().collect(),
        receipts,
    )
    .map_err(|_| SnapshotCodecError::CorruptLedger)
}

/// A replayed update frame may be followed only by ordinary owner commands.
/// Coupled package/grant update commands must be supplied by the update journal itself.
pub(in crate::market) fn ordinary_extension(
    before: &[u8],
    after: &[u8],
) -> Result<(), SnapshotCodecError> {
    let before: serde_json::Value =
        serde_json::from_slice(before).map_err(|_| SnapshotCodecError::InvalidJson)?;
    let after: serde_json::Value =
        serde_json::from_slice(after).map_err(|_| SnapshotCodecError::InvalidJson)?;
    let before = before["records"]
        .as_array()
        .ok_or(SnapshotCodecError::CorruptLedger)?;
    let after = after["records"]
        .as_array()
        .ok_or(SnapshotCodecError::CorruptLedger)?;
    if !after.starts_with(before) {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    for record in &after[before.len()..] {
        if matches!(
            record["action"]["kind"].as_str(),
            Some("PackageUpdated" | "PackageRolledBack")
        ) || record["command_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("grant-cmd:update-"))
        {
            return Err(SnapshotCodecError::CorruptLedger);
        }
    }
    Ok(())
}
