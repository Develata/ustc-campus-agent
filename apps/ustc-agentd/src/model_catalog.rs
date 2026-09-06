//! Immutable operator-selected M50 profiles; only allowlisted identity reaches clients.
use crate::chat_provider::{ChatProvider, ProviderIdentity};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, sync::Arc};
#[cfg(unix)]
use std::{fs::OpenOptions, io::Read};

const MAX_FILE_BYTES: u64 = 64 * 1024;
pub(crate) const DEFAULT_MODEL_ID: &str = "default";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum ModelSelectionFieldDto {
    #[default]
    #[serde(skip)]
    Absent,
    Value(String),
}
impl ModelSelectionFieldDto {
    pub(crate) fn is_absent(&self) -> bool {
        matches!(self, Self::Absent)
    }
    pub(crate) fn selected(&self, explicit: bool) -> Option<&str> {
        match (explicit, self) {
            (false, Self::Absent) => Some(DEFAULT_MODEL_ID),
            (true, Self::Value(value)) if valid_id(value) => Some(value),
            _ => None,
        }
    }
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModelCatalogError;
#[derive(Clone)]
pub(crate) struct ModelCatalog {
    entries: Arc<BTreeMap<String, ModelEntry>>,
}
struct ModelEntry {
    label: String,
    provider: ChatProvider,
}
#[derive(Serialize)]
pub(crate) struct ModelCatalogDto {
    schema: &'static str,
    default_id: &'static str,
    models: Vec<ModelEntryDto>,
}
#[derive(Serialize)]
struct ModelEntryDto {
    id: String,
    label: String,
    provider: ProviderIdentity,
    tool_calling: bool,
    context_limit_tokens: Option<u64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogFile {
    schema: String,
    models: Vec<ModelConfig>,
}
#[derive(Deserialize)]
#[serde(tag = "mode", deny_unknown_fields)]
enum ModelConfig {
    #[serde(rename = "mock")]
    Mock { id: String, label: String },
    #[serde(rename = "local-chat")]
    LocalChat {
        id: String,
        label: String,
        base_url: String,
        model: String,
        api_key_file: String,
        timeout_ms: u64,
        context_limit_tokens: u64,
    },
    #[serde(rename = "openai-compatible")]
    OpenAiCompatible {
        id: String,
        label: String,
        base_url: String,
        model: String,
        api_key_file: String,
        timeout_ms: u64,
        context_limit_tokens: u64,
    },
}
impl ModelCatalog {
    pub(crate) fn single(provider: ChatProvider) -> Self {
        Self {
            entries: Arc::new(BTreeMap::from([(
                DEFAULT_MODEL_ID.to_owned(),
                ModelEntry {
                    label: default_label(&provider),
                    provider,
                },
            )])),
        }
    }
    pub(crate) fn from_env() -> Result<Self, ModelCatalogError> {
        let default = ChatProvider::from_env().map_err(|_| ModelCatalogError)?;
        match std::env::var_os("UCA_AGENT_MODELS_FILE") {
            None => Ok(Self::single(default)),
            Some(path) => Self::from_file(default, Path::new(&path)),
        }
    }
    #[cfg(unix)]
    pub(crate) fn from_file(default: ChatProvider, path: &Path) -> Result<Self, ModelCatalogError> {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(path).map_err(|_| ModelCatalogError)?;
        let metadata = file.metadata().map_err(|_| ModelCatalogError)?;
        if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
            return Err(ModelCatalogError);
        }
        let mut bytes = Vec::new();
        file.take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| ModelCatalogError)?;
        Self::from_bytes(default, &bytes)
    }
    #[cfg(not(unix))]
    pub(crate) fn from_file(
        _default: ChatProvider,
        _path: &Path,
    ) -> Result<Self, ModelCatalogError> {
        Err(ModelCatalogError)
    }
    fn from_bytes(default: ChatProvider, bytes: &[u8]) -> Result<Self, ModelCatalogError> {
        if bytes.len() > MAX_FILE_BYTES as usize {
            return Err(ModelCatalogError);
        }
        let file: CatalogFile = serde_json::from_slice(bytes).map_err(|_| ModelCatalogError)?;
        if file.schema != "uca-agent-models/v1" || file.models.len() > 15 {
            return Err(ModelCatalogError);
        }
        let mut entries = BTreeMap::from([(
            DEFAULT_MODEL_ID.to_owned(),
            ModelEntry {
                label: default_label(&default),
                provider: default,
            },
        )]);
        for config in file.models {
            let (id, label, provider) = match config {
                ModelConfig::Mock { id, label } => (id, label, ChatProvider::deterministic_mock()),
                ModelConfig::LocalChat {
                    id,
                    label,
                    base_url,
                    model,
                    api_key_file,
                    timeout_ms,
                    context_limit_tokens,
                } => {
                    if !Path::new(&api_key_file).is_absolute() {
                        return Err(ModelCatalogError);
                    }
                    (
                        id,
                        label,
                        ChatProvider::local_chat(
                            &base_url,
                            &model,
                            Path::new(&api_key_file),
                            timeout_ms,
                            context_limit_tokens,
                        )
                        .map_err(|_| ModelCatalogError)?,
                    )
                }
                ModelConfig::OpenAiCompatible {
                    id,
                    label,
                    base_url,
                    model,
                    api_key_file,
                    timeout_ms,
                    context_limit_tokens,
                } => {
                    if !Path::new(&api_key_file).is_absolute() {
                        return Err(ModelCatalogError);
                    }
                    (
                        id,
                        label,
                        ChatProvider::openai_compatible(
                            &base_url,
                            &model,
                            Path::new(&api_key_file),
                            timeout_ms,
                            context_limit_tokens,
                            false,
                        )
                        .map_err(|_| ModelCatalogError)?,
                    )
                }
            };
            if !valid_id(&id)
                || label.trim().is_empty()
                || label.len() > 128
                || label.chars().any(char::is_control)
                || entries.contains_key(&id)
            {
                return Err(ModelCatalogError);
            }
            entries.insert(id, ModelEntry { label, provider });
        }
        Ok(Self {
            entries: Arc::new(entries),
        })
    }
    pub(crate) fn resolve(&self, id: &str) -> Result<ChatProvider, ModelCatalogError> {
        self.entries
            .get(id)
            .map(|entry| entry.provider.clone())
            .ok_or(ModelCatalogError)
    }
    pub(crate) fn view(&self) -> ModelCatalogDto {
        ModelCatalogDto {
            schema: "uca-agent-models/v1",
            default_id: DEFAULT_MODEL_ID,
            models: self
                .entries
                .iter()
                .map(|(id, entry)| ModelEntryDto {
                    id: id.clone(),
                    label: entry.label.clone(),
                    provider: entry.provider.identity(),
                    tool_calling: entry.provider.tool_calling_enabled(),
                    context_limit_tokens: entry.provider.context_limit_tokens(),
                })
                .collect(),
        }
    }
}

fn default_label(provider: &ChatProvider) -> String {
    let model = provider.identity().model;
    let mut end = model.len().min(128);
    while !model.is_char_boundary(end) {
        end -= 1;
    }
    model[..end].to_owned()
}
impl From<ChatProvider> for ModelCatalog {
    fn from(provider: ChatProvider) -> Self {
        Self::single(provider)
    }
}

#[cfg(test)]
mod tests;
