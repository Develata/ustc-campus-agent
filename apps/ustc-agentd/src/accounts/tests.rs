use super::*;
use std::fs;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

fn fixture() -> (PathBuf, AccountService) {
    let root = std::env::temp_dir().join(format!(
        "uca-account-test-{}",
        random_hex().expect("random")
    ));
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .expect("directory");
    let config = serde_json::json!({"schema":"platform-local-accounts/v1", "tenant_id":"tenant:campus", "accounts":[
        {"user_id":"user:alice", "login_name":"alice", "password_hash":password::hash("alice-password-fixture").expect("hash"), "active":true,"administrator":true,"credential_generation":1},
        {"user_id":"user:bobby", "login_name":"bobby", "password_hash":password::hash("bobby-password-fixture").expect("hash"), "active":true,"administrator":false,"credential_generation":1}]});
    let config_path = root.join("config.json");
    fs::write(&config_path, serde_json::to_vec(&config).expect("JSON")).expect("write");
    fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600)).expect("mode");
    let service = AccountService::open(config_path, root.join("sessions.json")).expect("open");
    (root, service)
}

#[test]
fn accounts_two_users_restart_logout_and_current_configuration() {
    let (root, service) = fixture();
    let (alice_token, alice) = service
        .login("alice", "alice-password-fixture")
        .expect("alice login");
    let (bobby_token, bobby) = service
        .login("bobby", "bobby-password-fixture")
        .expect("bobby login");
    assert_ne!(alice.user(), bobby.user());
    assert_eq!(alice.tenant(), bobby.tenant());
    assert!(alice.is_administrator());
    assert!(!bobby.is_administrator());
    let worker = AccountService::open(service.config_path.clone(), service.state_path.clone())
        .expect("second worker");
    assert_eq!(
        worker
            .admit(&alice_token)
            .expect("restart admission")
            .user()
            .as_str(),
        "user:alice"
    );
    service.logout(&alice_token).expect("logout");
    assert!(matches!(
        worker.admit(&alice_token),
        Err(AccountError::Unauthenticated)
    ));
    assert!(worker.admit(&bobby_token).is_ok());
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(&service.config_path).expect("read config"))
            .expect("JSON");
    config["accounts"][1]["active"] = false.into();
    fs::write(
        &service.config_path,
        serde_json::to_vec(&config).expect("JSON"),
    )
    .expect("update config");
    assert!(matches!(
        worker.admit(&bobby_token),
        Err(AccountError::Unauthenticated)
    ));
    assert!(matches!(
        service.login("bobby", "bobby-password-fixture"),
        Err(AccountError::AuthenticationFailed)
    ));
    assert!(matches!(
        service.login("nobody", "nobody-password-fixture"),
        Err(AccountError::AuthenticationFailed)
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn accounts_fail_closed_on_snapshot_rollback_and_invalid_hash_parameters() {
    let (root, service) = fixture();
    let (token, _) = service
        .login("alice", "alice-password-fixture")
        .expect("login");
    let old = fs::read(&service.state_path).expect("snapshot");
    service.logout(&token).expect("logout");
    fs::write(&service.state_path, old).expect("restore stale snapshot");
    assert!(matches!(
        service.admit(&token),
        Err(AccountError::Unavailable)
    ));
    assert!(password::validate_record("$argon2id$v=19$m=999999999,t=2,p=1$salt$hash").is_err());
    assert!(!password::valid_submission("short"));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn accounts_rate_limits_before_hashing_and_no_secret_in_durable_session() {
    let (root, service) = fixture();
    for _ in 0..5 {
        assert!(matches!(
            service.login("alice", "wrong-password-fixture"),
            Err(AccountError::AuthenticationFailed)
        ));
    }
    assert!(matches!(
        service.login("alice", "alice-password-fixture"),
        Err(AccountError::RateLimited)
    ));
    let (token, _) = service
        .login("bobby", "bobby-password-fixture")
        .expect("other account");
    let bytes = fs::read_to_string(&service.state_path).expect("session state");
    assert!(!bytes.contains(&token));
    assert!(!bytes.contains("password-fixture"));
    assert!(!bytes.contains("$argon2"));
    fs::remove_dir_all(root).expect("cleanup");
}
