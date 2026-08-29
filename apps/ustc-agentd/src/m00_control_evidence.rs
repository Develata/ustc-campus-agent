use std::collections::BTreeSet;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use ustc_campus_agent_core::control_evidence::{
    ControlEvidenceAppendOutcome, ControlEvidenceAppendPort, ControlEvidenceJournalError,
    ControlEvidenceKey, ControlEvidenceReadPort, PlatformControlEvent,
};

const DEFAULT_MAX_EVENTS: usize = 4096;
const DEFAULT_MAX_BYTES: u64 = 16 * 1024 * 1024;
const TEMP_ATTEMPTS: usize = 16;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlEvidenceState {
    schema_version: u8,
    events: Vec<PlatformControlEvent>,
}

impl Default for ControlEvidenceState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            events: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct DurableControlEvidenceJournal {
    path: PathBuf,
    state: Arc<Mutex<ControlEvidenceState>>,
    max_events: usize,
    max_bytes: u64,
}

impl DurableControlEvidenceJournal {
    pub(crate) fn open(path: &Path) -> Result<Self, ControlEvidenceJournalError> {
        Self::open_with_limits(path, DEFAULT_MAX_EVENTS, DEFAULT_MAX_BYTES)
    }

    fn open_with_limits(
        path: &Path,
        max_events: usize,
        max_bytes: u64,
    ) -> Result<Self, ControlEvidenceJournalError> {
        if max_events == 0 || max_bytes == 0 {
            return Err(ControlEvidenceJournalError::InternalInvariant);
        }
        validate_secure_parent(path)?;
        let state = match read_existing(path, max_bytes)? {
            None => ControlEvidenceState::default(),
            Some(bytes) => {
                let state: ControlEvidenceState = serde_json::from_slice(&bytes)
                    .map_err(|_| ControlEvidenceJournalError::Corrupt)?;
                validate_state(&state, max_events)?;
                let canonical = serde_json::to_vec(&state)
                    .map_err(|_| ControlEvidenceJournalError::InternalInvariant)?;
                if canonical != bytes {
                    return Err(ControlEvidenceJournalError::Corrupt);
                }
                state
            }
        };
        Ok(Self {
            path: path.to_owned(),
            state: Arc::new(Mutex::new(state)),
            max_events,
            max_bytes,
        })
    }

    fn persist(&self, state: &ControlEvidenceState) -> Result<(), ControlEvidenceJournalError> {
        validate_state(state, self.max_events)?;
        let bytes = serde_json::to_vec(state)
            .map_err(|_| ControlEvidenceJournalError::InternalInvariant)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > self.max_bytes {
            return Err(ControlEvidenceJournalError::LimitExceeded);
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
                Err(_) => return Err(ControlEvidenceJournalError::Unavailable),
            }
        }
        let temporary = temporary.ok_or(ControlEvidenceJournalError::Unavailable)?;
        let mut file = file.ok_or(ControlEvidenceJournalError::InternalInvariant)?;
        let result = (|| {
            let metadata = file
                .metadata()
                .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
            if !metadata.file_type().is_file()
                || metadata.uid() != current_uid()?
                || metadata.permissions().mode() & 0o777 != 0o600
                || metadata.nlink() != 1
            {
                return Err(ControlEvidenceJournalError::Corrupt);
            }
            file.write_all(&bytes)
                .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
            file.sync_all()
                .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
            drop(file);
            fs::rename(&temporary, &self.path)
                .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
            File::open(parent)
                .and_then(|directory| directory.sync_all())
                .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    fn verify_durable_matches(
        &self,
        state: &ControlEvidenceState,
    ) -> Result<(), ControlEvidenceJournalError> {
        let expected = serde_json::to_vec(state)
            .map_err(|_| ControlEvidenceJournalError::InternalInvariant)?;
        match read_existing(&self.path, self.max_bytes)? {
            None if state.events.is_empty() => Ok(()),
            Some(actual) if actual == expected => Ok(()),
            _ => Err(ControlEvidenceJournalError::Corrupt),
        }
    }

    pub(crate) fn event_count(&self) -> usize {
        self.state.lock().map_or(0, |state| state.events.len())
    }
}

impl ControlEvidenceReadPort for DurableControlEvidenceJournal {
    fn load_control_event(
        &mut self,
        key: &ControlEvidenceKey,
    ) -> Result<Option<PlatformControlEvent>, ControlEvidenceJournalError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ControlEvidenceJournalError::InternalInvariant)?;
        self.verify_durable_matches(&state)?;
        Ok(state
            .events
            .binary_search_by_key(key, PlatformControlEvent::key)
            .ok()
            .map(|index| state.events[index].clone()))
    }
}

impl ControlEvidenceAppendPort for DurableControlEvidenceJournal {
    fn append_once(
        &mut self,
        event: &PlatformControlEvent,
    ) -> Result<ControlEvidenceAppendOutcome, ControlEvidenceJournalError> {
        let key = event.key();
        let mut state = self
            .state
            .lock()
            .map_err(|_| ControlEvidenceJournalError::InternalInvariant)?;
        self.verify_durable_matches(&state)?;
        match state
            .events
            .binary_search_by_key(&key, PlatformControlEvent::key)
        {
            Ok(index) => {
                return Ok(if state.events[index] == *event {
                    ControlEvidenceAppendOutcome::AlreadySame
                } else {
                    ControlEvidenceAppendOutcome::Conflict
                });
            }
            Err(_) if state.events.len() >= self.max_events => {
                return Err(ControlEvidenceJournalError::LimitExceeded);
            }
            Err(_) => {}
        }

        let mut next = state.clone();
        next.events.push(event.clone());
        next.events.sort_by_key(PlatformControlEvent::key);
        self.persist(&next)?;
        *state = next;
        Ok(ControlEvidenceAppendOutcome::Appended)
    }
}

pub(crate) fn ensure_secure_state_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .ok_or_else(|| "state path requires a direct parent".to_owned())?;
    match fs::symlink_metadata(parent) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut builder = DirBuilder::new();
            builder.mode(0o700);
            builder
                .create(parent)
                .map_err(|error| format!("state parent create failed: {error}"))?;
        }
        Err(error) => return Err(format!("state parent metadata failed: {error}")),
    }
    validate_secure_parent(path).map_err(|error| format!("unsafe state parent: {error:?}"))
}

fn validate_state(
    state: &ControlEvidenceState,
    max_events: usize,
) -> Result<(), ControlEvidenceJournalError> {
    if state.schema_version != 1 {
        return Err(ControlEvidenceJournalError::Corrupt);
    }
    if state.events.len() > max_events {
        return Err(ControlEvidenceJournalError::LimitExceeded);
    }
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for event in &state.events {
        let key = event.key();
        if !seen.insert(key.clone()) || previous.as_ref().is_some_and(|value| value >= &key) {
            return Err(ControlEvidenceJournalError::Corrupt);
        }
        previous = Some(key);
    }
    Ok(())
}

fn read_existing(
    path: &Path,
    max_bytes: u64,
) -> Result<Option<Vec<u8>>, ControlEvidenceJournalError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(ControlEvidenceJournalError::Unavailable),
    };
    validate_primary_metadata(&metadata, max_bytes)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
    let opened = file
        .metadata()
        .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
    validate_primary_metadata(&opened, max_bytes)?;
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err(ControlEvidenceJournalError::Corrupt);
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(ControlEvidenceJournalError::LimitExceeded);
    }
    Ok(Some(bytes))
}

fn validate_existing_destination(
    path: &Path,
    max_bytes: u64,
) -> Result<(), ControlEvidenceJournalError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_primary_metadata(&metadata, max_bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(ControlEvidenceJournalError::Unavailable),
    }
}

fn validate_primary_metadata(
    metadata: &fs::Metadata,
    max_bytes: u64,
) -> Result<(), ControlEvidenceJournalError> {
    if !metadata.file_type().is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.nlink() != 1
        || metadata.uid() != current_uid()?
    {
        return Err(ControlEvidenceJournalError::Corrupt);
    }
    if metadata.len() > max_bytes {
        return Err(ControlEvidenceJournalError::LimitExceeded);
    }
    Ok(())
}

fn validate_secure_parent(path: &Path) -> Result<(), ControlEvidenceJournalError> {
    let parent = direct_parent(path)?;
    let metadata =
        fs::symlink_metadata(parent).map_err(|_| ControlEvidenceJournalError::Unavailable)?;
    if !metadata.file_type().is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
        || metadata.uid() != current_uid()?
    {
        return Err(ControlEvidenceJournalError::Corrupt);
    }
    Ok(())
}

fn direct_parent(path: &Path) -> Result<&Path, ControlEvidenceJournalError> {
    path.parent()
        .filter(|value| !value.as_os_str().is_empty())
        .ok_or(ControlEvidenceJournalError::Corrupt)
}

fn current_uid() -> Result<u32, ControlEvidenceJournalError> {
    fs::metadata("/proc/self")
        .map(|metadata| metadata.uid())
        .map_err(|_| ControlEvidenceJournalError::Unavailable)
}

fn unpredictable_temporary(
    parent: &Path,
    destination: &Path,
) -> Result<PathBuf, ControlEvidenceJournalError> {
    let mut random = [0_u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut random))
        .map_err(|_| ControlEvidenceJournalError::Unavailable)?;
    let nonce: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(ControlEvidenceJournalError::Corrupt)?;
    Ok(parent.join(format!(".{name}.{nonce}.tmp")))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};
    use ustc_campus_agent_core::identity::{CommandId, CorrelationId, RequestId};
    use ustc_campus_agent_core::request_context::{
        CausationId, DescriptorSnapshotId, EffectClass, OperationId, PermissionClass,
        PlatformPolicySnapshotId, SchemaDigest,
    };
    use ustc_campus_agent_core::session::SessionInstant;

    static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn directory(label: &str) -> PathBuf {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "m00-control-evidence-{}-{sequence}-{label}",
            std::process::id()
        ));
        let mut builder = DirBuilder::new();
        builder.mode(0o700).create(&path).unwrap();
        path
    }

    fn request_event_with_correlation(command: &str, correlation: &str) -> PlatformControlEvent {
        let digest = SchemaDigest::parse("a".repeat(64)).unwrap();
        PlatformControlEvent::RequestAdmitted {
            request_id: RequestId::parse(format!("request:{command}")).unwrap(),
            command_id: CommandId::parse(command).unwrap(),
            correlation_id: CorrelationId::parse(correlation).unwrap(),
            causation_id: None::<CausationId>,
            actor: ustc_campus_agent_core::control_evidence::PlatformControlActor::Public,
            operation_id: OperationId::parse("affairs.publish").unwrap(),
            descriptor_snapshot_id: DescriptorSnapshotId::from_canonical_identity(&digest, 1)
                .unwrap(),
            permission_class: PermissionClass::TenantPrivateWrite,
            effect_class: EffectClass::TenantLocalMutation,
            policy_snapshot_id: PlatformPolicySnapshotId::parse("policy:test").unwrap(),
            observed_at: SessionInstant::from_unix_millis(1),
        }
    }

    fn request_event(command: &str) -> PlatformControlEvent {
        request_event_with_correlation(command, &format!("correlation:{command}"))
    }

    #[test]
    fn append_reopen_and_exact_retry_are_stable() {
        let dir = directory("reopen");
        let path = dir.join("evidence.json");
        let event = request_event(&"a".repeat(64));
        let mut journal = DurableControlEvidenceJournal::open(&path).unwrap();
        assert_eq!(
            journal.append_once(&event).unwrap(),
            ControlEvidenceAppendOutcome::Appended
        );
        assert_eq!(
            journal.append_once(&event).unwrap(),
            ControlEvidenceAppendOutcome::AlreadySame
        );
        assert_eq!(journal.event_count(), 1);
        let mut reopened = DurableControlEvidenceJournal::open(&path).unwrap();
        assert_eq!(
            reopened.load_control_event(&event.key()).unwrap(),
            Some(event)
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn unsafe_primary_shapes_fail_closed() {
        let dir = directory("unsafe");
        let path = dir.join("evidence.json");
        let target = dir.join("target");
        fs::write(&target, b"{} ").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        symlink(&target, &path).unwrap();
        assert!(DurableControlEvidenceJournal::open(&path).is_err());
        fs::remove_file(&path).unwrap();
        fs::hard_link(&target, &path).unwrap();
        assert!(DurableControlEvidenceJournal::open(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn noncanonical_and_duplicate_state_fail_closed() {
        let dir = directory("corrupt");
        let path = dir.join("evidence.json");
        let event = request_event(&"b".repeat(64));
        let state = ControlEvidenceState {
            schema_version: 1,
            events: vec![event.clone(), event],
        };
        fs::write(&path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(DurableControlEvidenceJournal::open(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn same_key_conflict_never_overwrites_first_event() {
        let dir = directory("conflict");
        let path = dir.join("evidence.json");
        let command = "c".repeat(64);
        let first = request_event_with_correlation(&command, "correlation:first");
        let conflicting = request_event_with_correlation(&command, "correlation:second");
        let mut journal = DurableControlEvidenceJournal::open(&path).unwrap();
        assert_eq!(
            journal.append_once(&first).unwrap(),
            ControlEvidenceAppendOutcome::Appended
        );
        assert_eq!(
            journal.append_once(&conflicting).unwrap(),
            ControlEvidenceAppendOutcome::Conflict
        );
        assert_eq!(journal.event_count(), 1);
        assert_eq!(
            journal.load_control_event(&first.key()).unwrap(),
            Some(first)
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn runtime_replacement_and_persist_failure_leave_no_false_authority() {
        let dir = directory("runtime-drift");
        let path = dir.join("evidence.json");
        let first = request_event(&"d".repeat(64));
        let second = request_event(&"e".repeat(64));
        let mut journal = DurableControlEvidenceJournal::open(&path).unwrap();
        journal.append_once(&first).unwrap();

        fs::set_permissions(&dir, fs::Permissions::from_mode(0o500)).unwrap();
        assert!(journal.append_once(&second).is_err());
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            journal.load_control_event(&first.key()).unwrap(),
            Some(first)
        );
        assert_eq!(journal.load_control_event(&second.key()).unwrap(), None);

        let replacement = serde_json::to_vec(&ControlEvidenceState::default()).unwrap();
        fs::write(&path, replacement).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            journal.load_control_event(&second.key()),
            Err(ControlEvidenceJournalError::Corrupt)
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn event_and_byte_limits_fail_without_creating_durable_state() {
        let dir = directory("limits");
        let event_path = dir.join("event-limit.json");
        let mut event_limited =
            DurableControlEvidenceJournal::open_with_limits(&event_path, 1, 4096).unwrap();
        event_limited
            .append_once(&request_event(&"f".repeat(64)))
            .unwrap();
        assert_eq!(
            event_limited.append_once(&request_event(&"1".repeat(64))),
            Err(ControlEvidenceJournalError::LimitExceeded)
        );
        assert_eq!(event_limited.event_count(), 1);

        let byte_path = dir.join("byte-limit.json");
        let mut byte_limited =
            DurableControlEvidenceJournal::open_with_limits(&byte_path, 4, 1).unwrap();
        assert_eq!(
            byte_limited.append_once(&request_event(&"2".repeat(64))),
            Err(ControlEvidenceJournalError::LimitExceeded)
        );
        assert!(!byte_path.exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
