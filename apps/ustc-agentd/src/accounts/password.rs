//! Pinned RustCrypto Argon2id adapter. Passwords never enter session evidence.
use super::AccountError;
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};

fn algorithm() -> Result<Argon2<'static>, AccountError> {
    Ok(Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19_456, 2, 1, Some(32)).map_err(|_| AccountError::Unavailable)?,
    ))
}

pub(super) fn valid_submission(password: &str) -> bool {
    password.len() <= 1024 && (12..=256).contains(&password.chars().count())
}

pub(super) fn validate_record(record: &str) -> Result<(), AccountError> {
    if record.len() > 256 {
        return Err(AccountError::Unavailable);
    }
    let parsed = PasswordHash::new(record).map_err(|_| AccountError::Unavailable)?;
    if parsed.algorithm.as_str() != "argon2id"
        || parsed.version != Some(19)
        || parsed.params.get_decimal("m") != Some(19_456)
        || parsed.params.get_decimal("t") != Some(2)
        || parsed.params.get_decimal("p") != Some(1)
        || parsed.params.iter().count() != 3
        || parsed.salt.is_none_or(|salt| salt.len() < 22)
        || parsed.hash.is_none_or(|hash| hash.len() != 32)
    {
        return Err(AccountError::Unavailable);
    }
    Ok(())
}

pub(super) fn hash(password: &str) -> Result<String, AccountError> {
    let salt = SaltString::encode_b64(&super::random_bytes()?[..16])
        .map_err(|_| AccountError::Unavailable)?;
    algorithm()?
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AccountError::Unavailable)
}

pub(super) fn verify(password: &str, record: &str) -> Result<bool, AccountError> {
    validate_record(record)?;
    let record = PasswordHash::new(record).map_err(|_| AccountError::Unavailable)?;
    Ok(algorithm()?
        .verify_password(password.as_bytes(), &record)
        .is_ok())
}
