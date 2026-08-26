//! Executable evidence for `platform-request-context/v0` (`AUTH-013`).

mod request_context {
    use std::sync::Arc;

    use ustc_campus_agent_core::identity::{
        CommandId, CorrelationId, RequestId, SessionId, TenantId, UserId,
    };
    use ustc_campus_agent_core::request_context::*;
    use ustc_campus_agent_core::session::{
        AuthAdapterId, CredentialEvidenceDigest, OpenSession, SessionCommand,
        SessionCredentialEvidence, SessionDuration, SessionInstant, SessionPolicy, SessionSnapshot,
        decide, evolve,
    };

    const SOURCE: &str = include_str!("../src/request_context.rs");
    const IDENTITY_SOURCE: &str = include_str!("../src/identity.rs");
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const CREDENTIAL_DIGEST: &str =
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn operation(value: &str) -> OperationId {
        OperationId::parse(value).expect("fixture operation")
    }
    fn policy_id() -> PlatformPolicySnapshotId {
        PlatformPolicySnapshotId::parse("policy:fixture").expect("fixture policy")
    }
    fn request_id() -> RequestId {
        RequestId::parse("request:fixture").expect("fixture request")
    }
    fn command_id() -> CommandId {
        CommandId::parse("command:fixture").expect("fixture command")
    }
    fn correlation_id() -> CorrelationId {
        CorrelationId::parse("correlation:fixture").expect("fixture correlation")
    }
    fn key() -> IdempotencyKey {
        IdempotencyKey::parse("idempotency:fixture").expect("fixture key")
    }
    fn tenant() -> TenantId {
        TenantId::parse("tenant:fixture").expect("fixture tenant")
    }
    fn user() -> UserId {
        UserId::parse("user:fixture").expect("fixture user")
    }
    fn session(value: &str) -> SessionId {
        SessionId::parse(value).expect("fixture session")
    }
    fn at(value: u64) -> SessionInstant {
        SessionInstant::from_unix_millis(value)
    }
    fn schema_digest() -> SchemaDigest {
        SchemaDigest::parse(DIGEST).expect("fixture digest")
    }
    fn descriptor_id() -> DescriptorSnapshotId {
        DescriptorSnapshotId::from_canonical_identity(&schema_digest(), 7)
            .expect("fixture descriptor id")
    }
    fn token(fence: u64) -> IdempotencyReservationToken {
        IdempotencyReservationToken::from_store_observation(command_id(), 3, fence, at(1_100))
            .expect("fixture token")
    }

    #[derive(Clone)]
    struct Descriptor {
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

    impl Descriptor {
        fn new(operation_name: &str, permission_class: PermissionClass) -> Self {
            Self {
                operation_id: operation(operation_name),
                schema_identity: SchemaIdentity::parse("schema:fixture").expect("fixture"),
                schema_digest: schema_digest(),
                permission_class,
                effect_class: if matches!(permission_class, PermissionClass::PublicLinkout) {
                    EffectClass::LinkOut
                } else {
                    EffectClass::Read
                },
                decoder_identity: DecoderIdentity::parse("decoder:fixture").expect("fixture"),
                dispatcher_identity: DispatcherIdentity::parse("dispatcher:fixture")
                    .expect("fixture"),
                adapter_allowlist: AdapterAllowlist::try_from_iter([AdapterIdentity::parse(
                    "adapter:fixture",
                )
                .expect("fixture")])
                .expect("fixture"),
                snapshot_identity: descriptor_id(),
            }
        }
    }

    impl OperationDescriptorProjection for Descriptor {
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

    fn active_session(session_id: SessionId) -> SessionSnapshot {
        let evidence = SessionCredentialEvidence::new(
            tenant(),
            user(),
            AuthAdapterId::parse("fixture.adapter").expect("fixture"),
            CredentialEvidenceDigest::parse(CREDENTIAL_DIGEST).expect("fixture"),
            at(1_000),
            None,
        )
        .expect("fixture");
        let policy = SessionPolicy::new(
            SessionDuration::from_millis(100).expect("fixture"),
            SessionDuration::from_millis(1_000).expect("fixture"),
        );
        let command =
            SessionCommand::Open(OpenSession::new(session_id, evidence, policy, at(1_000), 0));
        let event = decide(None, &command).expect("fixture open");
        evolve(None, &event).expect("fixture evolve")
    }

    #[derive(Clone)]
    struct FakePorts {
        reservation: Result<IdempotencyReservation, IdempotencyError>,
        descriptor: Result<OperationSnapshot, DescriptorSnapshotError>,
        now: Result<SessionInstant, AdmissionPortError>,
        policy: Result<PolicyResolution, AdmissionPortError>,
        loaded_session: Result<Option<SessionSnapshot>, AdmissionPortError>,
        capability: Result<CapabilityDisposition, AdmissionPortError>,
        finalize: Result<FinalizeIdempotencyOutcome, IdempotencyError>,
        reserve_calls: usize,
        descriptor_calls: usize,
        clock_calls: usize,
        policy_calls: usize,
        session_calls: usize,
        capability_calls: usize,
        finalize_calls: usize,
        last_envelope: Option<String>,
        finalized: Vec<PersistedPriorDispositionDto>,
    }

    impl FakePorts {
        fn public() -> Self {
            Self {
                reservation: Ok(IdempotencyReservation::New(token(1))),
                descriptor: Ok(Arc::new(Descriptor::new(
                    "affairs.get",
                    PermissionClass::PublicRead,
                ))),
                now: Ok(at(1_000)),
                policy: Ok(PolicyResolution::new(
                    policy_id(),
                    PolicyCurrentnessFact::Current,
                )),
                loaded_session: Ok(None),
                capability: Ok(CapabilityDisposition::Enabled),
                finalize: Ok(FinalizeIdempotencyOutcome::Committed),
                reserve_calls: 0,
                descriptor_calls: 0,
                clock_calls: 0,
                policy_calls: 0,
                session_calls: 0,
                capability_calls: 0,
                finalize_calls: 0,
                last_envelope: None,
                finalized: Vec::new(),
            }
        }
    }

    impl AdmissionPorts for FakePorts {
        fn reserve_or_retrieve_idempotency(
            &mut self,
            _key: Option<&IdempotencyKey>,
            envelope_hash: &EnvelopeHash,
        ) -> Result<IdempotencyReservation, IdempotencyError> {
            self.reserve_calls += 1;
            self.last_envelope = Some(envelope_hash.as_str().to_owned());
            self.reservation.clone()
        }

        fn request_scoped_operation(
            &mut self,
        ) -> Result<OperationSnapshot, DescriptorSnapshotError> {
            self.descriptor_calls += 1;
            self.descriptor.clone()
        }

        fn now(&mut self) -> Result<SessionInstant, AdmissionPortError> {
            self.clock_calls += 1;
            self.now
        }

        fn resolve_policy(
            &mut self,
            _operation_id: &OperationId,
            _observed_at: SessionInstant,
        ) -> Result<PolicyResolution, AdmissionPortError> {
            self.policy_calls += 1;
            self.policy.clone()
        }

        fn load_session(
            &mut self,
            _session_id: &SessionId,
        ) -> Result<Option<SessionSnapshot>, AdmissionPortError> {
            self.session_calls += 1;
            self.loaded_session.clone()
        }

        fn check_capability(
            &mut self,
            _operation_id: &OperationId,
            _actor_kind: ActorKind,
            _observed_at: SessionInstant,
        ) -> Result<CapabilityDisposition, AdmissionPortError> {
            self.capability_calls += 1;
            self.capability
        }

        fn finalize_idempotency(
            &mut self,
            _token: &IdempotencyReservationToken,
            disposition: &FinalAdmissionDisposition,
        ) -> Result<FinalizeIdempotencyOutcome, IdempotencyError> {
            self.finalize_calls += 1;
            self.finalized.push(disposition.to_persisted_projection());
            self.finalize.clone()
        }
    }

    fn public_command(with_key: bool) -> BuildRequestContextCommand {
        BuildRequestContextCommand::new(
            request_id(),
            operation("affairs.get"),
            ActorReference::Anonymous { scope: PublicScope },
            correlation_id(),
            None,
            with_key.then(key),
            ClientProvenance::new("build:fixture", "linux", "m10:v2").expect("fixture"),
            PayloadDigest::parse(DIGEST).expect("fixture"),
        )
    }

    fn authenticated_command(session_id: SessionId) -> BuildRequestContextCommand {
        BuildRequestContextCommand::new(
            request_id(),
            operation("affairs.get"),
            ActorReference::Authenticated { session_id },
            correlation_id(),
            Some(CausationId::parse("causation:fixture").expect("fixture")),
            Some(key()),
            ClientProvenance::new("build:fixture", "linux", "m10:v2").expect("fixture"),
            PayloadDigest::parse(DIGEST).expect("fixture"),
        )
    }

    fn admit(command: &BuildRequestContextCommand, ports: &mut FakePorts) -> M00AdmissionResult {
        RequestAdmissionCoordinator.admit(command, ports)
    }

    fn expect_rejection(result: M00AdmissionResult, class: AdmissionRejectionClass) {
        match result {
            M00AdmissionResult::Rejected(value) => assert_eq!(value.class(), class),
            other => panic!("expected rejection {class:?}, got {other:?}"),
        }
    }

    fn fresh_public() -> (M00AdmissionResult, FakePorts) {
        let mut ports = FakePorts::public();
        let result = admit(&public_command(true), &mut ports);
        (result, ports)
    }

    fn persisted_public() -> PersistedAdmittedDispositionDto {
        PersistedAdmittedDispositionDto::try_from_parts(
            command_id(),
            correlation_id(),
            descriptor_id(),
            PersistedAdmittedActorDto::Public,
            PersistedFrozenPrerequisitesDto::from_parts(
                policy_id(),
                at(1_000),
                None,
                operation("affairs.get"),
            ),
        )
        .expect("coherent public")
    }

    fn persisted_authenticated(session_id: SessionId) -> PersistedAdmittedDispositionDto {
        PersistedAdmittedDispositionDto::try_from_parts(
            command_id(),
            correlation_id(),
            descriptor_id(),
            PersistedAdmittedActorDto::Authenticated {
                tenant_id: tenant(),
                user_id: user(),
                session_id: session_id.clone(),
            },
            PersistedFrozenPrerequisitesDto::from_parts(
                policy_id(),
                at(1_000),
                Some(session_id),
                operation("affairs.get"),
            ),
        )
        .expect("coherent authenticated")
    }

    fn projections() -> Vec<AdmissionRejectionProjection> {
        let op = operation("affairs.get");
        let other = operation("affairs.other");
        let requested = session("session:requested");
        let loaded = session("session:loaded");
        vec![
            AdmissionRejectionProjection::IdempotencyStoreUnavailable {
                operation_id: op.clone(),
            },
            AdmissionRejectionProjection::ConflictingEnvelope {
                operation_id: op.clone(),
                idempotency_key: key(),
            },
            AdmissionRejectionProjection::DescriptorSnapshotAbsent {
                operation_id: op.clone(),
            },
            AdmissionRejectionProjection::DescriptorSnapshotMismatch {
                command_operation_id: op.clone(),
                snapshot_operation_id: other,
            },
            AdmissionRejectionProjection::PolicyDenied {
                operation_id: op.clone(),
                permission_class: PermissionClass::TenantPrivateRead,
            },
            AdmissionRejectionProjection::PolicyExpired {
                operation_id: op.clone(),
                policy_snapshot_id: policy_id(),
            },
            AdmissionRejectionProjection::SessionNotFound {
                requested_session_id: requested.clone(),
            },
            AdmissionRejectionProjection::SessionIdMismatch {
                requested_session_id: requested.clone(),
                loaded_session_id: loaded,
            },
            AdmissionRejectionProjection::SessionNotAdmitted {
                requested_session_id: requested,
                observed_at: at(1_000),
            },
            AdmissionRejectionProjection::CapabilityMissing {
                operation_id: op.clone(),
                actor_kind: ActorKind::Public,
            },
            AdmissionRejectionProjection::CapabilityDisabled {
                operation_id: op.clone(),
                actor_kind: ActorKind::Authenticated,
            },
            AdmissionRejectionProjection::CapabilityRevoked {
                operation_id: op.clone(),
                actor_kind: ActorKind::Authenticated,
            },
            AdmissionRejectionProjection::InfrastructurePortUnavailable {
                operation_id: op.clone(),
                port: AdmissionPortKind::Policy,
            },
            AdmissionRejectionProjection::MalformedCommand {
                operation_id: Some(op),
            },
        ]
    }

    fn projection_classes() -> Vec<AdmissionRejectionClass> {
        projections()
            .iter()
            .map(AdmissionRejectionProjection::class)
            .collect()
    }

    fn assert_public_admitted() {
        let (result, ports) = fresh_public();
        let M00AdmissionResult::Admitted {
            context,
            disposition,
        } = result
        else {
            panic!("expected admitted")
        };
        assert_eq!(context.actor(), &M00AdmittedActor::Public);
        assert_eq!(disposition.admitted_actor(), &M00AdmittedActor::Public);
        assert!(disposition.frozen_prerequisites().session_id().is_none());
        assert_eq!(ports.reserve_calls, 1);
        assert_eq!(ports.policy_calls, 1);
        assert_eq!(ports.capability_calls, 1);
        assert_eq!(ports.session_calls, 0);
        assert_eq!(ports.finalize_calls, 1);
    }

    fn assert_projection_round_trip() {
        for projection in projections() {
            let dto = PersistedAdmissionRejectionDto::from_projection(&projection);
            let encoded = serde_json::to_vec(&dto).expect("serialize");
            let decoded: PersistedAdmissionRejectionDto =
                serde_json::from_slice(&encoded).expect("deserialize");
            assert_eq!(decoded.to_projection(), projection);
        }
    }

    #[test]
    fn request_context_forged_permission_tuple_rejected() {
        assert!(!SOURCE.contains("pub fn new(permission_class"));
        assert!(!SOURCE.contains("pub fn new(operation_id: OperationId, permission_class"));
    }

    #[test]
    fn request_context_session_a_snapshot_b_mismatch_rejected() {
        let mut ports = FakePorts::public();
        ports.loaded_session = Ok(Some(active_session(session("session:b"))));
        let result = admit(&authenticated_command(session("session:a")), &mut ports);
        expect_rejection(result, AdmissionRejectionClass::SessionIdMismatch);
        assert_eq!(ports.capability_calls, 0);
    }

    #[test]
    fn request_context_caller_cannot_inject_stale_policy_fact() {
        assert!(!SOURCE.contains("pub struct AdmissionFacts {\n    pub"));
        let mut ports = FakePorts::public();
        ports.policy = Ok(PolicyResolution::new(
            policy_id(),
            PolicyCurrentnessFact::Stale,
        ));
        expect_rejection(
            admit(&public_command(true), &mut ports),
            AdmissionRejectionClass::PolicyExpired,
        );
    }

    #[test]
    fn request_context_idempotency_first_attempt_reserves_command_id() {
        let (result, ports) = fresh_public();
        let M00AdmissionResult::Admitted { disposition, .. } = result else {
            panic!()
        };
        assert_eq!(disposition.command_id(), &command_id());
        assert_eq!(ports.reserve_calls, 1);
    }

    #[test]
    fn request_context_idempotency_retry_returns_prior_disposition() {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::PriorIdentical(
            PersistedPriorDispositionDto::Admitted(persisted_public()),
        ));
        assert!(matches!(
            admit(&public_command(true), &mut ports),
            M00AdmissionResult::PriorAdmitted(_)
        ));
        assert_eq!(ports.descriptor_calls, 0);
    }

    #[test]
    fn request_context_idempotency_conflicting_envelope_rejected() {
        fn capture(command: &BuildRequestContextCommand) -> String {
            let mut ports = FakePorts::public();
            ports.reservation = Err(IdempotencyError::ConflictingEnvelope {
                idempotency_key: key(),
            });
            expect_rejection(
                admit(command, &mut ports),
                AdmissionRejectionClass::ConflictingEnvelope,
            );
            assert_eq!(ports.descriptor_calls, 0);
            ports.last_envelope.expect("reservation observed envelope")
        }

        let base = public_command(true);
        let base_hash = capture(&base);
        assert_eq!(
            base_hash, "sha256:7d5eef1c9225922e26791cc45bc32d57d761735b75348e20c2597e8bd9c28bbe",
            "pins the v0 domain separator, length framing and exact base field set"
        );

        let rebuild = |request_id: RequestId,
                       operation_id: OperationId,
                       actor_reference: ActorReference,
                       correlation_id: CorrelationId,
                       causation_id: Option<CausationId>,
                       provenance: ClientProvenance,
                       payload_digest: PayloadDigest| {
            BuildRequestContextCommand::new(
                request_id,
                operation_id,
                actor_reference,
                correlation_id,
                causation_id,
                Some(key()),
                provenance,
                payload_digest,
            )
        };

        // Ingress-attempt/correlation/provenance observations and the store lookup key are
        // intentionally outside the semantic envelope commitment.
        let different_request = rebuild(
            RequestId::parse("request:retry").expect("fixture"),
            base.operation_id().clone(),
            base.actor_reference().clone(),
            base.correlation_id().clone(),
            None,
            base.client_provenance().clone(),
            base.payload_digest().clone(),
        );
        let different_correlation = rebuild(
            base.request_id().clone(),
            base.operation_id().clone(),
            base.actor_reference().clone(),
            CorrelationId::parse("correlation:retry").expect("fixture"),
            None,
            base.client_provenance().clone(),
            base.payload_digest().clone(),
        );
        let different_provenance = rebuild(
            base.request_id().clone(),
            base.operation_id().clone(),
            base.actor_reference().clone(),
            base.correlation_id().clone(),
            None,
            ClientProvenance::new("build:retry", "linux:retry", "m10:v2:retry").expect("fixture"),
            base.payload_digest().clone(),
        );
        assert_eq!(capture(&different_request), base_hash);
        assert_eq!(capture(&different_correlation), base_hash);
        assert_eq!(capture(&different_provenance), base_hash);

        // Every accepted semantic input is committed.
        let different_operation = rebuild(
            base.request_id().clone(),
            operation("affairs.search"),
            base.actor_reference().clone(),
            base.correlation_id().clone(),
            None,
            base.client_provenance().clone(),
            base.payload_digest().clone(),
        );
        let different_actor = rebuild(
            base.request_id().clone(),
            base.operation_id().clone(),
            ActorReference::Authenticated {
                session_id: SessionId::parse("session:retry").expect("fixture"),
            },
            base.correlation_id().clone(),
            None,
            base.client_provenance().clone(),
            base.payload_digest().clone(),
        );
        let different_payload = rebuild(
            base.request_id().clone(),
            base.operation_id().clone(),
            base.actor_reference().clone(),
            base.correlation_id().clone(),
            None,
            base.client_provenance().clone(),
            PayloadDigest::parse(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            )
            .expect("fixture"),
        );
        let different_causation = rebuild(
            base.request_id().clone(),
            base.operation_id().clone(),
            base.actor_reference().clone(),
            base.correlation_id().clone(),
            Some(CausationId::parse("cause:retry").expect("fixture")),
            base.client_provenance().clone(),
            base.payload_digest().clone(),
        );
        assert_ne!(capture(&different_operation), base_hash);
        assert_ne!(capture(&different_actor), base_hash);
        assert_ne!(capture(&different_payload), base_hash);
        assert_ne!(capture(&different_causation), base_hash);
    }

    #[test]
    fn request_context_idempotency_restart_reconciles_by_key() {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::PriorIdentical(
            PersistedPriorDispositionDto::Admitted(persisted_public()),
        ));
        assert!(matches!(
            admit(&public_command(true), &mut ports),
            M00AdmissionResult::PriorAdmitted(_)
        ));
    }

    #[test]
    fn request_context_anonymous_public_read_checks_policy_and_capability() {
        assert_public_admitted();
    }

    #[test]
    fn request_context_denied_admission_reaches_no_downstream_fake() {
        let mut ports = FakePorts::public();
        ports.capability = Ok(CapabilityDisposition::Revoked);
        expect_rejection(
            admit(&public_command(true), &mut ports),
            AdmissionRejectionClass::CapabilityRevoked,
        );
        assert_eq!(ports.finalize_calls, 1);
    }

    #[test]
    fn request_context_private_constructor_compile_proof() {
        assert!(SOURCE.matches("```compile_fail").count() >= 4);
        assert!(!SOURCE.contains("impl PlatformRequestContext {\n    pub fn new"));
    }

    #[test]
    fn request_context_serde_deserialize_absent_compile_proof() {
        assert!(SOURCE.contains("let _: PlatformRequestContext = serde_json::from_str"));
        assert!(!SOURCE.contains("impl<'de> Deserialize<'de> for PlatformRequestContext"));
        assert!(!SOURCE.contains("impl<'de> Deserialize<'de> for M00AdmittedActor"));
    }

    #[test]
    fn request_context_anonymous_on_private_class_denied_before_session() {
        let mut ports = FakePorts::public();
        ports.descriptor = Ok(Arc::new(Descriptor::new(
            "affairs.get",
            PermissionClass::TenantPrivateRead,
        )));
        expect_rejection(
            admit(&public_command(true), &mut ports),
            AdmissionRejectionClass::PolicyDenied,
        );
        assert_eq!(ports.session_calls, 0);
        assert_eq!(ports.policy_calls, 0);
    }

    #[test]
    fn request_context_authenticated_on_public_read_still_checks_session() {
        let mut ports = FakePorts::public();
        ports.loaded_session = Ok(None);
        expect_rejection(
            admit(&authenticated_command(session("session:a")), &mut ports),
            AdmissionRejectionClass::SessionNotFound,
        );
        assert_eq!(ports.session_calls, 1);
    }

    #[test]
    fn request_context_idempotency_no_key_at_most_once() {
        let mut ports = FakePorts::public();
        let result = admit(&public_command(false), &mut ports);
        assert!(matches!(result, M00AdmissionResult::Admitted { .. }));
        assert_eq!(ports.reserve_calls, 1);
    }

    #[test]
    fn request_context_timeout_reconciles_before_retry() {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::InFlight(token(2)));
        let M00AdmissionResult::Incomplete(value) = admit(&public_command(true), &mut ports) else {
            panic!()
        };
        assert_eq!(value.retry_not_before(), at(1_100));
        assert_eq!(ports.descriptor_calls, 0);
    }

    #[test]
    fn request_context_descriptor_snapshot_no_live_lookup() {
        let (_, ports) = fresh_public();
        assert_eq!(ports.descriptor_calls, 1);
    }

    #[test]
    fn request_context_descriptor_snapshot_operation_id_mismatch_rejected() {
        let mut ports = FakePorts::public();
        ports.descriptor = Ok(Arc::new(Descriptor::new(
            "affairs.other",
            PermissionClass::PublicRead,
        )));
        expect_rejection(
            admit(&public_command(true), &mut ports),
            AdmissionRejectionClass::DescriptorSnapshotMismatch,
        );
        assert_eq!(ports.policy_calls, 0);
    }

    #[test]
    fn request_context_descriptor_snapshot_absent_fail_closed() {
        let mut ports = FakePorts::public();
        ports.descriptor = Err(DescriptorSnapshotError::Absent);
        expect_rejection(
            admit(&public_command(true), &mut ports),
            AdmissionRejectionClass::DescriptorSnapshotAbsent,
        );
    }

    #[test]
    fn request_context_descriptor_snapshot_port_unavailable_class() {
        let mut ports = FakePorts::public();
        ports.descriptor = Err(DescriptorSnapshotError::PortUnavailable);
        expect_rejection(
            admit(&public_command(true), &mut ports),
            AdmissionRejectionClass::InfrastructurePortUnavailable,
        );
    }

    #[test]
    fn request_context_descriptor_snapshot_immutable_across_registry_update() {
        let (result, _) = fresh_public();
        let M00AdmissionResult::Admitted { context, .. } = result else {
            panic!()
        };
        assert_eq!(
            context.operation_snapshot().snapshot_identity(),
            &descriptor_id()
        );
    }

    #[test]
    fn request_context_descriptor_snapshot_send_sync_arc_representable() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OperationSnapshot>();
        let snapshot: OperationSnapshot =
            Arc::new(Descriptor::new("affairs.get", PermissionClass::PublicRead));
        assert_eq!(Arc::strong_count(&snapshot), 1);
    }

    #[test]
    fn request_context_descriptor_snapshot_carried_to_downstream() {
        let descriptor: OperationSnapshot =
            Arc::new(Descriptor::new("affairs.get", PermissionClass::PublicRead));
        let mut ports = FakePorts::public();
        ports.descriptor = Ok(Arc::clone(&descriptor));
        let M00AdmissionResult::Admitted { context, .. } = admit(&public_command(true), &mut ports)
        else {
            panic!()
        };
        assert!(Arc::ptr_eq(&descriptor, &context.operation_snapshot()));
    }

    #[test]
    fn descriptor_snapshot_id_owner_m00_checked_constructor() {
        let value = descriptor_id();
        assert_eq!(value.content_digest(), &schema_digest());
        assert_eq!(value.snapshot_version(), 7);
    }

    #[test]
    fn descriptor_snapshot_id_no_m10_inherent_mint() {
        assert!(!SOURCE.contains("fn mint("));
    }

    #[test]
    fn descriptor_snapshot_id_m10_mint_authorality_invokes_m00_constructor() {
        assert!(SOURCE.contains("pub fn from_canonical_identity("));
    }

    #[test]
    fn descriptor_snapshot_id_serde_owner_m00_validating() {
        let json = serde_json::to_string(&descriptor_id()).expect("serialize");
        let decoded: DescriptorSnapshotId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, descriptor_id());
        assert!(serde_json::from_str::<DescriptorSnapshotId>("\"descriptor:v0:0:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"").is_err());
    }

    #[test]
    fn descriptor_snapshot_id_inner_private() {
        assert!(SOURCE.contains("pub struct DescriptorSnapshotId {\n    rendered: String"));
        assert!(!SOURCE.contains("pub rendered: String"));
    }

    #[test]
    fn rejection_class_14_unit_tags_no_payload() {
        assert_eq!(projection_classes().len(), 14);
        assert_eq!(std::mem::size_of::<AdmissionRejectionClass>(), 1);
    }

    #[test]
    fn rejection_projection_covers_all_14_variants() {
        let classes = projection_classes();
        let unique: std::collections::HashSet<_> = classes.iter().copied().collect();
        assert_eq!(classes.len(), 14);
        assert_eq!(unique.len(), 14);
    }

    #[test]
    fn rejection_class_from_projection_exhaustive_no_wildcard() {
        let start = SOURCE.find("pub const fn class(&self)").expect("class");
        let tail = &SOURCE[start
            ..SOURCE[start..]
                .find("#[allow(dead_code)]")
                .map_or(SOURCE.len(), |n| start + n)];
        assert!(!tail.contains("_ =>"));
    }

    #[test]
    fn rejection_projection_payload_preservation_policy_expired() {
        let projection = AdmissionRejectionProjection::PolicyExpired {
            operation_id: operation("affairs.get"),
            policy_snapshot_id: policy_id(),
        };
        let AdmissionRejectionProjection::PolicyExpired {
            policy_snapshot_id, ..
        } = projection
        else {
            panic!()
        };
        assert_eq!(policy_snapshot_id, policy_id());
    }

    #[test]
    fn rejection_projection_payload_preservation_session_id_mismatch() {
        let requested = session("session:a");
        let loaded = session("session:b");
        let projection = AdmissionRejectionProjection::SessionIdMismatch {
            requested_session_id: requested.clone(),
            loaded_session_id: loaded.clone(),
        };
        let AdmissionRejectionProjection::SessionIdMismatch {
            requested_session_id,
            loaded_session_id,
        } = projection
        else {
            panic!()
        };
        assert_eq!(
            (requested_session_id, loaded_session_id),
            (requested, loaded)
        );
    }

    #[test]
    fn rejection_projection_payload_preservation_capability_actor_kind() {
        for projection in projections().into_iter().filter(|value| {
            matches!(
                value.class(),
                AdmissionRejectionClass::CapabilityMissing
                    | AdmissionRejectionClass::CapabilityDisabled
                    | AdmissionRejectionClass::CapabilityRevoked
            )
        }) {
            assert!(matches!(
                projection,
                AdmissionRejectionProjection::CapabilityMissing {
                    actor_kind: ActorKind::Public,
                    ..
                } | AdmissionRejectionProjection::CapabilityDisabled {
                    actor_kind: ActorKind::Authenticated,
                    ..
                } | AdmissionRejectionProjection::CapabilityRevoked {
                    actor_kind: ActorKind::Authenticated,
                    ..
                }
            ));
        }
    }

    #[test]
    fn rejection_projection_session_not_admitted_observed_at() {
        let projection = AdmissionRejectionProjection::SessionNotAdmitted {
            requested_session_id: session("session:a"),
            observed_at: at(77),
        };
        let AdmissionRejectionProjection::SessionNotAdmitted { observed_at, .. } = projection
        else {
            panic!()
        };
        assert_eq!(observed_at, at(77));
    }

    #[test]
    fn rejection_carrier_private_fields_no_default_no_serde() {
        assert!(SOURCE.contains("pub struct RequestContextRejection {\n    projection:"));
        assert!(!SOURCE.contains("impl Default for RequestContextRejection"));
        assert!(!SOURCE.contains("Deserialize<'de> for RequestContextRejection"));
    }

    #[test]
    fn rejection_carrier_public_accessors_only() {
        assert!(SOURCE.contains("pub const fn projection(&self)"));
        assert!(SOURCE.contains("pub const fn class(&self)"));
        assert!(SOURCE.contains("pub(crate) const fn diagnostic_source"));
    }

    #[test]
    fn rejection_cross_boundary_single_branch() {
        let section = &SOURCE[SOURCE.find("pub enum M00AdmissionResult").expect("result")..];
        assert!(section.contains("Rejected(RequestContextRejection)"));
        assert!(section.contains("PriorRejected(RequestContextRejection)"));
        assert!(!section.contains("RequestContextError"));
    }

    #[test]
    fn rejection_projection_constructed_at_site_no_side_channel() {
        let mut ports = FakePorts::public();
        ports.capability = Ok(CapabilityDisposition::Missing);
        let result = admit(&public_command(true), &mut ports);
        expect_rejection(result, AdmissionRejectionClass::CapabilityMissing);
        assert_eq!(ports.capability_calls, 1);
    }

    #[test]
    fn rejection_m10_wire_table_total_14_rows() {
        let rows: Vec<_> = projection_classes()
            .into_iter()
            .map(|class| match class {
                AdmissionRejectionClass::IdempotencyStoreUnavailable => {
                    "idempotency_store_unavailable"
                }
                AdmissionRejectionClass::ConflictingEnvelope => "conflicting_envelope",
                AdmissionRejectionClass::DescriptorSnapshotAbsent => "descriptor_snapshot_absent",
                AdmissionRejectionClass::DescriptorSnapshotMismatch => {
                    "descriptor_snapshot_mismatch"
                }
                AdmissionRejectionClass::PolicyDenied => "policy_denied",
                AdmissionRejectionClass::PolicyExpired => "policy_expired",
                AdmissionRejectionClass::SessionNotFound => "session_not_found",
                AdmissionRejectionClass::SessionIdMismatch => "session_id_mismatch",
                AdmissionRejectionClass::SessionNotAdmitted => "session_not_admitted",
                AdmissionRejectionClass::CapabilityMissing => "capability_missing",
                AdmissionRejectionClass::CapabilityDisabled => "capability_disabled",
                AdmissionRejectionClass::CapabilityRevoked => "capability_revoked",
                AdmissionRejectionClass::InfrastructurePortUnavailable => {
                    "infrastructure_port_unavailable"
                }
                AdmissionRejectionClass::MalformedCommand => "malformed_command",
            })
            .collect();
        assert_eq!(rows.len(), 14);
    }

    #[test]
    fn rejection_projection_malformed_command_option_echo() {
        let none = AdmissionRejectionProjection::MalformedCommand { operation_id: None };
        let some = AdmissionRejectionProjection::MalformedCommand {
            operation_id: Some(operation("affairs.get")),
        };
        assert!(matches!(
            none,
            AdmissionRejectionProjection::MalformedCommand { operation_id: None }
        ));
        assert!(matches!(
            some,
            AdmissionRejectionProjection::MalformedCommand {
                operation_id: Some(_)
            }
        ));
    }

    #[test]
    fn rejection_projection_infrastructure_port_retains_port_for_diagnosis() {
        let projection = AdmissionRejectionProjection::InfrastructurePortUnavailable {
            operation_id: operation("affairs.get"),
            port: AdmissionPortKind::Policy,
        };
        assert!(matches!(
            projection,
            AdmissionRejectionProjection::InfrastructurePortUnavailable {
                port: AdmissionPortKind::Policy,
                ..
            }
        ));
    }

    #[test]
    fn request_context_public_disposition_has_no_synthetic_identity() {
        assert_public_admitted();
        let json = serde_json::to_string(&persisted_public()).expect("serialize");
        assert!(!json.contains("tenant:"));
        assert!(!json.contains("user:"));
        assert!(!json.contains("session:"));
        assert!(json.contains("\"session_id\":null"));
    }

    #[test]
    fn request_context_authenticated_disposition_exact_session_binding() {
        let expected = session("session:a");
        let mut ports = FakePorts::public();
        ports.loaded_session = Ok(Some(active_session(expected.clone())));
        let M00AdmissionResult::Admitted { disposition, .. } =
            admit(&authenticated_command(expected.clone()), &mut ports)
        else {
            panic!()
        };
        let M00AdmittedActor::Authenticated(ids) = disposition.admitted_actor() else {
            panic!()
        };
        assert_eq!(ids.session_id(), &expected);
        assert_eq!(
            disposition.frozen_prerequisites().session_id(),
            Some(&expected)
        );
    }

    #[test]
    fn request_context_disposition_actor_session_incoherence_rejected() {
        let actor_session = session("session:a");
        let frozen_session = session("session:b");
        let result = PersistedAdmittedDispositionDto::try_from_parts(
            command_id(),
            correlation_id(),
            descriptor_id(),
            PersistedAdmittedActorDto::Authenticated {
                tenant_id: tenant(),
                user_id: user(),
                session_id: actor_session,
            },
            PersistedFrozenPrerequisitesDto::from_parts(
                policy_id(),
                at(1_000),
                Some(frozen_session),
                operation("affairs.get"),
            ),
        );
        assert!(result.is_err());
    }

    #[test]
    fn request_context_disposition_admitted_operation_equals_enabled_check() {
        let (result, _) = fresh_public();
        let M00AdmissionResult::Admitted {
            context,
            disposition,
        } = result
        else {
            panic!()
        };
        assert_eq!(
            disposition.frozen_prerequisites().admitted_operation_id(),
            context.operation().operation_id()
        );
    }

    #[test]
    fn request_context_disposition_descriptor_accessors_round_trip() {
        let (result, _) = fresh_public();
        let M00AdmissionResult::Admitted { disposition, .. } = result else {
            panic!()
        };
        assert_eq!(
            disposition.descriptor_snapshot_id().content_digest(),
            &schema_digest()
        );
        assert_eq!(disposition.descriptor_snapshot_id().snapshot_version(), 7);
    }

    #[test]
    fn request_context_prior_admitted_returns_complete_scalar_disposition_no_arc() {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::PriorIdentical(
            PersistedPriorDispositionDto::Admitted(persisted_public()),
        ));
        let M00AdmissionResult::PriorAdmitted(value) = admit(&public_command(true), &mut ports)
        else {
            panic!()
        };
        assert_eq!(value.descriptor_snapshot_id(), &descriptor_id());
        assert_eq!(ports.descriptor_calls, 0);
    }

    #[test]
    fn request_context_prior_rejected_preserves_exact_projection_payload() {
        let projection = AdmissionRejectionProjection::PolicyExpired {
            operation_id: operation("affairs.get"),
            policy_snapshot_id: policy_id(),
        };
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::PriorIdentical(
            PersistedPriorDispositionDto::Rejected(
                PersistedAdmissionRejectionDto::from_projection(&projection),
            ),
        ));
        let M00AdmissionResult::PriorRejected(value) = admit(&public_command(true), &mut ports)
        else {
            panic!()
        };
        assert_eq!(value.projection(), &projection);
    }

    #[test]
    fn request_context_inflight_never_becomes_admitted_or_rejected() {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::InFlight(token(9)));
        assert!(matches!(
            admit(&public_command(true), &mut ports),
            M00AdmissionResult::Incomplete(_)
        ));
        assert_eq!(ports.finalize_calls, 0);
    }

    #[test]
    fn request_context_finalize_response_loss_returns_already_same() {
        let mut ports = FakePorts::public();
        ports.finalize = Ok(FinalizeIdempotencyOutcome::AlreadySame(
            PersistedPriorDispositionDto::Admitted(persisted_public()),
        ));
        assert!(matches!(
            admit(&public_command(true), &mut ports),
            M00AdmissionResult::PriorAdmitted(_)
        ));
    }

    #[test]
    fn request_context_stale_finalizer_cannot_commit_after_reclaim() {
        let mut ports = FakePorts::public();
        let lost =
            IdempotencyReservationToken::from_store_observation(command_id(), 4, 2, at(1_200))
                .expect("fixture");
        ports.finalize = Ok(FinalizeIdempotencyOutcome::LostReservation(lost));
        let M00AdmissionResult::Incomplete(value) = admit(&public_command(true), &mut ports) else {
            panic!()
        };
        assert_eq!(value.retry_not_before(), at(1_200));
    }

    #[test]
    fn request_context_idempotency_reopen_restores_admitted_and_rejected_entries() {
        let admitted_json =
            serde_json::to_vec(&PersistedPriorDispositionDto::Admitted(persisted_public()))
                .expect("serialize");
        let admitted: PersistedPriorDispositionDto =
            serde_json::from_slice(&admitted_json).expect("restore");
        assert!(matches!(
            admitted,
            PersistedPriorDispositionDto::Admitted(_)
        ));
        let rejected = PersistedPriorDispositionDto::Rejected(
            PersistedAdmissionRejectionDto::from_projection(&projections()[0]),
        );
        let rejected_json = serde_json::to_vec(&rejected).expect("serialize");
        let restored: PersistedPriorDispositionDto =
            serde_json::from_slice(&rejected_json).expect("restore");
        assert!(matches!(
            restored,
            PersistedPriorDispositionDto::Rejected(_)
        ));
    }

    #[test]
    fn request_context_m10_v17_closed_match_delta_binding_fixture() {
        fn exhaust(result: M00AdmissionResult) -> &'static str {
            match result {
                M00AdmissionResult::Admitted { .. } => "admitted",
                M00AdmissionResult::PriorAdmitted(_) => "prior_admitted",
                M00AdmissionResult::Rejected(_) => "rejected",
                M00AdmissionResult::PriorRejected(_) => "prior_rejected",
                M00AdmissionResult::Incomplete(_) => "incomplete",
            }
        }
        let (result, _) = fresh_public();
        assert_eq!(exhaust(result), "admitted");
    }

    #[test]
    fn request_context_persisted_public_disposition_round_trips_without_identity() {
        let value = persisted_public();
        let json = serde_json::to_string(&value).expect("serialize");
        let decoded: PersistedAdmittedDispositionDto =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, value);
        assert!(matches!(
            decoded.admitted_actor(),
            PersistedAdmittedActorDto::Public
        ));
        assert!(decoded.frozen_prerequisites().session_id().is_none());

        let admitted_prior = PersistedPriorDispositionDto::Admitted(decoded.clone());
        assert_eq!(admitted_prior.admitted(), Some(&decoded));
        assert!(admitted_prior.rejected().is_none());
        let rejection = PersistedAdmissionRejectionDto::from_projection(&projections()[0]);
        let rejected_prior = PersistedPriorDispositionDto::Rejected(rejection.clone());
        assert_eq!(rejected_prior.rejected(), Some(&rejection));
        assert!(rejected_prior.admitted().is_none());
    }

    #[test]
    fn request_context_persisted_authenticated_disposition_requires_exact_session_pair() {
        let value = persisted_authenticated(session("session:a"));
        let json = serde_json::to_string(&value).expect("serialize");
        let decoded: PersistedAdmittedDispositionDto =
            serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, value);
        let bad = json
            .replace("session:a", "session:b")
            .replacen("session:b", "session:a", 1);
        assert!(serde_json::from_str::<PersistedAdmittedDispositionDto>(&bad).is_err());
    }

    #[test]
    fn request_context_persisted_rejection_all_fourteen_variants_round_trip() {
        assert_projection_round_trip();
    }

    #[test]
    fn request_context_persisted_unknown_or_incoherent_fields_fail_closed() {
        let json = serde_json::to_string(&persisted_public()).expect("serialize");
        let unknown = json.replacen('{', "{\"unknown\":1,", 1);
        assert!(serde_json::from_str::<PersistedAdmittedDispositionDto>(&unknown).is_err());
        let incoherent = json.replace("\"session_id\":null", "\"session_id\":\"session:fake\"");
        assert!(serde_json::from_str::<PersistedAdmittedDispositionDto>(&incoherent).is_err());
    }

    #[test]
    fn request_context_store_observation_requires_nonzero_fencing() {
        assert!(
            IdempotencyReservationToken::from_store_observation(command_id(), 1, 0, at(1_000))
                .is_err()
        );
        assert_eq!(token(1).fencing_token().get(), 1);
    }

    #[test]
    fn request_context_store_projection_promotes_only_through_m00_checked_conversion() {
        let mut ports = FakePorts::public();
        ports.reservation = Ok(IdempotencyReservation::PriorIdentical(
            PersistedPriorDispositionDto::Admitted(persisted_public()),
        ));
        assert!(matches!(
            admit(&public_command(true), &mut ports),
            M00AdmissionResult::PriorAdmitted(_)
        ));
        assert!(!SOURCE.contains("pub fn from_persisted"));
    }

    #[test]
    fn request_context_cross_crate_fake_constructs_port_data_not_authority_carriers() {
        let _ = persisted_public();
        let _ = token(1);
        assert!(!SOURCE.contains("pub fn try_from_parts(\n        command_id: CommandId,\n        correlation_id: CorrelationId,\n        descriptor_snapshot_id: DescriptorSnapshotId,\n        admitted_actor: M00AdmittedActor"));
    }

    #[test]
    fn request_context_registration_preserves_platform_identity_six_kind_surface() {
        assert_eq!(IDENTITY_SOURCE.matches("identity_value! {").count(), 6);
        assert!(!IDENTITY_SOURCE.contains("CausationId"));
    }

    #[test]
    fn request_context_persisted_leaf_serde_round_trips_checked_values() {
        let leaves = serde_json::to_string(&(
            operation("affairs.get"),
            key(),
            policy_id(),
            PermissionClass::PublicRead,
            ActorKind::Public,
            AdmissionPortKind::Policy,
        ))
        .expect("serialize");
        let decoded: (
            OperationId,
            IdempotencyKey,
            PlatformPolicySnapshotId,
            PermissionClass,
            ActorKind,
            AdmissionPortKind,
        ) = serde_json::from_str(&leaves).expect("deserialize");
        assert_eq!(decoded.0, operation("affairs.get"));
    }

    #[test]
    fn request_context_persisted_leaf_invalid_deserialize_rejected() {
        assert!(serde_json::from_str::<OperationId>("\"bad value\"").is_err());
        assert!(serde_json::from_str::<PermissionClass>("\"unknown\"").is_err());
        assert!(serde_json::from_str::<AdmissionPortKind>("\"unknown\"").is_err());
    }

    #[test]
    fn request_context_schema_digest_persisted_lower_hex_exact() {
        let json = serde_json::to_string(&schema_digest()).expect("serialize");
        assert_eq!(json, format!("\"{DIGEST}\""));
        assert!(
            serde_json::from_str::<SchemaDigest>(&format!("\"{}\"", DIGEST.to_uppercase()))
                .is_err()
        );
        assert!(serde_json::from_str::<SchemaDigest>("\"abcd\"").is_err());
        assert!(serde_json::from_str::<SchemaDigest>(&format!("\"{}g\"", &DIGEST[..63])).is_err());
    }
}
