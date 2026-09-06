//! One private atomic container for the original M20 installation and grant ledgers.
//! No installation/grant semantics or duplicate command ledger live in this adapter.
use super::{AuthorityState, PluginError};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use ustc_campus_agent_core::market::{
    grant::persistence as grants, installation::persistence as installations,
};
const MAGIC: &[u8] = b"uca-plugin-authority/v1\0";
const MAX_BYTES: usize = 40 * 1024 * 1024 + 128;
pub(super) struct Disk {
    path: PathBuf,
    _lock: File,
}
impl Disk {
    pub(super) fn open(path: PathBuf) -> Result<(Self, AuthorityState), PluginError> {
        crate::durable_path::ensure_secure_parent(&path, true)
            .map_err(|_| PluginError::Unavailable)?;
        let parent = path.parent().ok_or(PluginError::Unavailable)?;
        let lock_path = parent.join(".plugin-authority.lock");
        let (lock, fresh) = match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&lock_path)
        {
            Ok(file) => (file, true),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => (
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                    .open(&lock_path)
                    .map_err(|_| PluginError::Unavailable)?,
                false,
            ),
            Err(_) => return Err(PluginError::Unavailable),
        };
        validate(&lock)?;
        lock.try_lock().map_err(|_| PluginError::Unavailable)?;
        let disk = Self { path, _lock: lock };
        let state = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&disk.path)
        {
            Ok(mut file) => {
                if fresh {
                    return Err(PluginError::Unavailable);
                }
                validate(&file)?;
                if file.metadata().map_err(|_| PluginError::Unavailable)?.len() > MAX_BYTES as u64 {
                    return Err(PluginError::Capacity);
                }
                let mut bytes = Vec::new();
                Read::by_ref(&mut file)
                    .take((MAX_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|_| PluginError::Unavailable)?;
                decode(&bytes)?
            }
            Err(error) if fresh && error.kind() == std::io::ErrorKind::NotFound => {
                let state = AuthorityState::default();
                disk.save(&state)?;
                state
            }
            Err(_) => return Err(PluginError::Unavailable),
        };
        Ok((disk, state))
    }
    pub(super) fn save(&self, state: &AuthorityState) -> Result<(), PluginError> {
        let bytes = encode(state)?;
        crate::durable_path::ensure_secure_parent(&self.path, false)
            .map_err(|_| PluginError::Unavailable)?;
        if self.path.symlink_metadata().is_ok() {
            let existing = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(&self.path)
                .map_err(|_| PluginError::Unavailable)?;
            validate(&existing)?;
        }
        let parent = self.path.parent().ok_or(PluginError::Unavailable)?;
        let mut random = [0u8; 16];
        File::open("/dev/urandom")
            .and_then(|mut f| f.read_exact(&mut random))
            .map_err(|_| PluginError::Unavailable)?;
        let name: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        let temporary = parent.join(format!(".plugin-{name}.tmp"));
        let result = replace(&temporary, &self.path, parent, &bytes);
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}
fn validate(file: &File) -> Result<(), PluginError> {
    let meta = file.metadata().map_err(|_| PluginError::Unavailable)?;
    let uid = fs::metadata("/proc/self")
        .map_err(|_| PluginError::Unavailable)?
        .uid();
    if !meta.is_file()
        || meta.uid() != uid
        || meta.nlink() != 1
        || meta.permissions().mode() & 0o7777 != 0o600
    {
        return Err(PluginError::Unavailable);
    }
    Ok(())
}
fn replace(temp: &Path, target: &Path, parent: &Path, bytes: &[u8]) -> Result<(), PluginError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(temp)
        .map_err(|_| PluginError::Unavailable)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| PluginError::Unavailable)?;
    fs::rename(temp, target).map_err(|_| PluginError::Unavailable)?;
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| PluginError::Unavailable)
}
fn encode(state: &AuthorityState) -> Result<Vec<u8>, PluginError> {
    let installation =
        installations::encode_snapshot(&state.installations).map_err(|error| match error {
            installations::SnapshotCodecError::TooLarge => PluginError::Capacity,
            _ => PluginError::Unavailable,
        })?;
    let grant = grants::encode_snapshot(&state.grants).map_err(|error| match error {
        grants::SnapshotCodecError::TooLarge => PluginError::Capacity,
        _ => PluginError::Unavailable,
    })?;
    let runs = serde_json::to_vec(&state.runs).map_err(|_| PluginError::Unavailable)?;
    if runs.len() > 8 * 1024 * 1024 || state.runs.len() > 1024 {
        return Err(PluginError::Capacity);
    }
    let size = MAGIC.len() + 24 + installation.len() + grant.len() + runs.len();
    if size > MAX_BYTES {
        return Err(PluginError::Capacity);
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(installation.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&(grant.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&(runs.len() as u64).to_be_bytes());
    bytes.extend_from_slice(&installation);
    bytes.extend_from_slice(&grant);
    bytes.extend_from_slice(&runs);
    Ok(bytes)
}
fn decode(bytes: &[u8]) -> Result<AuthorityState, PluginError> {
    let start = MAGIC.len() + 24;
    if bytes.len() > MAX_BYTES || bytes.len() < start || !bytes.starts_with(MAGIC) {
        return Err(PluginError::Unavailable);
    }
    let length = |part: &[u8]| -> Result<usize, PluginError> {
        let raw: [u8; 8] = part.try_into().map_err(|_| PluginError::Unavailable)?;
        usize::try_from(u64::from_be_bytes(raw)).map_err(|_| PluginError::Unavailable)
    };
    let installation_len = length(&bytes[MAGIC.len()..MAGIC.len() + 8])?;
    let grant_len = length(&bytes[MAGIC.len() + 8..MAGIC.len() + 16])?;
    let runs_len = length(&bytes[MAGIC.len() + 16..start])?;
    let split = start
        .checked_add(installation_len)
        .ok_or(PluginError::Unavailable)?;
    let grant_end = split
        .checked_add(grant_len)
        .ok_or(PluginError::Unavailable)?;
    if grant_end.checked_add(runs_len) != Some(bytes.len()) || runs_len > 8 * 1024 * 1024 {
        return Err(PluginError::Unavailable);
    }
    let runs: Vec<super::invocation::JournalRun> =
        serde_json::from_slice(&bytes[grant_end..]).map_err(|_| PluginError::Unavailable)?;
    if runs.len() > 1024 {
        return Err(PluginError::Capacity);
    }
    for run in &runs {
        run.validate()?;
    }
    Ok(AuthorityState {
        installations: installations::decode_snapshot(&bytes[start..split])
            .map_err(|_| PluginError::Unavailable)?,
        grants: grants::decode_snapshot(&bytes[split..grant_end])
            .map_err(|_| PluginError::Unavailable)?,
        runs,
    })
}
