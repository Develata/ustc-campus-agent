//! Unix private-file adapter; domain transitions live in the parent module.
use super::{ConversationError, State};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
const MAX_STORE_BYTES: usize = 32 * 1024 * 1024;
// 16 KiB answer at worst six JSON bytes per input byte plus bounded traces/identity.
const RUNNING_RESERVE_BYTES: usize = 128 * 1024;
pub(super) struct Disk {
    path: PathBuf,
    _lock: File,
}
impl Drop for Disk {
    fn drop(&mut self) {
        // Release at the owner's lifetime boundary: an unrelated fork may still
        // hold this open file description until exec closes inherited handles.
        let _ = self._lock.unlock();
    }
}
impl Disk {
    pub(super) fn open(path: PathBuf) -> Result<(Self, State), ConversationError> {
        crate::durable_path::ensure_secure_parent(&path, true)
            .map_err(|_| ConversationError::Unavailable)?;
        let parent = path.parent().ok_or(ConversationError::Unavailable)?;
        let lock_path = parent.join(".conversations.lock");
        let (lock, fresh) = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&lock_path)
        {
            Ok(lock) => (lock, true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                    .open(&lock_path)
                    .map_err(|_| ConversationError::Unavailable)?,
                false,
            ),
            Err(_) => return Err(ConversationError::Unavailable),
        };
        validate_file(&lock)?;
        lock.try_lock()
            .map_err(|_| ConversationError::Unavailable)?;
        let disk = Self { path, _lock: lock };
        let state = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&disk.path)
        {
            Ok(mut file) => {
                if fresh {
                    return Err(ConversationError::Unavailable);
                }
                validate_file(&file)?;
                if file
                    .metadata()
                    .map_err(|_| ConversationError::Unavailable)?
                    .len()
                    > MAX_STORE_BYTES as u64
                {
                    return Err(ConversationError::Unavailable);
                }
                let mut bytes = Vec::new();
                Read::by_ref(&mut file)
                    .take((MAX_STORE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|_| ConversationError::Unavailable)?;
                if bytes.len() > MAX_STORE_BYTES {
                    return Err(ConversationError::Unavailable);
                }
                serde_json::from_slice(&bytes).map_err(|_| ConversationError::Unavailable)?
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && fresh => {
                let state = State {
                    version: 1,
                    conversations: Vec::new(),
                };
                disk.save(&state)?;
                state
            }
            Err(_) => return Err(ConversationError::Unavailable),
        };
        Ok((disk, state))
    }
    pub(super) fn save(&self, state: &State) -> Result<(), ConversationError> {
        let bytes = serde_json::to_vec(state).map_err(|_| ConversationError::Unavailable)?;
        check_capacity(
            bytes.len(),
            state
                .conversations
                .iter()
                .flat_map(|c| &c.turns)
                .filter(|t| t.view.phase == super::TurnPhase::Running)
                .count(),
        )?;
        crate::durable_path::ensure_secure_parent(&self.path, false)
            .map_err(|_| ConversationError::Unavailable)?;
        if self.path.symlink_metadata().is_ok() {
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(&self.path)
                .map_err(|_| ConversationError::Unavailable)?;
            validate_file(&file)?;
        }
        let parent = self.path.parent().ok_or(ConversationError::Unavailable)?;
        let temporary = parent.join(format!(".conversation-{}.tmp", random_id()?));
        let result = write_replace(&temporary, &self.path, parent, &bytes);
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}
fn write_replace(
    temporary: &Path,
    target: &Path,
    parent: &Path,
    bytes: &[u8],
) -> Result<(), ConversationError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(temporary)
        .map_err(|_| ConversationError::Unavailable)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| ConversationError::Unavailable)?;
    fs::rename(temporary, target).map_err(|_| ConversationError::Unavailable)?;
    File::open(parent)
        .and_then(|dir| dir.sync_all())
        .map_err(|_| ConversationError::Unavailable)
}
fn validate_file(file: &File) -> Result<(), ConversationError> {
    let meta = file
        .metadata()
        .map_err(|_| ConversationError::Unavailable)?;
    let uid = fs::metadata("/proc/self")
        .map_err(|_| ConversationError::Unavailable)?
        .uid();
    if !meta.is_file()
        || meta.uid() != uid
        || meta.nlink() != 1
        || meta.permissions().mode() & 0o7777 != 0o600
    {
        return Err(ConversationError::Unavailable);
    }
    Ok(())
}
pub(super) fn random_id() -> Result<String, ConversationError> {
    let mut bytes = [0u8; 32];
    File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(|_| ConversationError::Unavailable)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn check_capacity(bytes: usize, running: usize) -> Result<(), ConversationError> {
    if running
        .checked_mul(RUNNING_RESERVE_BYTES)
        .and_then(|reserve| bytes.checked_add(reserve))
        .is_none_or(|total| total > MAX_STORE_BYTES)
    {
        Err(ConversationError::Capacity)
    } else {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dropped_owner_releases_lock_even_with_an_inherited_descriptor() {
        let directory = std::env::temp_dir().join(format!(
            "uca-conversation-lock-{}",
            random_id().expect("random directory")
        ));
        fs::create_dir(&directory).expect("create directory");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .expect("private directory");
        let path = directory.join("conversations.json");
        let (disk, _) = Disk::open(path.clone()).expect("first writer");
        // dup and fork retain the same open file description. Holding this clone
        // models the window before an unrelated child closes its inherited fd.
        let inherited = disk._lock.try_clone().expect("inherited descriptor");
        assert!(matches!(
            Disk::open(path.clone()),
            Err(ConversationError::Unavailable)
        ));
        drop(disk);
        let replacement = Disk::open(path.clone());
        let reopened = replacement.is_ok();
        drop(inherited);
        let competitor = Disk::open(path);
        let exclusive = matches!(&competitor, Err(ConversationError::Unavailable));
        drop(competitor);
        drop(replacement);
        fs::remove_dir_all(directory).expect("cleanup");
        assert!(
            reopened,
            "a dropped owner must not leave its lock in a child"
        );
        assert!(
            exclusive,
            "the replacement must remain the exclusive writer"
        );
    }

    #[test]
    fn running_reservations_leave_room_for_terminal_response() {
        assert_eq!(
            check_capacity(MAX_STORE_BYTES - RUNNING_RESERVE_BYTES, 1),
            Ok(())
        );
        assert_eq!(
            check_capacity(MAX_STORE_BYTES - RUNNING_RESERVE_BYTES + 1, 1),
            Err(ConversationError::Capacity)
        );
        assert_eq!(
            check_capacity(MAX_STORE_BYTES - RUNNING_RESERVE_BYTES + 100 * 1024, 0),
            Ok(())
        );
        assert_eq!(check_capacity(MAX_STORE_BYTES, 0), Ok(()));
        assert_eq!(
            check_capacity(usize::MAX, 1),
            Err(ConversationError::Capacity)
        );
    }
}
