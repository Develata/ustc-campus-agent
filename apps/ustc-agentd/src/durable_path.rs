use std::collections::BTreeSet;
use std::fs::{self, DirBuilder, File};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

fn current_uid() -> Result<u32, String> {
    crate::unix_identity::effective_uid()
}

fn traversal_exposure_after(
    owner_uid: u32,
    mode: u32,
    current_uid: u32,
    exposed_by_writable_ancestor: bool,
) -> Result<bool, String> {
    if owner_uid != current_uid && owner_uid != 0 {
        return Err("durable state traversal ancestor has foreign ownership".to_owned());
    }
    let owner_private_boundary = owner_uid == current_uid && mode & 0o077 == 0;
    let root_owned_sticky = owner_uid == 0 && mode & 0o1000 != 0;
    if owner_private_boundary {
        Ok(false)
    } else if mode & 0o022 != 0 && !root_owned_sticky {
        Ok(true)
    } else {
        Ok(exposed_by_writable_ancestor)
    }
}

fn validate_traversal_chain(start: &Path) -> Result<(), String> {
    let uid = current_uid()?;
    let mut exposed_by_writable_ancestor = false;
    for ancestor in start.ancestors() {
        let metadata = fs::symlink_metadata(ancestor)
            .map_err(|error| format!("durable state ancestor metadata: {error}"))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err("durable state traversal contains a non-directory or symlink".to_owned());
        }
        exposed_by_writable_ancestor = traversal_exposure_after(
            metadata.uid(),
            metadata.permissions().mode() & 0o7777,
            uid,
            exposed_by_writable_ancestor,
        )?;
    }
    if exposed_by_writable_ancestor {
        return Err("durable state traversal is exposed by a writable ancestor".to_owned());
    }
    Ok(())
}

pub(crate) fn ensure_secure_parent(path: &Path, allow_create: bool) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "durable state parent is unavailable".to_owned())?;

    match fs::symlink_metadata(parent) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && allow_create => {
            let existing_ancestor = parent
                .ancestors()
                .find(|ancestor| fs::symlink_metadata(ancestor).is_ok())
                .ok_or_else(|| "durable state has no existing traversal ancestor".to_owned())?;
            validate_traversal_chain(existing_ancestor)?;
            let mut builder = DirBuilder::new();
            builder.recursive(true).mode(0o700);
            builder
                .create(parent)
                .map_err(|error| format!("durable state parent create: {error}"))?;
        }
        Err(error) => return Err(format!("durable state parent metadata: {error}")),
    }

    validate_traversal_chain(parent)?;
    let metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("durable state parent metadata: {error}"))?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_dir()
        || metadata.permissions().mode() & 0o7777 != 0o700
        || metadata.uid() != current_uid()?
    {
        return Err("durable state parent must be a current-owner 0700 directory".to_owned());
    }
    Ok(())
}

/// Removes only owner-private canonical members created by a fresh
/// composition bootstrap and durably syncs each affected parent directory.
pub(crate) fn rollback_fresh_state_paths(paths: &[std::path::PathBuf]) -> Result<(), String> {
    let uid = current_uid()?;
    let mut parents = BTreeSet::new();
    for path in paths.iter().rev() {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("fresh rollback metadata failed: {error}")),
        };
        ensure_secure_parent(path, false)?;
        let mode = metadata.permissions().mode() & 0o7777;
        if !metadata.file_type().is_file()
            || metadata.uid() != uid
            || metadata.nlink() != 1
            || mode != 0o600
        {
            return Err("fresh rollback member is not an owner-private regular file".to_owned());
        }
        fs::remove_file(path).map_err(|error| format!("fresh rollback remove failed: {error}"))?;
        parents.insert(
            path.parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .ok_or_else(|| "fresh rollback path has no parent".to_owned())?
                .to_path_buf(),
        );
    }
    for parent in parents {
        File::open(&parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| format!("fresh rollback parent sync failed: {error}"))?;
    }
    for path in paths {
        if fs::symlink_metadata(path).is_ok() {
            return Err("fresh rollback left a durable state member".to_owned());
        }
    }
    Ok(())
}

/// Cross-process ownership for one complete durable state set. Directory
/// locks avoid introducing another canonical member and are acquired in path
/// order so overlapping state sets cannot deadlock.
pub(crate) struct StateSetBootstrapLock {
    _directories: Vec<File>,
}

impl StateSetBootstrapLock {
    pub(crate) fn acquire(paths: &[PathBuf]) -> Result<Self, String> {
        let uid = current_uid()?;
        let mut parents = BTreeSet::new();
        for path in paths {
            ensure_secure_parent(path, true)?;
            parents.insert(
                path.parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .ok_or_else(|| "durable state path has no parent".to_owned())?
                    .to_path_buf(),
            );
        }
        let mut directories = Vec::with_capacity(parents.len());
        for parent in parents {
            let directory = File::open(&parent)
                .map_err(|error| format!("bootstrap lock directory open failed: {error}"))?;
            let metadata = directory
                .metadata()
                .map_err(|error| format!("bootstrap lock metadata failed: {error}"))?;
            let parent_mode = metadata.permissions().mode() & 0o7777;
            if !metadata.file_type().is_dir() || metadata.uid() != uid || parent_mode != 0o700 {
                return Err("bootstrap lock directory is not owner-private".to_owned());
            }
            directory
                .lock()
                .map_err(|error| format!("bootstrap directory lock failed: {error}"))?;
            directories.push(directory);
        }
        Ok(Self {
            _directories: directories,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_non_root_traversal_owner_fails_closed() {
        let uid = current_uid().expect("current uid");
        assert!(traversal_exposure_after(uid.saturating_add(1), 0o755, uid, false).is_err());
    }

    #[test]
    fn root_sticky_does_not_erase_existing_exposure() {
        let uid = current_uid().expect("current uid");
        let exposed = traversal_exposure_after(uid, 0o777, uid, false).expect("current owner");
        assert!(exposed);
        assert!(traversal_exposure_after(0, 0o1777, uid, exposed).expect("root sticky"));
    }
}
