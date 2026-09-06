//! Admitted subject selects a private Calendar owner; public composition is shared.
use std::{collections::BTreeMap, path::PathBuf};
use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_simple_calendar::{CalendarError, CalendarStore};

pub(crate) struct CalendarWorkspaces {
    root: PathBuf,
    stores: BTreeMap<String, Workspace>,
}
impl CalendarWorkspaces {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self {
            root,
            stores: BTreeMap::new(),
        }
    }
    pub(crate) fn with<T>(
        &mut self,
        owner: &(TenantId, UserId),
        f: impl FnOnce(&mut CalendarStore) -> Result<T, CalendarError>,
    ) -> Result<T, CalendarError> {
        let key = format!("{}-{}", encode(owner.0.as_str()), encode(owner.1.as_str()));
        if !self.stores.contains_key(&key) {
            if self.stores.len() >= 1000 {
                return Err(CalendarError::ItemLimitExceeded);
            }
            private_directory(&self.root)?;
            let dir = self.root.join(&key);
            let fresh = !dir.try_exists().map_err(|_| CalendarError::InvalidPath)?;
            private_directory(&dir)?;
            let store = Workspace::open(&dir, fresh)?;
            self.stores.insert(key.clone(), store);
        }
        f(&mut self
            .stores
            .get_mut(&key)
            .ok_or(CalendarError::PersistenceUnavailable)?
            .store)
    }
    /// Discover only canonical workspace names, so overdue reminders survive restart.
    pub(crate) fn tick(&mut self, now: u64) -> Result<(), CalendarError> {
        let mut failure = None;
        if self.root.exists() {
            private_directory(&self.root)?;
            for entry in std::fs::read_dir(&self.root).map_err(|_| CalendarError::InvalidPath)? {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => {
                        failure = Some(CalendarError::InvalidPath);
                        continue;
                    }
                };
                let Some(key) = entry.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                if !valid_key(&key) || self.stores.contains_key(&key) {
                    continue;
                }
                if self.stores.len() >= 1000 {
                    failure = Some(CalendarError::ItemLimitExceeded);
                    break;
                }
                match private_directory(&entry.path())
                    .and_then(|()| Workspace::open(&entry.path(), false))
                {
                    Ok(store) => {
                        self.stores.insert(key, store);
                    }
                    Err(e) => {
                        failure = Some(e);
                    }
                }
            }
        }
        for workspace in self.stores.values_mut() {
            if let Err(error) = workspace.store.dispatch_reminders(now) {
                failure = Some(error);
            }
        }
        failure.map_or(Ok(()), Err)
    }
}
fn encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn valid_key(value: &str) -> bool {
    let parts = value.split('-').collect::<Vec<_>>();
    parts.len() == 2
        && parts.iter().all(|s| {
            !s.is_empty()
                && s.len() <= 512
                && s.len() % 2 == 0
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}
#[cfg(unix)]
fn private_directory(path: &std::path::Path) -> Result<(), CalendarError> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
    match std::fs::symlink_metadata(path) {
        Ok(m)
            if m.is_dir()
                && !m.file_type().is_symlink()
                && m.permissions().mode() & 0o077 == 0
                && m.uid()
                    == std::fs::metadata("/proc/self")
                        .map_err(|_| CalendarError::InvalidPath)?
                        .uid() =>
        {
            Ok(())
        }
        Ok(_) => Err(CalendarError::InvalidPath),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(path)
                .map_err(|_| CalendarError::InvalidPath)?;
            std::fs::File::open(path.parent().ok_or(CalendarError::InvalidPath)?)
                .and_then(|parent| parent.sync_all())
                .map_err(|_| CalendarError::PersistenceUnavailable)
        }
        Err(_) => Err(CalendarError::InvalidPath),
    }
}

#[cfg(not(unix))]
fn private_directory(_path: &std::path::Path) -> Result<(), CalendarError> {
    Err(CalendarError::InvalidPath)
}

struct Workspace {
    store: CalendarStore,
    writer: std::fs::File,
}
impl Workspace {
    #[cfg(unix)]
    fn open(dir: &std::path::Path, fresh: bool) -> Result<Self, CalendarError> {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
        let writer = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(dir.join("writer.lock"))
            .map_err(|_| CalendarError::InvalidPath)?;
        let m = writer.metadata().map_err(|_| CalendarError::InvalidPath)?;
        if !m.is_file() || m.nlink() != 1 || m.permissions().mode() & 0o077 != 0 {
            return Err(CalendarError::InvalidPath);
        }
        writer
            .try_lock()
            .map_err(|_| CalendarError::PersistenceUnavailable)?;
        let store = CalendarStore::open_for_state_set(dir.join("calendar.json"), fresh)?;
        Ok(Self { store, writer })
    }
    #[cfg(not(unix))]
    fn open(_dir: &std::path::Path, _fresh: bool) -> Result<Self, CalendarError> {
        Err(CalendarError::InvalidPath)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = self.writer.unlock();
    }
}
