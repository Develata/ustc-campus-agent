use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ustc_campus_agent_core::identity::SessionId;
use ustc_campus_agent_core::session::SessionEvent;
use ustc_campus_agent_core::session_port::{
    SessionHistory, SessionHistoryReadPort, SessionRepositoryError,
};

const SCHEMA_VERSION: u64 = 1;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SESSIONS: usize = 64;
const MAX_EVENTS_PER_SESSION: usize = 1_024;

#[cfg(all(test, unix))]
type BootstrapControl = TestControl;
#[cfg(all(not(test), unix))]
type BootstrapControl = ();

#[derive(Clone)]
pub(crate) struct DurableCurrentSessionStore {
    histories: Arc<BTreeMap<SessionId, SessionHistory>>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionStoreFile {
    schema_version: u64,
    sessions: Vec<SessionStoreRecord>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionStoreRecord {
    session_id: SessionId,
    events: Vec<SessionEvent>,
}

impl DurableCurrentSessionStore {
    #[cfg(unix)]
    pub(crate) fn open_or_bootstrap(
        path: &Path,
        bootstrap_events: &[SessionEvent],
    ) -> Result<Self, SessionRepositoryError> {
        Self::open_or_bootstrap_inner(path, bootstrap_events, None)
    }

    #[cfg(not(unix))]
    pub(crate) fn open_or_bootstrap(
        _path: &Path,
        _bootstrap_events: &[SessionEvent],
    ) -> Result<Self, SessionRepositoryError> {
        Err(SessionRepositoryError::Unavailable)
    }

    #[cfg(unix)]
    fn open_or_bootstrap_inner(
        path: &Path,
        bootstrap_events: &[SessionEvent],
        control: Option<&BootstrapControl>,
    ) -> Result<Self, SessionRepositoryError> {
        validate_parent(path)?;
        match std::fs::symlink_metadata(path) {
            Ok(_) => read_existing(path, control),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                bootstrap(path, bootstrap_events, control)?;
                read_existing(path, control)
            }
            Err(_) => Err(SessionRepositoryError::Unavailable),
        }
    }
}

impl SessionHistoryReadPort for DurableCurrentSessionStore {
    fn load_history(
        &mut self,
        session_id: &SessionId,
    ) -> Result<Option<SessionHistory>, SessionRepositoryError> {
        Ok(self.histories.get(session_id).cloned())
    }
}

fn validate_state(
    state: SessionStoreFile,
    bytes: &[u8],
) -> Result<DurableCurrentSessionStore, SessionRepositoryError> {
    if state.schema_version != SCHEMA_VERSION
        || state.sessions.is_empty()
        || state.sessions.len() > MAX_SESSIONS
    {
        return Err(SessionRepositoryError::Corrupt);
    }
    let canonical = serde_json::to_vec(&state).map_err(|_| SessionRepositoryError::Corrupt)?;
    if canonical != bytes {
        return Err(SessionRepositoryError::Corrupt);
    }

    let mut histories = BTreeMap::new();
    let mut previous: Option<&SessionId> = None;
    for record in &state.sessions {
        if record.events.is_empty() || record.events.len() > MAX_EVENTS_PER_SESSION {
            return Err(SessionRepositoryError::Corrupt);
        }
        if previous.is_some_and(|value| value >= &record.session_id) {
            return Err(SessionRepositoryError::Corrupt);
        }
        let history = SessionHistory::try_from_events(record.events.clone())?;
        if history.session_id() != &record.session_id {
            return Err(SessionRepositoryError::Corrupt);
        }
        previous = Some(&record.session_id);
        if histories
            .insert(record.session_id.clone(), history)
            .is_some()
        {
            return Err(SessionRepositoryError::Corrupt);
        }
    }
    Ok(DurableCurrentSessionStore {
        histories: Arc::new(histories),
    })
}

#[cfg(unix)]
fn validate_parent(path: &Path) -> Result<(), SessionRepositoryError> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

    let parent = path.parent().ok_or(SessionRepositoryError::Unavailable)?;
    let metadata =
        std::fs::symlink_metadata(parent).map_err(|_| SessionRepositoryError::Unavailable)?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_dir()
        || metadata.file_type().is_file()
        || metadata.file_type().is_socket()
        || metadata.permissions().mode() & 0o7777 != 0o700
        || metadata.uid() != current_uid()?
    {
        return Err(SessionRepositoryError::Unavailable);
    }
    Ok(())
}

#[cfg(unix)]
fn current_uid() -> Result<u32, SessionRepositoryError> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata("/proc/self")
        .map(|metadata| metadata.uid())
        .map_err(|_| SessionRepositoryError::Unavailable)
}

#[cfg(unix)]
fn read_existing(
    path: &Path,
    control: Option<&BootstrapControl>,
) -> Result<DurableCurrentSessionStore, SessionRepositoryError> {
    use std::fs::OpenOptions;
    use std::io::Read;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    #[cfg(not(test))]
    let _ = control;

    let before =
        std::fs::symlink_metadata(path).map_err(|_| SessionRepositoryError::Unavailable)?;
    if before.file_type().is_symlink()
        || !before.file_type().is_file()
        || before.permissions().mode() & 0o7777 != 0o600
        || before.uid() != current_uid()?
        || before.nlink() != 1
        || before.len() == 0
        || before.len() > MAX_FILE_BYTES
    {
        return Err(SessionRepositoryError::Unavailable);
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| SessionRepositoryError::Unavailable)?;

    #[cfg(test)]
    if let Some(control) = control
        && let Some(replacement) = &control.swap_after_open
    {
        std::fs::rename(replacement, path).map_err(|_| SessionRepositoryError::Unavailable)?;
    }

    let descriptor = file
        .metadata()
        .map_err(|_| SessionRepositoryError::Unavailable)?;
    let after = std::fs::symlink_metadata(path).map_err(|_| SessionRepositoryError::Unavailable)?;
    if before.dev() != descriptor.dev()
        || before.ino() != descriptor.ino()
        || after.dev() != descriptor.dev()
        || after.ino() != descriptor.ino()
        || descriptor.permissions().mode() & 0o7777 != 0o600
        || descriptor.uid() != current_uid()?
        || descriptor.nlink() != 1
    {
        return Err(SessionRepositoryError::Unavailable);
    }
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| SessionRepositoryError::Unavailable)?;
    if bytes.is_empty() || u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(SessionRepositoryError::Corrupt);
    }
    let state: SessionStoreFile =
        serde_json::from_slice(&bytes).map_err(|_| SessionRepositoryError::Corrupt)?;
    validate_state(state, &bytes)
}

#[cfg(unix)]
fn bootstrap(
    path: &Path,
    bootstrap_events: &[SessionEvent],
    control: Option<&BootstrapControl>,
) -> Result<(), SessionRepositoryError> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[cfg(not(test))]
    let _ = control;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    if bootstrap_events.is_empty() || bootstrap_events.len() > MAX_EVENTS_PER_SESSION {
        return Err(SessionRepositoryError::InvalidEvent);
    }
    let history = SessionHistory::try_from_events(bootstrap_events.to_vec())
        .map_err(|_| SessionRepositoryError::InvalidEvent)?;
    let state = SessionStoreFile {
        schema_version: SCHEMA_VERSION,
        sessions: vec![SessionStoreRecord {
            session_id: history.session_id().clone(),
            events: bootstrap_events.to_vec(),
        }],
    };
    let bytes =
        serde_json::to_vec(&state).map_err(|_| SessionRepositoryError::InternalInvariant)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(SessionRepositoryError::LimitExceeded);
    }

    let parent = path.parent().ok_or(SessionRepositoryError::Unavailable)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(SessionRepositoryError::Unavailable)?;
    let mut temporary = None;
    let mut temporary_file = None;
    for _ in 0..128 {
        let candidate = parent.join(format!(
            ".{file_name}.tmp-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&candidate)
        {
            Ok(file) => {
                temporary = Some(candidate);
                temporary_file = Some(file);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return Err(SessionRepositoryError::Unavailable),
        }
    }
    let temporary = temporary.ok_or(SessionRepositoryError::Unavailable)?;
    let mut file = temporary_file.ok_or(SessionRepositoryError::InternalInvariant)?;

    #[cfg(test)]
    if fault(control) == Some(BootstrapFault::AfterTempCreate) {
        return fail_before_publish(&temporary, false);
    }

    if file.write_all(&bytes).is_err() {
        drop(file);
        return cleanup_before_publish(&temporary);
    }
    #[cfg(test)]
    if fault(control) == Some(BootstrapFault::AfterWrite) {
        return fail_before_publish(&temporary, false);
    }
    if file.sync_all().is_err() {
        drop(file);
        return cleanup_before_publish(&temporary);
    }
    #[cfg(test)]
    if fault(control) == Some(BootstrapFault::AfterFileSync) {
        return fail_before_publish(&temporary, false);
    }
    drop(file);

    #[cfg(test)]
    if let Some(control) = control {
        if let Some(barrier) = &control.before_publish {
            barrier.wait();
        }
        if fault(Some(control)) == Some(BootstrapFault::BeforePublish) {
            return fail_before_publish(&temporary, false);
        }
        if fault(Some(control)) == Some(BootstrapFault::CleanupRemoveFailure) {
            return fail_before_publish(&temporary, true);
        }
    }

    if std::fs::hard_link(&temporary, path).is_err() {
        let _ = std::fs::remove_file(&temporary);
        return Err(SessionRepositoryError::Unavailable);
    }

    #[cfg(test)]
    if fault(control) == Some(BootstrapFault::AfterPublishBeforeTempUnlink) {
        let _ = std::fs::remove_file(&temporary);
        return Err(SessionRepositoryError::Unavailable);
    }

    std::fs::remove_file(&temporary).map_err(|_| SessionRepositoryError::Unavailable)?;
    #[cfg(test)]
    if fault(control) == Some(BootstrapFault::AfterTempUnlinkBeforeParentSync) {
        return Err(SessionRepositoryError::Unavailable);
    }
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| SessionRepositoryError::Unavailable)
}

#[cfg(unix)]
fn cleanup_before_publish(temporary: &Path) -> Result<(), SessionRepositoryError> {
    let _ = std::fs::remove_file(temporary);
    Err(SessionRepositoryError::Unavailable)
}

#[cfg(all(test, unix))]
fn fail_before_publish(
    temporary: &Path,
    simulate_remove_failure: bool,
) -> Result<(), SessionRepositoryError> {
    if simulate_remove_failure {
        return Err(SessionRepositoryError::Unavailable);
    }
    cleanup_before_publish(temporary)
}

#[cfg(all(test, unix))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum BootstrapFault {
    AfterTempCreate,
    AfterWrite,
    AfterFileSync,
    BeforePublish,
    AfterPublishBeforeTempUnlink,
    AfterTempUnlinkBeforeParentSync,
    CleanupRemoveFailure,
}

#[cfg(all(test, unix))]
struct TestControl {
    fault: Option<BootstrapFault>,
    before_publish: Option<Arc<std::sync::Barrier>>,
    swap_after_open: Option<std::path::PathBuf>,
}

#[cfg(all(test, unix))]
fn fault(control: Option<&TestControl>) -> Option<BootstrapFault> {
    control.and_then(|control| control.fault)
}

#[cfg(all(test, unix))]
mod tests {
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicU64, Ordering};
    use ustc_campus_agent_core::identity::{TenantId, UserId};
    use ustc_campus_agent_core::session::{
        AuthAdapterId, CredentialEvidenceDigest, OpenSession, SessionCommand,
        SessionCredentialEvidence, SessionDuration, SessionInstant, SessionPolicy,
        SessionRefreshed, decide,
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "m00-session-store-{}-{}-{label}",
            std::process::id(),
            TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    fn bootstrap_events(session: &str, tenant: &str, user: &str) -> Vec<SessionEvent> {
        let evidence = SessionCredentialEvidence::new(
            TenantId::parse(tenant).unwrap(),
            UserId::parse(user).unwrap(),
            AuthAdapterId::parse("fixture.adapter").unwrap(),
            CredentialEvidenceDigest::parse(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .unwrap(),
            SessionInstant::from_unix_millis(1_000),
            None,
        )
        .unwrap();
        let command = SessionCommand::Open(OpenSession::new(
            SessionId::parse(session).unwrap(),
            evidence,
            SessionPolicy::new(
                SessionDuration::from_millis(1_000).unwrap(),
                SessionDuration::from_millis(10_000).unwrap(),
            ),
            SessionInstant::from_unix_millis(1_000),
            0,
        ));
        vec![decide(None, &command).unwrap()]
    }

    fn control(fault: Option<BootstrapFault>) -> TestControl {
        TestControl {
            fault,
            before_publish: None,
            swap_after_open: None,
        }
    }

    fn write_state(path: &Path, state: &SessionStoreFile) {
        fs::write(path, serde_json::to_vec(state).unwrap()).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    fn record(session: &str, tenant: &str, user: &str) -> SessionStoreRecord {
        SessionStoreRecord {
            session_id: SessionId::parse(session).unwrap(),
            events: bootstrap_events(session, tenant, user),
        }
    }

    #[test]
    fn bootstrap_absent_store_then_restart_reads_equal_history() {
        let dir = test_dir("restart");
        let path = dir.join("sessions.json");
        let events = bootstrap_events("session:one", "tenant:one", "user:one");
        let first = DurableCurrentSessionStore::open_or_bootstrap(&path, &events).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let bytes = fs::read(&path).unwrap();
        let mut reopened = DurableCurrentSessionStore::open_or_bootstrap(&path, &events).unwrap();
        let retained = reopened
            .load_history(&SessionId::parse("session:one").unwrap())
            .unwrap()
            .unwrap();
        let mut first_clone = first.clone();
        let original = first_clone
            .load_history(&SessionId::parse("session:one").unwrap())
            .unwrap()
            .unwrap();
        assert!(retained.events() == original.events());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn retained_store_wins_over_changed_bootstrap_and_missing_session_stays_absent() {
        let dir = test_dir("retained-wins");
        let path = dir.join("sessions.json");
        let original = bootstrap_events("session:one", "tenant:one", "user:one");
        DurableCurrentSessionStore::open_or_bootstrap(&path, &original).unwrap();
        let bytes = fs::read(&path).unwrap();
        let changed = bootstrap_events("session:two", "tenant:two", "user:two");
        let mut reopened = DurableCurrentSessionStore::open_or_bootstrap(&path, &changed).unwrap();
        assert!(
            reopened
                .load_history(&SessionId::parse("session:one").unwrap())
                .unwrap()
                .is_some()
        );
        assert!(
            reopened
                .load_history(&SessionId::parse("session:two").unwrap())
                .unwrap()
                .is_none()
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn malformed_unknown_version_duplicate_cross_session_and_forged_event_fail_closed() {
        let dir = test_dir("malformed");
        let path = dir.join("sessions.json");
        let bootstrap = bootstrap_events("session:one", "tenant:one", "user:one");
        for bytes in [
            b"not-json".as_slice(),
            br#"{"schema_version":2,"sessions":[]}"#,
            br#"{"schema_version":1,"sessions":[],"unknown":true}"#,
        ] {
            fs::write(&path, bytes).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            assert!(matches!(
                DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap),
                Err(SessionRepositoryError::Corrupt)
            ));
            fs::remove_file(&path).unwrap();
        }

        let duplicate = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: vec![
                record("session:one", "tenant:one", "user:one"),
                record("session:one", "tenant:one", "user:one"),
            ],
        };
        write_state(&path, &duplicate);
        assert!(matches!(
            DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap),
            Err(SessionRepositoryError::Corrupt)
        ));
        fs::remove_file(&path).unwrap();

        let cross_session = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: vec![SessionStoreRecord {
                session_id: SessionId::parse("session:one").unwrap(),
                events: bootstrap_events("session:two", "tenant:two", "user:two"),
            }],
        };
        write_state(&path, &cross_session);
        assert!(matches!(
            DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap),
            Err(SessionRepositoryError::Corrupt)
        ));
        fs::remove_file(&path).unwrap();

        let mut forged_events = bootstrap.clone();
        forged_events.push(SessionEvent::Refreshed(SessionRefreshed::new(
            2,
            SessionId::parse("session:one").unwrap(),
            SessionInstant::from_unix_millis(1_100),
            SessionInstant::from_unix_millis(9_999),
        )));
        let forged = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: vec![SessionStoreRecord {
                session_id: SessionId::parse("session:one").unwrap(),
                events: forged_events,
            }],
        };
        write_state(&path, &forged);
        assert!(matches!(
            DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap),
            Err(SessionRepositoryError::Corrupt)
        ));
    }

    #[test]
    fn noncanonical_order_empty_history_and_limit_fail_closed() {
        let dir = test_dir("noncanonical");
        let path = dir.join("sessions.json");
        let bootstrap = bootstrap_events("session:one", "tenant:one", "user:one");
        DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap).unwrap();
        let canonical = fs::read_to_string(&path).unwrap();
        fs::write(&path, format!("{canonical}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(matches!(
            DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap),
            Err(SessionRepositoryError::Corrupt)
        ));

        let empty = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: vec![SessionStoreRecord {
                session_id: SessionId::parse("session:one").unwrap(),
                events: Vec::new(),
            }],
        };
        write_state(&path, &empty);
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap).is_err());

        let reversed = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: vec![
                record("session:two", "tenant:two", "user:two"),
                record("session:one", "tenant:one", "user:one"),
            ],
        };
        write_state(&path, &reversed);
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap).is_err());

        let too_many_sessions = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: (0..=MAX_SESSIONS)
                .map(|index| {
                    let id = format!("session:s{index:03}");
                    record(&id, "tenant:one", "user:one")
                })
                .collect(),
        };
        write_state(&path, &too_many_sessions);
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap).is_err());

        let too_many_events = SessionStoreFile {
            schema_version: SCHEMA_VERSION,
            sessions: vec![SessionStoreRecord {
                session_id: SessionId::parse("session:one").unwrap(),
                events: vec![bootstrap[0].clone(); MAX_EVENTS_PER_SESSION + 1],
            }],
        };
        write_state(&path, &too_many_events);
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &bootstrap).is_err());
    }

    #[test]
    fn unsafe_file_type_symlink_parent_mode_identity_and_oversize_fail_closed() {
        let events = bootstrap_events("session:one", "tenant:one", "user:one");

        let dir = test_dir("unsafe-symlink");
        let sentinel = dir.join("sentinel");
        fs::write(&sentinel, "sentinel").unwrap();
        fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o600)).unwrap();
        let symlink = dir.join("sessions.json");
        std::os::unix::fs::symlink(&sentinel, &symlink).unwrap();
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&symlink, &events).is_err());
        assert_eq!(fs::read_to_string(&sentinel).unwrap(), "sentinel");

        let dir = test_dir("unsafe-mode");
        let path = dir.join("sessions.json");
        DurableCurrentSessionStore::open_or_bootstrap(&path, &events).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &events).is_err());

        fs::set_permissions(&path, fs::Permissions::from_mode(0o1600)).unwrap();
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &events).is_err());

        let dir = test_dir("unsafe-oversize");
        let path = dir.join("sessions.json");
        fs::File::create(&path)
            .unwrap()
            .set_len(MAX_FILE_BYTES + 1)
            .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &events).is_err());

        let dir = test_dir("unsafe-parent-mode");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            DurableCurrentSessionStore::open_or_bootstrap(&dir.join("sessions.json"), &events)
                .is_err()
        );
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o1700)).unwrap();
        assert!(
            DurableCurrentSessionStore::open_or_bootstrap(&dir.join("sessions.json"), &events)
                .is_err()
        );
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();

        let dir = test_dir("unsafe-hardlink");
        let path = dir.join("sessions.json");
        DurableCurrentSessionStore::open_or_bootstrap(&path, &events).unwrap();
        fs::hard_link(&path, dir.join("second-link")).unwrap();
        assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &events).is_err());

        let dir = test_dir("unsafe-swap");
        let path = dir.join("sessions.json");
        DurableCurrentSessionStore::open_or_bootstrap(&path, &events).unwrap();
        let replacement = dir.join("replacement.json");
        DurableCurrentSessionStore::open_or_bootstrap(
            &replacement,
            &bootstrap_events("session:two", "tenant:two", "user:two"),
        )
        .unwrap();
        let control = TestControl {
            fault: None,
            before_publish: None,
            swap_after_open: Some(replacement),
        };
        assert!(matches!(
            read_existing(&path, Some(&control)),
            Err(SessionRepositoryError::Unavailable)
        ));
    }

    #[test]
    fn bootstrap_is_atomic_bounded_and_leaves_no_temporary_residue() {
        for fault in [
            BootstrapFault::AfterTempCreate,
            BootstrapFault::AfterWrite,
            BootstrapFault::AfterFileSync,
            BootstrapFault::BeforePublish,
        ] {
            let dir = test_dir("fault-before");
            let path = dir.join("sessions.json");
            let events = bootstrap_events("session:one", "tenant:one", "user:one");
            assert!(
                DurableCurrentSessionStore::open_or_bootstrap_inner(
                    &path,
                    &events,
                    Some(&control(Some(fault)))
                )
                .is_err()
            );
            assert!(!path.exists());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        }

        let dir = test_dir("cleanup-remove-failure");
        let path = dir.join("sessions.json");
        let events = bootstrap_events("session:one", "tenant:one", "user:one");
        assert!(
            DurableCurrentSessionStore::open_or_bootstrap_inner(
                &path,
                &events,
                Some(&control(Some(BootstrapFault::CleanupRemoveFailure)))
            )
            .is_err()
        );
        assert!(!path.exists());
        let residue: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(residue.len(), 1);
        assert!(
            residue[0]
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".sessions.json.tmp-"))
        );
        fs::remove_file(&residue[0]).unwrap();

        for fault in [
            BootstrapFault::AfterPublishBeforeTempUnlink,
            BootstrapFault::AfterTempUnlinkBeforeParentSync,
        ] {
            let dir = test_dir("fault-after-publish");
            let path = dir.join("sessions.json");
            let events = bootstrap_events("session:one", "tenant:one", "user:one");
            assert!(
                DurableCurrentSessionStore::open_or_bootstrap_inner(
                    &path,
                    &events,
                    Some(&control(Some(fault)))
                )
                .is_err()
            );
            assert!(path.exists());
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
            assert!(DurableCurrentSessionStore::open_or_bootstrap(&path, &events).is_ok());
        }

        let dir = test_dir("concurrent");
        let path = dir.join("sessions.json");
        let barrier = Arc::new(Barrier::new(2));
        let mut joins = Vec::new();
        for _ in 0..2 {
            let path = path.clone();
            let barrier = Arc::clone(&barrier);
            joins.push(std::thread::spawn(move || {
                let events = bootstrap_events("session:one", "tenant:one", "user:one");
                let control = TestControl {
                    fault: None,
                    before_publish: Some(barrier),
                    swap_after_open: None,
                };
                DurableCurrentSessionStore::open_or_bootstrap_inner(&path, &events, Some(&control))
            }));
        }
        let outcomes: Vec<_> = joins.into_iter().map(|join| join.join().unwrap()).collect();
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(outcomes.iter().filter(|result| result.is_err()).count(), 1);
        assert!(
            DurableCurrentSessionStore::open_or_bootstrap(
                &path,
                &bootstrap_events("session:one", "tenant:one", "user:one")
            )
            .is_ok()
        );
    }
}
