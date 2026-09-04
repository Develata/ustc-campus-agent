//! plan_ref: docs/plan/modules/20-application-api-host.md#bounded-loopback-local-access-gate
//! Process-local deployment access for the bounded loopback Web composition.
//!
//! This adapter is not M00 identity or product authorization. It verifies one
//! operator-supplied Argon2id PHC string and retains only digests of opaque
//! browser session tokens in memory.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use argon2::password_hash::{PasswordHash, PasswordVerifier};
#[cfg(test)]
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

const HASH_FILE_MAX_BYTES: u64 = 256;
const USERNAME_MAX_BYTES: usize = 64;
const PASSWORD_MIN_BYTES: usize = 12;
const PASSWORD_MAX_BYTES: usize = 1024;
const SESSION_TOKEN_BYTES: usize = 32;
const MAX_SESSIONS: usize = 8;
const LOGIN_FAILURE_LIMIT: u8 = 5;
const LOGIN_COOLDOWN: Duration = Duration::from_secs(60);
const SESSION_IDLE: Duration = Duration::from_secs(30 * 60);
const SESSION_ABSOLUTE: Duration = Duration::from_secs(12 * 60 * 60);
const ARGON2_MEMORY_KIB: u32 = 19_456;
const ARGON2_ITERATIONS: u32 = 2;
const ARGON2_LANES: u32 = 1;
const ARGON2_OUTPUT_BYTES: usize = 32;
const ARGON2_PREFIX: &str = "$argon2id$v=19$m=19456,t=2,p=1$";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoginError {
    InvalidRequest,
    InvalidCredentials,
    RateLimited,
    Internal,
}

#[derive(Clone)]
pub(crate) struct LocalAccessControl {
    inner: Arc<LocalAccessInner>,
}

struct LocalAccessInner {
    username: String,
    password_hash: String,
    runtime: Mutex<RuntimeState>,
}

#[derive(Default)]
struct RuntimeState {
    sessions: BTreeMap<[u8; 32], LocalSession>,
    consecutive_failures: u8,
    blocked_until: Option<Instant>,
}

struct LocalSession {
    issued_at: Instant,
    idle_deadline: Instant,
    absolute_deadline: Instant,
}

impl LocalAccessControl {
    pub(crate) fn from_env() -> Result<Self, String> {
        let username = match std::env::var("UCA_ADMIN_USERNAME") {
            Ok(value) => value,
            Err(std::env::VarError::NotPresent) => "admin".to_owned(),
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err("local access username configuration is invalid".to_owned());
            }
        };
        let verifier_path = match std::env::var("UCA_ADMIN_PASSWORD_HASH_FILE") {
            Ok(value) if !value.is_empty() => value,
            Ok(_) | Err(std::env::VarError::NotPresent) => {
                return Err("local access password verifier path is required".to_owned());
            }
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err("local access password verifier path is invalid".to_owned());
            }
        };
        Self::open(username, Path::new(&verifier_path))
    }

    pub(crate) fn open(username: String, password_hash_path: &Path) -> Result<Self, String> {
        validate_username(&username)?;
        let password_hash = read_password_hash(password_hash_path)?;
        validate_password_hash(&password_hash)?;
        Ok(Self {
            inner: Arc::new(LocalAccessInner {
                username,
                password_hash,
                runtime: Mutex::new(RuntimeState::default()),
            }),
        })
    }

    #[must_use]
    pub(crate) fn username(&self) -> &str {
        &self.inner.username
    }

    pub(crate) fn login(&self, username: &str, password: &str) -> Result<String, LoginError> {
        if !valid_login_field(username, USERNAME_MAX_BYTES) || !valid_password(password) {
            return Err(LoginError::InvalidRequest);
        }

        let now = Instant::now();
        let mut runtime = self
            .inner
            .runtime
            .lock()
            .map_err(|_| LoginError::Internal)?;
        runtime.remove_expired(now);
        if runtime.blocked_until.is_some_and(|deadline| now < deadline) {
            return Err(LoginError::RateLimited);
        }
        runtime.blocked_until = None;

        let parsed =
            PasswordHash::new(&self.inner.password_hash).map_err(|_| LoginError::Internal)?;
        let password_matches = configured_argon2()
            .map_err(|_| LoginError::Internal)?
            .verify_password(password.as_bytes(), &parsed)
            .is_ok();
        let username_matches = username.as_bytes() == self.inner.username.as_bytes();
        if !(password_matches && username_matches) {
            runtime.consecutive_failures = runtime.consecutive_failures.saturating_add(1);
            if runtime.consecutive_failures >= LOGIN_FAILURE_LIMIT {
                runtime.consecutive_failures = 0;
                runtime.blocked_until = now.checked_add(LOGIN_COOLDOWN);
                return Err(LoginError::RateLimited);
            }
            return Err(LoginError::InvalidCredentials);
        }

        runtime.consecutive_failures = 0;
        runtime.blocked_until = None;
        let token = random_bytes::<SESSION_TOKEN_BYTES>().map_err(|_| LoginError::Internal)?;
        let digest = token_digest(&token);
        let absolute_deadline = now
            .checked_add(SESSION_ABSOLUTE)
            .ok_or(LoginError::Internal)?;
        let idle_deadline = now
            .checked_add(SESSION_IDLE)
            .map_or(absolute_deadline, |deadline| {
                deadline.min(absolute_deadline)
            });
        if runtime.sessions.len() >= MAX_SESSIONS
            && let Some(oldest) = runtime
                .sessions
                .iter()
                .min_by_key(|(_, session)| session.issued_at)
                .map(|(digest, _)| *digest)
        {
            runtime.sessions.remove(&oldest);
        }
        runtime.sessions.insert(
            digest,
            LocalSession {
                issued_at: now,
                idle_deadline,
                absolute_deadline,
            },
        );
        Ok(URL_SAFE_NO_PAD.encode(token))
    }

    pub(crate) fn authenticate_cookie(&self, cookie_headers: &[&str]) -> bool {
        let Some(token) = presented_session_token(cookie_headers) else {
            return false;
        };
        let Ok(decoded) = URL_SAFE_NO_PAD.decode(token) else {
            return false;
        };
        let Ok(token): Result<[u8; SESSION_TOKEN_BYTES], _> = decoded.try_into() else {
            return false;
        };
        let digest = token_digest(&token);
        let now = Instant::now();
        let Ok(mut runtime) = self.inner.runtime.lock() else {
            return false;
        };
        runtime.remove_expired(now);
        let Some(session) = runtime.sessions.get_mut(&digest) else {
            return false;
        };
        let Some(next_idle) = now.checked_add(SESSION_IDLE) else {
            runtime.sessions.remove(&digest);
            return false;
        };
        session.idle_deadline = next_idle.min(session.absolute_deadline);
        true
    }

    pub(crate) fn logout_cookie(&self, cookie_headers: &[&str]) {
        let Some(token) = presented_session_token(cookie_headers) else {
            return;
        };
        let Ok(decoded) = URL_SAFE_NO_PAD.decode(token) else {
            return;
        };
        let Ok(token): Result<[u8; SESSION_TOKEN_BYTES], _> = decoded.try_into() else {
            return;
        };
        if let Ok(mut runtime) = self.inner.runtime.lock() {
            runtime.sessions.remove(&token_digest(&token));
        }
    }
}

impl RuntimeState {
    fn remove_expired(&mut self, now: Instant) {
        self.sessions
            .retain(|_, session| now < session.idle_deadline && now < session.absolute_deadline);
    }
}

#[cfg(test)]
pub(crate) fn deterministic_access_for_tests() -> LocalAccessControl {
    let password_hash = hash_password(b"correct horse battery staple", &[7_u8; 16])
        .expect("fixed password is hashable");
    LocalAccessControl {
        inner: Arc::new(LocalAccessInner {
            username: "admin".to_owned(),
            password_hash,
            runtime: Mutex::new(RuntimeState::default()),
        }),
    }
}

fn validate_username(username: &str) -> Result<(), String> {
    if username.is_empty()
        || username.len() > USERNAME_MAX_BYTES
        || !username.bytes().all(|byte| matches!(byte, b'!'..=b'~'))
    {
        return Err("local access username must be 1..64 printable ASCII bytes".to_owned());
    }
    Ok(())
}

fn valid_login_field(value: &str, max_bytes: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max_bytes && !value.contains('\0')
}

fn valid_password(password: &str) -> bool {
    valid_login_field(password, PASSWORD_MAX_BYTES) && password.len() >= PASSWORD_MIN_BYTES
}

fn configured_argon2() -> Result<Argon2<'static>, String> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_LANES,
        Some(ARGON2_OUTPUT_BYTES),
    )
    .map_err(|_| "invalid fixed Argon2 parameters".to_owned())?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

#[cfg(test)]
fn hash_password(password: &[u8], salt: &[u8; 16]) -> Result<String, String> {
    let salt =
        SaltString::encode_b64(salt).map_err(|_| "failed to encode random salt".to_owned())?;
    configured_argon2()?
        .hash_password(password, &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "failed to hash password".to_owned())
}

fn validate_password_hash(value: &str) -> Result<(), String> {
    if value.len() > HASH_FILE_MAX_BYTES as usize || !value.starts_with(ARGON2_PREFIX) {
        return Err("local access password verifier has unsupported Argon2 parameters".to_owned());
    }
    let parts: Vec<&str> = value.split('$').collect();
    if parts.len() != 6
        || !parts[0].is_empty()
        || parts[1] != "argon2id"
        || parts[2] != "v=19"
        || parts[3] != "m=19456,t=2,p=1"
        || STANDARD_NO_PAD
            .decode(parts[4])
            .map_or(true, |salt| salt.len() != 16)
        || STANDARD_NO_PAD
            .decode(parts[5])
            .map_or(true, |output| output.len() != ARGON2_OUTPUT_BYTES)
        || PasswordHash::new(value).is_err()
    {
        return Err("local access password verifier is invalid".to_owned());
    }
    Ok(())
}

fn read_password_hash(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| "local access password verifier is missing or unreadable".to_owned())?;
    let current_uid = crate::unix_identity::effective_uid()
        .map_err(|_| "local access current uid is unavailable".to_owned())?;
    if !metadata.file_type().is_file()
        || metadata.permissions().mode() & 0o777 != 0o600
        || metadata.nlink() != 1
        || metadata.uid() != current_uid
        || metadata.len() > HASH_FILE_MAX_BYTES
    {
        return Err("local access password verifier metadata is unsafe".to_owned());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "local access password verifier is missing or unreadable".to_owned())?;
    let opened = file
        .metadata()
        .map_err(|_| "local access password verifier metadata is unavailable".to_owned())?;
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err("local access password verifier changed while opening".to_owned());
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(HASH_FILE_MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "local access password verifier is unreadable".to_owned())?;
    if bytes.len() > HASH_FILE_MAX_BYTES as usize {
        return Err("local access password verifier is too large".to_owned());
    }
    let value = String::from_utf8(bytes)
        .map_err(|_| "local access password verifier must be UTF-8".to_owned())?;
    if value.trim() != value {
        return Err("local access password verifier must contain exactly one PHC value".to_owned());
    }
    Ok(value)
}

fn random_bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut bytes = [0_u8; N];
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(|_| "operating-system randomness is unavailable".to_owned())?;
    Ok(bytes)
}

fn token_digest(token: &[u8; SESSION_TOKEN_BYTES]) -> [u8; 32] {
    Sha256::digest(token).into()
}

fn presented_session_token<'a>(cookie_headers: &'a [&'a str]) -> Option<&'a str> {
    let mut token = None;
    for header in cookie_headers {
        for pair in header.split(';') {
            let Some((name, value)) = pair.trim().split_once('=') else {
                continue;
            };
            if name == "uca_session" {
                if token.is_some() || value.is_empty() {
                    return None;
                }
                token = Some(value);
            }
        }
    }
    token
}

#[cfg(test)]
mod tests {
    use super::*;

    fn access() -> LocalAccessControl {
        let password_hash = hash_password(b"correct horse battery staple", &[7_u8; 16])
            .expect("fixed test password hash");
        LocalAccessControl {
            inner: Arc::new(LocalAccessInner {
                username: "admin".to_owned(),
                password_hash,
                runtime: Mutex::new(RuntimeState::default()),
            }),
        }
    }

    #[test]
    fn login_mints_one_usable_logout_capable_cookie() {
        let access = access();
        let token = access
            .login("admin", "correct horse battery staple")
            .expect("valid test login");
        let cookie = format!("other=x; uca_session={token}");
        assert!(access.authenticate_cookie(&[&cookie]));
        access.logout_cookie(&[&cookie]);
        assert!(!access.authenticate_cookie(&[&cookie]));
    }

    #[test]
    fn fifth_bad_login_enters_cooldown_without_session() {
        let access = access();
        for _ in 0..4 {
            assert_eq!(
                access.login("admin", "wrong password value"),
                Err(LoginError::InvalidCredentials)
            );
        }
        assert_eq!(
            access.login("admin", "wrong password value"),
            Err(LoginError::RateLimited)
        );
        assert_eq!(
            access.login("admin", "correct horse battery staple"),
            Err(LoginError::RateLimited)
        );
    }

    #[test]
    fn duplicate_or_malformed_cookie_is_rejected() {
        let access = access();
        let token = access
            .login("admin", "correct horse battery staple")
            .expect("valid test login");
        assert!(
            !access.authenticate_cookie(&[&format!("uca_session={token}; uca_session={token}")])
        );
        assert!(!access.authenticate_cookie(&["uca_session=not-base64!"]));
    }

    #[test]
    fn idle_and_absolute_expiry_both_reject_and_remove_the_session() {
        for expire_absolute_deadline in [false, true] {
            let access = access();
            let token = access
                .login("admin", "correct horse battery staple")
                .expect("valid test login");
            let cookie = format!("uca_session={token}");
            let now = Instant::now();
            let past = now
                .checked_sub(Duration::from_secs(1))
                .expect("test clock supports one second of history");
            let future = now
                .checked_add(Duration::from_secs(60))
                .expect("test clock supports one minute of future");
            {
                let mut runtime = access.inner.runtime.lock().expect("test runtime lock");
                let session = runtime
                    .sessions
                    .values_mut()
                    .next()
                    .expect("login inserted one session");
                if expire_absolute_deadline {
                    session.idle_deadline = future;
                    session.absolute_deadline = past;
                } else {
                    session.idle_deadline = past;
                    session.absolute_deadline = future;
                }
            }

            assert!(!access.authenticate_cookie(&[&cookie]));
            assert!(
                access
                    .inner
                    .runtime
                    .lock()
                    .expect("test runtime lock")
                    .sessions
                    .is_empty()
            );
        }
    }
}
