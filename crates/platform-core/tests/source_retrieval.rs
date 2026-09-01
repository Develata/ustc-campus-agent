//! Offline-only `source-retrieval/v0` integration evidence.
//!
//! Every source/observation is synthetic. This suite opens no socket, performs
//! no DNS lookup, reads no clock, and approves no concrete campus source.

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use ustc_campus_agent_core::SourceAuthority;
use ustc_campus_agent_core::identity::CommandId;
use ustc_campus_agent_core::source_registry::{
    PublicIpPolicyVersion, RetrievalSubject, SourceAuthorityRevision, SourceDefinition, SourceId,
    SourceMediaType, SourceOwner, SourceRegistry, SourceRetrievalPolicy,
    SourceRetrievalProtocolVersion, SourceReviewEvidenceId, SourceReviewReceipt, SourceReviewerId,
    SourceUrl, SourceValueErrorKind,
};
use ustc_campus_agent_core::source_retrieval::{
    BodyAdmissionCandidate, BodyObservation, DnsTransportObservation, PeerBoundRetrievalCandidate,
    RateOverrideId, ResponseHeadObservation, RetrievalAttemptCommand, RetrievalAttemptId,
    RetrievalBodyFraming, RetrievalDnsName, RetrievalEpochSeconds, RetrievalOverrideEvidenceId,
    RetrievalOverrideFacts, RetrievalPlanCandidate, RetrievalPolicy, RetrievalPolicyError,
    RetrievalRateDecision, RetrievalRateOverrideRequest, RetrievalTransportSuccess,
    SourceOperatorId, SourceTransportError,
};

const BODY_CAP: u32 = 1_048_576;
const WIRE_OVERHEAD: u64 = 65_536;

fn source_id(value: &str) -> SourceId {
    SourceId::parse(value.to_owned()).expect("synthetic source id")
}

fn review_receipt() -> SourceReviewReceipt {
    SourceReviewReceipt::new(
        SourceReviewerId::parse("reviewer:synthetic").expect("reviewer"),
        SourceReviewEvidenceId::parse("evidence:review").expect("review evidence"),
        SourceReviewEvidenceId::parse("evidence:permission").expect("permission evidence"),
        SourceReviewEvidenceId::parse("evidence:rate").expect("rate evidence"),
        SourceReviewEvidenceId::parse("evidence:fixture").expect("fixture evidence"),
    )
}

fn subject_with_policy(
    source: &str,
    url: &str,
    minimum_interval_seconds: u32,
    maximum_response_bytes: u32,
    maximum_elapsed_seconds: u32,
) -> (RetrievalSubject, SourceAuthorityRevision) {
    let source = source_id(source);
    let policy = SourceRetrievalPolicy::new(
        minimum_interval_seconds,
        maximum_response_bytes,
        maximum_elapsed_seconds,
        SourceMediaType::parse("text/plain").expect("media"),
        SourceRetrievalProtocolVersion::V0StrictHttpsIpv4Http11_20260809,
        PublicIpPolicyVersion::V0Ipv4Only20260809,
    )
    .expect("policy");
    let definition = SourceDefinition::proposed(
        source.clone(),
        SourceOwner::parse("Synthetic Offline Fixture").expect("owner"),
        SourceUrl::parse(url).expect("url"),
        SourceAuthority::ReviewedOfficialSource,
        policy,
    )
    .expect("definition");
    let stale = definition.authority_revision();
    let mut registry = SourceRegistry::new();
    registry
        .propose(definition)
        .expect("propose synthetic source");
    registry
        .approve(&source, stale, review_receipt())
        .expect("approve synthetic fixture only");
    (registry.retrieval_subject(&source).expect("subject"), stale)
}

fn command(
    subject: &RetrievalSubject,
    override_request: Option<RetrievalRateOverrideRequest>,
) -> RetrievalAttemptCommand {
    RetrievalAttemptCommand::new(
        CommandId::parse("command:synthetic").expect("command"),
        RetrievalAttemptId::new("attempt:synthetic".to_owned()).expect("attempt"),
        subject.source_id().clone(),
        subject.source_authority_revision(),
        override_request,
    )
}

fn candidate(subject: &RetrievalSubject) -> RetrievalPlanCandidate {
    RetrievalPolicy::derive_candidate(subject, &command(subject, None)).expect("candidate")
}

fn peer_bound(subject: &RetrievalSubject) -> PeerBoundRetrievalCandidate {
    let observation = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        Vec::new(),
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("DNS shape");
    let resolved = RetrievalPolicy::authorize_resolution(candidate(subject), observation)
        .expect("resolution policy");
    RetrievalPolicy::authorize_peer(
        resolved,
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 443)),
    )
    .expect("peer policy")
}

fn authorized_body(subject: &RetrievalSubject, extra_headers: &str) -> BodyAdmissionCandidate {
    let raw = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n{extra_headers}\r\n");
    let head = RetrievalPolicy::parse_strict_response_head(raw.as_bytes()).expect("head");
    RetrievalPolicy::authorize_response_head(peer_bound(subject), head).expect("head policy")
}

#[test]
fn nominal_id_families_and_dns_names_are_exact_and_redacted() {
    macro_rules! check_id {
        ($kind:ty) => {{
            let value = <$kind>::new("a.b:c-d_e".to_owned()).expect("canonical id");
            assert_eq!(value.as_str(), "a.b:c-d_e");
            assert_eq!(value.clone().into_inner(), "a.b:c-d_e");
            let secret = "SECRET-RAW";
            let error = <$kind>::new(secret.to_owned()).expect_err("uppercase rejected");
            assert!(!format!("{error:?} {error}").contains(secret));
        }};
    }
    check_id!(RetrievalAttemptId);
    check_id!(RateOverrideId);
    check_id!(RetrievalOverrideEvidenceId);
    check_id!(SourceOperatorId);

    let dns = RetrievalDnsName::parse("a.example").expect("DNS name");
    assert_eq!(format!("{dns:?}"), "RetrievalDnsName(<redacted>)");
    for invalid in ["A.example", "a.example.", "-a.example", "a..example"] {
        let error = RetrievalDnsName::parse(invalid).expect_err("invalid DNS name");
        assert_eq!(error.kind(), SourceValueErrorKind::InvalidHost);
        assert!(!format!("{error:?} {error}").contains(invalid));
    }
}

#[test]
fn epoch_and_override_window_cover_boundaries_without_authority() {
    assert_eq!(RetrievalEpochSeconds::from_unix_seconds(0).get(), 0);
    assert_eq!(
        RetrievalEpochSeconds::from_unix_seconds(u64::MAX).get(),
        u64::MAX
    );
    let source = source_id("source:synthetic");
    let revision = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    )
    .0
    .source_authority_revision();
    let make = |issued, not_after| {
        RetrievalOverrideFacts::new(
            RetrievalOverrideEvidenceId::new("evidence:override".to_owned()).expect("evidence"),
            RateOverrideId::new("override:one".to_owned()).expect("override"),
            RetrievalAttemptId::new("attempt:synthetic".to_owned()).expect("attempt"),
            SourceOperatorId::new("operator:synthetic".to_owned()).expect("operator"),
            source.clone(),
            revision,
            RetrievalEpochSeconds::from_unix_seconds(issued),
            RetrievalEpochSeconds::from_unix_seconds(not_after),
        )
    };
    assert!(make(7, 7).is_ok());
    assert_eq!(
        make(8, 7).expect_err("reversed window").kind(),
        SourceValueErrorKind::InvalidOverrideWindow
    );
}

#[test]
fn derive_rejects_source_before_revision_and_retains_no_caller_url_knobs() {
    let (subject, stale_revision) = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    );
    let wrong_source_command = RetrievalAttemptCommand::new(
        CommandId::parse("command:synthetic").expect("command"),
        RetrievalAttemptId::new("attempt:synthetic".to_owned()).expect("attempt"),
        source_id("source:other"),
        stale_revision,
        None,
    );
    assert_eq!(
        RetrievalPolicy::derive_candidate(&subject, &wrong_source_command)
            .expect_err("source mismatch"),
        RetrievalPolicyError::AttemptSourceMismatch
    );
    let stale_command = RetrievalAttemptCommand::new(
        CommandId::parse("command:synthetic").expect("command"),
        RetrievalAttemptId::new("attempt:synthetic".to_owned()).expect("attempt"),
        subject.source_id().clone(),
        stale_revision,
        None,
    );
    assert_eq!(
        RetrievalPolicy::derive_candidate(&subject, &stale_command).expect_err("revision mismatch"),
        RetrievalPolicyError::StaleSourceAuthorityRevision
    );
}

#[test]
fn rate_table_is_exhaustive_at_boundaries_and_checks_override_exactness() {
    let (subject, _) = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    );
    let override_request = RetrievalRateOverrideRequest::new(
        RateOverrideId::new("override:one".to_owned()).expect("override"),
        RetrievalOverrideEvidenceId::new("evidence:one".to_owned()).expect("evidence"),
    );
    let command = command(&subject, Some(override_request));
    let rate_candidate = RetrievalPolicy::derive_candidate(&subject, &command).expect("candidate");
    let at = RetrievalEpochSeconds::from_unix_seconds;
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&rate_candidate, at(100), None, None, false),
        Ok(RetrievalRateDecision::Allowed)
    );
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&rate_candidate, at(100), Some(at(90)), None, false),
        Ok(RetrievalRateDecision::Allowed)
    );
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&rate_candidate, at(99), Some(at(100)), None, false),
        Err(RetrievalPolicyError::ClockRegression)
    );
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&rate_candidate, at(100), Some(at(95)), None, false),
        Err(RetrievalPolicyError::OverrideEvidenceUnavailable)
    );

    let facts = |evidence: &str, not_after: u64| {
        RetrievalOverrideFacts::new(
            RetrievalOverrideEvidenceId::new(evidence.to_owned()).expect("evidence"),
            RateOverrideId::new("override:one".to_owned()).expect("override"),
            RetrievalAttemptId::new("attempt:synthetic".to_owned()).expect("attempt"),
            SourceOperatorId::new("operator:synthetic".to_owned()).expect("operator"),
            subject.source_id().clone(),
            subject.source_authority_revision(),
            at(90),
            at(not_after),
        )
        .expect("facts")
    };
    let mismatched = facts("evidence:other", 110);
    assert_eq!(
        RetrievalPolicy::evaluate_rate(
            &rate_candidate,
            at(100),
            Some(at(95)),
            Some(&mismatched),
            false,
        ),
        Err(RetrievalPolicyError::InvalidRateOverride)
    );
    let expired = facts("evidence:one", 99);
    assert_eq!(
        RetrievalPolicy::evaluate_rate(
            &rate_candidate,
            at(100),
            Some(at(95)),
            Some(&expired),
            false,
        ),
        Err(RetrievalPolicyError::InvalidRateOverride)
    );
    let exact = facts("evidence:one", 100);
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&rate_candidate, at(100), Some(at(95)), Some(&exact), true,),
        Err(RetrievalPolicyError::RateOverrideAlreadyConsumed)
    );
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&rate_candidate, at(100), Some(at(95)), Some(&exact), false,),
        Ok(RetrievalRateDecision::AllowedWithOverride(
            RateOverrideId::new("override:one".to_owned()).expect("override")
        ))
    );

    let no_override_candidate = candidate(&subject);
    assert_eq!(
        RetrievalPolicy::evaluate_rate(&no_override_candidate, at(100), Some(at(95)), None, false,),
        Err(RetrievalPolicyError::RateLimitNotElapsed)
    );
}

#[test]
fn dns_observation_is_shape_only_and_has_exact_move_accessors() {
    let addresses = vec![Ipv4Addr::new(10, 0, 0, 1)];
    let observation = DnsTransportObservation::new(
        "not-canonical-but-shape-valid".to_owned(),
        vec!["ALIAS".to_owned()],
        addresses.clone(),
    )
    .expect("constructor must not apply domain policy");
    assert_eq!(observation.queried_host(), "not-canonical-but-shape-valid");
    assert_eq!(observation.cname_chain(), &["ALIAS".to_owned()]);
    assert_eq!(observation.complete_addresses(), addresses);
    let parts = observation.into_parts();
    assert_eq!(parts.0, "not-canonical-but-shape-valid");
    assert_eq!(parts.2, addresses);

    assert_eq!(
        DnsTransportObservation::new(String::new(), Vec::new(), addresses.clone())
            .expect_err("empty host shape"),
        SourceTransportError::ObservationShapeRejected
    );
    assert_eq!(
        DnsTransportObservation::new("x".to_owned(), Vec::new(), Vec::new())
            .expect_err("empty answers"),
        SourceTransportError::ObservationShapeRejected
    );
    assert_eq!(
        DnsTransportObservation::new(
            "x".to_owned(),
            Vec::new(),
            vec![Ipv4Addr::new(8, 8, 8, 8); 65],
        )
        .expect_err("answer cap"),
        SourceTransportError::ObservationShapeRejected
    );
}

#[test]
fn resolution_rejects_each_frozen_cidr_and_accepts_neighbors() {
    let (subject, _) = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    );
    let denied = [
        Ipv4Addr::new(0, 0, 0, 0),
        Ipv4Addr::new(10, 0, 0, 0),
        Ipv4Addr::new(100, 64, 0, 0),
        Ipv4Addr::new(127, 255, 255, 255),
        Ipv4Addr::new(169, 254, 255, 255),
        Ipv4Addr::new(172, 31, 255, 255),
        Ipv4Addr::new(192, 0, 0, 255),
        Ipv4Addr::new(192, 0, 2, 255),
        Ipv4Addr::new(192, 88, 99, 255),
        Ipv4Addr::new(192, 168, 255, 255),
        Ipv4Addr::new(198, 19, 255, 255),
        Ipv4Addr::new(198, 51, 100, 255),
        Ipv4Addr::new(203, 0, 113, 255),
        Ipv4Addr::new(239, 255, 255, 255),
        Ipv4Addr::new(255, 255, 255, 255),
    ];
    for address in denied {
        let observation =
            DnsTransportObservation::new("example.invalid".to_owned(), Vec::new(), vec![address])
                .expect("shape");
        assert_eq!(
            RetrievalPolicy::authorize_resolution(candidate(&subject), observation)
                .expect_err("reserved address"),
            RetrievalPolicyError::NonPublicAddress,
            "{address}"
        );
    }
    for address in [
        Ipv4Addr::new(9, 255, 255, 255),
        Ipv4Addr::new(11, 0, 0, 0),
        Ipv4Addr::new(100, 63, 255, 255),
        Ipv4Addr::new(100, 128, 0, 0),
        Ipv4Addr::new(172, 15, 255, 255),
        Ipv4Addr::new(172, 32, 0, 0),
        Ipv4Addr::new(193, 0, 0, 1),
        Ipv4Addr::new(198, 20, 0, 0),
        Ipv4Addr::new(203, 0, 114, 0),
        Ipv4Addr::new(223, 255, 255, 255),
    ] {
        let observation =
            DnsTransportObservation::new("example.invalid".to_owned(), Vec::new(), vec![address])
                .expect("shape");
        assert!(
            RetrievalPolicy::authorize_resolution(candidate(&subject), observation).is_ok(),
            "{address}"
        );
    }
}

#[test]
fn resolution_and_peer_binding_fail_closed_on_host_alias_count_family_and_port() {
    let (subject, _) = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    );
    let wrong_host = DnsTransportObservation::new(
        "other.invalid".to_owned(),
        Vec::new(),
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("shape");
    assert_eq!(
        RetrievalPolicy::authorize_resolution(candidate(&subject), wrong_host)
            .expect_err("host mismatch"),
        RetrievalPolicyError::DnsAliasViolation
    );
    let looped = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        vec!["alias.invalid".to_owned(), "alias.invalid".to_owned()],
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("shape");
    assert_eq!(
        RetrievalPolicy::authorize_resolution(candidate(&subject), looped).expect_err("alias loop"),
        RetrievalPolicyError::DnsAliasViolation
    );
    let too_deep = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        (0..9)
            .map(|index| format!("alias-{index}.invalid"))
            .collect(),
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("raw shape");
    assert_eq!(
        RetrievalPolicy::authorize_resolution(candidate(&subject), too_deep)
            .expect_err("CNAME depth cap"),
        RetrievalPolicyError::DnsAliasViolation
    );
    let canonical_alias = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        vec!["example.invalid".to_owned()],
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("raw shape");
    assert_eq!(
        RetrievalPolicy::authorize_resolution(candidate(&subject), canonical_alias)
            .expect_err("canonical host alias loop"),
        RetrievalPolicyError::DnsAliasViolation
    );
    let too_many = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        Vec::new(),
        (1..=17).map(|last| Ipv4Addr::new(8, 8, 8, last)).collect(),
    )
    .expect("raw shape");
    assert_eq!(
        RetrievalPolicy::authorize_resolution(candidate(&subject), too_many)
            .expect_err("policy answer cap"),
        RetrievalPolicyError::DnsAnswerCountViolation
    );

    let observation = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        Vec::new(),
        vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(9, 9, 9, 9)],
    )
    .expect("shape");
    let resolved = RetrievalPolicy::authorize_resolution(candidate(&subject), observation)
        .expect("resolution");
    assert_eq!(
        RetrievalPolicy::authorize_peer(
            resolved,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(9, 9, 9, 9), 443)),
        )
        .expect_err("not selected lowest peer"),
        RetrievalPolicyError::PeerAddressMismatch
    );

    let observation = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        Vec::new(),
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("shape");
    let resolved = RetrievalPolicy::authorize_resolution(candidate(&subject), observation)
        .expect("resolution");
    assert_eq!(
        RetrievalPolicy::authorize_peer(
            resolved,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 80)),
        )
        .expect_err("wrong port"),
        RetrievalPolicyError::PeerAddressMismatch
    );

    let observation = DnsTransportObservation::new(
        "example.invalid".to_owned(),
        Vec::new(),
        vec![Ipv4Addr::new(8, 8, 8, 8)],
    )
    .expect("shape");
    let resolved = RetrievalPolicy::authorize_resolution(candidate(&subject), observation)
        .expect("resolution");
    assert_eq!(
        RetrievalPolicy::authorize_peer(
            resolved,
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 443, 0, 0)),
        )
        .expect_err("IPv6 denied"),
        RetrievalPolicyError::UnsupportedAddressFamily
    );
}

#[test]
fn strict_response_parser_rejects_line_folding_limits_and_status_classes() {
    for raw in [
        b"HTTP/1.1 200 OK\nContent-Type: text/plain\n\n".as_slice(),
        b"HTTP/1.1 200 OK\r\n folded: no\r\n\r\n".as_slice(),
        b"HTTP/1.1 200 OK\r\nBad Name: x\r\n\r\n".as_slice(),
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n".as_slice(),
    ] {
        assert_eq!(
            RetrievalPolicy::parse_strict_response_head(raw),
            Err(RetrievalPolicyError::MalformedResponseHead)
        );
    }
    assert_eq!(
        RetrievalPolicy::parse_strict_response_head(&vec![b'x'; 32_769]),
        Err(RetrievalPolicyError::HeaderLimitExceeded)
    );
    let too_many_fields = format!(
        "HTTP/1.1 200 OK\r\n{}\r\n",
        "X-Field: value\r\n".repeat(129)
    );
    assert_eq!(
        RetrievalPolicy::parse_strict_response_head(too_many_fields.as_bytes()),
        Err(RetrievalPolicyError::HeaderLimitExceeded)
    );
    let long_name = format!("HTTP/1.1 200 OK\r\n{}: x\r\n\r\n", "a".repeat(65));
    assert_eq!(
        RetrievalPolicy::parse_strict_response_head(long_name.as_bytes()),
        Err(RetrievalPolicyError::HeaderLimitExceeded)
    );
    let long_value = format!("HTTP/1.1 200 OK\r\nX: {}\r\n\r\n", "a".repeat(8_193));
    assert_eq!(
        RetrievalPolicy::parse_strict_response_head(long_value.as_bytes()),
        Err(RetrievalPolicyError::HeaderLimitExceeded)
    );

    let (subject, _) = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    );
    for (status, expected) in [
        ("101 Switching", RetrievalPolicyError::InterimResponseDenied),
        ("302 Found", RetrievalPolicyError::RedirectDenied),
        ("404 Missing", RetrievalPolicyError::UnexpectedStatus),
    ] {
        let raw = format!("HTTP/1.1 {status}\r\nContent-Type: text/plain\r\n\r\n");
        let head = RetrievalPolicy::parse_strict_response_head(raw.as_bytes()).expect("head");
        assert_eq!(
            RetrievalPolicy::authorize_response_head(peer_bound(&subject), head)
                .expect_err("status rejected"),
            expected
        );
    }
    let head = RetrievalPolicy::parse_strict_response_head(
        b"HTTP/1.0 200 OK\r\nContent-Type: text/plain\r\n\r\n",
    )
    .expect("head");
    assert_eq!(
        RetrievalPolicy::authorize_response_head(peer_bound(&subject), head)
            .expect_err("version rejected"),
        RetrievalPolicyError::UnexpectedHttpVersion
    );
}

#[test]
fn response_authorization_enforces_content_codings_framing_and_trailers() {
    let (subject, _) = subject_with_policy(
        "source:synthetic",
        "https://example.invalid/data",
        10,
        64,
        1,
    );
    let cases = [
        (
            "Content-Type: text/plain\r\nContent-Type: text/plain\r\n",
            RetrievalPolicyError::InvalidContentType,
        ),
        (
            "Content-Type: application/json\r\n",
            RetrievalPolicyError::UnexpectedContentType,
        ),
        (
            "Content-Type: text/plain; charset=utf-8; charset=utf-8\r\n",
            RetrievalPolicyError::InvalidContentType,
        ),
        (
            "Content-Type: text/plain; charset=utf 8\r\n",
            RetrievalPolicyError::InvalidContentType,
        ),
        (
            "Content-Type: text/plain\r\nContent-Encoding: gzip\r\n",
            RetrievalPolicyError::UnsupportedContentEncoding,
        ),
        (
            "Content-Type: text/plain\r\nTransfer-Encoding: gzip\r\n",
            RetrievalPolicyError::UnsupportedTransferCoding,
        ),
        (
            "Content-Type: text/plain\r\nTransfer-Encoding: gzip\r\nContent-Length: 1\r\n",
            RetrievalPolicyError::UnsupportedTransferCoding,
        ),
        (
            "Content-Type: text/plain\r\nTransfer-Encoding: chunked\r\nContent-Length: 1\r\n",
            RetrievalPolicyError::AmbiguousFraming,
        ),
        (
            "Content-Type: text/plain\r\nContent-Length: 65\r\n",
            RetrievalPolicyError::DeclaredBodyTooLarge,
        ),
    ];
    for (headers, expected) in cases {
        let raw = format!("HTTP/1.1 200 OK\r\n{headers}\r\n");
        let head = RetrievalPolicy::parse_strict_response_head(raw.as_bytes()).expect("head");
        assert_eq!(
            RetrievalPolicy::authorize_response_head(peer_bound(&subject), head)
                .expect_err("head policy"),
            expected,
            "{headers:?}"
        );
    }

    let head = RetrievalPolicy::parse_strict_response_head(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=\"utf 8\"\r\nContent-Length: 0\r\n\r\n",
    )
    .expect("quoted parameter head");
    assert!(RetrievalPolicy::authorize_response_head(peer_bound(&subject), head).is_ok());

    let admission = authorized_body(&subject, "Trailer: checksum\r\n");
    let body = BodyObservation::new(Vec::new(), 0, 0, 0, false, 0, true, 0).expect("body shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("trailer denied"),
        RetrievalPolicyError::TrailerDenied
    );
}

#[test]
fn body_shape_and_policy_bounds_are_independent_and_ordered() {
    assert!(
        BodyObservation::new(
            vec![0; 1_048_577],
            u64::MAX,
            u32::MAX,
            u16::MAX,
            true,
            u16::MAX,
            false,
            u64::MAX
        )
        .is_ok()
    );
    assert_eq!(
        BodyObservation::new(vec![0; 1_048_578], 0, 0, 0, false, 0, true, 0)
            .expect_err("representation cap"),
        SourceTransportError::ObservationShapeRejected
    );

    let (subject, _) =
        subject_with_policy("source:synthetic", "https://example.invalid/data", 10, 4, 1);
    let admission = authorized_body(&subject, "Transfer-Encoding: chunked\r\n");
    let body = BodyObservation::new(vec![0; 5], 5, 4_097, 0, false, 0, true, 0).expect("shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("chunk precedence"),
        RetrievalPolicyError::ChunkLimitExceeded
    );
    let admission = authorized_body(&subject, "Transfer-Encoding: chunked\r\n");
    let body = BodyObservation::new(vec![0; 5], 5, 4_097, 129, true, 1, false, 0)
        .expect("combined invalid body shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body)
            .expect_err("ambiguous framing has global precedence"),
        RetrievalPolicyError::AmbiguousFraming
    );
    let admission = authorized_body(&subject, "Transfer-Encoding: chunked\r\n");
    let body = BodyObservation::new(Vec::new(), 0, 1, 129, false, 0, true, 0).expect("shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("chunk line cap"),
        RetrievalPolicyError::ChunkLimitExceeded
    );
    for invalid_width in [0, 17] {
        let admission = authorized_body(&subject, "Transfer-Encoding: chunked\r\n");
        let body = BodyObservation::new(Vec::new(), 0, 1, invalid_width, false, 0, true, 0)
            .expect("shape");
        assert_eq!(
            RetrievalPolicy::finish_body(admission, body).expect_err("chunk-size digit bound"),
            RetrievalPolicyError::ChunkLimitExceeded
        );
    }
    let admission = authorized_body(&subject, "Transfer-Encoding: chunked\r\n");
    let body = BodyObservation::new(vec![0; 4], 4, 1, 16, false, 0, true, 0).expect("shape");
    assert!(RetrievalPolicy::finish_body(admission, body).is_ok());
    let admission = authorized_body(&subject, "Transfer-Encoding: chunked\r\n");
    let body = BodyObservation::new(Vec::new(), 0, 1, 1, true, 0, true, 0).expect("shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("chunk extension denied"),
        RetrievalPolicyError::ChunkLimitExceeded
    );

    let admission = authorized_body(&subject, "");
    let body = BodyObservation::new(
        vec![0; 5],
        4 + WIRE_OVERHEAD + 1,
        0,
        0,
        false,
        0,
        true,
        1_001,
    )
    .expect("shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("wire precedence"),
        RetrievalPolicyError::WireLimitExceeded
    );

    let admission = authorized_body(&subject, "");
    let body = BodyObservation::new(vec![0; 5], 5, 0, 0, false, 0, true, 1_001).expect("shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("body precedence"),
        RetrievalPolicyError::BodyLimitExceeded
    );

    let admission = authorized_body(&subject, "");
    let body = BodyObservation::new(vec![0; 4], 4, 0, 0, false, 0, true, 1_001).expect("shape");
    assert_eq!(
        RetrievalPolicy::finish_body(admission, body).expect_err("deadline"),
        RetrievalPolicyError::DeadlineExceeded
    );

    let admission = authorized_body(&subject, "Content-Length: 4\r\n");
    let body =
        BodyObservation::new(vec![1, 2, 3, 4], 4, 0, 0, false, 0, true, 1_000).expect("shape");
    assert!(RetrievalPolicy::finish_body(admission, body).is_ok());
}

#[test]
fn transport_success_is_shape_only_bounded_and_moves_parts_once() {
    let dns = DnsTransportObservation::new(
        "not-domain-authorized".to_owned(),
        Vec::new(),
        vec![Ipv4Addr::new(10, 0, 0, 1)],
    )
    .expect("shape only");
    let success = RetrievalTransportSuccess::new(
        b"raw".to_vec(),
        vec![7; 8],
        9,
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 1)),
        dns,
        RetrievalBodyFraming::CloseDelimited,
        10,
    )
    .expect("success constructor is shape-only");
    assert_eq!(success.response_head_bytes(), b"raw");
    assert_eq!(success.body_bytes(), &[7; 8]);
    assert_eq!(success.wire_bytes_after_headers(), 9);
    assert_eq!(success.elapsed_milliseconds(), 10);
    let parts = success.into_parts();
    assert_eq!(parts.response_head_bytes, b"raw");
    assert_eq!(parts.body_bytes, vec![7; 8]);
    assert_eq!(parts.framing, RetrievalBodyFraming::CloseDelimited);

    let dns =
        DnsTransportObservation::new("x".to_owned(), Vec::new(), vec![Ipv4Addr::new(8, 8, 8, 8)])
            .expect("shape");
    assert_eq!(
        RetrievalTransportSuccess::new(
            Vec::new(),
            vec![0; 1_048_578],
            0,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 443)),
            dns,
            RetrievalBodyFraming::CloseDelimited,
            0,
        )
        .expect_err("body representation cap"),
        SourceTransportError::ObservationShapeRejected
    );

    let dns =
        DnsTransportObservation::new("x".to_owned(), Vec::new(), vec![Ipv4Addr::new(8, 8, 8, 8)])
            .expect("shape");
    assert_eq!(
        RetrievalTransportSuccess::new(
            Vec::new(),
            Vec::new(),
            u64::from(BODY_CAP) + WIRE_OVERHEAD + 1,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(8, 8, 8, 8), 443)),
            dns,
            RetrievalBodyFraming::CloseDelimited,
            0,
        )
        .expect_err("wire representation cap"),
        SourceTransportError::ObservationShapeRejected
    );
}

#[test]
fn complete_error_families_are_payload_free_and_stable() {
    let transport = [
        SourceTransportError::DnsUnavailable,
        SourceTransportError::ConnectFailed,
        SourceTransportError::TlsFailed,
        SourceTransportError::WriteFailed,
        SourceTransportError::ReadFailed,
        SourceTransportError::EofFramingFailure,
        SourceTransportError::ExecutionDeadline,
        SourceTransportError::TransportCancelled,
        SourceTransportError::ObservationShapeRejected,
    ];
    assert_eq!(transport.len(), 9);
    let policy = [
        RetrievalPolicyError::AttemptIdConflict,
        RetrievalPolicyError::CommandIdConflict,
        RetrievalPolicyError::RetrievalProtocolVersionMismatch,
        RetrievalPolicyError::AttemptSourceMismatch,
        RetrievalPolicyError::ValidatedCandidateMismatch,
        RetrievalPolicyError::MissingAttempt,
        RetrievalPolicyError::AttemptCompletionConflict,
        RetrievalPolicyError::MissingOrTerminalSession,
        RetrievalPolicyError::RequestContextMismatch,
        RetrievalPolicyError::OperatorPolicyUnavailable,
        RetrievalPolicyError::UnauthorizedSourceOperator,
        RetrievalPolicyError::SourceNotRetrievable,
        RetrievalPolicyError::StaleSourceAuthorityRevision,
        RetrievalPolicyError::ClockUnavailable,
        RetrievalPolicyError::ClockRegression,
        RetrievalPolicyError::OverrideEvidenceUnavailable,
        RetrievalPolicyError::InvalidRateOverride,
        RetrievalPolicyError::RateOverrideAlreadyConsumed,
        RetrievalPolicyError::RateLimitNotElapsed,
        RetrievalPolicyError::LeaseUnavailable,
        RetrievalPolicyError::LeaseTimeOverflow,
        RetrievalPolicyError::LeaseExpired,
        RetrievalPolicyError::InvalidStartAuthorization,
        RetrievalPolicyError::StartAuthorizationAlreadyConsumed,
        RetrievalPolicyError::AdmissionStoreUnavailable,
        RetrievalPolicyError::PublicIpPolicyVersionMismatch,
        RetrievalPolicyError::DnsAliasViolation,
        RetrievalPolicyError::DnsAnswerCountViolation,
        RetrievalPolicyError::UnsupportedAddressFamily,
        RetrievalPolicyError::NonPublicAddress,
        RetrievalPolicyError::PeerAddressMismatch,
        RetrievalPolicyError::MalformedResponseHead,
        RetrievalPolicyError::UnexpectedHttpVersion,
        RetrievalPolicyError::InterimResponseDenied,
        RetrievalPolicyError::RedirectDenied,
        RetrievalPolicyError::UnexpectedStatus,
        RetrievalPolicyError::HeaderLimitExceeded,
        RetrievalPolicyError::InvalidContentType,
        RetrievalPolicyError::UnexpectedContentType,
        RetrievalPolicyError::UnsupportedContentEncoding,
        RetrievalPolicyError::UnsupportedTransferCoding,
        RetrievalPolicyError::AmbiguousFraming,
        RetrievalPolicyError::DeclaredBodyTooLarge,
        RetrievalPolicyError::ChunkLimitExceeded,
        RetrievalPolicyError::TrailerDenied,
        RetrievalPolicyError::WireLimitExceeded,
        RetrievalPolicyError::BodyLimitExceeded,
        RetrievalPolicyError::DeadlineExceeded,
    ];
    assert_eq!(policy.len(), 48);
    for rendered in transport
        .iter()
        .map(ToString::to_string)
        .chain(policy.iter().map(ToString::to_string))
    {
        assert!(!rendered.contains("example"));
        assert!(!rendered.contains("SECRET"));
    }
}

// Name the intentionally opaque public outputs in external code. Their values
// remain constructible only through the pure phase chain.
#[allow(dead_code)]
fn opaque_output_types_are_public(
    _: ResponseHeadObservation,
    _: RetrievalPlanCandidate,
    _: PeerBoundRetrievalCandidate,
    _: BodyAdmissionCandidate,
) {
}
