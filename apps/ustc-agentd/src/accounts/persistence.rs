//! Owner-private, cross-worker transactions. Separate revision fence detects rollback
//! of the session snapshot; restoring both files is outside this local profile.
use super::{AccountError, Store};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

pub(super) struct Lock(File);
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

fn validate(file: &File) -> Result<(), AccountError> {
    let metadata = file.metadata().map_err(|_| AccountError::Unavailable)?;
    let uid = fs::metadata("/proc/self")
        .map_err(|_| AccountError::Unavailable)?
        .uid();
    if !metadata.is_file()
        || metadata.uid() != uid
        || metadata.mode() & 0o7777 != 0o600
        || metadata.nlink() != 1
    {
        return Err(AccountError::Unavailable);
    }
    Ok(())
}

pub(super) fn lock(path: &Path) -> Result<Lock, AccountError> {
    crate::durable_path::ensure_secure_parent(path, true).map_err(|_| AccountError::Unavailable)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path.with_extension("account-lock"))
        .map_err(|_| AccountError::Unavailable)?;
    validate(&file)?;
    file.lock().map_err(|_| AccountError::Unavailable)?;
    Ok(Lock(file))
}

pub(super) fn read_private(path: &Path, max: u64) -> Result<Vec<u8>, AccountError> {
    crate::durable_path::ensure_secure_parent(path, false)
        .map_err(|_| AccountError::Unavailable)?;
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| AccountError::Unavailable)?;
    validate(&file)?;
    if file
        .metadata()
        .map_err(|_| AccountError::Unavailable)?
        .len()
        > max
    {
        return Err(AccountError::Unavailable);
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| AccountError::Unavailable)?;
    if bytes.len() as u64 > max {
        return Err(AccountError::Unavailable);
    }
    Ok(bytes)
}

pub(super) fn initialize(path: &Path) -> Result<(), AccountError> {
    let fence = path.with_extension("account-fence");
    match (fs::symlink_metadata(path), fs::symlink_metadata(&fence)) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(a), Err(b))
            if a.kind() == std::io::ErrorKind::NotFound
                && b.kind() == std::io::ErrorKind::NotFound =>
        {
            atomic_write(
                path,
                &serde_json::to_vec(&Store::default()).map_err(|_| AccountError::Unavailable)?,
            )?;
            atomic_write(&fence, b"0")
        }
        _ => Err(AccountError::Unavailable),
    }
}

pub(super) fn read_store(path: &Path) -> Result<Store, AccountError> {
    let bytes = read_private(path, 16 * 1024 * 1024)?;
    let store: Store = serde_json::from_slice(&bytes).map_err(|_| AccountError::Unavailable)?;
    let fence = read_private(&path.with_extension("account-fence"), 32)?;
    if fence != store.revision.to_string().as_bytes() {
        return Err(AccountError::Unavailable);
    }
    Ok(store)
}

pub(super) fn commit(path: &Path, store: &mut Store) -> Result<(), AccountError> {
    store.revision = store
        .revision
        .checked_add(1)
        .ok_or(AccountError::Unavailable)?;
    let bytes = serde_json::to_vec(store).map_err(|_| AccountError::Unavailable)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(AccountError::Unavailable);
    }
    atomic_write(path, &bytes)?;
    // A crash between these publications leaves a mismatch and closes admission.
    atomic_write(
        &path.with_extension("account-fence"),
        store.revision.to_string().as_bytes(),
    )
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AccountError> {
    let parent = path.parent().ok_or(AccountError::Unavailable)?;
    let temporary = parent.join(format!(".account-{}.tmp", super::random_hex()?));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&temporary)
        .map_err(|_| AccountError::Unavailable)?;
    let result = (|| {
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| AccountError::Unavailable)?;
        fs::rename(&temporary, path).map_err(|_| AccountError::Unavailable)?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| AccountError::Unavailable)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
