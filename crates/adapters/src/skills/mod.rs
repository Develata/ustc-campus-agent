//! Bounded Agent Skills parsing. Parsed content never grants execution authority.

mod resources;
pub use resources::{
    DeclaredTextResource, VerifiedTextResource, load_declared_skill, load_declared_text_resource,
};

use serde::Deserialize;
use std::{collections::BTreeMap, fmt};

pub const MAX_SKILL_BYTES: usize = 64 * 1024;
pub const MAX_FRONTMATTER_BYTES: usize = 16 * 1024;
pub const MAX_RESOURCE_BYTES: usize = 64 * 1024;
pub const MAX_SELECTED_CONTEXT_BYTES: usize = 256 * 1024;
const MAX_METADATA_ENTRIES: usize = 32;

/// Stable, redacted errors deliberately discard parser snippets and filesystem details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillError {
    TooLarge,
    InvalidUtf8,
    MissingFrontmatter,
    InvalidFrontmatter,
    InvalidName,
    DirectoryMismatch,
    InvalidDescription,
    InvalidCompatibility,
    TooManyMetadataEntries,
    InvalidResourcePath,
    UndeclaredResource,
    DuplicateResource,
    UnsafeResource,
    ResourceUnavailable,
    DigestMismatch,
    UnsupportedPlatform,
}

impl fmt::Display for SkillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "skill context rejected: {self:?}")
    }
}
impl std::error::Error for SkillError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    name: String,
    description: String,
    #[serde(default, deserialize_with = "present_string")]
    license: Option<String>,
    #[serde(default, deserialize_with = "present_string")]
    compatibility: Option<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
    #[serde(rename = "allowed-tools", default, deserialize_with = "present_string")]
    allowed_tools: Option<String>,
}

fn present_string<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    String::deserialize(deserializer).map(Some)
}

/// Validated syntax only. The application must authorize each context projection.
pub struct ParsedSkill {
    frontmatter: Frontmatter,
    body: String,
}

impl fmt::Debug for ParsedSkill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedSkill")
            .field("body_bytes", &self.body.len())
            .finish_non_exhaustive()
    }
}

impl ParsedSkill {
    pub fn parse(directory_name: &str, bytes: &[u8]) -> Result<Self, SkillError> {
        if bytes.len() > MAX_SKILL_BYTES {
            return Err(SkillError::TooLarge);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| SkillError::InvalidUtf8)?;
        let (header, body) = split_frontmatter(text)?;
        let options = serde_saphyr::options! {
            budget: serde_saphyr::budget! {
                max_reader_input_bytes: Some(MAX_FRONTMATTER_BYTES),
                max_depth: 4,
                flow_nesting_limit: 4,
                max_documents: 1,
                max_nodes: 160,
                max_events: 256,
                max_total_scalar_bytes: MAX_FRONTMATTER_BYTES,
                max_total_comment_bytes: MAX_FRONTMATTER_BYTES,
                max_aliases: 0,
                max_anchors: 0,
                max_merge_keys: 0,
                max_inclusion_depth: 0,
            },
            alias_limits: serde_saphyr::alias_limits! {
                max_total_replayed_events: 0,
                max_replay_stack_depth: 0,
                max_alias_expansions_per_anchor: 0,
            },
            duplicate_keys: serde_saphyr::DuplicateKeyPolicy::Error,
            merge_keys: serde_saphyr::MergeKeyPolicy::Error,
            reject_unsupported_tags: true,
            no_schema: true,
            strict_booleans: true,
            emit_comments: false,
            with_snippet: false,
        };
        let frontmatter: Frontmatter = serde_saphyr::from_str_with_options(header, options)
            .map_err(|_| SkillError::InvalidFrontmatter)?;
        if !valid_name(&frontmatter.name) {
            return Err(SkillError::InvalidName);
        }
        if frontmatter.name != directory_name {
            return Err(SkillError::DirectoryMismatch);
        }
        if frontmatter.description.trim().is_empty()
            || frontmatter.description.chars().count() > 1024
        {
            return Err(SkillError::InvalidDescription);
        }
        if frontmatter
            .compatibility
            .as_ref()
            .is_some_and(|value| value.trim().is_empty() || value.chars().count() > 500)
        {
            return Err(SkillError::InvalidCompatibility);
        }
        if frontmatter.metadata.len() > MAX_METADATA_ENTRIES {
            return Err(SkillError::TooManyMetadataEntries);
        }
        Ok(Self {
            frontmatter,
            body: body.to_owned(),
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.frontmatter.name
    }
    #[must_use]
    pub fn description(&self) -> &str {
        &self.frontmatter.description
    }
    #[must_use]
    pub fn license(&self) -> Option<&str> {
        self.frontmatter.license.as_deref()
    }
    #[must_use]
    pub fn compatibility(&self) -> Option<&str> {
        self.frontmatter.compatibility.as_deref()
    }
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.frontmatter.metadata
    }
    /// Advisory source text only: never an effective grant or a tool definition.
    #[must_use]
    pub fn advisory_allowed_tools(&self) -> Option<&str> {
        self.frontmatter.allowed_tools.as_deref()
    }
    /// Read only lower-trust guidance; caller owns lazy disclosure and authorization.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn split_frontmatter(text: &str) -> Result<(&str, &str), SkillError> {
    let mut lines = text.split_inclusive('\n');
    let first = lines.next().ok_or(SkillError::MissingFrontmatter)?;
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return Err(SkillError::MissingFrontmatter);
    }
    let start = first.len();
    let mut offset = start;
    for line in lines {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            if offset - start > MAX_FRONTMATTER_BYTES {
                return Err(SkillError::TooLarge);
            }
            return Ok((&text[start..offset], &text[offset + line.len()..]));
        }
        offset += line.len();
        if offset - start > MAX_FRONTMATTER_BYTES {
            return Err(SkillError::TooLarge);
        }
    }
    Err(SkillError::MissingFrontmatter)
}

#[cfg(test)]
mod tests;
