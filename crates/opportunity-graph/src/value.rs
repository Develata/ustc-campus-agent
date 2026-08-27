use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use time::OffsetDateTime;
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_course_planning::UserAcademicSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum OpportunityValueError {
    InvalidId,
    InvalidPrincipal,
    InvalidConsentFields,
    InvalidCreditBounds,
    InvalidCourseCode,
    SerializationFailed,
}

impl fmt::Display for OpportunityValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid Opportunity Graph value: {self:?}")
    }
}

impl Error for OpportunityValueError {}

macro_rules! checked_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, OpportunityValueError> {
                let value = value.into();
                let Some(tail) = value.strip_prefix($prefix) else {
                    return Err(OpportunityValueError::InvalidId);
                };
                if tail.is_empty()
                    || tail.len() > 128
                    || !tail.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-')
                    })
                {
                    return Err(OpportunityValueError::InvalidId);
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

checked_id!(ConsentId, "consent:opportunity:");
checked_id!(ProfileSnapshotId, "profile-snapshot:opportunity:");
checked_id!(DeletionReceiptId, "profile-deletion:opportunity:");

#[derive(Clone, PartialEq, Eq)]
pub struct AuthenticatedPrincipal {
    tenant_id: TenantId,
    user_id: UserId,
}

impl AuthenticatedPrincipal {
    pub fn new(tenant_id: TenantId, user_id: UserId) -> Result<Self, OpportunityValueError> {
        if tenant_id.as_str().trim().is_empty() || user_id.as_str().trim().is_empty() {
            return Err(OpportunityValueError::InvalidPrincipal);
        }
        Ok(Self { tenant_id, user_id })
    }

    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }
}

impl fmt::Debug for AuthenticatedPrincipal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticatedPrincipal")
            .field("tenant_id", &self.tenant_id.as_str())
            .field("user_id", &self.user_id.as_str())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentField {
    CompletedCourses,
    CreditBounds,
    PreferenceWeights,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentPurpose {
    OpportunityPlanning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentGrant {
    purpose: ConsentPurpose,
    fields: BTreeSet<ConsentField>,
    consented_at: OffsetDateTime,
}

impl ConsentGrant {
    pub fn new(
        purpose: ConsentPurpose,
        fields: impl IntoIterator<Item = ConsentField>,
        consented_at: OffsetDateTime,
    ) -> Result<Self, OpportunityValueError> {
        let fields: BTreeSet<_> = fields.into_iter().collect();
        let required = BTreeSet::from([
            ConsentField::CompletedCourses,
            ConsentField::CreditBounds,
            ConsentField::PreferenceWeights,
        ]);
        if fields != required {
            return Err(OpportunityValueError::InvalidConsentFields);
        }
        Ok(Self {
            purpose,
            fields,
            consented_at,
        })
    }

    #[must_use]
    pub const fn purpose(&self) -> ConsentPurpose {
        self.purpose
    }

    #[must_use]
    pub const fn fields(&self) -> &BTreeSet<ConsentField> {
        &self.fields
    }

    #[must_use]
    pub const fn consented_at(&self) -> OffsetDateTime {
        self.consented_at
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AcademicProfileInput {
    snapshot: UserAcademicSnapshot,
}

impl AcademicProfileInput {
    pub fn new(
        completed_courses: Vec<String>,
        min_credits: u16,
        max_credits: u16,
        preference_weights: BTreeMap<String, i32>,
    ) -> Result<Self, OpportunityValueError> {
        if max_credits == 0 || min_credits > max_credits {
            return Err(OpportunityValueError::InvalidCreditBounds);
        }
        let mut completed_courses = completed_courses;
        for code in completed_courses.iter().chain(preference_weights.keys()) {
            if !valid_course_code(code) {
                return Err(OpportunityValueError::InvalidCourseCode);
            }
        }
        completed_courses.sort();
        completed_courses.dedup();
        Ok(Self {
            snapshot: UserAcademicSnapshot {
                completed_courses,
                min_credits,
                max_credits,
                preference_weights,
            },
        })
    }

    #[must_use]
    pub const fn snapshot(&self) -> &UserAcademicSnapshot {
        &self.snapshot
    }

    pub(crate) fn canonical_bytes(&self) -> Result<Vec<u8>, OpportunityValueError> {
        serde_json::to_vec(&self.snapshot).map_err(|_| OpportunityValueError::SerializationFailed)
    }
}

impl fmt::Debug for AcademicProfileInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AcademicProfileInput")
            .field(
                "completed_course_count",
                &self.snapshot.completed_courses.len(),
            )
            .field("min_credits", &self.snapshot.min_credits)
            .field("max_credits", &self.snapshot.max_credits)
            .field("preference_count", &self.snapshot.preference_weights.len())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeletionReceipt {
    receipt_id: DeletionReceiptId,
    profile_snapshot_id: ProfileSnapshotId,
    consent_id: ConsentId,
    deleted_at_unix_seconds: i64,
}

impl DeletionReceipt {
    pub(crate) fn mint(
        principal: &AuthenticatedPrincipal,
        profile_snapshot_id: ProfileSnapshotId,
        consent_id: ConsentId,
        deleted_at: OffsetDateTime,
    ) -> Result<Self, OpportunityValueError> {
        let receipt_id = DeletionReceiptId::parse(format!(
            "profile-deletion:opportunity:sha256:{}",
            hash_parts(
                b"opportunity-profile-deletion/v1\0",
                &[
                    principal.tenant_id().as_str().as_bytes(),
                    principal.user_id().as_str().as_bytes(),
                    profile_snapshot_id.as_str().as_bytes(),
                    consent_id.as_str().as_bytes(),
                    &deleted_at.unix_timestamp_nanos().to_be_bytes(),
                ],
            )
        ))?;
        Ok(Self {
            receipt_id,
            profile_snapshot_id,
            consent_id,
            deleted_at_unix_seconds: deleted_at.unix_timestamp(),
        })
    }

    #[must_use]
    pub const fn receipt_id(&self) -> &DeletionReceiptId {
        &self.receipt_id
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
    pub const fn deleted_at_unix_seconds(&self) -> i64 {
        self.deleted_at_unix_seconds
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileTombstone {
    tenant_id: TenantId,
    user_id: UserId,
    consent_id: ConsentId,
    deletion_receipt: DeletionReceipt,
}

impl ProfileTombstone {
    pub(crate) fn new(
        principal: &AuthenticatedPrincipal,
        consent_id: ConsentId,
        deletion_receipt: DeletionReceipt,
    ) -> Self {
        Self {
            tenant_id: principal.tenant_id().clone(),
            user_id: principal.user_id().clone(),
            consent_id,
            deletion_receipt,
        }
    }

    #[must_use]
    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    #[must_use]
    pub const fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub const fn consent_id(&self) -> &ConsentId {
        &self.consent_id
    }

    #[must_use]
    pub const fn deletion_receipt(&self) -> &DeletionReceipt {
        &self.deletion_receipt
    }
}

pub(crate) fn mint_consent_id(
    principal: &AuthenticatedPrincipal,
    grant: &ConsentGrant,
) -> Result<ConsentId, OpportunityValueError> {
    let fields = serde_json::to_vec(grant.fields())
        .map_err(|_| OpportunityValueError::SerializationFailed)?;
    let purpose = match grant.purpose() {
        ConsentPurpose::OpportunityPlanning => b"opportunity_planning".as_slice(),
    };
    ConsentId::parse(format!(
        "consent:opportunity:sha256:{}",
        hash_parts(
            b"opportunity-consent/v1\0",
            &[
                principal.tenant_id().as_str().as_bytes(),
                principal.user_id().as_str().as_bytes(),
                purpose,
                &fields,
                &grant.consented_at().unix_timestamp_nanos().to_be_bytes(),
            ],
        )
    ))
}

pub(crate) fn mint_profile_snapshot_id(
    principal: &AuthenticatedPrincipal,
    consent_id: &ConsentId,
    profile: &AcademicProfileInput,
) -> Result<ProfileSnapshotId, OpportunityValueError> {
    let profile_bytes = profile.canonical_bytes()?;
    ProfileSnapshotId::parse(format!(
        "profile-snapshot:opportunity:sha256:{}",
        hash_parts(
            b"opportunity-profile-snapshot/v1\0",
            &[
                principal.tenant_id().as_str().as_bytes(),
                principal.user_id().as_str().as_bytes(),
                consent_id.as_str().as_bytes(),
                &profile_bytes,
            ],
        )
    ))
}

fn valid_course_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

pub(crate) fn hash_parts(domain: &[u8], parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    update_part(&mut hasher, domain);
    for part in parts {
        update_part(&mut hasher, part);
    }
    format!("{:x}", hasher.finalize())
}

fn update_part(hasher: &mut Sha256, part: &[u8]) {
    hasher.update((part.len() as u64).to_be_bytes());
    hasher.update(part);
}
