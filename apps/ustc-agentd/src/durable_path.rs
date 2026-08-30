use std::fs::{self, DirBuilder};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::Path;

fn current_uid() -> Result<u32, String> {
    fs::metadata("/proc/self")
        .map(|metadata| metadata.uid())
        .map_err(|error| format!("durable state current uid: {error}"))
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
