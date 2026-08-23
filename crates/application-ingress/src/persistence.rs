use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ustc_campus_agent_client_protocol::{AdmittedActorDto, DispatchCapsuleBodyV2, M71TerminalDto};

use crate::capability::StoredPublicAuthorization;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StoredReadPolicy {
    Public {
        authorization: StoredPublicAuthorization,
    },
    Authenticated {
        tenant_id: String,
        user_id: String,
    },
}

impl std::fmt::Debug for StoredReadPolicy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public { .. } => formatter
                .debug_struct("StoredReadPolicy")
                .field("kind", &"public")
                .field("authorization", &"[REDACTED]")
                .finish(),
            Self::Authenticated { .. } => formatter
                .debug_struct("StoredReadPolicy")
                .field("kind", &"authenticated")
                .field("tenant_id", &"[REDACTED]")
                .field("user_id", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionReceipt {
    pub fencing_token: NonZeroU64,
    pub terminal_digest: String,
}

impl std::fmt::Debug for CompletionReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CompletionReceipt")
            .field("fencing_token", &"[REDACTED]")
            .field("terminal_digest", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecordState {
    Pending {
        version: u64,
        highest_fencing: u64,
    },
    Claimed {
        version: u64,
        highest_fencing: NonZeroU64,
        fencing_token: NonZeroU64,
        lease_deadline_ms: i64,
    },
    Terminal {
        version: u64,
        highest_fencing: NonZeroU64,
        terminal: Box<M71TerminalDto>,
        completion: CompletionReceipt,
    },
}

impl std::fmt::Debug for RecordState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending { .. } => formatter
                .debug_struct("RecordState")
                .field("kind", &"pending")
                .field("version", &"[REDACTED]")
                .field("highest_fencing", &"[REDACTED]")
                .finish(),
            Self::Claimed { .. } => formatter
                .debug_struct("RecordState")
                .field("kind", &"claimed")
                .field("version", &"[REDACTED]")
                .field("highest_fencing", &"[REDACTED]")
                .field("fencing_token", &"[REDACTED]")
                .field("lease_deadline_ms", &"[REDACTED]")
                .finish(),
            Self::Terminal { .. } => formatter
                .debug_struct("RecordState")
                .field("kind", &"terminal")
                .field("version", &"[REDACTED]")
                .field("highest_fencing", &"[REDACTED]")
                .field("terminal", &"[REDACTED]")
                .field("completion", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredRecord {
    pub capsule: DispatchCapsuleBodyV2,
    pub capsule_digest: String,
    pub read_policy: StoredReadPolicy,
    pub state: RecordState,
}

impl std::fmt::Debug for StoredRecord {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StoredRecord")
            .field("capsule", &"[REDACTED]")
            .field("capsule_digest", &"[REDACTED]")
            .field("read_policy", &"[REDACTED]")
            .field("state", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ClaimToken {
    command_id: String,
    version: u64,
    fencing_token: NonZeroU64,
    lease_deadline_ms: i64,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum InsertOutcome {
    Created,
    Existing(Box<StoredRecord>),
    InvariantCorruption,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum ClaimOutcome {
    Claimed(ClaimToken),
    Busy,
    AlreadyTerminal(Box<StoredRecord>),
    Missing,
}

#[derive(Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum RenewOutcome {
    Renewed(ClaimToken),
    Stale,
    Expired,
    AlreadyTerminal,
    Missing,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum AbandonOutcome {
    Abandoned,
    Stale,
    AlreadyTerminal,
    Missing,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum CompleteOutcome {
    Completed(Box<StoredRecord>),
    AlreadyTerminal(Box<StoredRecord>),
    LostToWinner(Box<StoredRecord>),
    Stale,
    Missing,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct StoreState {
    schema_version: u8,
    records: BTreeMap<String, StoredRecord>,
}

#[derive(Clone)]
pub struct FileRecordStore {
    path: PathBuf,
    state: Arc<Mutex<StoreState>>,
}

impl FileRecordStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let path = path.into();
        let state = if path.exists() {
            let bytes = fs::read(&path).map_err(StoreError::Io)?;
            let state: StoreState =
                serde_json::from_slice(&bytes).map_err(StoreError::Corrupted)?;
            validate_state(&state)?;
            state
        } else {
            StoreState {
                schema_version: 1,
                records: BTreeMap::new(),
            }
        };
        Ok(Self {
            path,
            state: Arc::new(Mutex::new(state)),
        })
    }

    pub(crate) fn insert_admitted_once(
        &self,
        command_id: &str,
        capsule: DispatchCapsuleBodyV2,
        read_policy: StoredReadPolicy,
    ) -> Result<InsertOutcome, StoreError> {
        let capsule_digest = capsule_digest(&capsule)?;
        self.transaction(|state| {
            if let Some(existing) = state.records.get(command_id) {
                return Ok(
                    if existing.capsule_digest == capsule_digest && existing.capsule == capsule {
                        InsertOutcome::Existing(Box::new(existing.clone()))
                    } else {
                        InsertOutcome::InvariantCorruption
                    },
                );
            }
            let record = StoredRecord {
                capsule,
                capsule_digest,
                read_policy,
                state: RecordState::Pending {
                    version: 0,
                    highest_fencing: 0,
                },
            };
            validate_record(command_id, &record)?;
            state.records.insert(command_id.to_owned(), record);
            Ok(InsertOutcome::Created)
        })
    }

    pub(crate) fn claim(
        &self,
        command_id: &str,
        now_ms: i64,
        lease_duration_ms: i64,
    ) -> Result<ClaimOutcome, StoreError> {
        if lease_duration_ms <= 0 {
            return Err(StoreError::Invariant);
        }
        self.transaction(|state| {
            let Some(record) = state.records.get_mut(command_id) else {
                return Ok(ClaimOutcome::Missing);
            };
            if matches!(record.state, RecordState::Terminal { .. }) {
                return Ok(ClaimOutcome::AlreadyTerminal(Box::new(record.clone())));
            }
            if matches!(
                record.state,
                RecordState::Claimed {
                    lease_deadline_ms,
                    ..
                } if lease_deadline_ms >= now_ms
            ) {
                return Ok(ClaimOutcome::Busy);
            }
            let (version, highest_raw) = match &record.state {
                RecordState::Pending {
                    version,
                    highest_fencing,
                } => (*version, *highest_fencing),
                RecordState::Claimed {
                    version,
                    highest_fencing,
                    ..
                } => (*version, highest_fencing.get()),
                _ => return Err(StoreError::Invariant),
            };
            let fence = highest_raw
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .ok_or(StoreError::CounterExhausted)?;
            let next_version = version.checked_add(1).ok_or(StoreError::CounterExhausted)?;
            let deadline = now_ms
                .checked_add(lease_duration_ms)
                .ok_or(StoreError::CounterExhausted)?;
            record.state = RecordState::Claimed {
                version: next_version,
                highest_fencing: fence,
                fencing_token: fence,
                lease_deadline_ms: deadline,
            };
            Ok(ClaimOutcome::Claimed(ClaimToken {
                command_id: command_id.to_owned(),
                version: next_version,
                fencing_token: fence,
                lease_deadline_ms: deadline,
            }))
        })
    }

    #[allow(dead_code)]
    pub(crate) fn renew(
        &self,
        token: &ClaimToken,
        now_ms: i64,
        lease_duration_ms: i64,
    ) -> Result<RenewOutcome, StoreError> {
        if lease_duration_ms <= 0 {
            return Err(StoreError::Invariant);
        }
        self.transaction(|state| {
            let Some(record) = state.records.get_mut(&token.command_id) else {
                return Ok(RenewOutcome::Missing);
            };
            match &record.state {
                RecordState::Terminal { .. } => Ok(RenewOutcome::AlreadyTerminal),
                RecordState::Claimed {
                    version,
                    fencing_token,
                    lease_deadline_ms,
                    highest_fencing,
                } if *version == token.version && *fencing_token == token.fencing_token => {
                    if *lease_deadline_ms < now_ms {
                        return Ok(RenewOutcome::Expired);
                    }
                    let next_version =
                        version.checked_add(1).ok_or(StoreError::CounterExhausted)?;
                    let deadline = now_ms
                        .checked_add(lease_duration_ms)
                        .ok_or(StoreError::CounterExhausted)?;
                    let fence = *fencing_token;
                    record.state = RecordState::Claimed {
                        version: next_version,
                        highest_fencing: *highest_fencing,
                        fencing_token: fence,
                        lease_deadline_ms: deadline,
                    };
                    Ok(RenewOutcome::Renewed(ClaimToken {
                        command_id: token.command_id.clone(),
                        version: next_version,
                        fencing_token: fence,
                        lease_deadline_ms: deadline,
                    }))
                }
                _ => Ok(RenewOutcome::Stale),
            }
        })
    }

    pub(crate) fn abandon(&self, token: &ClaimToken) -> Result<AbandonOutcome, StoreError> {
        self.transaction(|state| {
            let Some(record) = state.records.get_mut(&token.command_id) else {
                return Ok(AbandonOutcome::Missing);
            };
            match &record.state {
                RecordState::Terminal { .. } => Ok(AbandonOutcome::AlreadyTerminal),
                RecordState::Claimed {
                    version,
                    fencing_token,
                    highest_fencing,
                    ..
                } if *version == token.version && *fencing_token == token.fencing_token => {
                    let next_version =
                        version.checked_add(1).ok_or(StoreError::CounterExhausted)?;
                    record.state = RecordState::Pending {
                        version: next_version,
                        highest_fencing: highest_fencing.get(),
                    };
                    Ok(AbandonOutcome::Abandoned)
                }
                _ => Ok(AbandonOutcome::Stale),
            }
        })
    }

    pub(crate) fn complete(
        &self,
        token: &ClaimToken,
        terminal: M71TerminalDto,
    ) -> Result<CompleteOutcome, StoreError> {
        let terminal_digest = terminal_digest(&terminal)?;
        self.transaction(|state| {
            let Some(record) = state.records.get_mut(&token.command_id) else {
                return Ok(CompleteOutcome::Missing);
            };
            match &record.state {
                RecordState::Terminal {
                    terminal: existing,
                    completion,
                    ..
                } => {
                    if existing.as_ref() == &terminal
                        && completion.terminal_digest == terminal_digest
                    {
                        Ok(CompleteOutcome::AlreadyTerminal(Box::new(record.clone())))
                    } else {
                        Ok(CompleteOutcome::LostToWinner(Box::new(record.clone())))
                    }
                }
                RecordState::Claimed {
                    version,
                    fencing_token,
                    highest_fencing,
                    ..
                } if *version == token.version && *fencing_token == token.fencing_token => {
                    let next_version =
                        version.checked_add(1).ok_or(StoreError::CounterExhausted)?;
                    let completion = CompletionReceipt {
                        fencing_token: *fencing_token,
                        terminal_digest,
                    };
                    record.state = RecordState::Terminal {
                        version: next_version,
                        highest_fencing: *highest_fencing,
                        terminal: Box::new(terminal),
                        completion,
                    };
                    Ok(CompleteOutcome::Completed(Box::new(record.clone())))
                }
                _ => Ok(CompleteOutcome::Stale),
            }
        })
    }

    pub(crate) fn get(&self, command_id: &str) -> Result<Option<StoredRecord>, StoreError> {
        let state = self.state.lock().map_err(|_| StoreError::Poisoned)?;
        Ok(state.records.get(command_id).cloned())
    }

    #[cfg(feature = "test-helpers")]
    pub fn test_get(&self, command_id: &str) -> Result<Option<StoredRecord>, StoreError> {
        self.get(command_id)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn transaction<T>(
        &self,
        operation: impl FnOnce(&mut StoreState) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        let mut state = self.state.lock().map_err(|_| StoreError::Poisoned)?;
        let before = state.clone();
        let result = match operation(&mut state) {
            Ok(value) => value,
            Err(error) => {
                *state = before;
                return Err(error);
            }
        };
        if let Err(error) = persist(&self.path, &state) {
            *state = before;
            return Err(error);
        }
        Ok(result)
    }
}

fn persist(path: &Path, state: &StoreState) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(StoreError::Io)?;
    }
    let bytes = serde_json::to_vec(state).map_err(StoreError::Corrupted)?;
    let temporary = path.with_extension("tmp");
    let mut file = File::create(&temporary).map_err(StoreError::Io)?;
    file.write_all(&bytes).map_err(StoreError::Io)?;
    file.sync_all().map_err(StoreError::Io)?;
    fs::rename(&temporary, path).map_err(StoreError::Io)
}

fn validate_state(state: &StoreState) -> Result<(), StoreError> {
    if state.schema_version != 1 {
        return Err(StoreError::Invariant);
    }
    for (command_id, record) in &state.records {
        validate_record(command_id, record)?;
    }
    Ok(())
}

fn validate_record(command_id: &str, record: &StoredRecord) -> Result<(), StoreError> {
    if record.capsule.command_id().as_str() != command_id
        || capsule_digest(&record.capsule)? != record.capsule_digest
        || !is_lowercase_hex_64(&record.capsule_digest)
    {
        return Err(StoreError::Invariant);
    }
    match (&record.capsule.admitted_actor(), &record.read_policy) {
        (AdmittedActorDto::Public, StoredReadPolicy::Public { authorization }) => {
            if !is_lowercase_hex_64(authorization.digest_hex()) || authorization.key_version() == 0
            {
                return Err(StoreError::Invariant);
            }
        }
        (
            AdmittedActorDto::Authenticated {
                tenant_id, user_id, ..
            },
            StoredReadPolicy::Authenticated {
                tenant_id: stored_tenant,
                user_id: stored_user,
            },
        ) if tenant_id.as_str() == stored_tenant && user_id.as_str() == stored_user => {}
        _ => return Err(StoreError::Invariant),
    }
    match &record.state {
        RecordState::Pending {
            version,
            highest_fencing,
        } => {
            let is_initial = *version == 0 && *highest_fencing == 0;
            let is_post_abandon = *highest_fencing > 0 && *version >= *highest_fencing;
            if !is_initial && !is_post_abandon {
                return Err(StoreError::Invariant);
            }
        }
        RecordState::Claimed {
            version,
            highest_fencing,
            fencing_token,
            ..
        } => {
            if highest_fencing != fencing_token || *version < highest_fencing.get() {
                return Err(StoreError::Invariant);
            }
        }
        RecordState::Terminal {
            version,
            highest_fencing,
            completion,
            terminal,
        } => {
            if highest_fencing != &completion.fencing_token
                || *version < highest_fencing.get()
                || terminal_digest(terminal)? != completion.terminal_digest
                || !is_lowercase_hex_64(&completion.terminal_digest)
            {
                return Err(StoreError::Invariant);
            }
        }
    }
    Ok(())
}

fn is_lowercase_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

pub fn capsule_digest(capsule: &DispatchCapsuleBodyV2) -> Result<String, StoreError> {
    let bytes = serde_json::to_vec(capsule).map_err(StoreError::Corrupted)?;
    Ok(sha256_hex(&bytes))
}

pub fn terminal_digest(terminal: &M71TerminalDto) -> Result<String, StoreError> {
    let bytes = serde_json::to_vec(terminal).map_err(StoreError::Corrupted)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Corrupted(serde_json::Error),
    Invariant,
    CounterExhausted,
    Poisoned,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Io(_) => "record store I/O failure",
            Self::Corrupted(_) => "record store encoding corruption",
            Self::Invariant => "record store invariant failure",
            Self::CounterExhausted => "record store counter exhausted",
            Self::Poisoned => "record store synchronization failure",
        })
    }
}
impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityIssuer;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use ustc_campus_agent_client_protocol::{
        AdmittedActorDto, AffairsGetPayloadDto, FrozenPrerequisitesDto, M71LineageDto,
        M71OutcomeDto, UnixMillis, WireText,
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path() -> PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir()
            .join(format!("m10-persist-test-{}-{}", std::process::id(), id))
            .join("store.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create temp dir");
        }
        path
    }

    fn wire(value: &str) -> WireText {
        WireText::parse(value).expect("fixture wire text")
    }

    fn public_capsule(command_id: &str) -> DispatchCapsuleBodyV2 {
        DispatchCapsuleBodyV2::try_new(
            wire(command_id),
            wire("corr:fixture"),
            AdmittedActorDto::Public,
            AffairsGetPayloadDto {
                procedure_id: wire("proc:fixture"),
                as_of: None,
            },
            wire(
                "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            wire("0000000000000000000000000000000000000000000000000000000000000000"),
            1,
            FrozenPrerequisitesDto {
                policy_snapshot_id: wire("policy:fixture:v1"),
                observed_at: UnixMillis::new(1_700_000_000_000),
                session_id: None,
                admitted_operation_id: wire("affairs.get"),
            },
        )
        .expect("fixture capsule")
    }

    fn authenticated_capsule(command_id: &str) -> DispatchCapsuleBodyV2 {
        DispatchCapsuleBodyV2::try_new(
            wire(command_id),
            wire("corr:fixture"),
            AdmittedActorDto::Authenticated {
                tenant_id: wire("tenant:fixture"),
                user_id: wire("user:fixture"),
                session_id: wire("session:fixture"),
            },
            AffairsGetPayloadDto {
                procedure_id: wire("proc:fixture"),
                as_of: None,
            },
            wire(
                "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            wire("0000000000000000000000000000000000000000000000000000000000000000"),
            1,
            FrozenPrerequisitesDto {
                policy_snapshot_id: wire("policy:fixture:v1"),
                observed_at: UnixMillis::new(1_700_000_000_000),
                session_id: Some(wire("session:fixture")),
                admitted_operation_id: wire("affairs.get"),
            },
        )
        .expect("fixture capsule")
    }

    fn not_found_terminal() -> M71TerminalDto {
        M71TerminalDto::try_new(
            M71OutcomeDto::NotFound {
                procedure_id: wire("proc:missing"),
            },
            M71LineageDto::NotRequired {
                materialization_receipt_id: wire("receipt:002"),
                reason: wire("no_visible_artifact"),
            },
        )
        .expect("fixture terminal")
    }

    fn found_terminal() -> M71TerminalDto {
        M71TerminalDto::try_new(
            M71OutcomeDto::NotFound {
                procedure_id: wire("proc:found"),
            },
            M71LineageDto::NotRequired {
                materialization_receipt_id: wire("receipt:003"),
                reason: wire("no_visible_artifact"),
            },
        )
        .expect("fixture terminal")
    }

    fn public_policy() -> StoredReadPolicy {
        let mut keys = BTreeMap::new();
        keys.insert(1u16, [0u8; 32]);
        let issuer = CapabilityIssuer::new(keys, 1).expect("fixture issuer");
        let (_bearer, authorization) = issuer.mint("cmd:fixture", "0000").expect("fixture mint");
        StoredReadPolicy::Public { authorization }
    }

    fn authenticated_policy() -> StoredReadPolicy {
        StoredReadPolicy::Authenticated {
            tenant_id: "tenant:fixture".to_owned(),
            user_id: "user:fixture".to_owned(),
        }
    }

    #[test]
    fn insert_created_then_existing_same_capsule() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        let outcome = store
            .insert_admitted_once("cmd:001", capsule.clone(), public_policy())
            .expect("insert");
        assert!(matches!(outcome, InsertOutcome::Created));
        let outcome2 = store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert2");
        match outcome2 {
            InsertOutcome::Existing(record) => {
                assert_eq!(record.capsule.command_id().as_str(), "cmd:001");
            }
            _ => panic!("expected Existing"),
        }
    }

    #[test]
    fn insert_invariant_corruption_on_different_capsule_same_command_id() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule1 = public_capsule("cmd:001");
        let capsule2 = DispatchCapsuleBodyV2::try_new(
            wire("cmd:001"),
            wire("corr:DIFFERENT"),
            AdmittedActorDto::Public,
            AffairsGetPayloadDto {
                procedure_id: wire("proc:fixture"),
                as_of: None,
            },
            wire(
                "descriptor:v0:1:0000000000000000000000000000000000000000000000000000000000000000",
            ),
            wire("0000000000000000000000000000000000000000000000000000000000000000"),
            1,
            FrozenPrerequisitesDto {
                policy_snapshot_id: wire("policy:fixture:v1"),
                observed_at: UnixMillis::new(1_700_000_000_000),
                session_id: None,
                admitted_operation_id: wire("affairs.get"),
            },
        )
        .expect("fixture capsule2");
        store
            .insert_admitted_once("cmd:001", capsule1, public_policy())
            .expect("insert1");
        let outcome = store
            .insert_admitted_once("cmd:001", capsule2, public_policy())
            .expect("insert2");
        assert!(matches!(outcome, InsertOutcome::InvariantCorruption));
    }

    #[test]
    fn claim_pending_succeeds() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let outcome = store.claim("cmd:001", 1_000_000, 30_000).expect("claim");
        match outcome {
            ClaimOutcome::Claimed(token) => {
                assert_eq!(token.command_id, "cmd:001");
                assert_eq!(token.version, 1);
                assert_eq!(token.fencing_token.get(), 1);
                assert_eq!(token.lease_deadline_ms, 1_030_000);
            }
            _ => panic!("expected Claimed"),
        }
    }

    #[test]
    fn claim_busy_when_lease_active() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        store.claim("cmd:001", 1_000_000, 30_000).expect("claim1");
        let outcome = store.claim("cmd:001", 1_010_000, 30_000).expect("claim2");
        assert!(matches!(outcome, ClaimOutcome::Busy));
    }

    #[test]
    fn claim_after_lease_expiry_reclaims() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token1 = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim1") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let outcome = store.claim("cmd:001", 1_040_000, 30_000).expect("claim2");
        match outcome {
            ClaimOutcome::Claimed(token2) => {
                assert_eq!(token2.fencing_token.get(), 2);
                assert_eq!(token2.version, 2);
                assert!(token2.fencing_token > token1.fencing_token);
            }
            _ => panic!("expected Claimed after expiry"),
        }
    }

    #[test]
    fn claim_missing_returns_missing() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let outcome = store
            .claim("nonexistent", 1_000_000, 30_000)
            .expect("claim");
        assert!(matches!(outcome, ClaimOutcome::Missing));
    }

    #[test]
    fn claim_terminal_returns_already_terminal() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        store
            .complete(&token, not_found_terminal())
            .expect("complete");
        let outcome = store.claim("cmd:001", 1_100_000, 30_000).expect("claim2");
        assert!(matches!(outcome, ClaimOutcome::AlreadyTerminal(_)));
    }

    #[test]
    fn renew_extends_lease() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let outcome = store.renew(&token, 1_010_000, 60_000).expect("renew");
        match outcome {
            RenewOutcome::Renewed(new_token) => {
                assert_eq!(new_token.version, token.version + 1);
                assert_eq!(new_token.lease_deadline_ms, 1_070_000);
                assert_eq!(new_token.fencing_token, token.fencing_token);
            }
            _ => panic!("expected Renewed"),
        }
    }

    #[test]
    fn renew_stale_after_reclaim_by_other() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token1 = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim1") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        store.claim("cmd:001", 1_040_000, 30_000).expect("claim2");
        let outcome = store.renew(&token1, 1_050_000, 30_000).expect("renew");
        assert!(matches!(outcome, RenewOutcome::Stale));
    }

    #[test]
    fn renew_expired_after_lease_deadline() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let outcome = store.renew(&token, 1_040_000, 30_000).expect("renew");
        assert!(matches!(outcome, RenewOutcome::Expired));
    }

    // R5: renew rejects nonpositive duration and leaves state bytes unchanged
    #[test]
    fn renew_rejects_zero_duration() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let result = store.renew(&token, 1_010_000, 0);
        assert!(result.is_err());
    }

    #[test]
    fn renew_rejects_negative_duration() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let result = store.renew(&token, 1_010_000, -1);
        assert!(result.is_err());
    }

    #[test]
    fn renew_nonpositive_duration_leaves_state_unchanged() {
        let path = temp_path();
        let store = FileRecordStore::open(path.clone()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let before = std::fs::read(&path).expect("read before");
        let _ = store.renew(&token, 1_010_000, 0);
        let _ = store.renew(&token, 1_010_000, -5);
        let after = std::fs::read(&path).expect("read after");
        assert_eq!(
            before, after,
            "state bytes must be unchanged on rejected renew"
        );
    }

    #[test]
    fn abandon_releases_claim_back_to_pending() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let outcome = store.abandon(&token).expect("abandon");
        assert!(matches!(outcome, AbandonOutcome::Abandoned));
        let outcome2 = store.claim("cmd:001", 1_010_000, 30_000).expect("claim2");
        assert!(matches!(outcome2, ClaimOutcome::Claimed(_)));
    }

    #[test]
    fn abandon_stale_returns_stale() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token1 = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim1") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        store.claim("cmd:001", 1_040_000, 30_000).expect("claim2");
        let outcome = store.abandon(&token1).expect("abandon");
        assert!(matches!(outcome, AbandonOutcome::Stale));
    }

    #[test]
    fn complete_writes_terminal() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let terminal = not_found_terminal();
        let outcome = store.complete(&token, terminal.clone()).expect("complete");
        match outcome {
            CompleteOutcome::Completed(record) => match &record.state {
                RecordState::Terminal {
                    terminal: stored,
                    completion,
                    ..
                } => {
                    assert_eq!(stored.as_ref(), &terminal);
                    assert_eq!(completion.fencing_token, token.fencing_token);
                }
                _ => panic!("expected Terminal"),
            },
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn complete_same_terminal_after_terminal_returns_already_terminal() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let terminal = not_found_terminal();
        store.complete(&token, terminal.clone()).expect("complete1");
        let outcome = store.complete(&token, terminal).expect("complete2");
        assert!(matches!(outcome, CompleteOutcome::AlreadyTerminal(_)));
    }

    #[test]
    fn complete_different_terminal_after_terminal_returns_lost_to_winner() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        store
            .complete(&token, not_found_terminal())
            .expect("complete1");
        let outcome = store.complete(&token, found_terminal()).expect("complete2");
        assert!(matches!(outcome, CompleteOutcome::LostToWinner(_)));
    }

    #[test]
    fn complete_stale_token_returns_stale() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let token1 = match store.claim("cmd:001", 1_000_000, 30_000).expect("claim1") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let _token2 = match store.claim("cmd:001", 1_040_000, 30_000).expect("claim2") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!("expected Claimed"),
        };
        let outcome = store
            .complete(&token1, not_found_terminal())
            .expect("complete");
        assert!(matches!(outcome, CompleteOutcome::Stale));
    }

    #[test]
    fn authenticated_policy_rejects_public_actor() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let public_cap = public_capsule("cmd:001");
        let result = store.insert_admitted_once("cmd:001", public_cap, authenticated_policy());
        assert!(result.is_err());
    }

    #[test]
    fn public_policy_rejects_authenticated_actor() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let auth_cap = authenticated_capsule("cmd:001");
        let result = store.insert_admitted_once("cmd:001", auth_cap, public_policy());
        assert!(result.is_err());
    }

    #[test]
    fn claim_rejects_nonpositive_lease() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        assert!(store.claim("cmd:001", 1_000_000, 0).is_err());
        assert!(store.claim("cmd:001", 1_000_000, -1).is_err());
    }

    #[test]
    fn fencing_token_monotonically_increases_across_claims() {
        let store = FileRecordStore::open(temp_path()).expect("open");
        let capsule = public_capsule("cmd:001");
        store
            .insert_admitted_once("cmd:001", capsule, public_policy())
            .expect("insert");
        let t1 = match store.claim("cmd:001", 1_000_000, 10_000).expect("claim1") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!(),
        };
        store.abandon(&t1).expect("abandon1");
        let t2 = match store.claim("cmd:001", 1_011_000, 10_000).expect("claim2") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!(),
        };
        store.abandon(&t2).expect("abandon2");
        let t3 = match store.claim("cmd:001", 1_022_000, 10_000).expect("claim3") {
            ClaimOutcome::Claimed(t) => t,
            _ => panic!(),
        };
        assert!(t2.fencing_token > t1.fencing_token);
        assert!(t3.fencing_token > t2.fencing_token);
    }

    #[test]
    fn durable_carrier_debug_redacts_secret_and_fencing_material() {
        let policy = public_policy();
        let authorization_digest = match &policy {
            StoredReadPolicy::Public { authorization } => authorization.digest_hex().to_owned(),
            StoredReadPolicy::Authenticated { .. } => panic!("expected public policy"),
        };
        let capsule = public_capsule("capsule-secret-command");
        let record = StoredRecord {
            capsule,
            capsule_digest: "capsule-secret-digest".to_owned(),
            read_policy: policy.clone(),
            state: RecordState::Pending {
                version: 0,
                highest_fencing: 0,
            },
        };
        let receipt = CompletionReceipt {
            fencing_token: NonZeroU64::new(424_242).expect("nonzero"),
            terminal_digest: "completion-secret-digest".to_owned(),
        };
        let claimed = RecordState::Claimed {
            version: 17,
            highest_fencing: NonZeroU64::new(13).expect("nonzero"),
            fencing_token: NonZeroU64::new(13).expect("nonzero"),
            lease_deadline_ms: 9_999_999,
        };

        for (name, debug, forbidden) in [
            (
                "policy",
                format!("{policy:?}"),
                authorization_digest.as_str(),
            ),
            ("record", format!("{record:?}"), "capsule-secret-command"),
            (
                "record-digest",
                format!("{record:?}"),
                "capsule-secret-digest",
            ),
            (
                "receipt",
                format!("{receipt:?}"),
                "completion-secret-digest",
            ),
            ("receipt-fence", format!("{receipt:?}"), "424242"),
            ("claimed-deadline", format!("{claimed:?}"), "9999999"),
        ] {
            assert!(
                !debug.contains(forbidden),
                "{name} leaked forbidden material: {debug}"
            );
            assert!(
                debug.contains("REDACTED"),
                "{name} did not expose an explicit redaction marker: {debug}"
            );
        }
    }

    // S1: transaction restores cloned pre-state both when the operation closure returns Err
    // and when persistence fails. This test proves in-memory and on-disk bytes remain
    // unchanged after a closure mutates state then returns Err.
    #[test]
    fn s1_transaction_rollback_on_operation_err_restores_state() {
        let path = temp_path();
        let store = FileRecordStore::open(path.clone()).expect("open");
        let capsule = public_capsule("cmd:rollback");
        store
            .insert_admitted_once("cmd:rollback", capsule, public_policy())
            .expect("insert");

        let disk_before = std::fs::read(&path).expect("read disk before");
        let memory_before = store
            .get("cmd:rollback")
            .expect("get before")
            .expect("record exists");

        let result = store.transaction(|state| {
            state.records.insert(
                "cmd:transient".to_owned(),
                StoredRecord {
                    capsule: public_capsule("cmd:transient"),
                    capsule_digest: "0".repeat(64),
                    read_policy: public_policy(),
                    state: RecordState::Pending {
                        version: 0,
                        highest_fencing: 0,
                    },
                },
            );
            Err::<(), StoreError>(StoreError::Invariant)
        });
        assert!(result.is_err(), "transaction must propagate the error");

        assert!(
            store.get("cmd:transient").expect("get transient").is_none(),
            "transient record must not exist in memory after rollback"
        );
        let memory_after = store
            .get("cmd:rollback")
            .expect("get after")
            .expect("record exists");
        assert_eq!(
            memory_before, memory_after,
            "in-memory record must be unchanged after rollback"
        );

        let disk_after = std::fs::read(&path).expect("read disk after");
        assert_eq!(
            disk_before, disk_after,
            "disk bytes must be unchanged after rollback"
        );
    }

    #[test]
    fn s2_concurrent_claims_yield_one_claimed_one_busy() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let store = Arc::new(FileRecordStore::open(temp_path()).expect("open"));
        let capsule = public_capsule("cmd:s2-claim");
        store
            .insert_admitted_once("cmd:s2-claim", capsule, public_policy())
            .expect("insert");

        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                barrier.wait();
                store
                    .claim("cmd:s2-claim", 1_000_000, 30_000)
                    .expect("claim")
            }));
        }
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("thread"))
            .collect();
        let claimed = results
            .iter()
            .filter(|r| matches!(r, ClaimOutcome::Claimed(_)))
            .count();
        let busy = results
            .iter()
            .filter(|r| matches!(r, ClaimOutcome::Busy))
            .count();
        assert_eq!(claimed, 1, "exactly one Claimed");
        assert_eq!(busy, 1, "exactly one Busy");
    }

    #[test]
    fn s2_concurrent_completions_yield_one_completed_one_already_terminal() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let store = Arc::new(FileRecordStore::open(temp_path()).expect("open"));
        let capsule = public_capsule("cmd:s2-complete");
        store
            .insert_admitted_once("cmd:s2-complete", capsule, public_policy())
            .expect("insert");
        let token = match store
            .claim("cmd:s2-complete", 1_000_000, 30_000)
            .expect("claim")
        {
            ClaimOutcome::Claimed(token) => token,
            _ => panic!("expected Claimed"),
        };
        let terminal = not_found_terminal();
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            let token = token.clone();
            let terminal = terminal.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                store.complete(&token, terminal).expect("complete")
            }));
        }
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("thread"))
            .collect();
        let completed = results
            .iter()
            .filter(|r| matches!(r, CompleteOutcome::Completed(_)))
            .count();
        let already_terminal = results
            .iter()
            .filter(|r| matches!(r, CompleteOutcome::AlreadyTerminal(_)))
            .count();
        assert_eq!(completed, 1, "exactly one Completed");
        assert_eq!(already_terminal, 1, "exactly one AlreadyTerminal");
    }
}
