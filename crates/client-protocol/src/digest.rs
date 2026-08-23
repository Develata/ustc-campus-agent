use sha2::{Digest, Sha256};

use crate::value::{UnixMillis, WireText};

const AFFAIRS_GET_DOMAIN: &[u8] = b"m10/affairs.get/payload-digest/v1\0";
const PROCEDURE_ID_TAG: &[u8] = b"procedure_id\0";
const AS_OF_TAG_ABSENT: &[u8] = b"as_of:absent\0";
const AS_OF_TAG_PRESENT: &[u8] = b"as_of:present\0";

pub fn affairs_get_payload_digest(
    procedure_id: &WireText,
    as_of: Option<UnixMillis>,
) -> Result<WireText, AffairsGetDigestError> {
    let mut hasher = Sha256::new();
    hasher.update(AFFAIRS_GET_DOMAIN);
    hasher.update(PROCEDURE_ID_TAG);
    let procedure_bytes = procedure_id.as_str().as_bytes();
    hasher.update(
        (u32::try_from(procedure_bytes.len()).map_err(|_| AffairsGetDigestError::Length)?)
            .to_be_bytes(),
    );
    hasher.update(procedure_bytes);
    match as_of {
        None => hasher.update(AS_OF_TAG_ABSENT),
        Some(value) => {
            hasher.update(AS_OF_TAG_PRESENT);
            hasher.update(value.get().to_be_bytes());
        }
    }
    let digest = hasher.finalize();
    let hex = hex_lower(&digest);
    WireText::parse(hex).map_err(|_| AffairsGetDigestError::Encoding)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AffairsGetDigestError {
    Length,
    Encoding,
}

impl std::fmt::Display for AffairsGetDigestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Length => "affairs.get payload digest input length overflow",
            Self::Encoding => "affairs.get payload digest encoding failed",
        })
    }
}

impl std::error::Error for AffairsGetDigestError {}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn proc_id(value: &str) -> WireText {
        WireText::parse(value).unwrap()
    }

    #[test]
    fn digest_is_lowercase_sha256_hex_without_prefix() {
        let digest = affairs_get_payload_digest(&proc_id("proc:fixture"), None).unwrap();
        assert_eq!(digest.as_str().len(), 64);
        assert!(
            digest
                .as_str()
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        );
    }

    #[test]
    fn digest_is_deterministic_for_same_inputs() {
        let left = affairs_get_payload_digest(&proc_id("proc:fixture"), None).unwrap();
        let right = affairs_get_payload_digest(&proc_id("proc:fixture"), None).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn digest_changes_when_procedure_id_changes() {
        let left = affairs_get_payload_digest(&proc_id("proc:a"), None).unwrap();
        let right = affairs_get_payload_digest(&proc_id("proc:b"), None).unwrap();
        assert_ne!(left, right);
    }

    #[test]
    fn digest_changes_when_as_of_presence_changes() {
        let absent = affairs_get_payload_digest(&proc_id("proc:fixture"), None).unwrap();
        let present =
            affairs_get_payload_digest(&proc_id("proc:fixture"), Some(UnixMillis::new(0))).unwrap();
        assert_ne!(absent, present);
    }

    #[test]
    fn digest_changes_when_as_of_value_changes() {
        let earlier =
            affairs_get_payload_digest(&proc_id("proc:fixture"), Some(UnixMillis::new(1))).unwrap();
        let later =
            affairs_get_payload_digest(&proc_id("proc:fixture"), Some(UnixMillis::new(2))).unwrap();
        assert_ne!(earlier, later);
    }

    #[test]
    fn digest_is_domain_separated_from_plain_sha256() {
        use sha2::Digest;
        let mut naive = Sha256::new();
        naive.update(b"proc:fixture");
        let naive_hex = {
            let bytes = naive.finalize();
            let mut s = String::with_capacity(64);
            for b in bytes {
                s.push(char::from(b"0123456789abcdef"[usize::from(b >> 4)]));
                s.push(char::from(b"0123456789abcdef"[usize::from(b & 0x0f)]));
            }
            s
        };
        let domain = affairs_get_payload_digest(&proc_id("proc:fixture"), None).unwrap();
        assert_ne!(naive_hex, domain.as_str());
    }

    #[test]
    fn digest_is_length_framed_so_prefix_collisions_are_impossible() {
        let left = affairs_get_payload_digest(&proc_id("ab"), None).unwrap();
        let right = affairs_get_payload_digest(&proc_id("a"), None).unwrap();
        assert_ne!(left, right);
        let combined = affairs_get_payload_digest(&proc_id("a:b"), None).unwrap();
        assert_ne!(left, combined);
    }
}
