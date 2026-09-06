//! One opened package directory owns every read and the exclusive output creation.
use rustix::fs::{Mode, OFlags, open, openat};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path},
};

pub(super) struct PackageDirectory(File);
impl PackageDirectory {
    pub(super) fn open(root: &Path) -> Result<Self, String> {
        let absolute = if root.is_absolute() {
            root.to_owned()
        } else {
            std::env::current_dir()
                .map_err(|_| "working directory unavailable")?
                .join(root)
        };
        let flags = directory_flags();
        let mut directory =
            open("/", flags, Mode::empty()).map_err(|_| "package directory unavailable")?;
        for component in absolute.components() {
            match component {
                Component::RootDir | Component::CurDir => {}
                Component::Normal(name) => {
                    directory = openat(&directory, name, flags, Mode::empty())
                        .map_err(|_| "package directory is unavailable or unsafe")?;
                }
                _ => return Err("package directory parent traversal is rejected".to_owned()),
            }
        }
        Ok(Self(File::from(directory)))
    }
    pub(super) fn read(&self, relative: &str, limit: usize) -> Result<Vec<u8>, String> {
        if relative.is_empty()
            || relative.len() > 1024
            || relative.contains(['\\', ':'])
            || relative.chars().any(char::is_control)
            || relative
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err("unsafe package resource path".to_owned());
        }
        let mut directory = self
            .0
            .try_clone()
            .map_err(|_| "package directory unavailable")?;
        let mut parts = relative.split('/').peekable();
        while let Some(part) = parts.next() {
            if parts.peek().is_some() {
                directory = File::from(
                    openat(&directory, part, directory_flags(), Mode::empty())
                        .map_err(|_| "package resource directory is unavailable or unsafe")?,
                );
            } else {
                let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
                let mut file = File::from(
                    openat(&directory, part, flags, Mode::empty())
                        .map_err(|_| "package resource is unavailable or unsafe")?,
                );
                let metadata = file
                    .metadata()
                    .map_err(|_| "package resource unavailable")?;
                if !metadata.is_file() || metadata.len() > limit as u64 {
                    return Err("package resource rejected".to_owned());
                }
                let mut bytes = Vec::new();
                Read::by_ref(&mut file)
                    .take((limit + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(|_| "package resource read failed")?;
                if bytes.len() > limit {
                    return Err("package resource too large".to_owned());
                }
                return Ok(bytes);
            }
        }
        Err("unsafe package resource path".to_owned())
    }
    pub(super) fn create_sidecar(&self, bytes: &[u8]) -> Result<(), String> {
        let flags =
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let mut file = File::from(
            openat(
                &self.0,
                "configuration.json",
                flags,
                Mode::from_raw_mode(0o644),
            )
            .map_err(|_| "configuration.json already exists or cannot be created")?,
        );
        file.write_all(bytes)
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.sync_all())
            .and_then(|()| self.0.sync_all())
            .map_err(|_| "configuration write failed".to_owned())
    }
}
fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}
