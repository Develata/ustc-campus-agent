#![allow(clippy::expect_used)]

use ustc_campus_agent_core::identity::{TenantId, UserId};
use ustc_campus_agent_core::invocation::{
    CapabilityId, GrantSnapshotId, GrantVersion, InstallationId,
};
use ustc_campus_agent_core::market::grant::{
    GrantCommand, GrantCommandId, GrantConstructionError, GrantDecisionError, GrantEventSequence,
    GrantInvalidationReason, GrantReplayError, GrantRepository, GrantRepositoryError, GrantScope,
    InMemoryGrantRepository, replay,
};

#[test]
fn checked_grant_ids_versions_and_sequences_are_canonical() {
    assert_eq!(
        GrantCommandId::parse("grant-cmd:ok")
            .expect("canonical id")
            .as_str(),
        "grant-cmd:ok"
    );
    assert_eq!(
        GrantCommandId::parse("grant-cmd:").expect_err("empty command suffix must fail"),
        GrantConstructionError::InvalidCommandId
    );
    assert_eq!(GrantEventSequence::new(1).expect("first sequence").get(), 1);
    assert_eq!(
        GrantEventSequence::new(0).expect_err("zero sequence must fail"),
        GrantConstructionError::InvalidEventSequence
    );
}

#[test]
fn closed_scope_algebra_projects_exact_public_and_tenant_private_scopes() {
    let public = GrantScope::campus_public().expect("closed public scope");
    assert_eq!(public.object_scope().as_str(), "scope:campus-public");
    assert!(public.tenant_id().is_none());

    let tenant = TenantId::parse("tenant:t1").expect("tenant");
    let user = UserId::parse("user:u1").expect("user");
    let private =
        GrantScope::tenant_private_user(tenant.clone(), user.clone()).expect("private scope");
    assert_eq!(private.tenant_id(), Some(&tenant));
    assert_eq!(private.user_id(), Some(&user));
    assert_eq!(
        private.object_scope().as_str(),
        "scope:tenant-user:sha256:f9e912fbb4d0cde319d86a1c368fe9b3278ca1c293165cd2af319f7f4cb2a056"
    );
}

#[test]
fn non_issue_commands_validate_snapshot_and_expected_version() {
    let command_id = GrantCommandId::parse("grant-cmd:stale").expect("command id");
    let invalid_snapshot = GrantSnapshotId::parse("snapshot:not-a-grant").expect("generic id");
    let valid_version = GrantVersion::parse("grant-version:1").expect("generic version");
    assert_eq!(
        GrantCommand::mark_stale(
            command_id.clone(),
            invalid_snapshot,
            valid_version,
            GrantInvalidationReason::PolicyChanged
        )
        .expect_err("non-grant snapshot must fail"),
        GrantConstructionError::InvalidSnapshotId,
    );
    let snapshot = GrantSnapshotId::parse("grant:one").expect("snapshot");
    let invalid_version = GrantVersion::parse("grant-version:01").expect("generic version");
    assert_eq!(
        GrantCommand::expire(command_id, snapshot, invalid_version)
            .expect_err("noncanonical version must fail"),
        GrantConstructionError::InvalidGrantVersion,
    );
}

#[test]
fn empty_replay_and_repository_queries_are_deterministic() {
    assert_eq!(replay(std::iter::empty()).expect("empty replay"), None);
    let repository = InMemoryGrantRepository::new();
    let snapshot = GrantSnapshotId::parse("grant:missing").expect("snapshot");
    assert_eq!(repository.load_exact(&snapshot).expect("query"), None);
    assert!(
        repository
            .event_history(&snapshot)
            .expect("history")
            .is_empty()
    );
    let tenant = TenantId::parse("tenant:missing").expect("tenant");
    let user = UserId::parse("user:missing").expect("user");
    let installation = InstallationId::parse("installation:missing").expect("installation");
    let capability = CapabilityId::parse("capability:missing").expect("capability");
    let scope = GrantScope::campus_public().expect("scope");
    assert_eq!(
        repository
            .load_current_for_authority(&tenant, &user, &installation, &capability, &scope)
            .expect("authority query"),
        None
    );
}

#[test]
fn public_errors_are_category_only_and_secret_safe() {
    let secret = "raw-secret-marker";
    let values = [
        format!("{}", GrantConstructionError::EvidenceIncoherent),
        format!("{}", GrantDecisionError::AdmissionEvidenceMismatch),
        format!("{}", GrantReplayError::AdmissionEvidenceMismatch),
        format!(
            "{}",
            GrantRepositoryError::DecisionRejected(GrantDecisionError::AuthorityConflict)
        ),
    ];
    assert!(values.iter().all(|value| !value.contains(secret)));
    assert!(values.iter().all(|value| value.len() < 128));
}
