use super::{MAX_RESOURCE_BYTES, ParsedSkill, SkillError};
use std::{fmt, fs::File, io::Read, path::Path};
use ustc_campus_agent_core::invocation::Sha256Digest;

/// Exact reviewed package declaration; construction is not admission or a grant.
#[derive(Clone)]
pub struct DeclaredTextResource {
    path: String,
    digest: Sha256Digest,
}

impl fmt::Debug for DeclaredTextResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeclaredTextResource")
            .finish_non_exhaustive()
    }
}

impl DeclaredTextResource {
    pub fn new(path: &str, digest: Sha256Digest) -> Result<Self, SkillError> {
        validate_relative_path(path)?;
        Ok(Self {
            path: path.to_owned(),
            digest,
        })
    }
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
}

/// Bounded UTF-8 text checked against its exact declared artifact digest.
pub struct VerifiedTextResource {
    path: String,
    digest: Sha256Digest,
    text: String,
}
impl fmt::Debug for VerifiedTextResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerifiedTextResource")
            .field("bytes", &self.text.len())
            .finish_non_exhaustive()
    }
}
impl VerifiedTextResource {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Reads only an allowlisted artifact. The application supplies a reviewed package root
/// and rechecks installation/grants before calling. Never scans directories.
/// Unix uses descriptor-relative no-follow traversal; unsupported hosts fail closed.
pub fn load_declared_text_resource(
    package_root: &Path,
    declarations: &[DeclaredTextResource],
    requested_path: &str,
) -> Result<VerifiedTextResource, SkillError> {
    validate_relative_path(requested_path)?;
    if declarations.len() > 256 {
        return Err(SkillError::TooLarge);
    }
    let mut matches = declarations
        .iter()
        .filter(|entry| entry.path == requested_path);
    let declaration = matches.next().ok_or(SkillError::UndeclaredResource)?;
    if matches.next().is_some() {
        return Err(SkillError::DuplicateResource);
    }
    let file = open_contained_regular_file(package_root, requested_path)?;
    let metadata = file
        .metadata()
        .map_err(|_| SkillError::ResourceUnavailable)?;
    if !metadata.is_file() {
        return Err(SkillError::UnsafeResource);
    }
    if metadata.len() > MAX_RESOURCE_BYTES as u64 {
        return Err(SkillError::TooLarge);
    }
    let mut bytes = Vec::new();
    file.take((MAX_RESOURCE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| SkillError::ResourceUnavailable)?;
    if bytes.len() > MAX_RESOURCE_BYTES {
        return Err(SkillError::TooLarge);
    }
    if Sha256Digest::from_bytes(&bytes) != declaration.digest {
        return Err(SkillError::DigestMismatch);
    }
    let text = String::from_utf8(bytes).map_err(|_| SkillError::InvalidUtf8)?;
    Ok(VerifiedTextResource {
        path: declaration.path.clone(),
        digest: declaration.digest.clone(),
        text,
    })
}

/// Loads SKILL.md from a declared directory after checking bytes/digest/containment.
pub fn load_declared_skill(
    package_root: &Path,
    declarations: &[DeclaredTextResource],
    requested_path: &str,
) -> Result<ParsedSkill, SkillError> {
    validate_relative_path(requested_path)?;
    let mut parts = requested_path.rsplit('/');
    if parts.next() != Some("SKILL.md") {
        return Err(SkillError::InvalidResourcePath);
    }
    let directory_name = parts.next().ok_or(SkillError::InvalidResourcePath)?;
    let resource = load_declared_text_resource(package_root, declarations, requested_path)?;
    ParsedSkill::parse(directory_name, resource.text.as_bytes())
}

fn validate_relative_path(path: &str) -> Result<(), SkillError> {
    if path.is_empty()
        || path.len() > 1024
        || path.contains(['\\', ':', '\0'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(SkillError::InvalidResourcePath);
    }
    Ok(())
}

#[cfg(unix)]
fn open_contained_regular_file(package_root: &Path, relative: &str) -> Result<File, SkillError> {
    use rustix::fs::{Mode, OFlags, open, openat};
    use std::path::Component;
    // Walk root too: no trusted-root ancestor may quietly become a symlink.
    if !package_root.is_absolute() {
        return Err(SkillError::InvalidResourcePath);
    }
    let directory_flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory =
        open("/", directory_flags, Mode::empty()).map_err(|_| SkillError::ResourceUnavailable)?;
    for component in package_root.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                directory = openat(&directory, name, directory_flags, Mode::empty())
                    .map_err(|_| SkillError::UnsafeResource)?;
            }
            _ => return Err(SkillError::InvalidResourcePath),
        }
    }
    let mut parts = relative.split('/').peekable();
    while let Some(part) = parts.next() {
        if parts.peek().is_some() {
            directory = openat(&directory, part, directory_flags, Mode::empty())
                .map_err(|_| SkillError::UnsafeResource)?;
        } else {
            // NONBLOCK ensures a hostile FIFO cannot stall before regular-file validation.
            let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
            let file = openat(&directory, part, flags, Mode::empty())
                .map_err(|_| SkillError::UnsafeResource)?;
            return Ok(File::from(file));
        }
    }
    Err(SkillError::InvalidResourcePath)
}

#[cfg(not(unix))]
fn open_contained_regular_file(_: &Path, _: &str) -> Result<File, SkillError> {
    Err(SkillError::UnsupportedPlatform)
}
