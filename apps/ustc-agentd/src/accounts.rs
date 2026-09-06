//! Bounded local-operator account application and durable session adapter.
//! Shared catalog/source composition stays outside this module.
mod password;
mod persistence;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::PathBuf;
use ustc_campus_agent_core::identity::{SessionId, TenantId, UserId};
use ustc_campus_agent_core::session::{
    self, AuthAdapterId, CredentialEvidenceDigest, OpenSession, RevokeSession, SessionCommand,
    SessionCredentialEvidence, SessionDuration, SessionEvent, SessionInstant, SessionPolicy,
    SessionSnapshot,
};

const SESSION_MILLIS: u64 = 8 * 60 * 60 * 1000;
const WINDOW_MILLIS: u64 = 15 * 60 * 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AccountError {
    InvalidRequest,
    AuthenticationFailed,
    Unauthenticated,
    RateLimited,
    Unavailable,
}

#[derive(Clone)]
pub(crate) struct AccountService {
    config_path: PathBuf,
    state_path: PathBuf,
    dummy_hash: String,
}

#[derive(Clone, Serialize)]
pub(crate) struct AdmittedSubject {
    tenant_id: TenantId,
    user_id: UserId,
    login_name: String,
    administrator: bool,
    expires_at: u64,
}
impl AdmittedSubject {
    pub(crate) fn tenant(&self) -> &TenantId {
        &self.tenant_id
    }
    pub(crate) fn user(&self) -> &UserId {
        &self.user_id
    }
    pub(crate) fn is_administrator(&self) -> bool {
        self.administrator
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    schema: String,
    tenant_id: TenantId,
    accounts: Vec<Account>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Account {
    user_id: UserId,
    login_name: String,
    password_hash: String,
    active: bool,
    administrator: bool,
    credential_generation: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Store {
    schema: String,
    revision: u64,
    sessions: BTreeMap<String, Binding>,
    attempts: BTreeMap<String, Vec<u64>>,
}
impl Default for Store {
    fn default() -> Self {
        Self {
            schema: "platform-local-account-sessions/v1".into(),
            revision: 0,
            sessions: BTreeMap::new(),
            attempts: BTreeMap::new(),
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    tenant_id: TenantId,
    user_id: UserId,
    generation: u64,
    credential_reference: String,
    events: Vec<SessionEvent>,
}

impl AccountService {
    pub(crate) fn from_environment() -> Result<Option<Self>, AccountError> {
        let Some(config) = std::env::var_os("USTC_ACCOUNT_CONFIG") else {
            if std::env::var_os("USTC_ACCOUNT_STATE").is_some() {
                return Err(AccountError::Unavailable);
            }
            return Ok(None);
        };
        let state = std::env::var_os("USTC_ACCOUNT_STATE").ok_or(AccountError::Unavailable)?;
        Self::open(PathBuf::from(config), PathBuf::from(state)).map(Some)
    }

    pub(crate) fn open(config_path: PathBuf, state_path: PathBuf) -> Result<Self, AccountError> {
        if !config_path.is_absolute() || !state_path.is_absolute() || config_path == state_path {
            return Err(AccountError::Unavailable);
        }
        let service = Self {
            config_path,
            state_path,
            dummy_hash: password::hash(&random_hex()?)?,
        };
        let _lock = persistence::lock(&service.state_path)?;
        service.configuration()?;
        persistence::initialize(&service.state_path)?;
        service.read_store()?;
        Ok(service)
    }

    fn configuration(&self) -> Result<Configuration, AccountError> {
        let bytes = persistence::read_private(&self.config_path, 256 * 1024)?;
        let config: Configuration =
            serde_json::from_slice(&bytes).map_err(|_| AccountError::Unavailable)?;
        if config.schema != "platform-local-accounts/v1"
            || config.accounts.is_empty()
            || config.accounts.len() > 128
        {
            return Err(AccountError::Unavailable);
        }
        let mut names = BTreeSet::new();
        let mut users = BTreeSet::new();
        for account in &config.accounts {
            if !valid_name(&account.login_name)
                || !names.insert(&account.login_name)
                || !users.insert(&account.user_id)
                || account.credential_generation == 0
            {
                return Err(AccountError::Unavailable);
            }
            password::validate_record(&account.password_hash)?;
        }
        if config
            .accounts
            .iter()
            .filter(|a| a.active && a.administrator)
            .count()
            != 1
        {
            return Err(AccountError::Unavailable);
        }
        Ok(config)
    }

    fn read_store(&self) -> Result<Store, AccountError> {
        let store = persistence::read_store(&self.state_path)?;
        if store.schema != "platform-local-account-sessions/v1"
            || store.sessions.len() > 4096
            || store.attempts.len() > 1024
            || store.attempts.iter().any(|(key, timestamps)| {
                (key != "loopback" && !is_verifier(key)) || timestamps.len() > 30
            })
        {
            return Err(AccountError::Unavailable);
        }
        for (verifier, binding) in &store.sessions {
            if !is_verifier(verifier)
                || binding.generation == 0
                || !is_verifier(&binding.credential_reference)
                || binding.events.is_empty()
                || binding.events.len() > 2
            {
                return Err(AccountError::Unavailable);
            }
            let snapshot = snapshot(binding)?;
            if snapshot.tenant_id() != &binding.tenant_id || snapshot.user_id() != &binding.user_id
            {
                return Err(AccountError::Unavailable);
            }
        }
        Ok(store)
    }

    pub(crate) fn login(
        &self,
        name: &str,
        submitted: &str,
    ) -> Result<(String, AdmittedSubject), AccountError> {
        if !valid_name(name) || !password::valid_submission(submitted) {
            return Err(AccountError::InvalidRequest);
        }
        let _lock = persistence::lock(&self.state_path)?;
        let config = self.configuration()?;
        let mut store = self.read_store()?;
        let now = now()?;
        store.attempts.retain(|_, timestamps| {
            timestamps.retain(|t| t.saturating_add(WINDOW_MILLIS) > now);
            !timestamps.is_empty()
        });
        let bucket = digest(&format!(
            "account-login/v1:{}:{name}",
            config.tenant_id.as_str()
        ));
        if store.attempts.get(&bucket).is_some_and(|v| v.len() >= 5)
            || store
                .attempts
                .get("loopback")
                .is_some_and(|v| v.len() >= 30)
            || store.attempts.len() >= 1024
        {
            return Err(AccountError::RateLimited);
        }
        store.attempts.entry(bucket.clone()).or_default().push(now);
        store
            .attempts
            .entry("loopback".into())
            .or_default()
            .push(now);
        persistence::commit(&self.state_path, &mut store)?;
        let account = config
            .accounts
            .iter()
            .find(|account| account.login_name == name);
        let accepted = password::verify(
            submitted,
            account.map_or(&self.dummy_hash, |a| &a.password_hash),
        )?;
        let Some(account) = account.filter(|account| account.active && accepted) else {
            return Err(AccountError::AuthenticationFailed);
        };
        if store.sessions.len() >= 4096 {
            return Err(AccountError::Unavailable);
        }
        let bearer = random_hex()?;
        let verifier = digest(&bearer);
        if store.sessions.contains_key(&verifier) {
            return Err(AccountError::Unavailable);
        }
        let instant = SessionInstant::from_unix_millis(now);
        let evidence = SessionCredentialEvidence::new(
            config.tenant_id.clone(),
            account.user_id.clone(),
            AuthAdapterId::parse("local-argon2id-v1").map_err(|_| AccountError::Unavailable)?,
            CredentialEvidenceDigest::parse(format!(
                "sha256:{}",
                digest(&format!("account-authentication/v1:{}", random_hex()?))
            ))
            .map_err(|_| AccountError::Unavailable)?,
            instant,
            None,
        )
        .map_err(|_| AccountError::Unavailable)?;
        let duration =
            SessionDuration::from_millis(SESSION_MILLIS).map_err(|_| AccountError::Unavailable)?;
        let event = session::decide(
            None,
            &SessionCommand::Open(OpenSession::new(
                SessionId::parse(format!("session:{}", random_hex()?))
                    .map_err(|_| AccountError::Unavailable)?,
                evidence,
                SessionPolicy::new(duration, duration),
                instant,
                0,
            )),
        )
        .map_err(|_| AccountError::Unavailable)?;
        let binding = Binding {
            tenant_id: config.tenant_id.clone(),
            user_id: account.user_id.clone(),
            generation: account.credential_generation,
            credential_reference: credential_reference(account),
            events: vec![event],
        };
        let subject = project(account, &snapshot(&binding)?);
        store.sessions.insert(verifier, binding);
        store.attempts.remove(&bucket);
        persistence::commit(&self.state_path, &mut store)?;
        Ok((bearer, subject))
    }

    pub(crate) fn admit(&self, bearer: &str) -> Result<AdmittedSubject, AccountError> {
        if !is_verifier(bearer) {
            return Err(AccountError::Unauthenticated);
        }
        let _lock = persistence::lock(&self.state_path)?;
        let config = self.configuration()?;
        let store = self.read_store()?;
        admit_binding(&config, &store, bearer, now()?)
            .map(|(account, snapshot)| project(account, &snapshot))
    }

    pub(crate) fn logout(&self, bearer: &str) -> Result<(), AccountError> {
        if !is_verifier(bearer) {
            return Err(AccountError::Unauthenticated);
        }
        let _lock = persistence::lock(&self.state_path)?;
        let config = self.configuration()?;
        let mut store = self.read_store()?;
        let now = now()?;
        let (_, snapshot) = admit_binding(&config, &store, bearer, now)?;
        let event = session::decide(
            Some(&snapshot),
            &SessionCommand::Revoke(RevokeSession::new(
                snapshot.session_id().clone(),
                SessionInstant::from_unix_millis(now),
                snapshot.revision(),
            )),
        )
        .map_err(|_| AccountError::Unavailable)?;
        store
            .sessions
            .get_mut(&digest(bearer))
            .ok_or(AccountError::Unauthenticated)?
            .events
            .push(event);
        persistence::commit(&self.state_path, &mut store)
    }
}

fn admit_binding<'a>(
    config: &'a Configuration,
    store: &Store,
    bearer: &str,
    now: u64,
) -> Result<(&'a Account, SessionSnapshot), AccountError> {
    let binding = store
        .sessions
        .get(&digest(bearer))
        .ok_or(AccountError::Unauthenticated)?;
    let account = config
        .accounts
        .iter()
        .find(|a| {
            a.user_id == binding.user_id
                && a.active
                && a.credential_generation == binding.generation
                && credential_reference(a) == binding.credential_reference
        })
        .ok_or(AccountError::Unauthenticated)?;
    let snapshot = snapshot(binding)?;
    if binding.tenant_id != config.tenant_id
        || !snapshot.admits_at(SessionInstant::from_unix_millis(now))
    {
        return Err(AccountError::Unauthenticated);
    }
    Ok((account, snapshot))
}

fn credential_reference(account: &Account) -> String {
    digest(&format!("credential-record/v1:{}", account.password_hash))
}
fn project(account: &Account, snapshot: &SessionSnapshot) -> AdmittedSubject {
    AdmittedSubject {
        tenant_id: snapshot.tenant_id().clone(),
        user_id: account.user_id.clone(),
        login_name: account.login_name.clone(),
        administrator: account.administrator,
        expires_at: snapshot.effective_expires_at().as_unix_millis(),
    }
}
fn snapshot(binding: &Binding) -> Result<SessionSnapshot, AccountError> {
    let mut value = None;
    for event in &binding.events {
        value =
            Some(session::evolve(value.as_ref(), event).map_err(|_| AccountError::Unavailable)?);
    }
    value.ok_or(AccountError::Unavailable)
}
fn valid_name(name: &str) -> bool {
    (3..=64).contains(&name.len())
        && name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
        && name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && name
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}
fn is_verifier(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn random_bytes() -> Result<[u8; 32], AccountError> {
    let mut bytes = [0; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(|_| AccountError::Unavailable)?;
    Ok(bytes)
}
fn random_hex() -> Result<String, AccountError> {
    Ok(random_bytes()?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}
fn now() -> Result<u64, AccountError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .ok_or(AccountError::Unavailable)
}

#[cfg(test)]
mod tests;
