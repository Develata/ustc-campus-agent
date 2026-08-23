use std::collections::BTreeMap;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use ustc_campus_agent_client_protocol::WireText;

type HmacSha256 = Hmac<Sha256>;
const CAPABILITY_DOMAIN: &[u8] = b"m10/public-capability/v1\0";
const DIGEST_DOMAIN: &[u8] = b"m10/public-capability-digest/v1\0";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredPublicAuthorization {
    digest_hex: String,
    key_version: u16,
}

impl std::fmt::Debug for StoredPublicAuthorization {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StoredPublicAuthorization")
            .field("digest_hex", &"[REDACTED]")
            .field("key_version", &"[REDACTED]")
            .finish()
    }
}

impl StoredPublicAuthorization {
    #[must_use]
    pub fn digest_hex(&self) -> &str {
        &self.digest_hex
    }
    #[must_use]
    pub const fn key_version(&self) -> u16 {
        self.key_version
    }
}

#[derive(Clone)]
pub struct CapabilityIssuer {
    keys: BTreeMap<u16, [u8; 32]>,
    current_version: u16,
}

impl CapabilityIssuer {
    pub fn new(
        keys: BTreeMap<u16, [u8; 32]>,
        current_version: u16,
    ) -> Result<Self, CapabilityError> {
        if current_version == 0 || !keys.contains_key(&current_version) {
            return Err(CapabilityError::UnknownKeyVersion);
        }
        Ok(Self {
            keys,
            current_version,
        })
    }

    pub fn mint(
        &self,
        command_id: &str,
        capsule_digest: &str,
    ) -> Result<(WireText, StoredPublicAuthorization), CapabilityError> {
        self.reproduce_with_version(self.current_version, command_id, capsule_digest)
    }

    pub fn reproduce(
        &self,
        stored: &StoredPublicAuthorization,
        command_id: &str,
        capsule_digest: &str,
    ) -> Result<WireText, CapabilityError> {
        let (bearer, candidate) =
            self.reproduce_with_version(stored.key_version, command_id, capsule_digest)?;
        if !constant_time_eq(
            candidate.digest_hex.as_bytes(),
            stored.digest_hex.as_bytes(),
        ) {
            return Err(CapabilityError::StoredDigestMismatch);
        }
        Ok(bearer)
    }

    pub fn verify(&self, stored: &StoredPublicAuthorization, presented: &str) -> bool {
        let Ok(raw) = URL_SAFE_NO_PAD.decode(presented) else {
            return false;
        };
        if raw.len() != 32 {
            return false;
        }
        let digest = bearer_digest(&raw);
        constant_time_eq(digest.as_bytes(), stored.digest_hex.as_bytes())
    }

    fn reproduce_with_version(
        &self,
        key_version: u16,
        command_id: &str,
        capsule_digest: &str,
    ) -> Result<(WireText, StoredPublicAuthorization), CapabilityError> {
        let key = self
            .keys
            .get(&key_version)
            .ok_or(CapabilityError::UnknownKeyVersion)?;
        let command_len =
            u16::try_from(command_id.len()).map_err(|_| CapabilityError::CommandTooLong)?;
        let mut mac =
            HmacSha256::new_from_slice(key).map_err(|_| CapabilityError::UnknownKeyVersion)?;
        mac.update(CAPABILITY_DOMAIN);
        mac.update(&command_len.to_be_bytes());
        mac.update(command_id.as_bytes());
        mac.update(capsule_digest.as_bytes());
        let raw = mac.finalize().into_bytes();
        let bearer_text = URL_SAFE_NO_PAD.encode(raw);
        let bearer = WireText::parse(bearer_text).map_err(|_| CapabilityError::InvalidBearer)?;
        let digest_hex = bearer_digest(&raw);
        Ok((
            bearer,
            StoredPublicAuthorization {
                digest_hex,
                key_version,
            },
        ))
    }
}

fn bearer_digest(raw: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(DIGEST_DOMAIN);
    hasher.update(raw);
    hex_lower(&hasher.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

pub fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let max = left.len().max(right.len());
    for index in 0..max {
        difference |= usize::from(
            left.get(index).copied().unwrap_or(0) ^ right.get(index).copied().unwrap_or(0),
        );
    }
    difference == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityError {
    UnknownKeyVersion,
    CommandTooLong,
    InvalidBearer,
    StoredDigestMismatch,
}

impl std::fmt::Display for CapabilityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownKeyVersion => "capability key version is unavailable",
            Self::CommandTooLong => "command identity exceeds capability KDF bound",
            Self::InvalidBearer => "capability bearer encoding failed",
            Self::StoredDigestMismatch => "stored capability digest mismatch",
        })
    }
}
impl std::error::Error for CapabilityError {}
