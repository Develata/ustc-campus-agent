//! Minimal durable calendar-item store for the loopback MVP.
//!
//! This crate deliberately owns only bounded local item records. It does not
//! schedule reminders, synchronize external calendars, interpret natural
//! language, or perform network effects.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

pub const STORE_SCHEMA_VERSION: &str = "ustc-simple-calendar-store/v1";
pub const MAX_ITEMS: usize = 128;
pub const MAX_TITLE_BYTES: usize = 256;
pub const MAX_STORE_BYTES: u64 = 64 * 1024;
const MAX_SCHEDULED_FOR_BYTES: usize = 64;
const ITEM_ID_PREFIX: &str = "calendar:item:";

/// One owner-local calendar item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarItem {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub scheduled_for: Option<String>,
    pub created_at_unix_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedCalendar {
    schema: String,
    next_id: u64,
    items: Vec<CalendarItem>,
}

impl Default for PersistedCalendar {
    fn default() -> Self {
        Self {
            schema: STORE_SCHEMA_VERSION.to_owned(),
            next_id: 1,
            items: Vec::new(),
        }
    }
}

/// Stable, non-sensitive error classes for calendar operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarError {
    InvalidPath,
    InvalidStore,
    InvalidTitle,
    InvalidScheduledFor,
    ItemLimitExceeded,
    ItemNotFound,
    ClockUnavailable,
    CounterExhausted,
    PersistenceUnavailable,
}

impl fmt::Display for CalendarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "simple calendar operation failed: {self:?}")
    }
}

impl Error for CalendarError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistOutcome {
    Durable,
    RenamedParentSyncUncertainExact,
    RenamedParentSyncUncertainUnknown,
}

/// Durable owner-local store. Mutations persist before success is returned.
pub struct CalendarStore {
    path: PathBuf,
    state: PersistedCalendar,
    durability_uncertain: bool,
    #[cfg(test)]
    fail_next_parent_sync_after_rename: bool,
    #[cfg(test)]
    fail_next_post_rename_readback: bool,
}

impl CalendarStore {
    /// Opens an existing bounded store or an empty in-memory view when absent.
    /// The file is created only by the first successful mutation.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CalendarError> {
        let path = path.as_ref();
        validate_store_path(path)?;
        let state = match read_existing(path)? {
            Some(bytes) => serde_json::from_slice::<PersistedCalendar>(&bytes)
                .map_err(|_| CalendarError::InvalidStore)?,
            None => PersistedCalendar::default(),
        };
        validate_state(&state)?;
        Ok(Self {
            path: path.to_path_buf(),
            state,
            durability_uncertain: false,
            #[cfg(test)]
            fail_next_parent_sync_after_rename: false,
            #[cfg(test)]
            fail_next_post_rename_readback: false,
        })
    }

    /// Opens one required member of a caller-owned durable state set.
    ///
    /// A fresh state set durably materializes the canonical empty store. Once
    /// any state set exists, absence is corruption and fails closed.
    pub fn open_for_state_set(
        path: impl AsRef<Path>,
        bootstrap_is_fresh: bool,
    ) -> Result<Self, CalendarError> {
        let path = path.as_ref();
        validate_store_path(path)?;
        match fs::symlink_metadata(path) {
            Ok(_) => Self::open(path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if !bootstrap_is_fresh {
                    return Err(CalendarError::InvalidStore);
                }
                let mut store = Self {
                    path: path.to_path_buf(),
                    state: PersistedCalendar::default(),
                    durability_uncertain: false,
                    #[cfg(test)]
                    fail_next_parent_sync_after_rename: false,
                    #[cfg(test)]
                    fail_next_post_rename_readback: false,
                };
                match store.persist()? {
                    PersistOutcome::Durable => Ok(store),
                    PersistOutcome::RenamedParentSyncUncertainExact
                    | PersistOutcome::RenamedParentSyncUncertainUnknown => {
                        Err(CalendarError::PersistenceUnavailable)
                    }
                }
            }
            Err(_) => Err(CalendarError::InvalidPath),
        }
    }

    pub fn items(&mut self) -> Result<&[CalendarItem], CalendarError> {
        self.resolve_uncertain_durability()?;
        Ok(&self.state.items)
    }

    /// Records one bounded item and durably commits it before returning.
    pub fn record(
        &mut self,
        title: &str,
        scheduled_for: Option<&str>,
    ) -> Result<CalendarItem, CalendarError> {
        self.resolve_uncertain_durability()?;
        if self.state.items.len() >= MAX_ITEMS {
            return Err(CalendarError::ItemLimitExceeded);
        }
        let title = validate_title(title)?.to_owned();
        let scheduled_for = validate_scheduled_for(scheduled_for)?;
        let sequence = self.state.next_id;
        let next_id = sequence
            .checked_add(1)
            .ok_or(CalendarError::CounterExhausted)?;
        let created_at_unix_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CalendarError::ClockUnavailable)?
            .as_secs();
        let item = CalendarItem {
            id: format!("{ITEM_ID_PREFIX}{sequence}"),
            title,
            scheduled_for,
            created_at_unix_secs,
        };
        let previous = self.state.clone();
        self.state.next_id = next_id;
        self.state.items.push(item.clone());
        self.state
            .items
            .sort_by(|left, right| left.id.cmp(&right.id));
        match self.persist() {
            Ok(PersistOutcome::Durable) => Ok(item),
            Ok(PersistOutcome::RenamedParentSyncUncertainExact)
            | Ok(PersistOutcome::RenamedParentSyncUncertainUnknown) => {
                self.durability_uncertain = true;
                Err(CalendarError::PersistenceUnavailable)
            }
            Err(error) => {
                self.state = previous;
                Err(error)
            }
        }
    }

    /// Deletes one exact item id and durably commits the removal.
    pub fn delete(&mut self, item_id: &str) -> Result<CalendarItem, CalendarError> {
        self.resolve_uncertain_durability()?;
        let Some(index) = self.state.items.iter().position(|item| item.id == item_id) else {
            return Err(CalendarError::ItemNotFound);
        };
        let previous = self.state.clone();
        let removed = self.state.items.remove(index);
        match self.persist() {
            Ok(PersistOutcome::Durable) => Ok(removed),
            Ok(PersistOutcome::RenamedParentSyncUncertainExact)
            | Ok(PersistOutcome::RenamedParentSyncUncertainUnknown) => {
                self.durability_uncertain = true;
                Err(CalendarError::PersistenceUnavailable)
            }
            Err(error) => {
                self.state = previous;
                Err(error)
            }
        }
    }

    fn persist(&mut self) -> Result<PersistOutcome, CalendarError> {
        let bytes =
            serde_json::to_vec(&self.state).map_err(|_| CalendarError::PersistenceUnavailable)?;
        if bytes.len() as u64 > MAX_STORE_BYTES {
            return Err(CalendarError::ItemLimitExceeded);
        }
        let parent = self
            .path
            .parent()
            .ok_or(CalendarError::InvalidPath)?
            .to_path_buf();
        validate_store_path(&self.path)?;
        validate_existing_destination(&self.path)?;
        let file_name = self
            .path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(CalendarError::InvalidPath)?;
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CalendarError::ClockUnavailable)?
            .as_nanos();
        let temp_path = parent.join(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        let result = (|| {
            let mut file = options
                .open(&temp_path)
                .map_err(|_| CalendarError::PersistenceUnavailable)?;
            validate_primary_metadata(
                &file
                    .metadata()
                    .map_err(|_| CalendarError::PersistenceUnavailable)?,
            )?;
            file.write_all(&bytes)
                .and_then(|()| file.sync_all())
                .map_err(|_| CalendarError::PersistenceUnavailable)?;
            drop(file);
            fs::rename(&temp_path, &self.path)
                .map_err(|_| CalendarError::PersistenceUnavailable)?;
            if self.sync_parent(&parent).is_ok() {
                return Ok(PersistOutcome::Durable);
            }
            Ok(if self.post_rename_readback_is_exact(&bytes) {
                PersistOutcome::RenamedParentSyncUncertainExact
            } else {
                PersistOutcome::RenamedParentSyncUncertainUnknown
            })
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        result
    }

    fn resolve_uncertain_durability(&mut self) -> Result<(), CalendarError> {
        if !self.durability_uncertain {
            return Ok(());
        }
        let expected =
            serde_json::to_vec(&self.state).map_err(|_| CalendarError::PersistenceUnavailable)?;
        if !matches!(read_existing(&self.path), Ok(Some(actual)) if actual == expected) {
            return Err(CalendarError::InvalidStore);
        }
        let parent = self
            .path
            .parent()
            .ok_or(CalendarError::InvalidPath)?
            .to_path_buf();
        self.sync_parent(&parent)
            .map_err(|_| CalendarError::PersistenceUnavailable)?;
        self.durability_uncertain = false;
        Ok(())
    }

    fn sync_parent(&mut self, parent: &Path) -> std::io::Result<()> {
        #[cfg(test)]
        if self.fail_next_parent_sync_after_rename {
            self.fail_next_parent_sync_after_rename = false;
            return Err(std::io::Error::other(
                "injected calendar parent sync failure",
            ));
        }
        fs::File::open(parent).and_then(|directory| directory.sync_all())
    }

    fn post_rename_readback_is_exact(&mut self, expected: &[u8]) -> bool {
        #[cfg(test)]
        if self.fail_next_post_rename_readback {
            self.fail_next_post_rename_readback = false;
            return false;
        }
        matches!(read_existing(&self.path), Ok(Some(actual)) if actual == expected)
    }

    #[cfg(test)]
    fn fail_next_parent_sync_after_rename(&mut self) {
        self.fail_next_parent_sync_after_rename = true;
    }

    #[cfg(test)]
    fn fail_next_post_rename_readback(&mut self) {
        self.fail_next_post_rename_readback = true;
    }
}

fn read_existing(path: &Path) -> Result<Option<Vec<u8>>, CalendarError> {
    validate_store_path(path)?;
    let path_metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(CalendarError::InvalidPath),
    };
    validate_primary_metadata(&path_metadata)?;

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW);
    let mut file = options
        .open(path)
        .map_err(|_| CalendarError::InvalidStore)?;
    let opened_metadata = file.metadata().map_err(|_| CalendarError::InvalidStore)?;
    validate_primary_metadata(&opened_metadata)?;
    let after_metadata = fs::symlink_metadata(path).map_err(|_| CalendarError::InvalidStore)?;
    validate_primary_metadata(&after_metadata)?;
    #[cfg(unix)]
    if path_metadata.dev() != opened_metadata.dev()
        || path_metadata.ino() != opened_metadata.ino()
        || after_metadata.dev() != opened_metadata.dev()
        || after_metadata.ino() != opened_metadata.ino()
    {
        return Err(CalendarError::InvalidStore);
    }

    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_STORE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| CalendarError::InvalidStore)?;
    if bytes.len() as u64 > MAX_STORE_BYTES {
        return Err(CalendarError::InvalidStore);
    }
    Ok(Some(bytes))
}

fn validate_existing_destination(path: &Path) -> Result<(), CalendarError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => validate_primary_metadata(&metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(CalendarError::InvalidStore),
    }
}

fn validate_primary_metadata(metadata: &fs::Metadata) -> Result<(), CalendarError> {
    if !metadata.file_type().is_file() || metadata.len() > MAX_STORE_BYTES {
        return Err(CalendarError::InvalidStore);
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o7777 != 0o600
        || metadata.uid() != current_uid()?
        || metadata.nlink() != 1
    {
        return Err(CalendarError::InvalidStore);
    }
    Ok(())
}

#[cfg(unix)]
fn current_uid() -> Result<u32, CalendarError> {
    fs::metadata("/proc/self")
        .map(|metadata| metadata.uid())
        .map_err(|_| CalendarError::InvalidStore)
}

fn validate_store_path(path: &Path) -> Result<(), CalendarError> {
    if path.as_os_str().is_empty() || path.file_name().is_none() {
        return Err(CalendarError::InvalidPath);
    }
    let parent = path.parent().ok_or(CalendarError::InvalidPath)?;
    let metadata = fs::symlink_metadata(parent).map_err(|_| CalendarError::InvalidPath)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CalendarError::InvalidPath);
    }
    Ok(())
}

fn validate_state(state: &PersistedCalendar) -> Result<(), CalendarError> {
    if state.schema != STORE_SCHEMA_VERSION || state.next_id == 0 || state.items.len() > MAX_ITEMS {
        return Err(CalendarError::InvalidStore);
    }
    let mut ids = BTreeSet::new();
    let mut maximum_sequence = 0_u64;
    for item in &state.items {
        validate_title(&item.title).map_err(|_| CalendarError::InvalidStore)?;
        validate_scheduled_for(item.scheduled_for.as_deref())
            .map_err(|_| CalendarError::InvalidStore)?;
        let sequence = item
            .id
            .strip_prefix(ITEM_ID_PREFIX)
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .ok_or(CalendarError::InvalidStore)?;
        if !ids.insert(item.id.as_str()) {
            return Err(CalendarError::InvalidStore);
        }
        maximum_sequence = maximum_sequence.max(sequence);
    }
    if state.next_id <= maximum_sequence {
        return Err(CalendarError::InvalidStore);
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<&str, CalendarError> {
    let title = title.trim();
    if title.is_empty()
        || title.len() > MAX_TITLE_BYTES
        || title
            .chars()
            .any(|character| character.is_control() || is_unicode_format(character))
    {
        return Err(CalendarError::InvalidTitle);
    }
    Ok(title)
}

fn is_unicode_format(character: char) -> bool {
    matches!(
        character,
        '\u{00ad}'
            | '\u{0600}'..='\u{0605}'
            | '\u{061c}'
            | '\u{06dd}'
            | '\u{070f}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08e2}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
            | '\u{110bd}'
            | '\u{110cd}'
            | '\u{13430}'..='\u{1343f}'
            | '\u{1bca0}'..='\u{1bca3}'
            | '\u{1d173}'..='\u{1d17a}'
            | '\u{e0001}'
            | '\u{e0020}'..='\u{e007f}'
    )
}

fn validate_scheduled_for(value: Option<&str>) -> Result<Option<String>, CalendarError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_empty()
        || value.len() > MAX_SCHEDULED_FOR_BYTES
        || value.chars().any(char::is_control)
        || OffsetDateTime::parse(value, &Rfc3339).is_err()
    {
        return Err(CalendarError::InvalidScheduledFor);
    }
    Ok(Some(value.to_owned()))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_store() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "ustc-simple-calendar-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        (root.join("calendar.json"), root)
    }

    #[test]
    fn record_list_reopen_and_delete_are_durable() {
        let (path, root) = temp_store();
        let mut store = CalendarStore::open(&path).unwrap();
        assert!(store.items().unwrap().is_empty());
        let item = store
            .record("  提交开题报告  ", Some("2026-09-10T09:00:00+08:00"))
            .unwrap();
        assert_eq!(item.id, "calendar:item:1");
        assert_eq!(item.title, "提交开题报告");
        drop(store);

        let mut reopened = CalendarStore::open(&path).unwrap();
        assert_eq!(reopened.items().unwrap(), std::slice::from_ref(&item));
        assert_eq!(reopened.delete(&item.id).unwrap(), item);
        drop(reopened);
        assert!(
            CalendarStore::open(&path)
                .unwrap()
                .items()
                .unwrap()
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_titles_and_times_fail_without_creating_state() {
        let (path, root) = temp_store();
        let mut store = CalendarStore::open(&path).unwrap();
        assert_eq!(store.record("   ", None), Err(CalendarError::InvalidTitle));
        assert_eq!(
            store.record("事项", Some("tomorrow")),
            Err(CalendarError::InvalidScheduledFor)
        );
        for title in ["事项\u{202e}", "事项\u{200b}", "事项\u{feff}"] {
            assert_eq!(store.record(title, None), Err(CalendarError::InvalidTitle));
        }
        assert!(!path.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn malformed_or_unknown_persisted_fields_fail_closed() {
        let (path, root) = temp_store();
        fs::write(
            &path,
            br#"{"schema":"ustc-simple-calendar-store/v1","next_id":2,"items":[],"extra":true}"#,
        )
        .unwrap();
        #[cfg(unix)]
        fs::set_permissions(
            &path,
            <fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o600),
        )
        .unwrap();
        assert!(matches!(
            CalendarStore::open(&path),
            Err(CalendarError::InvalidStore)
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn state_set_open_persists_canonical_empty_mode_and_fails_on_nonfresh_absence() {
        let (path, root) = temp_store();
        let mut store = CalendarStore::open_for_state_set(&path, true).unwrap();
        assert!(store.items().unwrap().is_empty());
        assert_eq!(
            fs::read(&path).unwrap(),
            br#"{"schema":"ustc-simple-calendar-store/v1","next_id":1,"items":[]}"#
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        let item = store.record("提交开题报告", None).unwrap();
        drop(store);
        let mut reopened = CalendarStore::open_for_state_set(&path, false).unwrap();
        assert_eq!(reopened.items().unwrap(), std::slice::from_ref(&item));
        drop(reopened);

        fs::remove_file(&path).unwrap();
        assert!(matches!(
            CalendarStore::open_for_state_set(&path, false),
            Err(CalendarError::InvalidStore)
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn existing_store_rejects_unsafe_mode_hard_links_and_symlinks() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let (path, root) = temp_store();
        CalendarStore::open(&path)
            .unwrap()
            .record("安全事项", None)
            .unwrap();

        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            CalendarStore::open(&path),
            Err(CalendarError::InvalidStore)
        ));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

        let hard_link = root.join("calendar-hard-link.json");
        fs::hard_link(&path, &hard_link).unwrap();
        assert!(matches!(
            CalendarStore::open(&path),
            Err(CalendarError::InvalidStore)
        ));
        fs::remove_file(&hard_link).unwrap();

        let target = root.join("calendar-target.json");
        fs::rename(&path, &target).unwrap();
        symlink(&target, &path).unwrap();
        assert!(matches!(
            CalendarStore::open(&path),
            Err(CalendarError::InvalidStore)
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parent_sync_uncertainty_keeps_exact_published_memory_and_reconciles() {
        let (path, root) = temp_store();
        let mut store = CalendarStore::open(&path).unwrap();
        store.record("第一项", None).unwrap();
        store.fail_next_parent_sync_after_rename();
        assert_eq!(
            store.record("第二项", None),
            Err(CalendarError::PersistenceUnavailable)
        );
        assert_eq!(store.items().unwrap().len(), 2);

        let third = store.record("第三项", None).unwrap();
        assert_eq!(third.id, "calendar:item:3");
        drop(store);

        let mut reopened = CalendarStore::open(&path).unwrap();
        assert_eq!(reopened.items().unwrap().len(), 3);
        assert_eq!(reopened.items().unwrap()[1].title, "第二项");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unknown_parent_sync_uncertainty_blocks_list_until_reconciled() {
        let (path, root) = temp_store();
        let mut store = CalendarStore::open(&path).unwrap();
        store.record("第一项", None).unwrap();
        store.fail_next_parent_sync_after_rename();
        store.fail_next_post_rename_readback();
        assert_eq!(
            store.record("第二项", None),
            Err(CalendarError::PersistenceUnavailable)
        );

        store.fail_next_parent_sync_after_rename();
        assert_eq!(store.items(), Err(CalendarError::PersistenceUnavailable));
        assert_eq!(store.items().unwrap().len(), 2);
        fs::remove_dir_all(root).unwrap();
    }
}
