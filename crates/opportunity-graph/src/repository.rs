use crate::{
    AcademicProfileInput, AuthenticatedPrincipal, ConsentField, ConsentId, ConsentPurpose,
    DeletionReceipt, ProfileSnapshotId, ProfileTombstone,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use time::OffsetDateTime;
use ustc_campus_agent_core::identity::{TenantId, UserId};

#[derive(Clone, PartialEq, Eq)]
pub struct TenantProfileRecord {
    principal: AuthenticatedPrincipal,
    profile_snapshot_id: ProfileSnapshotId,
    consent_id: ConsentId,
    consent_purpose: ConsentPurpose,
    consent_fields: BTreeSet<ConsentField>,
    consented_at: OffsetDateTime,
    profile: AcademicProfileInput,
}

impl TenantProfileRecord {
    pub(crate) fn new(
        principal: AuthenticatedPrincipal,
        profile_snapshot_id: ProfileSnapshotId,
        consent_id: ConsentId,
        consent_purpose: ConsentPurpose,
        consent_fields: BTreeSet<ConsentField>,
        consented_at: OffsetDateTime,
        profile: AcademicProfileInput,
    ) -> Self {
        Self {
            principal,
            profile_snapshot_id,
            consent_id,
            consent_purpose,
            consent_fields,
            consented_at,
            profile,
        }
    }

    #[must_use]
    pub const fn principal(&self) -> &AuthenticatedPrincipal {
        &self.principal
    }

    #[must_use]
    pub const fn profile_snapshot_id(&self) -> &ProfileSnapshotId {
        &self.profile_snapshot_id
    }

    #[must_use]
    pub const fn consent_id(&self) -> &ConsentId {
        &self.consent_id
    }

    #[must_use]
    pub const fn consent_purpose(&self) -> ConsentPurpose {
        self.consent_purpose
    }

    #[must_use]
    pub const fn consent_fields(&self) -> &BTreeSet<ConsentField> {
        &self.consent_fields
    }

    #[must_use]
    pub const fn consented_at(&self) -> OffsetDateTime {
        self.consented_at
    }

    #[must_use]
    pub const fn profile(&self) -> &AcademicProfileInput {
        &self.profile
    }
}

impl fmt::Debug for TenantProfileRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TenantProfileRecord")
            .field("principal", &self.principal)
            .field("profile_snapshot_id", &self.profile_snapshot_id)
            .field("consent_id", &self.consent_id)
            .field("consent_purpose", &self.consent_purpose)
            .field("consent_fields", &self.consent_fields)
            .field("consented_at", &self.consented_at)
            .field("profile", &"<redacted-private-profile>")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileLookup {
    Active(TenantProfileRecord),
    Deleted(ProfileTombstone),
    AccessDenied,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryFailureMode {
    None,
    ReadUnavailable,
    WriteUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OpportunityRepositoryError {
    Unavailable,
    CapacityExceeded,
    PrincipalAlreadyHasProfile,
    ProfileIdentityConflict,
    DeleteIdentityConflict,
    AccessDenied,
}

impl fmt::Display for OpportunityRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Opportunity profile repository rejected operation: {self:?}"
        )
    }
}

impl Error for OpportunityRepositoryError {}

pub trait OpportunityProfileRepository: Send + Sync {
    fn create(&mut self, record: TenantProfileRecord) -> Result<(), OpportunityRepositoryError>;

    fn lookup(
        &self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
    ) -> Result<ProfileLookup, OpportunityRepositoryError>;

    fn delete(
        &mut self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
        receipt: DeletionReceipt,
    ) -> Result<DeletionReceipt, OpportunityRepositoryError>;
}

pub struct InMemoryOpportunityProfileRepository {
    profiles: BTreeMap<ProfileSnapshotId, TenantProfileRecord>,
    current_by_principal: BTreeMap<(TenantId, UserId), ProfileSnapshotId>,
    tombstones: BTreeMap<ProfileSnapshotId, ProfileTombstone>,
    max_profiles: usize,
    max_tombstones: usize,
    failure_mode: RepositoryFailureMode,
}

impl InMemoryOpportunityProfileRepository {
    pub fn new(
        max_profiles: usize,
        max_tombstones: usize,
    ) -> Result<Self, OpportunityRepositoryError> {
        if max_profiles == 0 || max_tombstones == 0 {
            return Err(OpportunityRepositoryError::CapacityExceeded);
        }
        Ok(Self {
            profiles: BTreeMap::new(),
            current_by_principal: BTreeMap::new(),
            tombstones: BTreeMap::new(),
            max_profiles,
            max_tombstones,
            failure_mode: RepositoryFailureMode::None,
        })
    }

    pub fn set_failure_mode(&mut self, failure_mode: RepositoryFailureMode) -> &mut Self {
        self.failure_mode = failure_mode;
        self
    }

    #[must_use]
    pub fn private_payload_count(&self) -> usize {
        self.profiles.len()
    }

    #[must_use]
    pub fn tombstone_count(&self) -> usize {
        self.tombstones.len()
    }
}

impl OpportunityProfileRepository for InMemoryOpportunityProfileRepository {
    fn create(&mut self, record: TenantProfileRecord) -> Result<(), OpportunityRepositoryError> {
        if self.failure_mode == RepositoryFailureMode::WriteUnavailable {
            return Err(OpportunityRepositoryError::Unavailable);
        }
        if self.profiles.len() >= self.max_profiles {
            return Err(OpportunityRepositoryError::CapacityExceeded);
        }
        let principal_key = (
            record.principal().tenant_id().clone(),
            record.principal().user_id().clone(),
        );
        if self.current_by_principal.contains_key(&principal_key) {
            return Err(OpportunityRepositoryError::PrincipalAlreadyHasProfile);
        }
        if self.profiles.contains_key(record.profile_snapshot_id())
            || self.tombstones.contains_key(record.profile_snapshot_id())
        {
            return Err(OpportunityRepositoryError::ProfileIdentityConflict);
        }
        self.current_by_principal
            .insert(principal_key, record.profile_snapshot_id().clone());
        self.profiles
            .insert(record.profile_snapshot_id().clone(), record);
        Ok(())
    }

    fn lookup(
        &self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
    ) -> Result<ProfileLookup, OpportunityRepositoryError> {
        if self.failure_mode == RepositoryFailureMode::ReadUnavailable {
            return Err(OpportunityRepositoryError::Unavailable);
        }
        if let Some(record) = self.profiles.get(profile_snapshot_id) {
            if record.principal() != principal {
                return Ok(ProfileLookup::AccessDenied);
            }
            return Ok(ProfileLookup::Active(record.clone()));
        }
        if let Some(tombstone) = self.tombstones.get(profile_snapshot_id) {
            if tombstone.tenant_id() != principal.tenant_id()
                || tombstone.user_id() != principal.user_id()
            {
                return Ok(ProfileLookup::AccessDenied);
            }
            return Ok(ProfileLookup::Deleted(tombstone.clone()));
        }
        Ok(ProfileLookup::Missing)
    }

    fn delete(
        &mut self,
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: &ProfileSnapshotId,
        receipt: DeletionReceipt,
    ) -> Result<DeletionReceipt, OpportunityRepositoryError> {
        if self.failure_mode == RepositoryFailureMode::WriteUnavailable {
            return Err(OpportunityRepositoryError::Unavailable);
        }
        if let Some(tombstone) = self.tombstones.get(profile_snapshot_id) {
            if tombstone.tenant_id() != principal.tenant_id()
                || tombstone.user_id() != principal.user_id()
                || tombstone.deletion_receipt() != &receipt
            {
                return Err(OpportunityRepositoryError::DeleteIdentityConflict);
            }
            return Ok(tombstone.deletion_receipt().clone());
        }
        let Some(record) = self.profiles.get(profile_snapshot_id) else {
            return Err(OpportunityRepositoryError::ProfileIdentityConflict);
        };
        if record.principal() != principal {
            return Err(OpportunityRepositoryError::AccessDenied);
        }
        if receipt.profile_snapshot_id() != profile_snapshot_id
            || receipt.consent_id() != record.consent_id()
        {
            return Err(OpportunityRepositoryError::DeleteIdentityConflict);
        }
        if self.tombstones.len() >= self.max_tombstones {
            return Err(OpportunityRepositoryError::CapacityExceeded);
        }

        let consent_id = record.consent_id().clone();
        let principal_key = (principal.tenant_id().clone(), principal.user_id().clone());
        self.profiles.remove(profile_snapshot_id);
        self.current_by_principal.remove(&principal_key);
        self.tombstones.insert(
            profile_snapshot_id.clone(),
            ProfileTombstone::new(principal, consent_id, receipt.clone()),
        );
        Ok(receipt)
    }
}
