#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use ustc_agentd::AffairsComposition;
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientProvenanceDto, ClientResponseDto,
    M72OpportunityTerminalDto, OpportunityCommandDto, OpportunityConsentFieldDto,
    OpportunityPlanDecisionDto, OpportunityPreferenceDto, OpportunityRejectionDto,
    OpportunitySourceHealthDto, SubmitAffairsGetDto, SubmitOpportunityDto, UnixMillis,
    WireErrorClassDto, WireText, affairs_get_payload_digest, opportunity_payload_digest,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace")
        .to_path_buf()
}

fn temp_dir(label: &str) -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!(
        "agentd-opportunity-composition-{}-{id}-{label}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create test directory");
    path
}

fn wire(value: impl Into<String>) -> WireText {
    WireText::parse(value).expect("wire text")
}

struct TestEnv {
    dir: PathBuf,
    affairs_fixture: PathBuf,
    opportunity_fixture: PathBuf,
    catalog: PathBuf,
    store: PathBuf,
    idempotency: PathBuf,
    profiles: PathBuf,
}

impl TestEnv {
    fn new(label: &str) -> Self {
        let dir = temp_dir(label);
        let affairs_fixture = dir.join("affairs.json");
        let opportunity_fixture = dir.join("opportunity.json");
        fs::copy(
            workspace().join("fixtures/affairs/proc-011-reviewed.json"),
            &affairs_fixture,
        )
        .expect("copy affairs fixture");
        fs::copy(
            workspace().join("fixtures/opportunity-graph/course-planning-demo-reviewed.json"),
            &opportunity_fixture,
        )
        .expect("copy opportunity fixture");
        Self {
            catalog: workspace().join("market/fixtures/course-planning/minimal-v0.json"),
            store: dir.join("records.json"),
            idempotency: dir.join("idempotency.json"),
            profiles: dir.join("opportunity-profiles.json"),
            dir,
            affairs_fixture,
            opportunity_fixture,
        }
    }

    fn set_opportunity(&self, key: &str, value: Value) {
        let mut fixture: Value = serde_json::from_slice(
            &fs::read(&self.opportunity_fixture).expect("read opportunity fixture"),
        )
        .expect("decode opportunity fixture");
        fixture[key] = value;
        fs::write(&self.opportunity_fixture, fixture.to_string())
            .expect("write opportunity fixture");
    }

    fn set_actor(&self, session_id: &str, tenant_id: &str, user_id: &str) {
        let mut fixture: Value =
            serde_json::from_slice(&fs::read(&self.affairs_fixture).expect("read affairs fixture"))
                .expect("decode affairs fixture");
        fixture["session_id"] = json!(session_id);
        fixture["tenant_id"] = json!(tenant_id);
        fixture["user_id"] = json!(user_id);
        fs::write(&self.affairs_fixture, fixture.to_string()).expect("write affairs fixture");
    }

    fn open(&self) -> AffairsComposition {
        AffairsComposition::open_with_opportunity(
            &self.affairs_fixture,
            &self.opportunity_fixture,
            &self.catalog,
            &self.profiles,
            &self.store,
            &self.idempotency,
        )
        .expect("open Opportunity composition")
    }

    fn open_with_idempotency(&self, idempotency: &Path) -> AffairsComposition {
        AffairsComposition::open_with_opportunity(
            &self.affairs_fixture,
            &self.opportunity_fixture,
            &self.catalog,
            &self.profiles,
            &self.store,
            idempotency,
        )
        .expect("open Opportunity composition")
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn authenticated_actor(session: &str) -> ActorIntentDto {
    ActorIntentDto::Authenticated {
        session_id: wire(session),
    }
}

fn request(
    command: OpportunityCommandDto,
    suffix: &str,
    actor: ActorIntentDto,
) -> SubmitOpportunityDto {
    SubmitOpportunityDto {
        request_id: wire(format!("req:opportunity:{suffix}")),
        correlation_id: wire(format!("corr:opportunity:{suffix}")),
        causation_id: None,
        idempotency_key: Some(wire(format!("idem:opportunity:{suffix}"))),
        actor,
        provenance: ClientProvenanceDto {
            build: wire("build:test"),
            target: wire("linux"),
            protocol: wire("m10:v2"),
        },
        payload_digest: opportunity_payload_digest(&command).expect("opportunity digest"),
        command,
    }
}

fn create_command() -> OpportunityCommandDto {
    OpportunityCommandDto::CreateProfile {
        consent_purpose: wire("opportunity_planning"),
        consent_fields: vec![
            OpportunityConsentFieldDto::CompletedCourses,
            OpportunityConsentFieldDto::CreditBounds,
            OpportunityConsentFieldDto::PreferenceWeights,
        ],
        consented_at: UnixMillis::new(1_787_792_400_000),
        completed_courses: vec![
            wire("MATH1001"),
            wire("MATH1002"),
            wire("CS1001"),
            wire("PHYS1001"),
        ],
        min_credits: 9,
        max_credits: 12,
        preference_weights: vec![
            OpportunityPreferenceDto {
                course_code: wire("MATH2001"),
                weight: 9,
            },
            OpportunityPreferenceDto {
                course_code: wire("MATH2003"),
                weight: 8,
            },
            OpportunityPreferenceDto {
                course_code: wire("CS2006"),
                weight: 7,
            },
            OpportunityPreferenceDto {
                course_code: wire("PHYS2003"),
                weight: 5,
            },
            OpportunityPreferenceDto {
                course_code: wire("HUM2001"),
                weight: 4,
            },
            OpportunityPreferenceDto {
                course_code: wire("GEN2001"),
                weight: 3,
            },
            OpportunityPreferenceDto {
                course_code: wire("LANG2001"),
                weight: 2,
            },
        ],
    }
}

fn create_profile(composition: &AffairsComposition, suffix: &str) -> WireText {
    let response = composition.handle_opportunity_submit(&request(
        create_command(),
        suffix,
        authenticated_actor("session:proc011-web-demo"),
    ));
    match response {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
            M72OpportunityTerminalDto::ProfileCreated { profile } => profile.profile_snapshot_id,
            _ => panic!("expected profile-created terminal"),
        },
        other => panic!("expected accepted profile creation, got {other:?}"),
    }
}

fn plan_command(profile_id: &WireText) -> OpportunityCommandDto {
    OpportunityCommandDto::GeneratePlan {
        profile_snapshot_id: profile_id.clone(),
        max_results: 3,
        beam_width: 1_024,
    }
}

fn view_command(profile_id: &WireText) -> OpportunityCommandDto {
    OpportunityCommandDto::ViewProfile {
        profile_snapshot_id: profile_id.clone(),
    }
}

#[test]
fn consent_profile_plan_delete_and_restart_cross_the_full_bounded_spine() {
    let env = TestEnv::new("journey");
    let composition = env.open();
    let profile_id = create_profile(&composition, "create");
    assert_eq!(composition.opportunity_private_state_counts(), (1, 0));
    assert_eq!(
        fs::metadata(&env.profiles)
            .expect("profile store metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert_eq!(composition.opportunity_invocation_counts(), (1, 1, 1));
    assert_eq!(composition.opportunity_m60_call_count(), 0);

    let view = composition.handle_opportunity_submit(&request(
        view_command(&profile_id),
        "view",
        authenticated_actor("session:proc011-web-demo"),
    ));
    match view {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
            M72OpportunityTerminalDto::ProfileFound { profile } => {
                assert_eq!(profile.profile_snapshot_id, profile_id);
                assert_eq!(profile.completed_course_count, 4);
                assert_eq!(profile.preference_count, 7);
            }
            _ => panic!("expected profile-found terminal"),
        },
        other => panic!("expected accepted profile view, got {other:?}"),
    }
    assert_eq!(composition.opportunity_m60_call_count(), 0);

    let plan = composition.handle_opportunity_submit(&request(
        plan_command(&profile_id),
        "plan",
        authenticated_actor("session:proc011-web-demo"),
    ));
    match plan {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
            M72OpportunityTerminalDto::PlanGenerated { plan } => {
                assert_eq!(plan.profile_snapshot_id, profile_id);
                assert!(!plan.qualifications.is_empty());
                assert!(plan.qualifications.iter().all(|qualification| {
                    !qualification.source_id.as_str().is_empty()
                        && qualification.source_revision_id == plan.source_revision_id
                }));
                match plan.decision {
                    OpportunityPlanDecisionDto::Planned {
                        candidates,
                        hard_constraint_violations,
                        ..
                    } => {
                        assert_eq!(hard_constraint_violations, 0);
                        assert!(!candidates.is_empty());
                        assert!(candidates.iter().all(|candidate| {
                            candidate.hard_constraint_violations.is_empty()
                                && !candidate.provenance.is_empty()
                        }));
                    }
                    OpportunityPlanDecisionDto::NoFeasiblePlan => {
                        panic!("reviewed fixture should produce a bounded plan")
                    }
                }
            }
            _ => panic!("expected plan-generated terminal"),
        },
        other => panic!(
            "expected accepted plan, got {}",
            serde_json::to_string(&other).expect("serialize unexpected plan response")
        ),
    }
    assert_eq!(composition.opportunity_m60_call_count(), 1);
    drop(composition);

    let reopened = env.open();
    assert_eq!(reopened.opportunity_private_state_counts(), (1, 0));
    assert!(matches!(
        reopened.handle_opportunity_submit(&request(
            view_command(&profile_id),
            "view-after-restart",
            authenticated_actor("session:proc011-web-demo"),
        )),
        ClientResponseDto::OpportunityAccepted { .. }
    ));

    let delete = reopened.handle_opportunity_submit(&request(
        OpportunityCommandDto::RevokeConsentAndDeleteProfile {
            profile_snapshot_id: profile_id.clone(),
            revoked_at: UnixMillis::new(1_787_792_500_000),
        },
        "delete",
        authenticated_actor("session:proc011-web-demo"),
    ));
    let deletion_receipt_id = match delete {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
            M72OpportunityTerminalDto::ProfileDeleted { deletion } => deletion.deletion_receipt_id,
            _ => panic!("expected profile-deleted terminal"),
        },
        other => panic!("expected accepted deletion, got {other:?}"),
    };
    assert_eq!(reopened.opportunity_private_state_counts(), (0, 1));
    drop(reopened);

    let persisted = fs::read_to_string(&env.profiles).expect("read durable private state");
    for forbidden in [
        "MATH1001",
        "MATH1002",
        "CS1001",
        "PHYS1001",
        "MATH2001",
        "MATH2003",
        "CS2006",
        "PHYS2003",
        "HUM2001",
        "GEN2001",
        "LANG2001",
        "completed_courses",
        "preference_weights",
    ] {
        assert!(
            !persisted.contains(forbidden),
            "deleted durable state leaked private payload marker {forbidden}"
        );
    }

    let after_delete_restart = env.open();
    assert_eq!(
        after_delete_restart.opportunity_private_state_counts(),
        (0, 1)
    );
    let deleted = after_delete_restart.handle_opportunity_submit(&request(
        plan_command(&profile_id),
        "plan-after-delete-restart",
        authenticated_actor("session:proc011-web-demo"),
    ));
    match deleted {
        ClientResponseDto::OpportunityRejected { rejection, .. } => {
            assert_eq!(rejection, OpportunityRejectionDto::ProfileDeleted);
        }
        other => panic!("expected typed deleted rejection, got {other:?}"),
    }
    assert_eq!(after_delete_restart.opportunity_m60_call_count(), 0);

    let delete_replay = after_delete_restart.handle_opportunity_submit(&request(
        OpportunityCommandDto::RevokeConsentAndDeleteProfile {
            profile_snapshot_id: profile_id,
            revoked_at: UnixMillis::new(1_787_792_500_000),
        },
        "delete-replay",
        authenticated_actor("session:proc011-web-demo"),
    ));
    match delete_replay {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
            M72OpportunityTerminalDto::ProfileDeleted { deletion } => {
                assert_eq!(deletion.deletion_receipt_id, deletion_receipt_id);
            }
            _ => panic!("expected replayed profile-deleted terminal"),
        },
        other => panic!("expected idempotent delete replay, got {other:?}"),
    }
}

#[test]
fn different_tenant_cannot_read_profile_or_reach_m60() {
    let env = TestEnv::new("tenant-isolation");
    let owner = env.open();
    let profile_id = create_profile(&owner, "owner-create");
    drop(owner);

    env.set_actor(
        "session:opportunity-other-tenant",
        "tenant:opportunity-other",
        "user:opportunity-other",
    );
    let other_idempotency = env.dir.join("other-idempotency.json");
    let other = env.open_with_idempotency(&other_idempotency);
    let response = other.handle_opportunity_submit(&request(
        plan_command(&profile_id),
        "cross-tenant-plan",
        authenticated_actor("session:opportunity-other-tenant"),
    ));
    match response {
        ClientResponseDto::OpportunityRejected { rejection, .. } => {
            assert_eq!(rejection, OpportunityRejectionDto::AccessDenied);
        }
        other => panic!("expected typed access denial, got {other:?}"),
    }
    assert_eq!(other.opportunity_invocation_counts(), (1, 1, 1));
    assert_eq!(other.opportunity_m60_call_count(), 0);
    assert_eq!(other.opportunity_private_state_counts(), (1, 0));
}

#[test]
fn market_disable_and_grant_revoke_deny_before_intent_executor_and_m60() {
    for (index, key) in ["market_enabled", "market_grant_active"]
        .into_iter()
        .enumerate()
    {
        let env = TestEnv::new(key);
        env.set_opportunity(key, json!(false));
        let composition = env.open();
        let response = composition.handle_opportunity_submit(&request(
            create_command(),
            &format!("deny-{index}"),
            authenticated_actor("session:proc011-web-demo"),
        ));
        match response {
            ClientResponseDto::Error {
                error: ClientErrorDto::Admission { error },
            } => assert_eq!(error.class, WireErrorClassDto::PolicyDenied),
            other => panic!("expected policy denial, got {other:?}"),
        }
        assert_eq!(composition.opportunity_invocation_counts(), (0, 0, 0));
        assert_eq!(composition.opportunity_m60_call_count(), 0);
        assert_eq!(composition.opportunity_private_state_counts(), (0, 0));
    }
}

#[test]
fn transaction_current_grant_recheck_denies_revocation_after_projection() {
    let env = TestEnv::new("grant-revoked-after-projection");
    env.set_opportunity("authority_change_after_projection", json!("revoke_grant"));
    let composition = env.open();
    let response = composition.handle_opportunity_submit(&request(
        create_command(),
        "grant-revoked-after-projection",
        authenticated_actor("session:proc011-web-demo"),
    ));
    match response {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => assert_eq!(error.class, WireErrorClassDto::PolicyDenied),
        other => panic!("expected transaction-current policy denial, got {other:?}"),
    }
    assert_eq!(composition.opportunity_invocation_counts(), (0, 0, 0));
    assert_eq!(composition.opportunity_m60_call_count(), 0);
    assert_eq!(composition.opportunity_private_state_counts(), (0, 0));
}

#[test]
fn syntactically_valid_but_unknown_profile_facts_are_typed_not_internal() {
    let env = TestEnv::new("invalid-profile-facts");
    let composition = env.open();
    let mut create = create_command();
    let OpportunityCommandDto::CreateProfile {
        completed_courses,
        preference_weights,
        ..
    } = &mut create
    else {
        unreachable!("create helper must return create command");
    };
    completed_courses.push(wire("UNKNOWN9998"));
    preference_weights.push(OpportunityPreferenceDto {
        course_code: wire("UNKNOWN9999"),
        weight: 1,
    });
    let created = composition.handle_opportunity_submit(&request(
        create,
        "invalid-profile-create",
        authenticated_actor("session:proc011-web-demo"),
    ));
    let profile_id = match created {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
            M72OpportunityTerminalDto::ProfileCreated { profile } => profile.profile_snapshot_id,
            _ => panic!("expected profile-created terminal"),
        },
        other => panic!("expected invalid-fact profile storage, got {other:?}"),
    };
    let planned = composition.handle_opportunity_submit(&request(
        plan_command(&profile_id),
        "invalid-profile-plan",
        authenticated_actor("session:proc011-web-demo"),
    ));
    assert!(matches!(
        planned,
        ClientResponseDto::OpportunityRejected {
            rejection: OpportunityRejectionDto::InvalidProfileFacts,
            ..
        }
    ));
    assert_eq!(composition.opportunity_m60_call_count(), 1);
}

#[test]
fn stale_conflicting_and_unavailable_sources_return_typed_refusal() {
    for (index, (health, expected)) in [
        (
            "stale",
            OpportunityRejectionDto::SourceNotCurrent {
                health: OpportunitySourceHealthDto::Stale,
            },
        ),
        (
            "conflicting",
            OpportunityRejectionDto::SourceNotCurrent {
                health: OpportunitySourceHealthDto::Conflicting,
            },
        ),
        ("unavailable", OpportunityRejectionDto::SourceUnavailable),
    ]
    .into_iter()
    .enumerate()
    {
        let env = TestEnv::new(health);
        env.set_opportunity("source_health", json!(health));
        let composition = env.open();
        let profile_id = create_profile(&composition, &format!("create-{index}"));
        let response = composition.handle_opportunity_submit(&request(
            plan_command(&profile_id),
            &format!("plan-{index}"),
            authenticated_actor("session:proc011-web-demo"),
        ));
        match response {
            ClientResponseDto::OpportunityRejected { rejection, .. } => {
                assert_eq!(rejection, expected);
            }
            other => panic!("expected typed source refusal, got {other:?}"),
        }
        assert_eq!(composition.opportunity_m60_call_count(), 1);
        assert_eq!(composition.opportunity_invocation_counts(), (2, 2, 2));
    }
}

#[test]
fn unknown_profile_is_typed_and_does_not_reach_source() {
    let env = TestEnv::new("unknown-profile");
    let composition = env.open();
    let missing = wire(
        "profile-snapshot:opportunity:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let response = composition.handle_opportunity_submit(&request(
        plan_command(&missing),
        "unknown-profile",
        authenticated_actor("session:proc011-web-demo"),
    ));
    assert!(matches!(
        response,
        ClientResponseDto::OpportunityRejected {
            rejection: OpportunityRejectionDto::MissingProfile,
            ..
        }
    ));
    assert_eq!(composition.opportunity_m60_call_count(), 0);
    assert_eq!(composition.opportunity_private_state_counts(), (0, 0));
    assert_eq!(composition.opportunity_invocation_counts(), (1, 1, 1));
}

#[test]
fn tool_failure_before_execution_and_outcome_unknown_are_distinguishable() {
    let blocked = TestEnv::new("tool-before-execution");
    blocked.set_opportunity("tool_failure", json!("before_execution"));
    let composition = blocked.open();
    let response = composition.handle_opportunity_submit(&request(
        create_command(),
        "tool-before-execution",
        authenticated_actor("session:proc011-web-demo"),
    ));
    assert!(matches!(
        response,
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { .. }
        }
    ));
    assert_eq!(composition.opportunity_invocation_counts(), (0, 0, 0));
    assert_eq!(composition.opportunity_private_state_counts(), (0, 0));
    assert_eq!(composition.opportunity_m60_call_count(), 0);

    let unknown = TestEnv::new("outcome-unknown");
    unknown.set_opportunity("tool_failure", json!("outcome_persistence_unavailable"));
    let composition = unknown.open();
    let command = create_command();
    let response = composition.handle_opportunity_submit(&request(
        command.clone(),
        "outcome-unknown",
        authenticated_actor("session:proc011-web-demo"),
    ));
    assert!(matches!(response, ClientResponseDto::Incomplete { .. }));
    assert_eq!(composition.opportunity_invocation_counts(), (1, 1, 0));
    assert_eq!(composition.opportunity_private_state_counts(), (1, 0));
    drop(composition);

    unknown.set_opportunity("tool_failure", json!("none"));
    let recovered = unknown.open();
    let response = recovered.handle_opportunity_submit(&request(
        command,
        "outcome-unknown",
        authenticated_actor("session:proc011-web-demo"),
    ));
    assert!(matches!(
        response,
        ClientResponseDto::OpportunityAccepted {
            terminal,
            ..
        } if matches!(terminal.as_ref(), M72OpportunityTerminalDto::ProfileCreated { .. })
    ));
    assert_eq!(recovered.opportunity_invocation_counts(), (1, 1, 1));
    assert_eq!(recovered.opportunity_private_state_counts(), (1, 0));
}

#[test]
fn public_actor_and_malformed_digest_never_reach_private_executor() {
    let env = TestEnv::new("pre-executor-denial");
    let composition = env.open();
    let public = composition.handle_opportunity_submit(&request(
        create_command(),
        "public",
        ActorIntentDto::Public,
    ));
    assert!(matches!(
        public,
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { .. }
        }
    ));
    assert_eq!(composition.opportunity_invocation_counts(), (0, 0, 0));

    let mut malformed = request(
        create_command(),
        "malformed",
        authenticated_actor("session:proc011-web-demo"),
    );
    malformed.payload_digest =
        wire("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    match composition.handle_opportunity_submit(&malformed) {
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } => assert_eq!(error.class, WireErrorClassDto::MalformedCommand),
        other => panic!("expected malformed command, got {other:?}"),
    }
    assert_eq!(composition.opportunity_invocation_counts(), (0, 0, 0));
    assert_eq!(composition.opportunity_private_state_counts(), (0, 0));
}

#[test]
fn opportunity_disable_does_not_disable_affairs() {
    let env = TestEnv::new("plugin-isolation");
    env.set_opportunity("market_enabled", json!(false));
    let composition = env.open();
    let procedure_id = wire("proc:ustc:undergraduate:transcript-certificate");
    let affairs = SubmitAffairsGetDto {
        request_id: wire("req:affairs:opportunity-disabled"),
        correlation_id: wire("corr:affairs:opportunity-disabled"),
        causation_id: None,
        idempotency_key: Some(wire("idem:affairs:opportunity-disabled")),
        actor: ActorIntentDto::Public,
        provenance: ClientProvenanceDto {
            build: wire("build:test"),
            target: wire("linux"),
            protocol: wire("m10:v2"),
        },
        payload_digest: affairs_get_payload_digest(&procedure_id, None).expect("affairs digest"),
        procedure_id,
        as_of: None,
    };
    assert!(matches!(
        composition.handle_submit(&affairs),
        ClientResponseDto::Accepted { .. }
    ));
    assert_eq!(composition.invocation_counts(), (1, 1, 1));
    assert_eq!(composition.opportunity_invocation_counts(), (0, 0, 0));
}

#[test]
fn oversized_state_and_preexisting_temporary_symlink_fail_closed() {
    let oversized = TestEnv::new("oversized-profile-state");
    fs::File::create(&oversized.profiles)
        .expect("create oversized state")
        .set_len(1_048_577)
        .expect("size oversized state");
    fs::set_permissions(&oversized.profiles, fs::Permissions::from_mode(0o600))
        .expect("set oversized state mode");
    let opened = AffairsComposition::open_with_opportunity(
        &oversized.affairs_fixture,
        &oversized.opportunity_fixture,
        &oversized.catalog,
        &oversized.profiles,
        &oversized.store,
        &oversized.idempotency,
    );
    assert!(opened.is_err());

    let insecure = TestEnv::new("insecure-profile-mode");
    fs::write(
        &insecure.profiles,
        br#"{"schema_version":1,"active":[],"tombstones":[]}"#,
    )
    .expect("write insecure profile state");
    fs::set_permissions(&insecure.profiles, fs::Permissions::from_mode(0o644))
        .expect("set insecure profile mode");
    let opened = AffairsComposition::open_with_opportunity(
        &insecure.affairs_fixture,
        &insecure.opportunity_fixture,
        &insecure.catalog,
        &insecure.profiles,
        &insecure.store,
        &insecure.idempotency,
    );
    assert!(opened.is_err(), "0644 private state must fail closed");

    let primary_symlink = TestEnv::new("primary-state-symlink");
    let primary_sentinel = primary_symlink.dir.join("primary-sentinel.txt");
    fs::write(&primary_sentinel, "UNCHANGED").expect("write primary sentinel");
    std::os::unix::fs::symlink(&primary_sentinel, &primary_symlink.profiles)
        .expect("create primary-state symlink");
    let opened = AffairsComposition::open_with_opportunity(
        &primary_symlink.affairs_fixture,
        &primary_symlink.opportunity_fixture,
        &primary_symlink.catalog,
        &primary_symlink.profiles,
        &primary_symlink.store,
        &primary_symlink.idempotency,
    );
    assert!(opened.is_err(), "primary-state symlink must fail closed");
    assert_eq!(
        fs::read_to_string(&primary_sentinel).expect("read primary sentinel"),
        "UNCHANGED"
    );

    let symlinked = TestEnv::new("temporary-symlink");
    let sentinel = symlinked.dir.join("sentinel.txt");
    fs::write(&sentinel, "UNCHANGED").expect("write sentinel");
    let temporary = symlinked.dir.join(format!(
        ".opportunity-profiles.json.tmp-{}",
        std::process::id()
    ));
    std::os::unix::fs::symlink(&sentinel, &temporary).expect("create temporary symlink");
    let composition = symlinked.open();
    let response = composition.handle_opportunity_submit(&request(
        create_command(),
        "temporary-symlink",
        authenticated_actor("session:proc011-web-demo"),
    ));
    assert!(matches!(
        response,
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { .. }
        }
    ));
    assert_eq!(composition.opportunity_private_state_counts(), (0, 0));
    assert_eq!(
        fs::read_to_string(&sentinel).expect("read sentinel"),
        "UNCHANGED"
    );
}

#[test]
fn retained_catalog_tamper_blocks_startup() {
    let env = TestEnv::new("catalog-tamper");
    let tampered_catalog = env.dir.join("tampered-catalog.json");
    fs::write(&tampered_catalog, b"{}\n").expect("write tampered catalog");
    let result = AffairsComposition::open_with_opportunity(
        &env.affairs_fixture,
        &env.opportunity_fixture,
        &tampered_catalog,
        &env.profiles,
        &env.store,
        &env.idempotency,
    );
    assert!(result.is_err(), "retained catalog tamper must fail startup");
}
