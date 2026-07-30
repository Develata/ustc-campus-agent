//! Executable evidence for `platform-session/v0`, bound by `AUTH-017`, `AUTH-018`, `AUTH-019` and
//! `AUTH-020`.
//!
//! Each bound acceptance command runs `scripts/check_repo_contracts.py` before its Rust leg,
//! because a redirected `[[test]]` target or a renamed function makes `--exact` match nothing,
//! which cargo reports as `running 0 tests` at exit zero — and a guard written inside this suite is
//! exactly what such a change replaces.

use ustc_campus_agent_core::identity::{SessionId, TenantId, UserId};
use ustc_campus_agent_core::session::{
    AuthAdapterId, CredentialEvidenceDigest, EventDerivedField, ExpireSession, OpenSession,
    RefreshSession, RevokeSession, SessionCommand, SessionCredentialEvidence, SessionDomainError,
    SessionDuration, SessionEvent, SessionExpired, SessionExpiryCause, SessionInstant,
    SessionOpened, SessionPolicy, SessionRefreshed, SessionRevoked, SessionSnapshot, SessionStatus,
    SessionValueErrorKind, decide, evolve,
};

/// The governed module source, read for the dependency and side-effect negative space of §11.
const SESSION_SOURCE: &str = include_str!("../src/session.rs");

/// A well-formed digest whose trailing bytes are the canary every redaction proof searches for.
const DIGEST: &str = "sha256:00000000000000000000000000000000000000000000000000000000cafebabe";

/// The canary substring of [`DIGEST`]. It must never reach a `Debug`, `Display` or error surface.
const DIGEST_CANARY: &str = "cafebabe";

/// Secret-like text a caller might mistakenly hand to a value constructor.
const RAW_CREDENTIAL_CANARY: &str = "hunter2 password canary";

fn tenant() -> TenantId {
    TenantId::parse("tenant:example").expect("fixture")
}

fn user() -> UserId {
    UserId::parse("user:example").expect("fixture")
}

fn session() -> SessionId {
    SessionId::parse("session:example").expect("fixture")
}

fn other_session() -> SessionId {
    SessionId::parse("session:other").expect("fixture")
}

fn adapter() -> AuthAdapterId {
    AuthAdapterId::parse("ustc.cas").expect("fixture")
}

fn digest() -> CredentialEvidenceDigest {
    CredentialEvidenceDigest::parse(DIGEST).expect("fixture")
}

fn at(millis: u64) -> SessionInstant {
    SessionInstant::from_unix_millis(millis)
}

fn span(millis: u64) -> SessionDuration {
    SessionDuration::from_millis(millis).expect("fixture")
}

fn policy(idle: u64, absolute: u64) -> SessionPolicy {
    SessionPolicy::new(span(idle), span(absolute))
}

fn evidence(authenticated_at: u64, credential_not_after: Option<u64>) -> SessionCredentialEvidence {
    SessionCredentialEvidence::new(
        tenant(),
        user(),
        adapter(),
        digest(),
        at(authenticated_at),
        credential_not_after.map(at),
    )
    .expect("fixture")
}

fn open_command(
    opened_at: u64,
    credential_evidence: SessionCredentialEvidence,
    resolved: SessionPolicy,
    expected_revision: u64,
) -> SessionCommand {
    SessionCommand::Open(OpenSession::new(
        session(),
        credential_evidence,
        resolved,
        at(opened_at),
        expected_revision,
    ))
}

fn refresh(observed_at: u64, expected_revision: u64) -> SessionCommand {
    SessionCommand::Refresh(RefreshSession::new(
        session(),
        at(observed_at),
        expected_revision,
    ))
}

fn expire(observed_at: u64, expected_revision: u64) -> SessionCommand {
    SessionCommand::Expire(ExpireSession::new(
        session(),
        at(observed_at),
        expected_revision,
    ))
}

fn revoke(observed_at: u64, expected_revision: u64) -> SessionCommand {
    SessionCommand::Revoke(RevokeSession::new(
        session(),
        at(observed_at),
        expected_revision,
    ))
}

/// Decides and applies one command, requiring both halves to succeed.
fn advance(state: Option<&SessionSnapshot>, command: &SessionCommand) -> SessionSnapshot {
    let Ok(event) = decide(state, command) else {
        panic!("fixture command was expected to be accepted");
    };
    let Ok(next) = evolve(state, &event) else {
        panic!("fixture event was expected to apply");
    };
    next
}

/// Opens the shared base session: authenticated and opened at `1_000`, idle `100`, absolute
/// `1_000`, no credential deadline. Its effective deadline is therefore `1_100` with cause `Idle`,
/// and its policy-absolute deadline is `2_000`.
fn base_session() -> SessionSnapshot {
    advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(100, 1_000), 0),
    )
}

/// Serializes credential evidence as the canonical JSON object the suite mutates.
fn evidence_json(credential_not_after: &str) -> String {
    format!(
        concat!(
            "{{\"tenant_id\":\"tenant:example\",\"user_id\":\"user:example\",",
            "\"auth_adapter_id\":\"ustc.cas\",\"evidence_digest\":\"{}\",",
            "\"authenticated_at\":1000{}}}"
        ),
        DIGEST, credential_not_after
    )
}

/// `AUTH-017` — immutable open scope and the checked deadline algebra.
#[test]
fn session_open_pins_immutable_scope_and_checked_deadlines() {
    // Open pins exact tenant/user/session/adapter/evidence/policy scope, and derives the three
    // deadlines separately.
    let opened = base_session();
    assert_eq!(opened.session_id(), &session());
    assert_eq!(opened.tenant_id(), &tenant());
    assert_eq!(opened.user_id(), &user());
    assert_eq!(opened.auth_adapter_id(), &adapter());
    assert_eq!(opened.evidence_digest(), &digest());
    assert_eq!(opened.authenticated_at(), at(1_000));
    assert_eq!(opened.credential_not_after(), None);
    assert_eq!(opened.opened_at(), at(1_000));
    assert_eq!(opened.last_transition_at(), at(1_000));
    assert_eq!(opened.idle_timeout(), span(100));
    assert_eq!(opened.absolute_timeout(), span(1_000));
    assert_eq!(opened.absolute_expires_at(), at(2_000));
    assert_eq!(opened.effective_expires_at(), at(1_100));
    assert_eq!(opened.status(), SessionStatus::Active);
    assert_eq!(opened.revision(), 1);

    // `effective_expires_at` is the minimum of the idle candidate, the policy-absolute deadline
    // and the credential deadline: idle before, equal to and after each of the other two.
    let idle_binds = advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(100, 1_000), 0),
    );
    assert_eq!(idle_binds.effective_expires_at(), at(1_100));
    let absolute_binds = advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(5_000, 1_000), 0),
    );
    assert_eq!(absolute_binds.effective_expires_at(), at(2_000));
    let credential_binds = advance(
        None,
        &open_command(1_000, evidence(1_000, Some(1_050)), policy(100, 1_000), 0),
    );
    assert_eq!(credential_binds.effective_expires_at(), at(1_050));
    // …and the direction that a fail-open mutation would survive: a credential deadline ABOVE
    // both other candidates must NOT bind. Without this, replacing the conditional in the
    // three-way minimum with an unconditional assignment passes every other fixture while
    // extending sessions past their idle and policy-absolute deadlines.
    let credential_does_not_bind = advance(
        None,
        &open_command(1_000, evidence(1_000, Some(5_000)), policy(100, 1_000), 0),
    );
    assert_eq!(credential_does_not_bind.effective_expires_at(), at(1_100));
    assert_eq!(
        credential_does_not_bind.credential_not_after(),
        Some(at(5_000))
    );
    // Equal idle and policy-absolute deadlines, and an equal credential deadline on top of them.
    let equal_idle_absolute = advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(1_000, 1_000), 0),
    );
    assert_eq!(equal_idle_absolute.effective_expires_at(), at(2_000));
    assert_eq!(equal_idle_absolute.absolute_expires_at(), at(2_000));

    // Open at, before and after the credential deadline. Equality is already expired.
    assert_eq!(
        decide(
            None,
            &open_command(1_000, evidence(999, Some(1_000)), policy(100, 1_000), 0)
        ),
        Err(SessionDomainError::CredentialEvidenceExpired)
    );
    assert_eq!(
        decide(
            None,
            &open_command(1_001, evidence(999, Some(1_000)), policy(100, 1_000), 0)
        ),
        Err(SessionDomainError::CredentialEvidenceExpired)
    );
    assert!(
        decide(
            None,
            &open_command(999, evidence(999, Some(1_000)), policy(100, 1_000), 0)
        )
        .is_ok()
    );

    // Open time ordering, checked-add overflow, and the empty-aggregate revision rule.
    assert_eq!(
        decide(
            None,
            &open_command(1_000, evidence(2_000, None), policy(100, 1_000), 0)
        ),
        Err(SessionDomainError::InvalidTimeOrder)
    );
    assert_eq!(
        decide(
            None,
            &open_command(u64::MAX, evidence(u64::MAX, None), policy(1, 1), 0)
        ),
        Err(SessionDomainError::DeadlineOverflow)
    );
    assert_eq!(
        decide(
            None,
            &open_command(1_000, evidence(1_000, None), policy(100, 1_000), 7)
        ),
        Err(SessionDomainError::RevisionMismatch {
            expected: 7,
            actual: 0
        })
    );
    // A non-open command against an empty aggregate is `SessionNotFound`.
    for command in [refresh(1_000, 0), expire(1_000, 0), revoke(1_000, 0)] {
        assert_eq!(
            decide(None, &command),
            Err(SessionDomainError::SessionNotFound)
        );
    }
    // Open precedence: an existing aggregate answers before stale time and before the revision
    // claim, so neither lower-precedence fault can hide it.
    assert_eq!(
        decide(
            Some(&opened),
            &open_command(1, evidence(1, None), policy(100, 1_000), 42)
        ),
        Err(SessionDomainError::SessionAlreadyExists)
    );

    // A zero duration cannot be built, and cannot arrive through Serde either.
    let Err(zero) = SessionDuration::from_millis(0) else {
        panic!("zero duration must be rejected");
    };
    assert_eq!(zero.kind(), SessionValueErrorKind::ZeroDuration);
    assert_eq!(zero.value_kind(), "SessionDuration");
    assert!(serde_json::from_str::<SessionDuration>("0").is_err());
    assert_eq!(
        serde_json::from_str::<SessionDuration>("5").expect("fixture"),
        span(5)
    );

    // The adapter grammar and digest shape, in their exact §9.1 precedence.
    for (candidate, expected) in [
        (String::new(), SessionValueErrorKind::Empty),
        (
            "a".repeat(129),
            SessionValueErrorKind::TooLong { max_bytes: 128 },
        ),
        (".leading".to_owned(), SessionValueErrorKind::InvalidStart),
        (
            "a b".to_owned(),
            SessionValueErrorKind::InvalidCharacter { byte_index: 1 },
        ),
        ("trailing.".to_owned(), SessionValueErrorKind::InvalidEnd),
    ] {
        let Err(error) = AuthAdapterId::parse(candidate.clone()) else {
            panic!("adapter grammar must reject {candidate:?}");
        };
        assert_eq!(error.kind(), expected);
        assert_eq!(error.value_kind(), "AuthAdapterId");
    }
    assert_eq!(AuthAdapterId::parse("a").expect("fixture").as_str(), "a");
    assert_eq!(
        AuthAdapterId::parse("ustc.cas").expect("fixture").as_str(),
        "ustc.cas"
    );
    for malformed in [
        "sha256:00000000000000000000000000000000000000000000000000000000CAFEBABE",
        "00000000000000000000000000000000000000000000000000000000cafebabe",
        "sha512:00000000000000000000000000000000000000000000000000000000cafebabe",
        "sha256:cafebabe",
        "sha256:00000000000000000000000000000000000000000000000000000000cafebabe0",
    ] {
        let Err(error) = CredentialEvidenceDigest::parse(malformed) else {
            panic!("digest shape must reject {malformed:?}");
        };
        assert_eq!(error.kind(), SessionValueErrorKind::MalformedDigest);
        assert_eq!(error.value_kind(), "CredentialEvidenceDigest");
    }

    // The credential window is a relation between two fields, so it is enforced by the checked
    // constructor and by the shadow-struct decode alike.
    let Err(window) = SessionCredentialEvidence::new(
        tenant(),
        user(),
        adapter(),
        digest(),
        at(1_000),
        Some(at(1_000)),
    ) else {
        panic!("a credential deadline at the authentication instant must be rejected");
    };
    assert_eq!(
        window.kind(),
        SessionValueErrorKind::CredentialWindowNotAfterAuthentication
    );
    let rejected = serde_json::from_str::<SessionCredentialEvidence>(&evidence_json(
        ",\"credential_not_after\":1000",
    ));
    let Err(decoded) = rejected else {
        panic!("Serde must route through the checked constructor");
    };
    assert!(
        decoded
            .to_string()
            .contains("credential deadline is not after authentication"),
        "the shadow-struct decode must report the cross-field rejection"
    );

    // A MISSING `credential_not_after` is a rejection, not an absent deadline; an explicit null is
    // the only spelling of "no credential deadline". These two fail against the bare derive.
    assert!(
        serde_json::from_str::<SessionCredentialEvidence>(&evidence_json("")).is_err(),
        "an omitted credential_not_after must not decode as None"
    );
    let explicit_null = serde_json::from_str::<SessionCredentialEvidence>(&evidence_json(
        ",\"credential_not_after\":null",
    ))
    .expect("an explicit null is 'no credential deadline'");
    assert_eq!(explicit_null.credential_not_after(), None);
    assert_eq!(explicit_null, evidence(1_000, None));
    // …and serialization always writes the field, so no value this module produces is the omitted
    // form.
    let round_tripped = serde_json::to_string(&evidence(1_000, None)).expect("fixture");
    assert!(
        round_tripped.contains("\"credential_not_after\":null"),
        "the serialized form must write the field rather than skip it"
    );
    assert_eq!(
        serde_json::from_str::<SessionCredentialEvidence>(&round_tripped).expect("fixture"),
        evidence(1_000, None)
    );
    // Unknown fields fail closed on the aggregate and on the events.
    assert!(
        serde_json::from_str::<SessionCredentialEvidence>(&evidence_json(
            ",\"credential_not_after\":null,\"role\":\"admin\""
        ))
        .is_err(),
        "unknown fields must fail closed"
    );
}

/// `AUTH-018` — refresh, expire and revoke precedence, and current admission.
#[test]
fn session_lifecycle_precedence_is_deterministic_and_terminal() {
    let opened = base_session();

    // Refresh before the effective deadline extends only idle expiry, and never scope, the
    // policy-absolute deadline or the credential deadline.
    let refreshed = advance(Some(&opened), &refresh(1_050, 1));
    assert_eq!(refreshed.effective_expires_at(), at(1_150));
    assert_eq!(refreshed.last_transition_at(), at(1_050));
    assert_eq!(refreshed.revision(), 2);
    assert_eq!(
        refreshed.absolute_expires_at(),
        opened.absolute_expires_at()
    );
    assert_eq!(
        refreshed.credential_not_after(),
        opened.credential_not_after()
    );
    assert_eq!(refreshed.tenant_id(), opened.tenant_id());
    assert_eq!(refreshed.user_id(), opened.user_id());
    assert_eq!(refreshed.session_id(), opened.session_id());
    assert_eq!(refreshed.auth_adapter_id(), opened.auth_adapter_id());
    assert_eq!(refreshed.evidence_digest(), opened.evidence_digest());
    assert_eq!(refreshed.idle_timeout(), opened.idle_timeout());
    assert_eq!(refreshed.absolute_timeout(), opened.absolute_timeout());
    assert_eq!(refreshed.opened_at(), opened.opened_at());

    // Refresh is capped by the credential deadline exactly as it is by the policy-absolute one.
    // No other fixture runs a refresh on a credential-capped session, so without this the
    // credential term of the refresh recomputation is dead: dropping it lets a refresh extend a
    // session past the deadline its credential set, which §3 forbids.
    let capped = advance(
        None,
        &open_command(1_000, evidence(1_000, Some(1_120)), policy(100, 1_000), 0),
    );
    assert_eq!(capped.effective_expires_at(), at(1_100));
    let capped_refresh = advance(Some(&capped), &refresh(1_050, 1));
    assert_eq!(
        capped_refresh.effective_expires_at(),
        at(1_120),
        "refresh must be clipped to the credential deadline, not the idle candidate"
    );
    assert_eq!(capped_refresh.credential_not_after(), Some(at(1_120)));
    // …and once the cap binds, no later refresh can move it.
    assert_eq!(
        decide(Some(&capped_refresh), &refresh(1_060, 2)),
        Err(SessionDomainError::NoEffectiveRefresh)
    );

    // A refresh that cannot advance the deadline is an ordinary steady state, not a liveness
    // failure: at the same instant as the last transition, and under a policy whose idle timeout
    // never binds.
    assert_eq!(
        decide(Some(&opened), &refresh(1_000, 1)),
        Err(SessionDomainError::NoEffectiveRefresh)
    );
    let never_idle = advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(5_000, 1_000), 0),
    );
    assert_eq!(
        decide(Some(&never_idle), &refresh(1_500, 1)),
        Err(SessionDomainError::NoEffectiveRefresh)
    );

    // Equality is expired, not a grace interval, and time-derived expiry answers before refresh or
    // revoke semantics — so an expired session cannot be refreshed or relabeled.
    for command in [refresh(1_100, 1), expire(1_100, 1), revoke(1_100, 1)] {
        let Ok(SessionEvent::Expired(event)) = decide(Some(&opened), &command) else {
            panic!("an observation at the effective deadline must expire the session");
        };
        assert_eq!(event.expired_at(), at(1_100));
        assert_eq!(event.observed_at(), at(1_100));
        assert_eq!(event.cause(), SessionExpiryCause::Idle);
    }
    // …and strictly after it, for a refresh specifically: time-derived expiry answers before
    // refresh semantics on both sides of the boundary, and the event still records the effective
    // deadline rather than the later observation.
    let Ok(SessionEvent::Refreshed(_)) = decide(Some(&opened), &refresh(1_050, 1)) else {
        panic!("a refresh below the deadline must still refresh");
    };
    let Ok(SessionEvent::Expired(late_refresh)) = decide(Some(&opened), &refresh(5_000, 1)) else {
        panic!("a refresh observed strictly after the effective deadline must expire the session");
    };
    assert_eq!(late_refresh.sequence(), 2);
    assert_eq!(late_refresh.session_id(), opened.session_id());
    assert_eq!(late_refresh.observed_at(), at(5_000));
    assert_eq!(late_refresh.expired_at(), at(1_100));
    assert_eq!(late_refresh.cause(), SessionExpiryCause::Idle);

    // A late observation records detection order without rewriting historical validity.
    let Ok(SessionEvent::Expired(late)) = decide(Some(&opened), &expire(9_999, 1)) else {
        panic!("a late observation must still expire the session");
    };
    assert_eq!(late.expired_at(), at(1_100));
    assert_eq!(late.observed_at(), at(9_999));
    let late_snapshot = advance(Some(&opened), &expire(9_999, 1));
    assert_eq!(
        late_snapshot.status(),
        SessionStatus::Expired {
            expired_at: at(1_100),
            observed_at: at(9_999),
            cause: SessionExpiryCause::Idle,
        }
    );
    assert_eq!(late_snapshot.effective_expires_at(), at(1_100));

    // `Credential > Absolute > Idle` resolves equal deadlines deterministically.
    let idle_cause = advance(Some(&opened), &expire(1_100, 1));
    assert_eq!(
        idle_cause.status(),
        SessionStatus::Expired {
            expired_at: at(1_100),
            observed_at: at(1_100),
            cause: SessionExpiryCause::Idle,
        }
    );
    let absolute_tie = advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(1_000, 1_000), 0),
    );
    let Ok(SessionEvent::Expired(absolute_event)) = decide(Some(&absolute_tie), &expire(2_000, 1))
    else {
        panic!("equal idle and absolute deadlines must expire as Absolute");
    };
    assert_eq!(absolute_event.cause(), SessionExpiryCause::Absolute);
    // …and strictly after that absolute-bound deadline: the derived `expired_at` is the deadline,
    // not the observation, and the cause does not drift.
    let Ok(SessionEvent::Expired(late_absolute)) = decide(Some(&absolute_tie), &expire(9_999, 1))
    else {
        panic!("an observation strictly after an absolute-bound deadline must expire the session");
    };
    assert_eq!(late_absolute.expired_at(), at(2_000));
    assert_eq!(late_absolute.observed_at(), at(9_999));
    assert_eq!(late_absolute.cause(), SessionExpiryCause::Absolute);
    let credential_tie = advance(
        None,
        &open_command(1_000, evidence(1_000, Some(2_000)), policy(1_000, 1_000), 0),
    );
    let Ok(SessionEvent::Expired(credential_event)) =
        decide(Some(&credential_tie), &expire(2_000, 1))
    else {
        panic!("a credential deadline sharing the effective deadline must expire as Credential");
    };
    assert_eq!(credential_event.cause(), SessionExpiryCause::Credential);
    // …and strictly after a deadline the credential alone binds, where the credential is the
    // strict minimum rather than sharing the effective deadline with the other two candidates.
    let credential_bound = advance(
        None,
        &open_command(1_000, evidence(1_000, Some(1_050)), policy(100, 1_000), 0),
    );
    assert_eq!(credential_bound.effective_expires_at(), at(1_050));
    let Ok(SessionEvent::Expired(late_credential)) =
        decide(Some(&credential_bound), &expire(9_999, 1))
    else {
        panic!("an observation strictly after a credential-bound deadline must expire the session");
    };
    assert_eq!(late_credential.expired_at(), at(1_050));
    assert_eq!(late_credential.observed_at(), at(9_999));
    assert_eq!(late_credential.cause(), SessionExpiryCause::Credential);
    let late_credential_snapshot = advance(Some(&credential_bound), &expire(9_999, 1));
    assert_eq!(late_credential_snapshot.effective_expires_at(), at(1_050));

    // Expire before the deadline, revoke before the deadline, and revocation at the same
    // millisecond as the prior transition.
    assert_eq!(
        decide(Some(&opened), &expire(1_050, 1)),
        Err(SessionDomainError::SessionNotYetExpired)
    );
    let revoked = advance(Some(&opened), &revoke(1_050, 1));
    assert_eq!(
        revoked.status(),
        SessionStatus::Revoked {
            revoked_at: at(1_050)
        }
    );
    assert_eq!(revoked.effective_expires_at(), at(1_100));
    assert!(advance(Some(&opened), &revoke(1_000, 1)).revision() == 2);

    // Terminal states cannot mutate or resurrect, including repeated expire and revoke.
    let expired = advance(Some(&opened), &expire(1_100, 1));
    for terminal in [&revoked, &expired] {
        for command in [refresh(2_000, 2), expire(2_000, 2), revoke(2_000, 2)] {
            assert_eq!(
                decide(Some(terminal), &command),
                Err(SessionDomainError::TerminalSession {
                    status: terminal.status()
                })
            );
        }
        assert_eq!(
            decide(
                Some(terminal),
                &open_command(2_000, evidence(2_000, None), policy(100, 1_000), 0)
            ),
            Err(SessionDomainError::SessionAlreadyExists)
        );
    }

    // Dual-fault precedence, in both orientations where relevant: identity before revision,
    // revision before terminal, terminal before time, time before command legality.
    assert_eq!(
        decide(
            Some(&revoked),
            &SessionCommand::Refresh(RefreshSession::new(other_session(), at(1), 99))
        ),
        Err(SessionDomainError::SessionIdMismatch)
    );
    assert_eq!(
        decide(Some(&revoked), &refresh(1, 99)),
        Err(SessionDomainError::RevisionMismatch {
            expected: 99,
            actual: 2
        })
    );
    assert_eq!(
        decide(Some(&revoked), &refresh(1, 2)),
        Err(SessionDomainError::TerminalSession {
            status: revoked.status()
        })
    );
    assert_eq!(
        decide(Some(&opened), &refresh(999, 1)),
        Err(SessionDomainError::NonMonotoneTime)
    );
    // Non-monotone time and time-derived expiry can never compete: while a session is `Active`,
    // `effective_expires_at > last_transition_at`, so the two conditions have an empty overlap.
    assert!(opened.effective_expires_at() > opened.last_transition_at());

    // A refresh whose own deadline arithmetic overflows while the session is still validly active.
    let at_ceiling = advance(
        None,
        &open_command(1, evidence(1, None), policy(u64::MAX - 1, u64::MAX - 1), 0),
    );
    assert_eq!(at_ceiling.effective_expires_at(), at(u64::MAX));
    assert_eq!(
        decide(Some(&at_ceiling), &refresh(u64::MAX - 1, 1)),
        Err(SessionDomainError::DeadlineOverflow)
    );

    // `admits_at` answers current admission under all three conjuncts.
    assert!(opened.admits_at(at(1_000)), "true at last_transition_at");
    assert!(opened.admits_at(at(1_099)));
    assert!(
        !opened.admits_at(at(1_100)),
        "false at exactly effective_expires_at"
    );
    assert!(!opened.admits_at(at(999)), "false before open");
    assert!(
        !refreshed.admits_at(at(1_020)),
        "a stale observation between opened_at and last_transition_at must not be admitted"
    );
    assert!(refreshed.admits_at(at(1_050)));
    assert!(
        !revoked.admits_at(at(1_060)),
        "a revoked session must not be admitted before its preserved effective deadline"
    );
    assert!(!expired.admits_at(at(1_050)));
    // The obvious consumer test is wrong for exactly the revocation case, which is why the
    // snapshot exposes the sanctioned question instead.
    assert!(at(1_060) < revoked.effective_expires_at());

    // Terminal state is checked BEFORE revision exhaustion, so §6.3's flat statement holds with
    // no exception at `u64::MAX`.
    //
    // §13 binds the dual-fault case itself — a terminal session whose `current_revision` is
    // `u64::MAX` — to the private same-module fixture §12 names on this row's library leg,
    // because no feasible sequence of public calls builds that aggregate from out here. That
    // fixture executes the real decide path against the real state. What follows is an
    // additional carrier over the same ordering, kept because §13's list is a floor rather than
    // a ceiling: it fails if the two guards are ever reordered in the source.
    let decide_body = source_of("decide_existing");
    assert!(
        !decide_body.contains("fn evolve"),
        "the decide-path slice must stop at its own function"
    );
    let Some(terminal_at) = decide_body.find("SessionDomainError::TerminalSession") else {
        panic!("the decide path lost its terminal guard");
    };
    let Some(overflow_at) = decide_body.find("SessionDomainError::RevisionOverflow") else {
        panic!("the decide path lost its revision-exhaustion guard");
    };
    assert!(
        terminal_at < overflow_at,
        "terminal state must be decided before revision exhaustion"
    );
}

/// `AUTH-019` — expected revision, ordered events and deterministic replay.
#[test]
fn session_revision_and_replay_are_exact_and_fail_closed() {
    let opened = base_session();

    // Stale and future expected revisions, including a claim at the counter ceiling.
    for claimed in [0_u64, 2, u64::MAX] {
        assert_eq!(
            decide(Some(&opened), &refresh(1_050, claimed)),
            Err(SessionDomainError::RevisionMismatch {
                expected: claimed,
                actual: 1
            })
        );
    }

    // Sequence must be the exact next revision: gap, duplicate, reorder, a forged wrapped zero and
    // a forged ceiling are all rejected, and none of them applies anything.
    //
    // The aggregate already at `current_revision == u64::MAX` is not buildable from here — a
    // snapshot has no public constructor and no `Deserialize`, and `evolve` only ever sets
    // `revision` to `current + 1` starting from `1` — so §13 binds that half to the private
    // same-module fixture named on this row's library leg in §12. What this loop covers is the
    // reachable half: a forged sequence, including the wrapped `0`, against a live aggregate.
    for forged in [0_u64, 1, 3, u64::MAX] {
        assert_eq!(
            evolve(
                Some(&opened),
                &SessionEvent::Refreshed(SessionRefreshed::new(
                    forged,
                    session(),
                    at(1_050),
                    at(1_150)
                ))
            ),
            Err(SessionDomainError::EventSequenceMismatch {
                expected: 2,
                actual: forged
            })
        );
    }

    // Cross-session event injection, and the decide-side identity mismatch the caller-supplied
    // state makes reachable.
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Refreshed(SessionRefreshed::new(
                2,
                other_session(),
                at(1_050),
                at(1_150)
            ))
        ),
        Err(SessionDomainError::SessionIdMismatch)
    );
    assert_eq!(
        decide(
            Some(&opened),
            &SessionCommand::Refresh(RefreshSession::new(other_session(), at(1_050), 1))
        ),
        Err(SessionDomainError::SessionIdMismatch)
    );

    // Forged derived fields. For the two `SessionExpired` fields, reaching
    // `EventDerivedFieldMismatch` at all requires `observed_at >= effective_expires_at`, or the
    // apply-time validity guard answers first — so those two are built that way rather than
    // silently testing guard 1 twice. `SessionRefreshed`'s guard runs the other way round
    // (`observed_at < effective_expires_at`), so its fixture sits below the deadline.
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Refreshed(SessionRefreshed::new(2, session(), at(1_050), at(1_999)))
        ),
        Err(SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::RefreshEffectiveExpiresAt
        })
    );
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Expired(SessionExpired::new(
                2,
                session(),
                at(1_100),
                at(1_099),
                SessionExpiryCause::Idle
            ))
        ),
        Err(SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::ExpiredAt
        })
    );
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Expired(SessionExpired::new(
                2,
                session(),
                at(1_100),
                at(1_100),
                SessionExpiryCause::Absolute
            ))
        ),
        Err(SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::ExpiryCause
        })
    );

    // The apply-guard failure has its own name, and it covers all three `Active` rows. Each event
    // below carries an exact sequence, an exact `SessionId` and an `observed_at` at or after
    // `last_transition_at`, so the earlier universal checks cannot answer first.
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Refreshed(SessionRefreshed::new(2, session(), at(1_100), at(1_200)))
        ),
        Err(SessionDomainError::EventTimeOutsideValidity)
    );
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Revoked(SessionRevoked::new(2, session(), at(1_100)))
        ),
        Err(SessionDomainError::EventTimeOutsideValidity)
    );

    // The exact-derived-field expiry event, reached two ways. Direct construction must SUCCEED,
    // because the constructor is total; the byte-equal payload must deserialize identically; and
    // both must then be rejected by `evolve` as the same failure. A test that only exercised the
    // evolve path would still pass if a fallible constructor and a third error channel were
    // quietly reintroduced.
    let exact = SessionExpired::new(2, session(), at(1_050), at(1_100), SessionExpiryCause::Idle);
    assert_eq!(exact.observed_at(), at(1_050));
    assert_eq!(exact.expired_at(), at(1_100));
    assert!(exact.observed_at() < exact.expired_at());
    let event = SessionEvent::Expired(exact);
    let encoded = serde_json::to_string(&event).expect("fixture");
    let decoded = serde_json::from_str::<SessionEvent>(&encoded).expect("fixture");
    assert_eq!(decoded, event);
    assert_eq!(
        evolve(Some(&opened), &event),
        Err(SessionDomainError::EventTimeOutsideValidity)
    );
    assert_eq!(
        evolve(Some(&opened), &decoded),
        Err(SessionDomainError::EventTimeOutsideValidity)
    );

    // …and an `expired_at` forged BELOW the true effective deadline, with
    // `last_transition_at <= observed_at < effective_expires_at` and `observed_at` above the forged
    // value, which pins the guard order: guard 1 answers, not the derived-field guard.
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Expired(SessionExpired::new(
                2,
                session(),
                at(1_050),
                at(1_020),
                SessionExpiryCause::Idle
            ))
        ),
        Err(SessionDomainError::EventTimeOutsideValidity)
    );

    // The evolve-side refresh guards, which no other fixture reaches. Order matters: the strict
    // advance is checked BEFORE exact agreement with the recomputed deadline, so a persisted
    // refresh that cannot advance reports `NoEffectiveRefresh` even when its derived field is
    // forged. Reporting `EventDerivedFieldMismatch` for the exact-field case would be a
    // falsehood — the field is not forged, the refresh is simply impossible.
    let never_idle = advance(
        None,
        &open_command(1_000, evidence(1_000, None), policy(5_000, 1_000), 0),
    );
    assert_eq!(never_idle.effective_expires_at(), at(2_000));
    for claimed in [at(2_000), at(1_999)] {
        assert_eq!(
            evolve(
                Some(&never_idle),
                &SessionEvent::Refreshed(SessionRefreshed::new(2, session(), at(1_500), claimed))
            ),
            Err(SessionDomainError::NoEffectiveRefresh)
        );
    }
    // …and the overflow arm of the same recomputation, on the evolve side.
    let at_ceiling = advance(
        None,
        &open_command(1, evidence(1, None), policy(u64::MAX - 1, u64::MAX - 1), 0),
    );
    assert_eq!(
        evolve(
            Some(&at_ceiling),
            &SessionEvent::Refreshed(SessionRefreshed::new(
                2,
                session(),
                at(u64::MAX - 1),
                at(u64::MAX)
            ))
        ),
        Err(SessionDomainError::DeadlineOverflow)
    );

    // Illegal event/state pairs fail closed, on an empty aggregate, on an active one and on a
    // terminal one.
    assert_eq!(
        evolve(
            None,
            &SessionEvent::Revoked(SessionRevoked::new(1, session(), at(1_000)))
        ),
        Err(SessionDomainError::IllegalEventForState)
    );
    assert_eq!(
        evolve(
            Some(&opened),
            &SessionEvent::Opened(SessionOpened::new(
                2,
                session(),
                evidence(1_000, None),
                policy(100, 1_000),
                at(1_000)
            ))
        ),
        Err(SessionDomainError::IllegalEventForState)
    );
    let revoked = advance(Some(&opened), &revoke(1_050, 1));
    assert_eq!(
        evolve(
            Some(&revoked),
            &SessionEvent::Refreshed(SessionRefreshed::new(3, session(), at(1_060), at(1_160)))
        ),
        Err(SessionDomainError::IllegalEventForState)
    );

    // Evolution revalidates every open invariant from the persisted event rather than trusting it,
    // which is why the three open-path domain errors stay reachable on this side too.
    assert_eq!(
        evolve(
            None,
            &SessionEvent::Opened(SessionOpened::new(
                1,
                session(),
                evidence(2_000, None),
                policy(100, 1_000),
                at(1_000)
            ))
        ),
        Err(SessionDomainError::InvalidTimeOrder)
    );
    assert_eq!(
        evolve(
            None,
            &SessionEvent::Opened(SessionOpened::new(
                1,
                session(),
                evidence(999, Some(1_000)),
                policy(100, 1_000),
                at(1_000)
            ))
        ),
        Err(SessionDomainError::CredentialEvidenceExpired)
    );
    assert_eq!(
        evolve(
            None,
            &SessionEvent::Opened(SessionOpened::new(
                1,
                session(),
                evidence(u64::MAX, None),
                policy(1, 1),
                at(u64::MAX)
            ))
        ),
        Err(SessionDomainError::DeadlineOverflow)
    );

    // No failure produces a partial snapshot: the prior state is untouched by every rejection
    // above, and a rejected decision emits no event.
    assert_eq!(opened, base_session());
    assert_eq!(opened.revision(), 1);

    // Replay reconstructs a structurally equal snapshot after each legal prefix and across the
    // full lifecycle, from the events alone.
    let history = [
        decide(
            None,
            &open_command(1_000, evidence(1_000, None), policy(100, 1_000), 0),
        )
        .expect("fixture"),
        decide(Some(&opened), &refresh(1_050, 1)).expect("fixture"),
    ];
    let refreshed = evolve(Some(&opened), &history[1]).expect("fixture");
    let final_event = decide(Some(&refreshed), &expire(1_150, 2)).expect("fixture");
    let expired = evolve(Some(&refreshed), &final_event).expect("fixture");
    let full: Vec<SessionEvent> = vec![history[0].clone(), history[1].clone(), final_event.clone()];
    let expected = [opened.clone(), refreshed.clone(), expired.clone()];
    for prefix in 1..=full.len() {
        let mut replayed: Option<SessionSnapshot> = None;
        for event in full.iter().take(prefix) {
            replayed = Some(evolve(replayed.as_ref(), event).expect("replay must accept"));
        }
        let Some(replayed) = replayed else {
            panic!("replay produced no snapshot");
        };
        assert_eq!(replayed, expected[prefix - 1], "replay prefix {prefix}");
        assert_eq!(replayed.revision(), prefix as u64);
    }
    assert_eq!(
        expired.status(),
        SessionStatus::Expired {
            expired_at: at(1_150),
            observed_at: at(1_150),
            cause: SessionExpiryCause::Idle,
        }
    );

    // Both paths increment the revision with CHECKED arithmetic and fail closed on exhaustion.
    // The behaviour at `current_revision == u64::MAX` is proved by the private same-module
    // fixture §12 binds to this row's library leg; this is an additional carrier over the same
    // property, kept because §13's list is a floor rather than a ceiling. A wrapping or unchecked
    // increment fails both.
    for name in ["decide_existing", "evolve"] {
        let body = source_of(name);
        assert_eq!(
            body.matches("checked_add(1)").count(),
            1,
            "{name} must contain exactly its own checked increment"
        );
        assert!(
            body.contains("checked_add(1)"),
            "{name} must use a checked revision increment"
        );
        assert!(
            body.contains("SessionDomainError::RevisionOverflow"),
            "{name} must fail closed on revision exhaustion"
        );
        assert!(
            !body.contains("wrapping_add") && !body.contains("saturating_add"),
            "{name} must not wrap or saturate the revision"
        );
    }
}

/// `AUTH-020` — credential and dependency negative space.
#[test]
fn session_domain_has_no_credential_or_adapter_surface() {
    let opened = base_session();
    let command = open_command(1_000, evidence(1_000, None), policy(100, 1_000), 0);
    let event = decide(None, &command).expect("fixture");

    // `Debug` redaction is discharged at the one type that IS the digest, so every holder inherits
    // it rather than each having to remember. The canary must not survive on any of them.
    let surfaces = [
        format!("{:?}", digest()),
        format!("{:?}", evidence(1_000, None)),
        format!("{opened:?}"),
        format!("{command:?}"),
        format!("{event:?}"),
        format!(
            "{:?}",
            SessionEvent::Revoked(SessionRevoked::new(2, session(), at(1_050)))
        ),
    ];
    for rendered in &surfaces {
        assert!(
            !rendered.contains(DIGEST_CANARY),
            "a Debug surface leaked the credential-evidence digest: {rendered}"
        );
        assert!(!rendered.contains(DIGEST));
    }
    assert_eq!(
        format!("{:?}", digest()),
        "CredentialEvidenceDigest(<redacted>)"
    );

    // Errors report a kind and safe revisions only. They never echo rejected input, and neither
    // taxonomy renders secret-derived text.
    let Err(rejected_adapter) = AuthAdapterId::parse(RAW_CREDENTIAL_CANARY) else {
        panic!("raw credential text must not parse as an adapter id");
    };
    let Err(rejected_digest) = CredentialEvidenceDigest::parse(RAW_CREDENTIAL_CANARY) else {
        panic!("raw credential text must not parse as a digest");
    };
    for rendered in [
        rejected_adapter.to_string(),
        format!("{rejected_adapter:?}"),
        rejected_digest.to_string(),
        format!("{rejected_digest:?}"),
    ] {
        for fragment in ["hunter2", "password", "canary", DIGEST_CANARY] {
            assert!(
                !rendered.contains(fragment),
                "a value error echoed rejected input: {rendered}"
            );
        }
    }
    let domain_errors = [
        SessionDomainError::CredentialEvidenceExpired,
        SessionDomainError::InvalidTimeOrder,
        SessionDomainError::DeadlineOverflow,
        SessionDomainError::SessionNotFound,
        SessionDomainError::SessionAlreadyExists,
        SessionDomainError::SessionIdMismatch,
        SessionDomainError::RevisionMismatch {
            expected: 7,
            actual: 1,
        },
        SessionDomainError::RevisionOverflow,
        SessionDomainError::TerminalSession {
            status: SessionStatus::Revoked {
                revoked_at: at(1_050),
            },
        },
        SessionDomainError::NonMonotoneTime,
        SessionDomainError::SessionNotYetExpired,
        SessionDomainError::NoEffectiveRefresh,
        SessionDomainError::EventSequenceMismatch {
            expected: 2,
            actual: 0,
        },
        SessionDomainError::EventTimeOutsideValidity,
        SessionDomainError::IllegalEventForState,
        SessionDomainError::EventDerivedFieldMismatch {
            field: EventDerivedField::ExpiredAt,
        },
    ];
    // A length check over a locally written array proves nothing about the enum: adding a
    // seventeenth variant would leave it green. An exhaustive match does prove it, because a new
    // variant stops this file compiling.
    assert_eq!(domain_errors.len(), 16);
    for error in &domain_errors {
        let named = match error {
            SessionDomainError::CredentialEvidenceExpired
            | SessionDomainError::InvalidTimeOrder
            | SessionDomainError::DeadlineOverflow
            | SessionDomainError::SessionNotFound
            | SessionDomainError::SessionAlreadyExists
            | SessionDomainError::SessionIdMismatch
            | SessionDomainError::RevisionMismatch { .. }
            | SessionDomainError::RevisionOverflow
            | SessionDomainError::TerminalSession { .. }
            | SessionDomainError::NonMonotoneTime
            | SessionDomainError::SessionNotYetExpired
            | SessionDomainError::NoEffectiveRefresh
            | SessionDomainError::EventSequenceMismatch { .. }
            | SessionDomainError::EventTimeOutsideValidity
            | SessionDomainError::IllegalEventForState
            | SessionDomainError::EventDerivedFieldMismatch { .. } => true,
        };
        assert!(named, "the domain taxonomy is a closed set");
    }
    for error in &domain_errors {
        for rendered in [error.to_string(), format!("{error:?}")] {
            for fragment in [DIGEST_CANARY, DIGEST, "hunter2", "tenant:example"] {
                assert!(
                    !rendered.contains(fragment),
                    "a domain error rendered non-safe material: {rendered}"
                );
            }
        }
    }
    // `RevisionMismatch` and `TerminalSession` are the two variants allowed to report a fact.
    assert!(
        SessionDomainError::RevisionMismatch {
            expected: 7,
            actual: 1
        }
        .to_string()
        .contains('7')
    );

    // The B2-owned half of §9's non-echo guarantee, at the Serde boundary. When a value this
    // contract owns rejects a decoded primitive, the message that reaches the caller is B2's own
    // `Display` mapped through `de::Error::custom`, and it must not echo the rejected text — even
    // when that text is credential-shaped. The deserializer's OWN syntax and type diagnostics are
    // a different class: §9 scopes them out as untrusted boundary diagnostics owned by
    // `M00-B4 control-evidence`, so nothing here asserts anything about them, and `AUTH-020`'s
    // assertion text carries that exclusion rather than leaving it to be explained.
    let echoed = serde_json::from_str::<SessionCredentialEvidence>(&format!(
        concat!(
            "{{\"tenant_id\":\"tenant:example\",\"user_id\":\"user:example\",",
            "\"auth_adapter_id\":\"ustc.cas\",\"evidence_digest\":\"{}\",",
            "\"authenticated_at\":1000,\"credential_not_after\":null}}"
        ),
        RAW_CREDENTIAL_CANARY
    ));
    let Err(echoed) = echoed else {
        panic!("credential-shaped text must not decode as a digest");
    };
    let rendered = echoed.to_string();
    assert!(
        rendered.contains("digest shape is malformed"),
        "the B2 validator must be the thing that rejected it: {rendered}"
    );
    for fragment in ["hunter2", "password", "canary"] {
        assert!(
            !rendered.contains(fragment),
            "B2's validation mapping echoed the rejected input: {rendered}"
        );
    }

    // Structural decoding is not authentication: a well-formed evidence payload decodes, and the
    // state machine still applies every check afterwards.
    let decoded = serde_json::from_str::<SessionCredentialEvidence>(&evidence_json(
        ",\"credential_not_after\":null",
    ))
    .expect("fixture");
    assert_eq!(decoded, evidence(1_000, None));
    assert_eq!(
        decide(None, &open_command(999, decoded, policy(100, 1_000), 0)),
        Err(SessionDomainError::InvalidTimeOrder),
        "a successfully decoded evidence value proves shape, never admission"
    );

    // The serialized snapshot and event surfaces carry bounded provenance only — no raw
    // credential, cookie, token, provider payload or reason string.
    let snapshot_json = serde_json::to_string(&opened).expect("fixture");
    let event_json = serde_json::to_string(&event).expect("fixture");
    for encoded in [&snapshot_json, &event_json] {
        for forbidden in [
            "password", "cookie", "token", "secret", "bearer", "subject", "email", "role",
        ] {
            assert!(
                !encoded.contains(forbidden),
                "a serialized surface carried forbidden material: {encoded}"
            );
        }
    }
    assert!(snapshot_json.contains("\"status\":\"active\""));
    assert!(event_json.contains("\"opened\""));

    // The pure module declares no clock, RNG, transport, database, framework, auth-adapter or
    // ID-generation dependency, and references neither the Agent-facing tool protocol nor package
    // version semantics — including by path inside a function body, which declares no item.
    //
    // The scan runs over CODE, with comments removed and string-literal payloads emptied. Prose
    // that names a prohibited technology in order to prohibit it is not a carrier, and a literal
    // is not an import; scanning the raw file would make this assertion fail on its own
    // documentation and pass or fail on unrelated text. `strip_code` keeps every token boundary,
    // so a comment between two keywords cannot weld them into one identifier.
    let code = strip_code(SESSION_SOURCE);
    for forbidden in [
        "ustc_agent_tool_protocol",
        "semver",
        "uuid",
        "Uuid",
        "ulid",
        "Ulid",
        "nanoid",
        "NanoId",
        "rand",
        "Rng",
        "random",
        "generate",
        "mint",
        "SystemTime",
        "Instant::now",
        "chrono",
        "std::time",
        "std::net",
        "std::fs",
        "std::process",
        "std::env",
        "std::io",
        "TcpStream",
        "reqwest",
        "hyper",
        "sqlx",
        "diesel",
        "rusqlite",
        "axum",
        "dioxus",
        "cookie",
        "Cookie",
        "oauth",
        "OAuth",
        "oidc",
        "Oidc",
        "sha2::",
        "Sha256",
        "hmac",
        "Hmac",
        "hex::",
    ] {
        assert!(
            !code.contains(forbidden),
            "the session module gained a forbidden dependency carrier: {forbidden}"
        );
    }
    // …and it computes no digest: the only digest work is byte-class validation of an
    // already-supplied string.
    for forbidden in ["fn hash", "::digest(", ".update(", ".finalize("] {
        assert!(
            !code.contains(forbidden),
            "the session module must not compute a digest: {forbidden}"
        );
    }
    // Every `use` item of the module, in full: three from `std`/`serde` plus the one enumerated
    // cross-file identity binding, spelled without renaming. An import allowlist stated as a
    // complete set is what makes an unlisted import drift rather than something to notice.
    let imports: Vec<&str> = code
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("use ") || line.starts_with("pub use "))
        .collect();
    assert_eq!(
        imports,
        [
            "use std::error::Error;",
            "use std::fmt;",
            "use serde::de;",
            "use serde::{Deserialize, Deserializer, Serialize};",
            "use crate::identity::{SessionId, TenantId, UserId};",
            // The `#[cfg(test)] mod tests` fixture module §12 binds to the library legs of
            // AUTH-018 and AUTH-019. It is gated out of every non-test build and adds no public
            // item, which the public-declaration allowlist in the repository checker pins.
            "use super::*;",
        ],
        "the session module import surface drifted"
    );
    // It re-exports nothing, declares no submodule, defines no macro and splices no source, so no
    // second path to an admitted identity kind and no unread file can exist.
    for forbidden in [
        "pub use",
        "pub mod",
        "macro_rules!",
        "include!",
        "include_str!",
        "#[path",
        "extern crate",
        "#![",
    ] {
        assert!(
            !code.contains(forbidden),
            "the session module gained a forbidden item carrier: {forbidden}"
        );
    }
}

/// Returns the code of `fn <name>` up to the next item, for the two ordering properties below
/// that no reachable fixture can express.
///
/// Both terminators matter: slicing only on `\nfn ` runs straight through a following `pub fn`,
/// which would let the next function's carriers answer for this one. Comments are stripped, so a
/// carrier named in prose cannot satisfy an assertion about code.
fn source_of(name: &str) -> String {
    let code = strip_code(SESSION_SOURCE);
    let Some(start) = code.find(&format!("fn {name}(")) else {
        panic!("the session module lost {name}");
    };
    let tail = code[start..].to_owned();
    let end = [
        "\nfn ",
        "\npub fn ",
        "\nconst ",
        "\npub const ",
        "\nstruct ",
        "\nimpl ",
    ]
    .iter()
    .filter_map(|marker| tail[1..].find(marker).map(|offset| offset + 1))
    .min()
    .unwrap_or(tail.len());
    tail[..end].to_owned()
}

/// Removes comments and empties string/char literal payloads, preserving every token boundary.
///
/// Each removed span becomes one space rather than nothing: a stripper that deletes them welds the
/// neighbours together, so `extern/**/crate` would become the single identifier `externcrate`,
/// which no `extern crate` scan can see while Rust still reads two keywords.
fn strip_code(source: &str) -> String {
    let bytes: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        let rest_is = |needle: &str| source_slice_starts_with(&bytes, index, needle);
        if rest_is("//") {
            while index < bytes.len() && bytes[index] != '\n' {
                index += 1;
            }
            out.push(' ');
        } else if rest_is("/*") {
            let mut depth = 0_usize;
            while index < bytes.len() {
                if source_slice_starts_with(&bytes, index, "/*") {
                    depth += 1;
                    index += 2;
                } else if source_slice_starts_with(&bytes, index, "*/") {
                    depth -= 1;
                    index += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    index += 1;
                }
            }
            out.push(' ');
        } else if bytes[index] == '"' {
            index += 1;
            while index < bytes.len() && bytes[index] != '"' {
                index += if bytes[index] == '\\' { 2 } else { 1 };
            }
            index += 1;
            out.push_str("\"\"");
        } else if bytes[index] == '\'' && index + 2 < bytes.len() && bytes[index + 1] == '\\' {
            // A byte/char escape such as `b'\n'`; the simple literal case below covers `b'-'`.
            while index < bytes.len() && bytes[index] != '\'' {
                index += 1;
            }
            index += 1;
            while index < bytes.len() && bytes[index] != '\'' {
                index += 1;
            }
            index += 1;
            out.push_str("''");
        } else if bytes[index] == '\''
            && index + 2 < bytes.len()
            && bytes[index + 2] == '\''
            && bytes[index + 1] != '\''
        {
            index += 3;
            out.push_str("''");
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    out
}

/// Returns whether `chars[at..]` begins with `needle`.
fn source_slice_starts_with(chars: &[char], at: usize, needle: &str) -> bool {
    needle
        .chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(at + offset) == Some(&expected))
}
