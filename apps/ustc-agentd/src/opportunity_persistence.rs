//! Composition-owned file adapter for tenant-private Opportunity profiles.
//!
//! Authority-bearing domain records are never deserialized directly. Active
//! records are rebuilt through `OpportunityProfileService`; tombstones are
//! rebuilt by recomputing the deterministic deletion receipt identity.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_opportunity_graph::{
    AcademicProfileInput, AuthenticatedPrincipal, ConsentField, ConsentGrant, ConsentId,
    ConsentPurpose, DeletionReceipt, InMemoryOpportunityProfileRepository,
    OpportunityProfileRepository, OpportunityProfileService, OpportunityRepositoryError,
    ProfileLookup, ProfileSnapshotId, ProfileTombstone, TenantProfileRecord,
};

const STATE_SCHEMA_VERSION: u8 = 1;
const MAX_PROFILE_STATE_BYTES: u64 = 1_048_576;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedState {
    schema_version: u8,
    active: Vec<PersistedProfile>,
    tombstones: Vec<PersistedTombstone>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            active: Vec::new(),
            tombstones: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedProfile {
    tenant_id: String,
    user_id: String,
    profile_snapshot_id: String,
    consent_id: String,
    consent_purpose: String,
    consent_fields: Vec<String>,
    consented_at_unix_nanos: String,
    completed_courses: Vec<String>,
    min_credits: u16,
    max_credits: u16,
    preference_weights: BTreeMap<String, i32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedTombstone {
    tenant_id: String,
    user_id: String,
    profile_snapshot_id: String,
    consent_id: String,
    deletion_receipt_id: String,
    deleted_at_unix_nanos: String,
}

pub(crate) struct DurableOpportunityProfileRepository {
    path: PathBuf,
    inner: InMemoryOpportunityProfileRepository,
    active: BTreeMap<ProfileSnapshotId, PersistedProfile>,
    tombstones: BTreeMap<ProfileSnapshotId, PersistedTombstone>,
    max_tombstones: usize,
}

fn read_existing_private_state(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let path_metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("profile state metadata: {error}")),
    };
    if !path_metadata.file_type().is_file() {
        return Err("profile state must be a regular non-symlink file".to_owned());
    }
    if path_metadata.permissions().mode() & 0o777 != 0o600 {
        return Err("profile state permissions must be 0600".to_owned());
    }
    if path_metadata.len() > MAX_PROFILE_STATE_BYTES {
        return Err("profile state exceeds byte limit".to_owned());
    }

    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| format!("profile state open: {error}"))?;
    let opened_metadata = file
        .metadata()
        .map_err(|error| format!("profile state opened metadata: {error}"))?;
    if !opened_metadata.file_type().is_file()
        || opened_metadata.dev() != path_metadata.dev()
        || opened_metadata.ino() != path_metadata.ino()
        || opened_metadata.permissions().mode() & 0o777 != 0o600
    {
        return Err("profile state changed or became unsafe during open".to_owned());
    }

    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(MAX_PROFILE_STATE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("profile state read: {error}"))?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > MAX_PROFILE_STATE_BYTES) {
        return Err("profile state exceeds byte limit".to_owned());
    }
    Ok(Some(bytes))
}

impl DurableOpportunityProfileRepository {
    pub(crate) fn open(
        path: &Path,
        max_profiles: usize,
        max_tombstones: usize,
    ) -> Result<Self, String> {
        let state = if let Some(bytes) = read_existing_private_state(path)? {
            serde_json::from_slice::<PersistedState>(&bytes)
                .map_err(|error| format!("profile state decode: {error}"))?
        } else {
            PersistedState::default()
        };
        if state.schema_version != STATE_SCHEMA_VERSION {
            return Err(format!(
                "profile state schema mismatch: expected {STATE_SCHEMA_VERSION}, got {}",
                state.schema_version
            ));
        }
        if state.active.len() > max_profiles || state.tombstones.len() > max_tombstones {
            return Err("profile state exceeds configured capacity".to_owned());
        }

        let mut inner = InMemoryOpportunityProfileRepository::new(max_profiles, max_tombstones)
            .map_err(|error| format!("profile repository capacity: {error}"))?;
        let mut active = BTreeMap::new();
        let mut active_principals = BTreeSet::new();
        for persisted in state.active {
            let principal = principal(&persisted.tenant_id, &persisted.user_id)?;
            let expected_profile_id = ProfileSnapshotId::parse(&persisted.profile_snapshot_id)
                .map_err(|error| format!("persisted profile id: {error}"))?;
            let expected_consent_id = ConsentId::parse(&persisted.consent_id)
                .map_err(|error| format!("persisted consent id: {error}"))?;
            let consented_at = timestamp_nanos(&persisted.consented_at_unix_nanos)?;
            let consent = ConsentGrant::new(
                consent_purpose(&persisted.consent_purpose)?,
                consent_fields(&persisted.consent_fields)?,
                consented_at,
            )
            .map_err(|error| format!("persisted consent: {error}"))?;
            let profile = AcademicProfileInput::new(
                persisted.completed_courses.clone(),
                persisted.min_credits,
                persisted.max_credits,
                persisted.preference_weights.clone(),
            )
            .map_err(|error| format!("persisted private profile: {error}"))?;
            let record = OpportunityProfileService::new(&mut inner)
                .create_profile(principal.clone(), consent, profile)
                .map_err(|error| format!("persisted profile replay: {error}"))?;
            if record.profile_snapshot_id() != &expected_profile_id
                || record.consent_id() != &expected_consent_id
            {
                return Err("persisted profile identity drift".to_owned());
            }
            let principal_key = (
                principal.tenant_id().as_str().to_owned(),
                principal.user_id().as_str().to_owned(),
            );
            if !active_principals.insert(principal_key)
                || active.insert(expected_profile_id, persisted).is_some()
            {
                return Err("persisted profile duplicate".to_owned());
            }
        }

        let mut tombstones = BTreeMap::new();
        for persisted in state.tombstones {
            let profile_id = ProfileSnapshotId::parse(&persisted.profile_snapshot_id)
                .map_err(|error| format!("persisted tombstone profile id: {error}"))?;
            if active.contains_key(&profile_id) || tombstones.contains_key(&profile_id) {
                return Err("persisted tombstone identity duplicate".to_owned());
            }
            rehydrate_tombstone(&persisted)?;
            tombstones.insert(profile_id, persisted);
        }

        Ok(Self {
            path: path.to_owned(),
            inner,
            active,
            tombstones,
            max_tombstones,
        })
    }

    #[must_use]
    pub(crate) fn private_payload_count(&self) -> usize {
        self.active.len()
    }

    #[must_use]
    pub(crate) fn tombstone_count(&self) -> usize {
        self.tombstones.len()
    }

    fn persist(
        &self,
        active: &BTreeMap<ProfileSnapshotId, PersistedProfile>,
        tombstones: &BTreeMap<ProfileSnapshotId, PersistedTombstone>,
    ) -> Result<(), OpportunityRepositoryError> {
        let state = PersistedState {
            schema_version: STATE_SCHEMA_VERSION,
            active: active.values().cloned().collect(),
            tombstones: tombstones.values().cloned().collect(),
        };
        let bytes =
            serde_json::to_vec(&state).map_err(|_| OpportunityRepositoryError::Unavailable)?;
        let length =
            u64::try_from(bytes.len()).map_err(|_| OpportunityRepositoryError::CapacityExceeded)?;
        if length > MAX_PROFILE_STATE_BYTES {
            return Err(OpportunityRepositoryError::CapacityExceeded);
        }
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|_| OpportunityRepositoryError::Unavailable)?;
        let temporary = parent.join(format!(
            ".{}.tmp-{}",
            self.path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("opportunity-profiles"),
            std::process::id()
        ));
        let write_result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            let parent_file = fs::File::open(parent)?;
            parent_file.sync_all()
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary);
            return Err(OpportunityRepositoryError::Unavailable);
        }
        Ok(())
    }
}

impl OpportunityProfileRepository for DurableOpportunityProfileRepository {
    fn create(&mut self, record: TenantProfileRecord) -> Result<(), OpportunityRepositoryError> {
        if self.tombstones.contains_key(record.profile_snapshot_id()) {
            return Err(OpportunityRepositoryError::ProfileIdentityConflict);
        }
        let persisted = profile_to_persisted(&record)?;
        let mut candidate_inner = self.inner.clone();
        candidate_inner.create(record.clone())?;
        let mut candidate_active = self.active.clone();
        match candidate_active.get(record.profile_snapshot_id()) {
            Some(existing) if persisted_profile_eq(existing, &persisted) => return Ok(()),
            Some(_) => return Err(OpportunityRepositoryError::ProfileIdentityConflict),
            None => {
                candidate_active.insert(record.profile_snapshot_id().clone(), persisted);
            }
        }
        self.persist(&candidate_active, &self.tombstones)?;
        self.inner = candidate_inner;
        self.active = candidate_active;
        Ok(())
    }

    fn lookup(
        &self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
    ) -> Result<ProfileLookup, OpportunityRepositoryError> {
        if let Some(persisted) = self.tombstones.get(profile_snapshot_id) {
            if persisted.tenant_id != principal.tenant_id().as_str()
                || persisted.user_id != principal.user_id().as_str()
            {
                return Ok(ProfileLookup::AccessDenied);
            }
            return rehydrate_tombstone(persisted)
                .map(ProfileLookup::Deleted)
                .map_err(|_| OpportunityRepositoryError::ProfileIdentityConflict);
        }
        self.inner.lookup(principal, profile_snapshot_id)
    }

    fn delete(
        &mut self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
        receipt: DeletionReceipt,
    ) -> Result<DeletionReceipt, OpportunityRepositoryError> {
        if let Some(persisted) = self.tombstones.get(profile_snapshot_id) {
            let tombstone = rehydrate_tombstone(persisted)
                .map_err(|_| OpportunityRepositoryError::DeleteIdentityConflict)?;
            if tombstone.tenant_id() != principal.tenant_id()
                || tombstone.user_id() != principal.user_id()
                || tombstone.deletion_receipt() != &receipt
            {
                return Err(OpportunityRepositoryError::DeleteIdentityConflict);
            }
            return Ok(receipt);
        }
        let mut candidate_inner = self.inner.clone();
        let committed = candidate_inner.delete(principal, profile_snapshot_id, receipt)?;
        let mut candidate_active = self.active.clone();
        if candidate_active.remove(profile_snapshot_id).is_none() {
            return Err(OpportunityRepositoryError::ProfileIdentityConflict);
        }
        let mut candidate_tombstones = self.tombstones.clone();
        if candidate_tombstones.len() >= self.max_tombstones {
            return Err(OpportunityRepositoryError::CapacityExceeded);
        }
        let persisted = tombstone_to_persisted(principal, &committed);
        if candidate_tombstones
            .insert(profile_snapshot_id.clone(), persisted)
            .is_some()
        {
            return Err(OpportunityRepositoryError::DeleteIdentityConflict);
        }
        self.persist(&candidate_active, &candidate_tombstones)?;
        self.inner = candidate_inner;
        self.active = candidate_active;
        self.tombstones = candidate_tombstones;
        Ok(committed)
    }
}

fn profile_to_persisted(
    record: &TenantProfileRecord,
) -> Result<PersistedProfile, OpportunityRepositoryError> {
    let snapshot = record.profile().snapshot();
    let mut fields = record
        .consent_fields()
        .iter()
        .map(|field| match field {
            ConsentField::CompletedCourses => "completed_courses".to_owned(),
            ConsentField::CreditBounds => "credit_bounds".to_owned(),
            ConsentField::PreferenceWeights => "preference_weights".to_owned(),
        })
        .collect::<Vec<_>>();
    fields.sort();
    Ok(PersistedProfile {
        tenant_id: record.principal().tenant_id().as_str().to_owned(),
        user_id: record.principal().user_id().as_str().to_owned(),
        profile_snapshot_id: record.profile_snapshot_id().as_str().to_owned(),
        consent_id: record.consent_id().as_str().to_owned(),
        consent_purpose: match record.consent_purpose() {
            ConsentPurpose::OpportunityPlanning => "opportunity_planning".to_owned(),
        },
        consent_fields: fields,
        consented_at_unix_nanos: record.consented_at().unix_timestamp_nanos().to_string(),
        completed_courses: snapshot.completed_courses.clone(),
        min_credits: snapshot.min_credits,
        max_credits: snapshot.max_credits,
        preference_weights: snapshot.preference_weights.clone(),
    })
}

fn tombstone_to_persisted(
    principal: &AuthenticatedPrincipal,
    receipt: &DeletionReceipt,
) -> PersistedTombstone {
    PersistedTombstone {
        tenant_id: principal.tenant_id().as_str().to_owned(),
        user_id: principal.user_id().as_str().to_owned(),
        profile_snapshot_id: receipt.profile_snapshot_id().as_str().to_owned(),
        consent_id: receipt.consent_id().as_str().to_owned(),
        deletion_receipt_id: receipt.receipt_id().as_str().to_owned(),
        deleted_at_unix_nanos: receipt.deleted_at_unix_nanos().to_string(),
    }
}

fn rehydrate_tombstone(persisted: &PersistedTombstone) -> Result<ProfileTombstone, String> {
    let principal = principal(&persisted.tenant_id, &persisted.user_id)?;
    let profile_id = ProfileSnapshotId::parse(&persisted.profile_snapshot_id)
        .map_err(|error| format!("persisted tombstone profile id: {error}"))?;
    let consent_id = ConsentId::parse(&persisted.consent_id)
        .map_err(|error| format!("persisted tombstone consent id: {error}"))?;
    let deleted_at = timestamp_nanos(&persisted.deleted_at_unix_nanos)?;
    let receipt = DeletionReceipt::restore(&principal, profile_id, consent_id.clone(), deleted_at)
        .map_err(|error| format!("persisted deletion receipt: {error}"))?;
    if receipt.receipt_id().as_str() != persisted.deletion_receipt_id {
        return Err("persisted deletion receipt identity drift".to_owned());
    }
    ProfileTombstone::restore(&principal, consent_id, receipt)
        .map_err(|error| format!("persisted tombstone: {error}"))
}

fn persisted_profile_eq(left: &PersistedProfile, right: &PersistedProfile) -> bool {
    left.tenant_id == right.tenant_id
        && left.user_id == right.user_id
        && left.profile_snapshot_id == right.profile_snapshot_id
        && left.consent_id == right.consent_id
        && left.consent_purpose == right.consent_purpose
        && left.consent_fields == right.consent_fields
        && left.consented_at_unix_nanos == right.consented_at_unix_nanos
        && left.completed_courses == right.completed_courses
        && left.min_credits == right.min_credits
        && left.max_credits == right.max_credits
        && left.preference_weights == right.preference_weights
}

fn principal(tenant_id: &str, user_id: &str) -> Result<AuthenticatedPrincipal, String> {
    let tenant_id = TenantId::parse(tenant_id).map_err(|error| format!("tenant id: {error}"))?;
    let user_id = UserId::parse(user_id).map_err(|error| format!("user id: {error}"))?;
    AuthenticatedPrincipal::new(tenant_id, user_id)
        .map_err(|error| format!("authenticated principal: {error}"))
}

fn consent_purpose(value: &str) -> Result<ConsentPurpose, String> {
    match value {
        "opportunity_planning" => Ok(ConsentPurpose::OpportunityPlanning),
        _ => Err("persisted consent purpose is invalid".to_owned()),
    }
}

fn consent_fields(values: &[String]) -> Result<Vec<ConsentField>, String> {
    let fields = values
        .iter()
        .map(|value| match value.as_str() {
            "completed_courses" => Ok(ConsentField::CompletedCourses),
            "credit_bounds" => Ok(ConsentField::CreditBounds),
            "preference_weights" => Ok(ConsentField::PreferenceWeights),
            _ => Err("persisted consent field is invalid".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if fields.len() != 3 || fields.iter().copied().collect::<BTreeSet<_>>().len() != 3 {
        return Err("persisted consent fields are not the exact required set".to_owned());
    }
    Ok(fields)
}

fn timestamp_nanos(value: &str) -> Result<OffsetDateTime, String> {
    let nanos = value
        .parse::<i128>()
        .map_err(|_| "persisted timestamp is not an i128".to_owned())?;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|error| format!("persisted timestamp: {error}"))
}
