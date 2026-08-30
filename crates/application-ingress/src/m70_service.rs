use ustc_campus_agent_change_radar::{BoardId, ChangeFeedQueryError, ChangeFeedReceipt};
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, ClientErrorDto, ClientResponseDto, EchoPayloadDto, M10WireErrorDto,
    M70ChangeFeedOutcomeDto, M70ChangeFeedTerminalDto, MAX_FRAME_BYTES, RetryabilityDto,
    SubmitChangeFeedDto, WireErrorClassDto, WireText, change_feed_payload_digest,
};
use ustc_campus_agent_core::identity::{CorrelationId, RequestId, SessionId};
use ustc_campus_agent_core::request_context::{
    ActorReference, ClientProvenance, M00AdmissionResult, M00AdmittedActor, OperationId,
    PayloadDigest, PublicScope, RequestAdmissionCoordinator,
};

use crate::capability::constant_time_eq;
use crate::m00_projection::project_rejection;
use crate::m70_projection::project_change_feed;
use crate::service::M10AdmissionPorts;

pub trait ChangeFeedInvocationPort: Send + Sync {
    fn invoke(
        &self,
        actor: &M00AdmittedActor,
        board_id: &BoardId,
    ) -> Result<ChangeFeedInvocationOutcome, ChangeFeedInvocationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeFeedInvocationOutcome {
    Found(ChangeFeedReceipt),
    NotFound(BoardId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeFeedInvocationError {
    Downstream(ChangeFeedQueryError),
    Denied,
    Unavailable,
    Internal,
}

pub struct M10ChangeFeedService<'a> {
    change_feed: &'a dyn ChangeFeedInvocationPort,
}

impl<'a> M10ChangeFeedService<'a> {
    #[must_use]
    pub const fn new(change_feed: &'a dyn ChangeFeedInvocationPort) -> Self {
        Self { change_feed }
    }

    pub fn submit<P: M10AdmissionPorts>(
        &self,
        request: &SubmitChangeFeedDto,
        ports: &mut P,
    ) -> ClientResponseDto {
        let expected_digest = match change_feed_payload_digest(&request.board_id) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(),
        };
        if !constant_time_eq(
            request.payload_digest.as_str().as_bytes(),
            expected_digest.as_str().as_bytes(),
        ) {
            return malformed_command_error();
        }
        let board_id = match BoardId::parse(request.board_id.as_str()) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(),
        };
        let command = match build_command(request) {
            Ok(value) => value,
            Err(_) => return malformed_command_error(),
        };
        let staged = ports.staged_operation();
        let staged_identity = staged.snapshot_identity().clone();
        let disposition = match RequestAdmissionCoordinator.admit(&command, ports) {
            M00AdmissionResult::Rejected(rejection)
            | M00AdmissionResult::PriorRejected(rejection) => {
                return ClientResponseDto::Error {
                    error: project_rejection(&rejection),
                };
            }
            M00AdmissionResult::Incomplete(value) => {
                return ClientResponseDto::Incomplete {
                    command_id: wire(value.command_id().as_str()),
                    retry_not_before: ustc_campus_agent_client_protocol::UnixMillis::new(
                        i64::try_from(value.retry_not_before().as_unix_millis())
                            .unwrap_or(i64::MAX),
                    ),
                };
            }
            M00AdmissionResult::Admitted { disposition, .. }
            | M00AdmissionResult::PriorAdmitted(disposition) => disposition,
        };
        if disposition.descriptor_snapshot_id() != &staged_identity {
            return internal_error("change_descriptor_identity_drift");
        }
        let outcome = match self
            .change_feed
            .invoke(disposition.admitted_actor(), &board_id)
        {
            Ok(value) => value,
            Err(error) => return map_invocation_error(error),
        };
        let terminal = match outcome {
            ChangeFeedInvocationOutcome::Found(receipt) => match project_change_feed(&receipt) {
                Ok(value) => value,
                Err(_) => return internal_error("change_projection_invariant"),
            },
            ChangeFeedInvocationOutcome::NotFound(board_id) => {
                M70ChangeFeedTerminalDto::new(M70ChangeFeedOutcomeDto::NotFound {
                    board_id: wire(board_id.as_str()),
                })
            }
        };
        let response = ClientResponseDto::ChangeFeedAccepted {
            command_id: wire(disposition.command_id().as_str()),
            terminal: Box::new(terminal),
        };
        match serde_json::to_vec(&response) {
            Ok(bytes) if bytes.len() <= MAX_FRAME_BYTES => response,
            _ => infrastructure_error("change_feed_frame_overflow"),
        }
    }
}

fn build_command(
    request: &SubmitChangeFeedDto,
) -> Result<ustc_campus_agent_core::request_context::BuildRequestContextCommand, &'static str> {
    let request_id = RequestId::parse(request.request_id.as_str()).map_err(|_| "request id")?;
    let correlation_id =
        CorrelationId::parse(request.correlation_id.as_str()).map_err(|_| "correlation id")?;
    let actor_reference = match &request.actor {
        ActorIntentDto::Public => ActorReference::Anonymous { scope: PublicScope },
        ActorIntentDto::Authenticated { session_id } => ActorReference::Authenticated {
            session_id: SessionId::parse(session_id.as_str()).map_err(|_| "session id")?,
        },
    };
    let causation_id = request
        .causation_id
        .as_ref()
        .map(|value| {
            ustc_campus_agent_core::request_context::CausationId::parse(value.as_str())
                .map_err(|_| "causation id")
        })
        .transpose()?;
    let idempotency_key = request
        .idempotency_key
        .as_ref()
        .map(|value| {
            ustc_campus_agent_core::request_context::IdempotencyKey::parse(value.as_str())
                .map_err(|_| "idempotency key")
        })
        .transpose()?;
    let provenance = ClientProvenance::new(
        request.provenance.build.as_str(),
        request.provenance.target.as_str(),
        request.provenance.protocol.as_str(),
    )
    .map_err(|_| "client provenance")?;
    let digest =
        PayloadDigest::parse(request.payload_digest.as_str()).map_err(|_| "payload digest")?;
    let operation_id = OperationId::parse("change.list").map_err(|_| "operation id")?;
    Ok(
        ustc_campus_agent_core::request_context::BuildRequestContextCommand::new(
            request_id,
            operation_id,
            actor_reference,
            correlation_id,
            causation_id,
            idempotency_key,
            provenance,
            digest,
        ),
    )
}

fn map_invocation_error(error: ChangeFeedInvocationError) -> ClientResponseDto {
    match error {
        ChangeFeedInvocationError::Downstream(ChangeFeedQueryError::Repository(_))
        | ChangeFeedInvocationError::Unavailable => {
            infrastructure_error("change_invocation_unavailable")
        }
        ChangeFeedInvocationError::Downstream(ChangeFeedQueryError::Projection)
        | ChangeFeedInvocationError::Internal => internal_error("change_invocation_internal"),
        ChangeFeedInvocationError::Denied => invocation_denied_error(),
    }
}

fn invocation_denied_error() -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::PolicyDenied,
        RetryabilityDto::NotRetryable,
        wire("policy_denied"),
        EchoPayloadDto::PolicyDenied {
            operation_id: wire("change.list"),
            permission_class: wire("public_read"),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("change_invocation_denial_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn infrastructure_error(code: &str) -> ClientResponseDto {
    ClientResponseDto::Error {
        error: ClientErrorDto::Infrastructure {
            retryable: code.ends_with("retry") || code.contains("unavailable"),
            wire_code: wire(code),
        },
    }
}

fn internal_error(code: &str) -> ClientResponseDto {
    ClientResponseDto::Error {
        error: ClientErrorDto::InternalInvariant {
            wire_code: wire(code),
        },
    }
}

fn malformed_command_error() -> ClientResponseDto {
    let error = match M10WireErrorDto::try_new(
        WireErrorClassDto::MalformedCommand,
        RetryabilityDto::RetryableAfterChange,
        wire("malformed_command"),
        EchoPayloadDto::Operation {
            operation_id: wire("change.list"),
        },
    ) {
        Ok(value) => value,
        Err(_) => return internal_error("change_malformed_command_projection"),
    };
    ClientResponseDto::Error {
        error: ClientErrorDto::Admission { error },
    }
}

fn wire(value: &str) -> WireText {
    WireText::parse(value).unwrap_or_else(|_| WireText::fallback())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use ustc_campus_agent_change_radar::{
        AcceptedObservation, BoardFeedPolicy, BoardPolicy, ChangeFeedQueryService,
        ChangePublicationService, ChangeRadarService, ChangeReviewReceipt,
        InMemoryChangeRadarRepository, M60ChangePublicationOutcome, M60ChangePublicationPort,
        M60ChangePublicationPortError, M60VerifiedChangeEvidence, NormalizedFacts,
        ObservationOutcome, SemanticField, SemanticValue,
    };
    use ustc_campus_agent_client_protocol::{ClientProvenanceDto, write_frame};
    use ustc_campus_agent_core::identity::{CommandId, UserId};
    use ustc_campus_agent_core::request_context::{
        ActorKind, AdapterAllowlist, AdapterIdentity, AdmissionPortError, AdmissionPorts,
        CapabilityDisposition, DecoderIdentity, DescriptorSnapshotError, DescriptorSnapshotId,
        DispatcherIdentity, EffectClass, EnvelopeHash, FinalAdmissionDisposition,
        FinalizeIdempotencyOutcome, IdempotencyError, IdempotencyKey, IdempotencyReservation,
        IdempotencyReservationToken, OperationDescriptorProjection, OperationSnapshot,
        PermissionClass, PlatformPolicySnapshotId, PolicyCurrentnessFact, PolicyResolution,
        SchemaDigest, SchemaIdentity,
    };
    use ustc_campus_agent_core::session::{SessionInstant, SessionSnapshot};
    use ustc_campus_agent_core::source_registry::{
        SourceId, SourceReviewEvidenceId, SourceReviewerId, SourceUrl,
    };
    use ustc_campus_agent_core::source_revision::{
        EffectiveInterval, NormalizedSnapshotId, ParserIdentity, RawSnapshotId, RevisionSha256,
        RevisionTimestamp, SourceRevision, SourceRevisionHealth,
    };

    const SCHEMA_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const LARGE_ENTRY_COUNT: usize = 12;

    struct ChangeListDescriptor {
        operation_id: OperationId,
        schema_identity: SchemaIdentity,
        schema_digest: SchemaDigest,
        permission_class: PermissionClass,
        effect_class: EffectClass,
        decoder_identity: DecoderIdentity,
        dispatcher_identity: DispatcherIdentity,
        adapter_allowlist: AdapterAllowlist,
        snapshot_identity: DescriptorSnapshotId,
    }

    impl ChangeListDescriptor {
        fn new() -> Self {
            let schema_digest = SchemaDigest::parse(SCHEMA_DIGEST).expect("fixture digest");
            Self {
                operation_id: OperationId::parse("change.list").expect("fixture operation"),
                schema_identity: SchemaIdentity::parse("schema:change-feed-test")
                    .expect("fixture schema identity"),
                schema_digest: schema_digest.clone(),
                permission_class: PermissionClass::PublicRead,
                effect_class: EffectClass::Read,
                decoder_identity: DecoderIdentity::parse("decoder:change-feed:v1")
                    .expect("fixture decoder"),
                dispatcher_identity: DispatcherIdentity::parse("dispatcher:change-feed:v1")
                    .expect("fixture dispatcher"),
                adapter_allowlist: AdapterAllowlist::try_from_iter([AdapterIdentity::parse(
                    "adapter:fixture",
                )
                .expect("fixture adapter")])
                .expect("fixture allowlist"),
                snapshot_identity: DescriptorSnapshotId::from_canonical_identity(&schema_digest, 1)
                    .expect("fixture descriptor id"),
            }
        }
    }

    impl OperationDescriptorProjection for ChangeListDescriptor {
        fn operation_id(&self) -> &OperationId {
            &self.operation_id
        }
        fn schema_identity(&self) -> &SchemaIdentity {
            &self.schema_identity
        }
        fn schema_digest(&self) -> &SchemaDigest {
            &self.schema_digest
        }
        fn permission_class(&self) -> PermissionClass {
            self.permission_class
        }
        fn effect_class(&self) -> EffectClass {
            self.effect_class
        }
        fn decoder_identity(&self) -> &DecoderIdentity {
            &self.decoder_identity
        }
        fn dispatcher_identity(&self) -> &DispatcherIdentity {
            &self.dispatcher_identity
        }
        fn adapter_allowlist(&self) -> &AdapterAllowlist {
            &self.adapter_allowlist
        }
        fn snapshot_identity(&self) -> &DescriptorSnapshotId {
            &self.snapshot_identity
        }
    }

    struct ChangeFeedTestPorts {
        snapshot: OperationSnapshot,
    }

    impl ChangeFeedTestPorts {
        fn new() -> Self {
            Self {
                snapshot: std::sync::Arc::new(ChangeListDescriptor::new()),
            }
        }
    }

    impl AdmissionPorts for ChangeFeedTestPorts {
        fn reserve_or_retrieve_idempotency(
            &mut self,
            _key: Option<&IdempotencyKey>,
            _envelope_hash: &EnvelopeHash,
        ) -> Result<IdempotencyReservation, IdempotencyError> {
            Ok(IdempotencyReservation::New(
                IdempotencyReservationToken::from_store_observation(
                    CommandId::parse("command:change-feed-test").expect("fixture command"),
                    1,
                    1,
                    SessionInstant::from_unix_millis(1_100),
                )
                .expect("fixture token"),
            ))
        }

        fn request_scoped_operation(
            &mut self,
        ) -> Result<OperationSnapshot, DescriptorSnapshotError> {
            Ok(std::sync::Arc::clone(&self.snapshot))
        }

        fn now(&mut self) -> Result<SessionInstant, AdmissionPortError> {
            Ok(SessionInstant::from_unix_millis(1_000))
        }

        fn resolve_policy(
            &mut self,
            _operation_id: &OperationId,
            _observed_at: SessionInstant,
        ) -> Result<PolicyResolution, AdmissionPortError> {
            Ok(PolicyResolution::new(
                PlatformPolicySnapshotId::parse("policy:fixture").expect("fixture policy"),
                PolicyCurrentnessFact::Current,
            ))
        }

        fn load_session(
            &mut self,
            _session_id: &SessionId,
        ) -> Result<Option<SessionSnapshot>, AdmissionPortError> {
            Ok(None)
        }

        fn check_capability(
            &mut self,
            _operation_id: &OperationId,
            _actor_kind: ActorKind,
            _observed_at: SessionInstant,
        ) -> Result<CapabilityDisposition, AdmissionPortError> {
            Ok(CapabilityDisposition::Enabled)
        }

        fn finalize_idempotency(
            &mut self,
            _token: &IdempotencyReservationToken,
            _disposition: &FinalAdmissionDisposition,
        ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError> {
            Ok(FinalizeIdempotencyOutcome::Committed)
        }
    }

    impl M10AdmissionPorts for ChangeFeedTestPorts {
        fn staged_operation(&self) -> OperationSnapshot {
            std::sync::Arc::clone(&self.snapshot)
        }
    }

    struct FixtureChangeFeedPort<'a> {
        outcome: ChangeFeedInvocationOutcome,
        calls: &'a AtomicUsize,
    }

    impl ChangeFeedInvocationPort for FixtureChangeFeedPort<'_> {
        fn invoke(
            &self,
            _actor: &M00AdmittedActor,
            _board_id: &BoardId,
        ) -> Result<ChangeFeedInvocationOutcome, ChangeFeedInvocationError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.outcome.clone())
        }
    }

    fn change_feed_request(board_id: &str) -> SubmitChangeFeedDto {
        let board_wire = WireText::parse(board_id).expect("fixture board");
        let digest = change_feed_payload_digest(&board_wire).expect("fixture digest");
        SubmitChangeFeedDto {
            request_id: WireText::parse("req:fixture").expect("fixture request id"),
            correlation_id: WireText::parse("corr:fixture").expect("fixture correlation"),
            causation_id: None,
            idempotency_key: Some(WireText::parse("idem:fixture").expect("fixture key")),
            actor: ActorIntentDto::Public,
            provenance: ClientProvenanceDto {
                build: WireText::parse("build:fixture").expect("fixture build"),
                target: WireText::parse("linux").expect("fixture target"),
                protocol: WireText::parse("m10:v2").expect("fixture protocol"),
            },
            payload_digest: digest,
            board_id: board_wire,
        }
    }

    struct AlwaysCurrentM60;

    impl M60ChangePublicationPort for AlwaysCurrentM60 {
        fn verify_publication(
            &self,
            old_revision: &SourceRevision,
            new_revision: &SourceRevision,
        ) -> Result<M60ChangePublicationOutcome, M60ChangePublicationPortError> {
            Ok(M60ChangePublicationOutcome::CurrentVerified(
                M60VerifiedChangeEvidence::for_revisions(old_revision, new_revision),
            ))
        }
    }

    fn field_value(tag: char, round: usize) -> String {
        let mut value = String::new();
        let mut seed = (tag as usize) * 10_000 + round;
        while value.len() < 512 {
            value.push_str(&format!("v{seed}."));
            seed += 1;
        }
        value.truncate(512);
        value
    }

    fn frame_facts(tag: char, round: usize) -> NormalizedFacts {
        NormalizedFacts::try_from_iter((0..64usize).map(|index| {
            (
                SemanticField::parse(format!("field.{index:02}")).expect("fixture field"),
                SemanticValue::parse(field_value(tag, round)).expect("fixture value"),
            )
        }))
        .expect("fixture facts")
    }

    fn frame_revision(number: usize, tag: char, round: usize) -> SourceRevision {
        SourceRevision::demo_reviewed(
            SourceId::parse("source:demo:frame").expect("fixture source"),
            SourceUrl::parse("https://example.test/frame").expect("fixture url"),
            RawSnapshotId::parse(format!("raw:frame:{number}")).expect("fixture raw id"),
            RevisionSha256::parse(format!("sha256:{:064x}", number)).expect("fixture raw digest"),
            NormalizedSnapshotId::parse(format!("normalized:frame:{number}"))
                .expect("fixture normalized id"),
            frame_facts(tag, round).sha256(),
            ParserIdentity::parse("parser:calendar:v1").expect("fixture parser"),
            RevisionTimestamp::from_unix_seconds(1_000 + number as i64),
            Some(RevisionTimestamp::from_unix_seconds(900)),
            EffectiveInterval::new(Some(RevisionTimestamp::from_unix_seconds(2_000)), None)
                .expect("fixture interval"),
            SourceReviewerId::parse("reviewer:demo").expect("fixture source reviewer"),
            SourceReviewEvidenceId::parse(format!("evidence:frame:{number}"))
                .expect("fixture evidence id"),
        )
    }

    fn frame_receipt(entries: usize) -> ChangeFeedReceipt {
        let board_id = BoardId::parse("board:frame-overflow").expect("fixture board");
        let source_id = SourceId::parse("source:demo:frame").expect("fixture source");
        let tracked_fields = (0..64usize)
            .map(|index| SemanticField::parse(format!("field.{index:02}")).expect("fixture field"))
            .collect::<Vec<_>>();
        let board_policy = BoardPolicy::new(
            board_id.clone(),
            source_id,
            1,
            tracked_fields,
            "all_students",
        )
        .expect("fixture board policy");
        let mut radar = ChangeRadarService::new(board_policy, InMemoryChangeRadarRepository::new());
        radar
            .observe(
                AcceptedObservation::new(
                    frame_revision(0, 'a', 0),
                    frame_facts('a', 0),
                    SourceRevisionHealth::Current,
                )
                .expect("fixture baseline observation"),
            )
            .expect("fixture baseline");
        let mut candidates = Vec::new();
        for round in 1..=entries {
            let outcome = radar
                .observe(
                    AcceptedObservation::new(
                        frame_revision(round, 'b', round),
                        frame_facts('b', round),
                        SourceRevisionHealth::Current,
                    )
                    .expect("fixture candidate observation"),
                )
                .expect("fixture candidate");
            let ObservationOutcome::SemanticChange(candidate) = outcome else {
                panic!("expected semantic candidate");
            };
            candidates.push(*candidate);
        }
        let feed_policy = BoardFeedPolicy::new(
            board_id,
            1,
            "Frame overflow feed",
            "USTC Campus Agent",
            "https://campus.example.test",
        )
        .expect("fixture feed policy");
        let mut repository = radar.into_repository();
        let m60 = AlwaysCurrentM60;
        let mut publication =
            ChangePublicationService::new(&mut repository, &m60, feed_policy.clone());
        for (index, candidate) in candidates.iter().enumerate() {
            let review = ChangeReviewReceipt::approve(
                candidate,
                UserId::parse("user:admin").expect("fixture reviewer"),
                RevisionTimestamp::from_unix_seconds(5_000 + index as i64),
            )
            .expect("fixture approval");
            publication.record_review(review).expect("fixture review");
            publication
                .publish(
                    candidate.event_id(),
                    RevisionTimestamp::from_unix_seconds(6_000 + index as i64),
                )
                .expect("fixture publication");
        }
        ChangeFeedQueryService::new(&repository)
            .execute(&feed_policy)
            .expect("fixture feed query")
    }

    fn assert_malformed_without_invocation(request: &SubmitChangeFeedDto) {
        let calls = AtomicUsize::new(0);
        let port = FixtureChangeFeedPort {
            outcome: ChangeFeedInvocationOutcome::NotFound(
                BoardId::parse("board:frame-overflow").expect("fixture board"),
            ),
            calls: &calls,
        };
        let service = M10ChangeFeedService::new(&port);
        let mut ports = ChangeFeedTestPorts::new();
        let response = service.submit(request, &mut ports);
        assert_eq!(response, malformed_command_error());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn change_invalid_request_id_is_malformed_without_invocation() {
        let mut request = change_feed_request("board:frame-overflow");
        request.request_id = WireText::parse("#invalid-request").expect("wire text");
        assert_malformed_without_invocation(&request);
    }

    #[test]
    fn change_invalid_correlation_id_is_malformed_without_invocation() {
        let mut request = change_feed_request("board:frame-overflow");
        request.correlation_id = WireText::parse("bad correlation!").expect("wire text");
        assert_malformed_without_invocation(&request);
    }

    #[test]
    fn change_invalid_session_id_is_malformed_without_invocation() {
        let mut request = change_feed_request("board:frame-overflow");
        request.actor = ActorIntentDto::Authenticated {
            session_id: WireText::parse("-invalid-session").expect("wire text"),
        };
        assert_malformed_without_invocation(&request);
    }

    #[test]
    fn change_invalid_idempotency_key_is_malformed_without_invocation() {
        let mut request = change_feed_request("board:frame-overflow");
        request.idempotency_key = Some(WireText::parse("bad key!").expect("wire text"));
        assert_malformed_without_invocation(&request);
    }

    #[test]
    fn change_invalid_provenance_is_malformed_without_invocation() {
        let mut request = change_feed_request("board:frame-overflow");
        request.provenance = ClientProvenanceDto {
            build: WireText::parse("build:fixture").expect("fixture build"),
            target: WireText::parse("linux").expect("fixture target"),
            protocol: WireText::parse("p".repeat(129)).expect("wire text"),
        };
        assert_malformed_without_invocation(&request);
    }

    #[test]
    fn change_feed_found_below_frame_returns_accepted() {
        let receipt = frame_receipt(1);
        let terminal = project_change_feed(&receipt).expect("fixture projection");
        let probe = ClientResponseDto::ChangeFeedAccepted {
            command_id: wire("command:fixture"),
            terminal: Box::new(terminal),
        };
        let bytes = serde_json::to_vec(&probe).expect("serialize probe");
        assert!(bytes.len() <= MAX_FRAME_BYTES);

        let calls = AtomicUsize::new(0);
        let port = FixtureChangeFeedPort {
            outcome: ChangeFeedInvocationOutcome::Found(receipt),
            calls: &calls,
        };
        let service = M10ChangeFeedService::new(&port);
        let mut ports = ChangeFeedTestPorts::new();
        let request = change_feed_request("board:frame-overflow");
        let response = service.submit(&request, &mut ports);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(matches!(
            response,
            ClientResponseDto::ChangeFeedAccepted { .. }
        ));
    }

    #[test]
    fn change_feed_frame_overflow_returns_bounded_frame_error() {
        let receipt = frame_receipt(LARGE_ENTRY_COUNT);
        let terminal = project_change_feed(&receipt).expect("fixture projection");
        let overflow_response = ClientResponseDto::ChangeFeedAccepted {
            command_id: wire("command:overflow"),
            terminal: Box::new(terminal),
        };
        let overflow_bytes = serde_json::to_vec(&overflow_response).expect("serialize overflow");
        assert!(
            overflow_bytes.len() > MAX_FRAME_BYTES,
            "fixture must exceed the frame bound: {} bytes",
            overflow_bytes.len()
        );

        let calls = AtomicUsize::new(0);
        let port = FixtureChangeFeedPort {
            outcome: ChangeFeedInvocationOutcome::Found(receipt),
            calls: &calls,
        };
        let service = M10ChangeFeedService::new(&port);
        let mut ports = ChangeFeedTestPorts::new();
        let request = change_feed_request("board:frame-overflow");
        let response = service.submit(&request, &mut ports);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let ClientResponseDto::Error { error } = &response else {
            panic!("expected error response, got {response:?}");
        };
        let ClientErrorDto::Infrastructure { wire_code, .. } = error else {
            panic!("expected infrastructure error, got {error:?}");
        };
        assert_eq!(wire_code.as_str(), "change_feed_frame_overflow");
        let mut frame = Vec::new();
        write_frame(&mut frame, &response).expect("bounded error frame");
    }
}
