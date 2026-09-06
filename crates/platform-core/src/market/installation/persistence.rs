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

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecord {
    command_id: String,
    installation_id: String,
    action: RawAction,
    pre: RawPre,
    outcome: RawOutcome,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RawAction {
    Install {
        tenant: String,
        user: String,
        pin: RawPin,
        configuration: RawConfiguration,
    },
    Configure {
        revision: String,
        configuration: RawConfiguration,
    },
    Enable {
        revision: String,
        evidence: RawEnable,
    },
    Disable {
        revision: String,
    },
    Revoke {
        revision: String,
    },
    Uninstall {
        revision: String,
    },
    PackageUpdated {
        revision: String,
        plan_digest: String,
        pin: RawPin,
    },
    PackageRolledBack {
        revision: String,
        plan_digest: String,
        pin: RawPin,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPin {
    catalog: String,
    package: String,
    version: String,
    digest: String,
    components: Vec<RawComponent>,
    component_set: String,
    capabilities: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawComponent {
    id: String,
    kind: u8,
    version: String,
    digest: String,
    execution: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfiguration {
    tenant: String,
    entries: Vec<RawEntry>,
    digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    key: String,
    value: RawValue,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum RawValue {
    Text { value: String },
    Integer { value: i64 },
    Boolean { value: bool },
    Secret { tenant: String, reference: String },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnable {
    installation: String,
    revision: String,
    package: String,
    components: String,
    configuration: String,
    capabilities: String,
    grants: String,
    policy: String,
    digest: String,
}

fn raw_configuration(value: &InstallationConfiguration) -> RawConfiguration {
    RawConfiguration {
        tenant: value.tenant_id.as_str().to_owned(),
        digest: value.digest.as_str().to_owned(),
        entries: value
            .entries
            .iter()
            .map(|(key, value)| RawEntry {
                key: key.as_str().to_owned(),
                value: match value {
                    ConfigurationValue::Text(v) => RawValue::Text {
                        value: v.as_str().to_owned(),
                    },
                    ConfigurationValue::Integer(v) => RawValue::Integer { value: *v },
                    ConfigurationValue::Boolean(v) => RawValue::Boolean { value: *v },
                    ConfigurationValue::Secret(v) => RawValue::Secret {
                        tenant: v.tenant_id().as_str().to_owned(),
                        reference: v.id().as_str().to_owned(),
                    },
                },
            })
            .collect(),
    }
}
fn configuration(raw: RawConfiguration) -> Result<InstallationConfiguration, SnapshotCodecError> {
    if raw.entries.len() > MAX_CONFIGURATION_ENTRIES {
        return Err(SnapshotCodecError::TooLarge);
    }
    let tenant = checked(TenantId::parse(raw.tenant))?;
    let mut entries = Vec::with_capacity(raw.entries.len());
    for entry in raw.entries {
        let value = match entry.value {
            RawValue::Text { value } => {
                ConfigurationValue::Text(checked(NonSecretText::parse(value))?)
            }
            RawValue::Integer { value } => ConfigurationValue::Integer(value),
            RawValue::Boolean { value } => ConfigurationValue::Boolean(value),
            RawValue::Secret { tenant, reference } => {
                ConfigurationValue::Secret(checked(SecretRef::new(
                    checked(TenantId::parse(tenant))?,
                    checked(SecretRefId::parse(reference))?,
                ))?)
            }
        };
        entries.push((checked(ConfigurationKey::parse(entry.key))?, value));
    }
    let result = checked(InstallationConfiguration::new(&tenant, entries))?;
    if result.digest().as_str() != raw.digest {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    Ok(result)
}
fn raw_pin(pin: &InstallationPackagePin) -> Result<RawPin, SnapshotCodecError> {
    if pin.components().len() > 64 {
        return Err(SnapshotCodecError::TooLarge);
    }
    Ok(RawPin {
        catalog: pin.catalog_revision().as_str().to_owned(),
        package: pin.package_id().as_str().to_owned(),
        version: pin.package_version().as_str(),
        digest: pin.package_digest().as_str().to_owned(),
        component_set: pin.component_set_digest().as_str().to_owned(),
        capabilities: pin.capability_manifest_digest().as_str().to_owned(),
        components: pin
            .components()
            .iter()
            .map(|c| RawComponent {
                id: c.component_id().as_str().to_owned(),
                kind: component_kind_tag(c.kind()),
                version: c.version().as_str().to_owned(),
                digest: c.digest().as_str().to_owned(),
                execution: c.execution_identity().as_str().to_owned(),
            })
            .collect(),
    })
}
fn pin(raw: RawPin) -> Result<InstallationPackagePin, SnapshotCodecError> {
    if raw.components.len() > 64 {
        return Err(SnapshotCodecError::TooLarge);
    }
    let mut components = Vec::with_capacity(raw.components.len());
    for c in raw.components {
        let kind = match c.kind {
            1 => ComponentKind::SkillComponent,
            2 => ComponentKind::DeclarativeResourcePack,
            3 => ComponentKind::McpServerComponent,
            4 => ComponentKind::NativeRustComponent,
            _ => return Err(SnapshotCodecError::InvalidValue),
        };
        components.push(checked(InstalledComponentPin::new(
            checked(ComponentId::parse(c.id))?,
            kind,
            checked(ComponentVersion::parse(c.version))?,
            checked(Sha256Digest::parse(c.digest))?,
            checked(ExecutionIdentity::parse(c.execution))?,
        ))?);
    }
    checked(InstallationPackagePin::new(
        checked(CatalogRevision::parse(raw.catalog))?,
        checked(PackageId::parse(raw.package))?,
        checked(PackageVersion::parse(&raw.version))?,
        checked(Sha256Digest::parse(raw.digest))?,
        components,
        checked(Sha256Digest::parse(raw.component_set))?,
        checked(Sha256Digest::parse(raw.capabilities))?,
    ))
}
fn raw_enable(e: &EnablePreconditionEvidence) -> RawEnable {
    RawEnable {
        installation: e.installation_id.as_str().to_owned(),
        revision: e.expected_installation_revision.as_str().to_owned(),
        package: e.package_digest.as_str().to_owned(),
        components: e.component_set_digest.as_str().to_owned(),
        configuration: e.configuration_digest.as_str().to_owned(),
        capabilities: e.capability_manifest_digest.as_str().to_owned(),
        grants: e.grant_set_snapshot_digest.as_str().to_owned(),
        policy: e.policy_admission_snapshot_digest.as_str().to_owned(),
        digest: e.evidence_digest.as_str().to_owned(),
    }
}
fn enable(e: RawEnable) -> Result<EnablePreconditionEvidence, SnapshotCodecError> {
    let value = checked(EnablePreconditionEvidence::from_authority_bindings(
        checked(InstallationId::parse(e.installation))?,
        checked(InstallationRevision::parse(e.revision))?,
        checked(Sha256Digest::parse(e.package))?,
        checked(Sha256Digest::parse(e.components))?,
        checked(Sha256Digest::parse(e.configuration))?,
        checked(Sha256Digest::parse(e.capabilities))?,
        checked(Sha256Digest::parse(e.grants))?,
        checked(Sha256Digest::parse(e.policy))?,
    ))?;
    if value.evidence_digest().as_str() != e.digest {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    Ok(value)
}
fn raw_action(command: &InstallationCommand) -> Result<RawAction, SnapshotCodecError> {
    Ok(match &command.action {
        InstallationCommandAction::Install {
            tenant_id,
            user_id,
            package_pin,
            configuration,
        } => RawAction::Install {
            tenant: tenant_id.as_str().to_owned(),
            user: user_id.as_str().to_owned(),
            pin: raw_pin(package_pin)?,
            configuration: raw_configuration(configuration),
        },
        InstallationCommandAction::Configure {
            expected_revision,
            configuration,
        } => RawAction::Configure {
            revision: expected_revision.as_str().to_owned(),
            configuration: raw_configuration(configuration),
        },
        InstallationCommandAction::Enable {
            expected_revision,
            evidence,
        } => RawAction::Enable {
            revision: expected_revision.as_str().to_owned(),
            evidence: raw_enable(evidence),
        },
        InstallationCommandAction::Disable { expected_revision } => RawAction::Disable {
            revision: expected_revision.as_str().to_owned(),
        },
        InstallationCommandAction::Revoke { expected_revision } => RawAction::Revoke {
            revision: expected_revision.as_str().to_owned(),
        },
        InstallationCommandAction::Uninstall { expected_revision } => RawAction::Uninstall {
            revision: expected_revision.as_str().to_owned(),
        },
        InstallationCommandAction::PackageUpdated {
            expected_revision,
            plan_digest,
            next_package_pin,
        } => RawAction::PackageUpdated {
            revision: expected_revision.as_str().to_owned(),
            plan_digest: plan_digest.as_str().to_owned(),
            pin: raw_pin(next_package_pin)?,
        },
        InstallationCommandAction::PackageRolledBack {
            expected_revision,
            plan_digest,
            rollback_package_pin,
        } => RawAction::PackageRolledBack {
            revision: expected_revision.as_str().to_owned(),
            plan_digest: plan_digest.as_str().to_owned(),
            pin: raw_pin(rollback_package_pin)?,
        },
    })
}
fn command(
    id: String,
    installation: String,
    action: RawAction,
) -> Result<InstallationCommand, SnapshotCodecError> {
    let id = checked(InstallationCommandId::parse(id))?;
    let installation = checked(InstallationId::parse(installation))?;
    checked(match action {
        RawAction::Install {
            tenant,
            user,
            pin: p,
            configuration: c,
        } => InstallationCommand::install(
            id,
            installation,
            checked(TenantId::parse(tenant))?,
            checked(UserId::parse(user))?,
            pin(p)?,
            configuration(c)?,
        ),
        RawAction::Configure {
            revision,
            configuration: c,
        } => InstallationCommand::configure(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
            configuration(c)?,
        ),
        RawAction::Enable { revision, evidence } => InstallationCommand::enable(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
            enable(evidence)?,
        ),
        RawAction::Disable { revision } => InstallationCommand::disable(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
        ),
        RawAction::Revoke { revision } => InstallationCommand::revoke(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
        ),
        RawAction::PackageUpdated {
            revision,
            plan_digest,
            pin: raw,
        } => InstallationCommand::package_updated(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
            checked(Sha256Digest::parse(plan_digest))?,
            pin(raw)?,
        ),
        RawAction::PackageRolledBack {
            revision,
            plan_digest,
            pin: raw,
        } => InstallationCommand::package_rolled_back(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
            checked(Sha256Digest::parse(plan_digest))?,
            pin(raw)?,
        ),
        RawAction::Uninstall { revision } => InstallationCommand::uninstall(
            id,
            installation,
            checked(InstallationRevision::parse(revision))?,
        ),
    })
}
fn raw_pre(pre: Option<&InstallationSnapshot>) -> RawPre {
    pre.map_or(RawPre::Absent {}, |s| RawPre::Present {
        revision: s.revision().as_str().to_owned(),
    })
}
fn raw_outcome(receipt: &InstallationCommandReceipt) -> RawOutcome {
    match receipt.outcome() {
        InstallationCommandOutcome::Accepted { event, snapshot } => RawOutcome::Accepted {
            event_digest: event.canonical_coupling_digest().as_str().to_owned(),
            revision: snapshot.revision().as_str().to_owned(),
        },
        InstallationCommandOutcome::Rejected { error } => RawOutcome::Rejected {
            category: format!("{error:?}"),
        },
    }
}

fn history_digest(repository: &InMemoryInstallationRepository) -> String {
    let mut bytes = b"market-installation-snapshot-history/v1\0".to_vec();
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

/// Encode the original ledger only, with historical preconditions and exact outcome checks.
pub fn encode_snapshot(
    repository: &InMemoryInstallationRepository,
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
            installation_id: entry.command.installation_id().as_str().to_owned(),
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
        version: "market-installation-ledger/v1".to_owned(),
        record_count: records.len(),
        history_digest: history_digest(repository),
        records,
    }))?;
    if bytes.len() > MAX_BYTES {
        return Err(SnapshotCodecError::TooLarge);
    }
    Ok(bytes)
}
/// Decode into a fresh isolated repository, then invoke the existing history/receipt validator.
/// This is for trusted storage recovery, never an installation or approval import endpoint.
pub fn decode_snapshot(bytes: &[u8]) -> Result<InMemoryInstallationRepository, SnapshotCodecError> {
    if bytes.len() > MAX_BYTES {
        return Err(SnapshotCodecError::TooLarge);
    }
    let raw: RawSnapshot =
        serde_json::from_slice(bytes).map_err(|_| SnapshotCodecError::InvalidJson)?;
    if raw.version != "market-installation-ledger/v1" {
        return Err(SnapshotCodecError::UnsupportedVersion);
    }
    if raw.record_count != raw.records.len() {
        return Err(SnapshotCodecError::CorruptLedger);
    }
    if raw.records.len() > MAX_COMMANDS {
        return Err(SnapshotCodecError::TooLarge);
    }
    let mut repository = InMemoryInstallationRepository::new();
    let mut receipts = Vec::with_capacity(raw.records.len());
    for record in raw.records {
        let command = command(record.command_id, record.installation_id, record.action)?;
        if repository.command_ledger.contains_key(command.command_id()) {
            return Err(SnapshotCodecError::CorruptLedger);
        }
        let pre = repository
            .aggregates
            .get(command.installation_id())
            .cloned();
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
    InMemoryInstallationRepository::try_from_histories_and_receipts(
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
