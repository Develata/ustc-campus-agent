//! Durable, fail-closed publication repository for the bounded Affairs fixture.
//!
//! Persisted bytes are private adapter DTOs. Every authority-bearing domain
//! value is reconstructed through `ProcedurePublicationRecoveryRecord` and the
//! checked in-memory repository; the file is never deserialized directly into
//! M71 domain state.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use affairs_navigator::{
    ActorRef, AffairsRepository, AffairsRepositoryReadError, ArtifactId,
    InMemoryPublishedAffairsRepository, ProcedureArtifact, ProcedureDraft, ProcedureId,
    ProcedurePublicationCommit, ProcedurePublicationReceipt, ProcedurePublicationReceiptId,
    ProcedurePublicationRecoveryAnchor, ProcedurePublicationRecoveryRecord,
    ProcedurePublicationRepository, ProcedurePublicationRepositoryError, ProcedurePublicationState,
    ProcedureReviewId, Sha256,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ustc_campus_agent_core::source_revision::SourceRevisionId;

const STATE_SCHEMA_VERSION: u8 = 1;
const DEFAULT_MAX_RECORDS: usize = 32;
const DEFAULT_MAX_BYTES: u64 = 1_048_576;
const TEMP_ATTEMPTS: usize = 16;

fn map_read_error(error: ProcedurePublicationRepositoryError) -> AffairsRepositoryReadError {
    match error {
        ProcedurePublicationRepositoryError::PersistenceUnavailable => {
            AffairsRepositoryReadError::Unavailable
        }
        _ => AffairsRepositoryReadError::Corrupted,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedPublicationRecord {
    source_revision_id: String,
    draft_digest: String,
    review_id: String,
    reviewer: String,
    reviewed_at_unix_nanos: String,
    receipt_id: String,
    artifact_id: String,
    expected_publication_revision: Option<u64>,
    publication_revision: u64,
    published_at_unix_nanos: String,
    m60_evidence_set_digest: String,
    m60_revision_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedPublicationState {
    schema_version: u8,
    source_revision_id: String,
    draft_digest: String,
    records: Vec<PersistedPublicationRecord>,
}

impl PersistedPublicationState {
    fn empty(draft: &ProcedureDraft) -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            source_revision_id: draft.source_revision().revision_id().as_str().to_owned(),
            draft_digest: draft.draft_digest().as_str().to_owned(),
            records: Vec::new(),
        }
    }
}

/// Composition-owned durable adapter. One instance is authoritative for both
/// direct administrator publication and ordinary M71 query reads.
pub(crate) struct DurablePublishedAffairsRepository {
    path: PathBuf,
    draft: ProcedureDraft,
    anchor: ProcedurePublicationRecoveryAnchor,
    inner: InMemoryPublishedAffairsRepository,
    records: Vec<PersistedPublicationRecord>,
    max_records: usize,
    max_bytes: u64,
    fail_next_persist: bool,
    fail_next_parent_sync_after_rename: bool,
}

impl DurablePublishedAffairsRepository {
    pub(crate) fn open(
        path: &Path,
        draft: ProcedureDraft,
        anchor: ProcedurePublicationRecoveryAnchor,
        allow_fresh_bootstrap: bool,
    ) -> Result<Self, String> {
        Self::open_with_limits(
            path,
            draft,
            anchor,
            allow_fresh_bootstrap,
            DEFAULT_MAX_RECORDS,
            DEFAULT_MAX_BYTES,
        )
    }

    fn open_with_limits(
        path: &Path,
        draft: ProcedureDraft,
        anchor: ProcedurePublicationRecoveryAnchor,
        allow_fresh_bootstrap: bool,
        max_records: usize,
        max_bytes: u64,
    ) -> Result<Self, String> {
        if max_records == 0 || max_bytes == 0 {
            return Err("publication persistence limits must be nonzero".to_owned());
        }
        validate_secure_parent(path).map_err(error_text)?;
        let state = match read_existing(path, max_bytes).map_err(error_text)? {
            None if allow_fresh_bootstrap => PersistedPublicationState::empty(&draft),
            None => {
                return Err("publication state is missing from an existing state set".to_owned());
            }
            Some(bytes) => {
                let state: PersistedPublicationState = serde_json::from_slice(&bytes)
                    .map_err(|_| "publication state is not valid schema-v1 JSON".to_owned())?;
                let canonical = serde_json::to_vec(&state)
                    .map_err(|_| "publication state canonicalization failed".to_owned())?;
                if canonical != bytes {
                    return Err("publication state is noncanonical".to_owned());
                }
                state
            }
        };
        validate_state_binding(&state, &draft, max_records)?;
        let inner = rebuild_inner(&draft, &anchor, &state.records)?;
        Ok(Self {
            path: path.to_owned(),
            draft,
            anchor,
            inner,
            records: state.records,
            max_records,
            max_bytes,
            fail_next_persist: false,
            fail_next_parent_sync_after_rename: false,
        })
    }

    #[must_use]
    pub(crate) fn record_count(&self) -> usize {
        self.records.len()
    }

    pub(crate) fn latest_receipt(
        &self,
    ) -> Result<Option<ProcedurePublicationReceipt>, ProcedurePublicationRepositoryError> {
        self.verify_durable_matches()?;
        let Some(record) = self.records.last() else {
            return Ok(None);
        };
        let receipt_id = ProcedurePublicationReceiptId::parse(&record.receipt_id)
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        self.inner.find_publication_replay(&receipt_id)
    }

    pub(crate) fn fail_next_persist(&mut self) {
        self.fail_next_persist = true;
    }

    pub(crate) fn fail_next_parent_sync_after_rename(&mut self) {
        self.fail_next_parent_sync_after_rename = true;
    }

    fn state_with(&self, records: Vec<PersistedPublicationRecord>) -> PersistedPublicationState {
        PersistedPublicationState {
            schema_version: STATE_SCHEMA_VERSION,
            source_revision_id: self
                .draft
                .source_revision()
                .revision_id()
                .as_str()
                .to_owned(),
            draft_digest: self.draft.draft_digest().as_str().to_owned(),
            records,
        }
    }

    fn verify_durable_matches(&self) -> Result<(), ProcedurePublicationRepositoryError> {
        let expected = serde_json::to_vec(&self.state_with(self.records.clone()))
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        match read_existing(&self.path, self.max_bytes)? {
            None if self.records.is_empty() => Ok(()),
            Some(actual) if actual == expected => Ok(()),
            _ => Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted),
        }
    }

    fn persist(
        &mut self,
        records: &[PersistedPublicationRecord],
    ) -> Result<(), ProcedurePublicationRepositoryError> {
        if self.fail_next_persist {
            self.fail_next_persist = false;
            return Err(ProcedurePublicationRepositoryError::FailureInjected);
        }
        let state = self.state_with(records.to_vec());
        validate_state_binding(&state, &self.draft, self.max_records)
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        let bytes = serde_json::to_vec(&state)
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > self.max_bytes {
            return Err(ProcedurePublicationRepositoryError::PersistenceLimitExceeded);
        }
        validate_secure_parent(&self.path)?;
        validate_existing_destination(&self.path, self.max_bytes)?;
        let parent = direct_parent(&self.path)?;

        let mut temporary = None;
        let mut file = None;
        for _ in 0..TEMP_ATTEMPTS {
            let candidate = unpredictable_temporary(parent, &self.path)?;
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&candidate)
            {
                Ok(opened) => {
                    temporary = Some(candidate);
                    file = Some(opened);
                    break;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err(ProcedurePublicationRepositoryError::PersistenceUnavailable),
            }
        }
        let temporary =
            temporary.ok_or(ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
        let mut file =
            file.ok_or(ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        let mut renamed = false;
        let result = (|| {
            validate_primary_metadata(
                &file
                    .metadata()
                    .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?,
                self.max_bytes,
            )?;
            file.write_all(&bytes)
                .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
            file.sync_all()
                .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
            drop(file);
            fs::rename(&temporary, &self.path)
                .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
            renamed = true;
            if self.fail_next_parent_sync_after_rename {
                self.fail_next_parent_sync_after_rename = false;
                return Err(ProcedurePublicationRepositoryError::PersistenceUnavailable);
            }
            File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
            if renamed
                && matches!(read_existing(&self.path, self.max_bytes), Ok(Some(actual)) if actual == bytes)
            {
                return Ok(());
            }
        }
        result
    }
}

pub(crate) fn recovery_anchor_and_commit_from_receipt(
    draft: &ProcedureDraft,
    receipt: &ProcedurePublicationReceipt,
) -> Result<
    (
        ProcedurePublicationRecoveryAnchor,
        ProcedurePublicationCommit,
    ),
    String,
> {
    let anchor = ProcedurePublicationRecoveryAnchor::from_receipt(draft, receipt)
        .map_err(|_| "fixture publication receipt cannot mint its recovery anchor".to_owned())?;
    let (_, commit) = ProcedurePublicationRecoveryRecord::try_recover(
        draft,
        &anchor,
        draft.source_revision().revision_id().clone(),
        receipt.draft_digest().clone(),
        receipt.review_id().clone(),
        receipt.reviewer().clone(),
        receipt.reviewed_at(),
        receipt.receipt_id().clone(),
        receipt.artifact_id().clone(),
        receipt.expected_publication_revision(),
        receipt.publication_revision(),
        receipt.published_at(),
        receipt.m60_evidence_set_digest().clone(),
        receipt.m60_revision_count(),
    )
    .map_err(|_| "fixture publication receipt cannot reconstruct its sealed commit".to_owned())?;
    Ok((anchor, commit))
}

impl AffairsRepository for DurablePublishedAffairsRepository {
    fn find_current_artifact(
        &self,
        procedure_id: &ProcedureId,
    ) -> Result<Option<ProcedureArtifact>, AffairsRepositoryReadError> {
        self.verify_durable_matches().map_err(map_read_error)?;
        self.inner.find_current_artifact(procedure_id)
    }

    fn find_publication_state(
        &self,
        procedure_id: &ProcedureId,
    ) -> Result<Option<ProcedurePublicationState>, AffairsRepositoryReadError> {
        self.verify_durable_matches().map_err(map_read_error)?;
        self.inner.find_publication_state(procedure_id)
    }
}

impl ProcedurePublicationRepository for DurablePublishedAffairsRepository {
    fn publication_revision(&self, procedure_id: &ProcedureId) -> Option<u64> {
        self.verify_durable_matches().ok()?;
        self.inner.publication_revision(procedure_id)
    }

    fn find_publication_replay(
        &self,
        receipt_id: &ProcedurePublicationReceiptId,
    ) -> Result<Option<ProcedurePublicationReceipt>, ProcedurePublicationRepositoryError> {
        self.verify_durable_matches()?;
        self.inner.find_publication_replay(receipt_id)
    }

    fn apply_publication(
        &mut self,
        commit: ProcedurePublicationCommit,
    ) -> Result<(), ProcedurePublicationRepositoryError> {
        self.verify_durable_matches()?;
        let record = ProcedurePublicationRecoveryRecord::from_commit(&commit, &self.draft)?;
        let persisted = persisted_record(&record);
        recover_record(&self.draft, &self.anchor, &persisted)
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        if let Some(existing) = self
            .records
            .iter()
            .find(|value| value.receipt_id == persisted.receipt_id)
        {
            return if existing == &persisted {
                self.inner.apply_publication(commit)
            } else {
                Err(ProcedurePublicationRepositoryError::ReceiptIdentityConflict)
            };
        }
        if self.records.len() >= self.max_records {
            return Err(ProcedurePublicationRepositoryError::PersistenceLimitExceeded);
        }

        let mut candidate_inner = rebuild_inner(&self.draft, &self.anchor, &self.records)
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        candidate_inner.apply_publication(commit)?;
        let mut candidate_records = self.records.clone();
        candidate_records.push(persisted);
        validate_record_sequence(&candidate_records)
            .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
        self.persist(&candidate_records)?;
        self.inner = candidate_inner;
        self.records = candidate_records;
        Ok(())
    }
}

fn persisted_record(record: &ProcedurePublicationRecoveryRecord) -> PersistedPublicationRecord {
    PersistedPublicationRecord {
        source_revision_id: record.source_revision_id().as_str().to_owned(),
        draft_digest: record.draft_digest().as_str().to_owned(),
        review_id: record.review_id().as_str().to_owned(),
        reviewer: record.reviewer().as_str().to_owned(),
        reviewed_at_unix_nanos: record.reviewed_at().unix_timestamp_nanos().to_string(),
        receipt_id: record.receipt_id().as_str().to_owned(),
        artifact_id: record.artifact_id().as_str().to_owned(),
        expected_publication_revision: record.expected_publication_revision(),
        publication_revision: record.publication_revision(),
        published_at_unix_nanos: record.published_at().unix_timestamp_nanos().to_string(),
        m60_evidence_set_digest: record.m60_evidence_set_digest().as_str().to_owned(),
        m60_revision_count: record.m60_revision_count(),
    }
}

fn recover_record(
    draft: &ProcedureDraft,
    anchor: &ProcedurePublicationRecoveryAnchor,
    persisted: &PersistedPublicationRecord,
) -> Result<
    (
        ProcedurePublicationRecoveryRecord,
        ProcedurePublicationCommit,
    ),
    String,
> {
    ProcedurePublicationRecoveryRecord::try_recover(
        draft,
        anchor,
        SourceRevisionId::parse(&persisted.source_revision_id)
            .map_err(|_| "persisted source revision id is invalid".to_owned())?,
        Sha256::new(&persisted.draft_digest)
            .map_err(|_| "persisted draft digest is invalid".to_owned())?,
        ProcedureReviewId::parse(&persisted.review_id)
            .map_err(|_| "persisted review id is invalid".to_owned())?,
        ActorRef::parse(&persisted.reviewer)
            .map_err(|_| "persisted reviewer is invalid".to_owned())?,
        timestamp_nanos(&persisted.reviewed_at_unix_nanos)?,
        ProcedurePublicationReceiptId::parse(&persisted.receipt_id)
            .map_err(|_| "persisted receipt id is invalid".to_owned())?,
        ArtifactId::parse(&persisted.artifact_id)
            .map_err(|_| "persisted artifact id is invalid".to_owned())?,
        persisted.expected_publication_revision,
        persisted.publication_revision,
        timestamp_nanos(&persisted.published_at_unix_nanos)?,
        Sha256::new(&persisted.m60_evidence_set_digest)
            .map_err(|_| "persisted M60 evidence digest is invalid".to_owned())?,
        persisted.m60_revision_count,
    )
    .map_err(|_| "persisted publication record is incoherent".to_owned())
}

fn rebuild_inner(
    draft: &ProcedureDraft,
    anchor: &ProcedurePublicationRecoveryAnchor,
    records: &[PersistedPublicationRecord],
) -> Result<InMemoryPublishedAffairsRepository, String> {
    validate_record_sequence(records)?;
    let mut inner = InMemoryPublishedAffairsRepository::new();
    for persisted in records {
        let (_, commit) = recover_record(draft, anchor, persisted)?;
        inner
            .apply_publication(commit)
            .map_err(|_| "persisted publication sequence is incoherent".to_owned())?;
    }
    Ok(inner)
}

fn validate_state_binding(
    state: &PersistedPublicationState,
    draft: &ProcedureDraft,
    max_records: usize,
) -> Result<(), String> {
    if state.schema_version != STATE_SCHEMA_VERSION {
        return Err("publication state schema version is unsupported".to_owned());
    }
    if state.source_revision_id != draft.source_revision().revision_id().as_str()
        || state.draft_digest != draft.draft_digest().as_str()
    {
        return Err("publication state is bound to a different fixture draft".to_owned());
    }
    if state.records.len() > max_records {
        return Err("publication state record capacity exceeded".to_owned());
    }
    validate_record_sequence(&state.records)
}

fn validate_record_sequence(records: &[PersistedPublicationRecord]) -> Result<(), String> {
    let mut receipt_ids = BTreeSet::new();
    let mut artifact_ids = BTreeSet::new();
    for (index, record) in records.iter().enumerate() {
        let publication_revision = u64::try_from(index + 1)
            .map_err(|_| "publication revision sequence overflow".to_owned())?;
        let expected = if publication_revision == 1 {
            None
        } else {
            Some(publication_revision - 1)
        };
        if record.expected_publication_revision != expected
            || record.publication_revision != publication_revision
            || !receipt_ids.insert(record.receipt_id.as_str())
            || !artifact_ids.insert(record.artifact_id.as_str())
        {
            return Err("publication records are duplicated, reordered, or gapped".to_owned());
        }
    }
    Ok(())
}

fn timestamp_nanos(value: &str) -> Result<OffsetDateTime, String> {
    let nanos = value
        .parse::<i128>()
        .map_err(|_| "persisted publication timestamp is invalid".to_owned())?;
    OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|_| "persisted publication timestamp is out of range".to_owned())
}

fn read_existing(
    path: &Path,
    max_bytes: u64,
) -> Result<Option<Vec<u8>>, ProcedurePublicationRepositoryError> {
    validate_secure_parent(path)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(ProcedurePublicationRepositoryError::PersistenceUnavailable),
    };
    validate_primary_metadata(&metadata, max_bytes)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
    validate_primary_metadata(&opened, max_bytes)?;
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(ProcedurePublicationRepositoryError::PersistenceLimitExceeded);
    }
    Ok(Some(bytes))
}

fn validate_existing_destination(
    path: &Path,
    max_bytes: u64,
) -> Result<(), ProcedurePublicationRepositoryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_primary_metadata(&metadata, max_bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ProcedurePublicationRepositoryError::PersistenceUnavailable),
    }
}

fn validate_primary_metadata(
    metadata: &fs::Metadata,
    max_bytes: u64,
) -> Result<(), ProcedurePublicationRepositoryError> {
    if !metadata.file_type().is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.nlink() != 1
        || metadata.uid() != current_uid()?
    {
        return Err(ProcedurePublicationRepositoryError::StoredPublicationCorrupted);
    }
    if metadata.len() > max_bytes {
        return Err(ProcedurePublicationRepositoryError::PersistenceLimitExceeded);
    }
    Ok(())
}

fn validate_secure_parent(path: &Path) -> Result<(), ProcedurePublicationRepositoryError> {
    crate::durable_path::ensure_secure_parent(path, false)
        .map_err(|_| ProcedurePublicationRepositoryError::StoredPublicationCorrupted)
}

fn direct_parent(path: &Path) -> Result<&Path, ProcedurePublicationRepositoryError> {
    path.parent()
        .filter(|value| !value.as_os_str().is_empty())
        .ok_or(ProcedurePublicationRepositoryError::StoredPublicationCorrupted)
}

fn current_uid() -> Result<u32, ProcedurePublicationRepositoryError> {
    fs::metadata("/proc/self")
        .map(|metadata| metadata.uid())
        .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)
}

fn unpredictable_temporary(
    parent: &Path,
    destination: &Path,
) -> Result<PathBuf, ProcedurePublicationRepositoryError> {
    let mut random = [0_u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut random))
        .map_err(|_| ProcedurePublicationRepositoryError::PersistenceUnavailable)?;
    let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ProcedurePublicationRepositoryError::StoredPublicationCorrupted)?;
    Ok(parent.join(format!(".{name}.{nonce}.tmp")))
}

fn error_text(error: ProcedurePublicationRepositoryError) -> String {
    format!("publication persistence rejected: {error:?}")
}
