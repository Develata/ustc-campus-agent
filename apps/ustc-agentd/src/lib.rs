//! Bounded composition root for the `ustc-agentd` product-path slice.
//!
//! Only this crate may simultaneously name M00 fixture ports, M10 ingress, M20
//! current invocation authority, the M30 deterministic harness path, the bounded
//! ToolGateway adapters, and M70/M71 publication/query services. M60 fixture
//! decisions enter only through the owning product ports; they never enter an
//! M10/client seam and remain explicitly noncanonical.

#![forbid(unsafe_code)]

mod affairs_fixture;
mod affairs_invocation;
mod affairs_persistence;
mod affairs_publication;
mod agent_chat;
mod change_fixture;
mod change_invocation;
mod change_persistence;
mod change_publication;
mod chat_provider;
mod chat_tools;
mod durable_path;
mod m00_control_evidence;
mod m00_session;
mod opportunity_authority;
mod opportunity_fixture;
mod opportunity_persistence;
mod opportunity_use_case;
mod web;

pub use web::web_router;

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use affairs_fixture::{AffairsFixture, DurableIdempotencyStore, FixturePorts};
use affairs_invocation::AffairsInvocationSpine;
use affairs_navigator::{AffairsGetService, ProcedurePublicationRepository};
use affairs_publication::{AffairsPublicationCounters, FixtureAffairsPublicationPort};
use change_fixture::ChangeRadarFixture;
use change_invocation::ChangeInvocationSpine;
use change_publication::{ChangePublicationCounters, FixtureChangePublicationPort};
use m00_control_evidence::{DurableControlEvidenceJournal, ensure_secure_state_parent};
use m00_session::DurableCurrentSessionStore;
use opportunity_authority::OpportunityMarketAuthorityStore;
use opportunity_fixture::OpportunityFixture;
use opportunity_persistence::DurableOpportunityProfileRepository;
use opportunity_use_case::OpportunityApplicationUseCase;
use ustc_campus_agent_application_ingress::{
    AffairsPublicationCommand, AffairsPublicationOutcome, ChangePublicationCommand,
    ChangePublicationOutcome, FileRecordStore, M10AffairsPublicationService, M10ChangeFeedService,
    M10ChangePublicationService, M10OpportunityService, M10Service,
    affairs_publication_payload_digest, change_publication_payload_digest,
};
use ustc_campus_agent_client_protocol::{
    ClientIntentDto, ClientResponseDto, SubmitAffairsGetDto, SubmitChangeFeedDto,
    SubmitOpportunityDto, ViewerAuthorizationDto, read_frame, write_frame,
};
use ustc_campus_agent_core::identity::{CorrelationId, RequestId, TenantId, UserId};
use ustc_campus_agent_core::request_context::{
    ActorReference, CapabilityDisposition, ClientProvenance, IdempotencyKey,
};
use ustc_campus_agent_core::session_port::{SessionHistoryReadPort, SessionRepositoryError};
use ustc_campus_agent_simple_calendar::{CalendarError, CalendarItem, CalendarStore};

const FRAMED_CONNECTION_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Bounded composition root owning the fixture, record store and idempotency
/// store for the product-path slice.
///
/// Per-request construction of `AffairsGetService` and `M10Service` avoids
/// self-referential lifetime issues: `AffairsGetService` borrows the repository,
/// M60 port and clock from the fixture, and `M10Service` borrows the
/// `AffairsGetService`. Both borrows are scoped to a single request.
pub struct AffairsComposition {
    fixture: AffairsFixture,
    change: Option<ChangeRadarFixture>,
    opportunity: Option<OpportunityComposition>,
    store: FileRecordStore,
    idempotency: DurableIdempotencyStore,
    control_evidence: DurableControlEvidenceJournal,
    sessions: DurableCurrentSessionStore,
    calendar: CalendarStore,
    publication_counters: AffairsPublicationCounters,
    publication_capability: CapabilityDisposition,
    change_publication_counters: ChangePublicationCounters,
    change_publication_capability: CapabilityDisposition,
    current_tenant_id: TenantId,
    current_user_id: UserId,
}

struct OpportunityComposition {
    fixture: OpportunityFixture,
    authority: OpportunityMarketAuthorityStore,
    profiles: Mutex<DurableOpportunityProfileRepository>,
}

fn map_session_store_error(error: SessionRepositoryError) -> String {
    match error {
        SessionRepositoryError::Unavailable => "session_store_unavailable",
        SessionRepositoryError::Corrupt => "session_store_corrupt",
        SessionRepositoryError::InvalidEvent => "session_store_invalid_event",
        SessionRepositoryError::LimitExceeded => "session_store_limit_exceeded",
        SessionRepositoryError::InternalInvariant => "session_store_internal_invariant",
    }
    .to_owned()
}

fn path_entry_exists(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err("state_path_metadata_unavailable".to_owned()),
    }
}

fn durable_state_paths(
    store_path: &Path,
    idempotency_path: &Path,
    session_store_path: &Path,
    additional_state_paths: &[&Path],
) -> Vec<std::path::PathBuf> {
    let mut paths = vec![
        idempotency_path.with_extension("affairs-publication.json"),
        idempotency_path.with_extension("control-evidence.json"),
        idempotency_path.with_extension("calendar-items.json"),
        store_path.to_path_buf(),
        idempotency_path.to_path_buf(),
        session_store_path.to_path_buf(),
    ];
    paths.extend(
        additional_state_paths
            .iter()
            .map(|path| (*path).to_path_buf()),
    );
    paths
}

fn state_set_bootstrap_is_fresh(
    store_path: &Path,
    idempotency_path: &Path,
    session_store_path: &Path,
    additional_state_paths: &[&Path],
) -> Result<bool, String> {
    let paths = durable_state_paths(
        store_path,
        idempotency_path,
        session_store_path,
        additional_state_paths,
    );
    let mut present = 0_usize;
    for path in &paths {
        present += usize::from(path_entry_exists(path)?);
    }
    if present == 0 {
        Ok(true)
    } else if present == paths.len() {
        Ok(false)
    } else {
        Err("durable_state_set_incomplete".to_owned())
    }
}

fn rollback_failed_fresh_bootstrap<T>(
    result: Result<T, String>,
    bootstrap_is_fresh: bool,
    paths: &[std::path::PathBuf],
) -> Result<T, String> {
    match result {
        Err(original) if bootstrap_is_fresh => {
            durable_path::rollback_fresh_state_paths(paths).map_err(|rollback| {
                format!("fresh_bootstrap_rollback_failed: {rollback}; original: {original}")
            })?;
            Err(original)
        }
        other => other,
    }
}

impl AffairsComposition {
    /// Opens the composition from durable fixture, record-store and
    /// idempotency-store paths.
    ///
    /// # Errors
    ///
    /// Returns a descriptive string when any component fails to load or open.
    pub fn open(
        fixture_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
        session_store_path: &Path,
    ) -> Result<Self, String> {
        let paths = durable_state_paths(store_path, idempotency_path, session_store_path, &[]);
        let _bootstrap_lock = durable_path::StateSetBootstrapLock::acquire(&paths)?;
        let bootstrap_is_fresh =
            state_set_bootstrap_is_fresh(store_path, idempotency_path, session_store_path, &[])?;
        let result = Self::open_with_required_state_paths(
            fixture_path,
            store_path,
            idempotency_path,
            session_store_path,
            bootstrap_is_fresh,
        );
        rollback_failed_fresh_bootstrap(result, bootstrap_is_fresh, &paths)
    }

    fn open_with_required_state_paths(
        fixture_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
        session_store_path: &Path,
        publication_bootstrap_is_fresh: bool,
    ) -> Result<Self, String> {
        ensure_secure_state_parent(idempotency_path)?;
        let publication_path = idempotency_path.with_extension("affairs-publication.json");
        let control_evidence_path = idempotency_path.with_extension("control-evidence.json");
        let calendar_path = idempotency_path.with_extension("calendar-items.json");
        let fixture = AffairsFixture::load(
            fixture_path,
            &publication_path,
            publication_bootstrap_is_fresh,
        )?;
        let control_evidence = DurableControlEvidenceJournal::open_for_state_set(
            &control_evidence_path,
            publication_bootstrap_is_fresh,
        )
        .map_err(|error| format!("control evidence open failed: {error:?}"))?;
        let store = FileRecordStore::open_for_state_set(store_path, publication_bootstrap_is_fresh)
            .map_err(|e| format!("store open failed: {e:?}"))?;
        let now_ms = fixture.now.as_unix_millis();
        let idempotency = DurableIdempotencyStore::open_for_state_set(
            idempotency_path,
            now_ms,
            fixture.idempotency_deadline_ms,
            publication_bootstrap_is_fresh,
        )?;
        let mut sessions = DurableCurrentSessionStore::open_or_bootstrap_for_state_set(
            session_store_path,
            &fixture.session_events,
            publication_bootstrap_is_fresh,
        )
        .map_err(map_session_store_error)?;
        let current = sessions
            .load_history(fixture.session.session_id())
            .map_err(map_session_store_error)?
            .ok_or_else(|| "session_store_current_session_absent".to_owned())?;
        if current.snapshot().tenant_id() != fixture.session.tenant_id()
            || current.snapshot().user_id() != fixture.session.user_id()
        {
            return Err("session_store_current_session_scope_mismatch".to_owned());
        }
        let current_tenant_id = current.snapshot().tenant_id().clone();
        let current_user_id = current.snapshot().user_id().clone();
        let calendar =
            CalendarStore::open_for_state_set(&calendar_path, publication_bootstrap_is_fresh)
                .map_err(|error| format!("simple calendar open failed: {error}"))?;
        Ok(Self {
            fixture,
            change: None,
            opportunity: None,
            store,
            idempotency,
            control_evidence,
            sessions,
            calendar,
            publication_counters: AffairsPublicationCounters::default(),
            publication_capability: CapabilityDisposition::Enabled,
            change_publication_counters: ChangePublicationCounters::default(),
            change_publication_capability: CapabilityDisposition::Enabled,
            current_tenant_id,
            current_user_id,
        })
    }

    /// Opens the shared Affairs + ChangeRadar composition from two reviewed
    /// fixture inputs and the common M10 stores.
    pub fn open_with_change(
        fixture_path: &Path,
        change_fixture_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
        session_store_path: &Path,
    ) -> Result<Self, String> {
        let change_publication_path = idempotency_path.with_extension("change-publication.json");
        let paths = durable_state_paths(
            store_path,
            idempotency_path,
            session_store_path,
            &[change_publication_path.as_path()],
        );
        let _bootstrap_lock = durable_path::StateSetBootstrapLock::acquire(&paths)?;
        let (composition, _) = Self::open_with_change_required_state_paths_unlocked(
            fixture_path,
            change_fixture_path,
            store_path,
            idempotency_path,
            session_store_path,
            &[],
        )?;
        Ok(composition)
    }

    fn open_with_change_required_state_paths_unlocked(
        fixture_path: &Path,
        change_fixture_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
        session_store_path: &Path,
        additional_state_paths: &[&Path],
    ) -> Result<(Self, bool), String> {
        let change_publication_path = idempotency_path.with_extension("change-publication.json");
        let mut required_state_paths = vec![change_publication_path.as_path()];
        required_state_paths.extend_from_slice(additional_state_paths);
        let bootstrap_is_fresh = state_set_bootstrap_is_fresh(
            store_path,
            idempotency_path,
            session_store_path,
            &required_state_paths,
        )?;
        let paths = durable_state_paths(
            store_path,
            idempotency_path,
            session_store_path,
            &required_state_paths,
        );
        let result = (|| {
            let mut composition = Self::open_with_required_state_paths(
                fixture_path,
                store_path,
                idempotency_path,
                session_store_path,
                bootstrap_is_fresh,
            )?;
            composition.change = Some(ChangeRadarFixture::load(
                change_fixture_path,
                &change_publication_path,
                bootstrap_is_fresh,
            )?);
            Ok((composition, bootstrap_is_fresh))
        })();
        rollback_failed_fresh_bootstrap(result, bootstrap_is_fresh, &paths)
    }

    /// Opens the shared Affairs + Opportunity Graph composition. This remains
    /// independent of ChangeRadar so one Plugin may be disabled without
    /// breaking the other product projections.
    pub fn open_with_opportunity(
        fixture_path: &Path,
        opportunity_fixture_path: &Path,
        opportunity_catalog_path: &Path,
        opportunity_profile_store_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
        session_store_path: &Path,
    ) -> Result<Self, String> {
        let paths = durable_state_paths(
            store_path,
            idempotency_path,
            session_store_path,
            &[opportunity_profile_store_path],
        );
        let _bootstrap_lock = durable_path::StateSetBootstrapLock::acquire(&paths)?;
        let bootstrap_is_fresh = state_set_bootstrap_is_fresh(
            store_path,
            idempotency_path,
            session_store_path,
            &[opportunity_profile_store_path],
        )?;
        let result = (|| {
            let mut composition = Self::open_with_required_state_paths(
                fixture_path,
                store_path,
                idempotency_path,
                session_store_path,
                bootstrap_is_fresh,
            )?;
            composition.attach_opportunity(
                opportunity_fixture_path,
                opportunity_catalog_path,
                opportunity_profile_store_path,
                bootstrap_is_fresh,
            )?;
            Ok(composition)
        })();
        rollback_failed_fresh_bootstrap(result, bootstrap_is_fresh, &paths)
    }

    /// Opens all three first-party Plugin product paths in one composition.
    #[allow(clippy::too_many_arguments)] // One explicit path per independently durable authority.
    pub fn open_with_change_and_opportunity(
        fixture_path: &Path,
        change_fixture_path: &Path,
        opportunity_fixture_path: &Path,
        opportunity_catalog_path: &Path,
        opportunity_profile_store_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
        session_store_path: &Path,
    ) -> Result<Self, String> {
        let change_publication_path = idempotency_path.with_extension("change-publication.json");
        let paths = durable_state_paths(
            store_path,
            idempotency_path,
            session_store_path,
            &[
                change_publication_path.as_path(),
                opportunity_profile_store_path,
            ],
        );
        let _bootstrap_lock = durable_path::StateSetBootstrapLock::acquire(&paths)?;
        let (mut composition, bootstrap_is_fresh) =
            Self::open_with_change_required_state_paths_unlocked(
                fixture_path,
                change_fixture_path,
                store_path,
                idempotency_path,
                session_store_path,
                &[opportunity_profile_store_path],
            )?;
        let result = composition
            .attach_opportunity(
                opportunity_fixture_path,
                opportunity_catalog_path,
                opportunity_profile_store_path,
                bootstrap_is_fresh,
            )
            .map(|()| composition);
        rollback_failed_fresh_bootstrap(result, bootstrap_is_fresh, &paths)
    }

    fn attach_opportunity(
        &mut self,
        fixture_path: &Path,
        catalog_path: &Path,
        profile_store_path: &Path,
        bootstrap_is_fresh: bool,
    ) -> Result<(), String> {
        let fixture = OpportunityFixture::load(fixture_path, catalog_path)?;
        let authority = OpportunityMarketAuthorityStore::new(
            self.current_tenant_id.clone(),
            self.current_user_id.clone(),
            fixture.market_enabled,
            fixture.market_grant_active,
            &fixture.source_evidence_digest,
            fixture.authority_mutation,
        )
        .map_err(|error| format!("opportunity Market authority open failed: {error:?}"))?;
        let profiles = DurableOpportunityProfileRepository::open(
            profile_store_path,
            64,
            256,
            bootstrap_is_fresh,
        )?;
        self.opportunity = Some(OpportunityComposition {
            fixture,
            authority,
            profiles: Mutex::new(profiles),
        });
        Ok(())
    }

    /// Handles one `SubmitAffairsGet` intent through the real M00 admission
    /// coordinator, M10 service, bounded Market/Agent/ToolGateway spine, owning
    /// M71 application service and M60 fixture port.
    #[must_use]
    pub fn handle_submit(&self, request: &SubmitAffairsGetDto) -> ClientResponseDto {
        let m71 =
            AffairsGetService::new(&self.fixture.repo, &self.fixture.m60, &self.fixture.clock);
        let affairs = AffairsInvocationSpine::new(
            m71,
            self.fixture.market_enabled,
            self.fixture.market_grant_active,
            self.fixture.source_evidence_digest.clone(),
            self.fixture.invocation_counters.clone(),
        );
        let m10 = M10Service::new(
            self.store.clone(),
            self.fixture.capabilities.clone(),
            &affairs,
            self.fixture.operator_grant_id.clone(),
        );
        let mut ports = FixturePorts::new(
            self.idempotency.clone(),
            Arc::clone(&self.fixture.descriptor),
            self.fixture.now,
            self.fixture.policy_snapshot_id.clone(),
            self.sessions.clone(),
        );
        let now_ms = i64::try_from(self.fixture.now.as_unix_millis()).unwrap_or(i64::MAX);
        m10.submit(request, &mut ports, now_ms)
    }

    pub(crate) fn calendar_items(&mut self) -> Result<Vec<CalendarItem>, CalendarError> {
        Ok(self.calendar.items()?.to_vec())
    }

    pub(crate) fn record_calendar_item(
        &mut self,
        title: &str,
        scheduled_for: Option<&str>,
    ) -> Result<CalendarItem, CalendarError> {
        self.calendar.record(title, scheduled_for)
    }

    pub(crate) fn delete_calendar_item(
        &mut self,
        item_id: &str,
    ) -> Result<CalendarItem, CalendarError> {
        self.calendar.delete(item_id)
    }

    /// Publishes the reviewed demo procedure through M10 admission, durable
    /// M00 control evidence, and the direct owning M71 application port.
    #[must_use]
    pub fn publish_demo_as_administrator(&mut self) -> AffairsPublicationOutcome {
        let procedure_id = self.fixture.publication_draft.procedure_id().clone();
        let payload_digest = affairs_publication_payload_digest(&procedure_id, Some(1));
        let request_id = match RequestId::parse("request:affairs-publication-demo") {
            Ok(value) => value,
            Err(_) => return AffairsPublicationOutcome::InternalInvariant,
        };
        let correlation_id = match CorrelationId::parse("correlation:affairs-publication-demo") {
            Ok(value) => value,
            Err(_) => return AffairsPublicationOutcome::InternalInvariant,
        };
        let idempotency_key = match IdempotencyKey::parse("idempotency:affairs-publication-demo") {
            Ok(value) => value,
            Err(_) => return AffairsPublicationOutcome::InternalInvariant,
        };
        let provenance =
            match ClientProvenance::new("ustc-agentd", "internal-affairs-administrator", "rust-v1")
            {
                Ok(value) => value,
                Err(_) => return AffairsPublicationOutcome::InternalInvariant,
            };
        let command = AffairsPublicationCommand::new(
            request_id,
            ActorReference::Authenticated {
                session_id: self.fixture.session.session_id().clone(),
            },
            correlation_id,
            None,
            Some(idempotency_key),
            provenance,
            payload_digest,
            procedure_id,
            Some(1),
        );
        let expected_tenant = self.fixture.publication_administrator_tenant_id.clone();
        let expected_user = self.fixture.publication_administrator_user_id.clone();
        let expected_session = self.fixture.publication_administrator_session_id.clone();
        let mut ports = FixturePorts::new(
            self.idempotency.clone(),
            Arc::clone(&self.fixture.publication_descriptor),
            self.fixture.now,
            self.fixture.policy_snapshot_id.clone(),
            self.sessions.clone(),
        )
        .with_capability(self.publication_capability);
        let outcome = {
            let mut publication = FixtureAffairsPublicationPort::new(
                &mut self.fixture.repo,
                &self.fixture.m60,
                &self.fixture.publication_draft,
                &self.fixture.publication_reviewer,
                self.fixture.publication_reviewed_at,
                self.fixture.publication_published_at,
                &expected_tenant,
                &expected_user,
                &expected_session,
                self.publication_counters.clone(),
            );
            let mut service =
                M10AffairsPublicationService::new(&mut publication, &mut self.control_evidence);
            service.submit(&command, &mut ports)
        };
        if let AffairsPublicationOutcome::Published(receipt) = &outcome {
            self.fixture.publication_receipt = receipt.clone();
        }
        outcome
    }

    /// Configures only the M00 capability fact for the publication command.
    pub fn set_publication_capability(&mut self, capability: CapabilityDisposition) {
        self.publication_capability = capability;
    }

    /// Injects one bounded fixture-adapter persistence failure. This is a test
    /// seam for proving durable-before-visible publication ordering.
    #[doc(hidden)]
    pub fn fail_next_publication_persistence_for_test(&mut self) {
        self.fixture.repo.fail_next_persist();
    }

    /// Injects one post-rename parent-sync uncertainty. Exact destination
    /// read-back must reconcile it before the durable candidate becomes visible.
    #[doc(hidden)]
    pub fn fail_next_publication_parent_sync_after_rename_for_test(&mut self) {
        self.fixture.repo.fail_next_parent_sync_after_rename();
    }

    #[must_use]
    pub fn publication_application_call_count(&self) -> u64 {
        self.publication_counters.applications()
    }

    #[must_use]
    pub fn control_evidence_event_count(&self) -> usize {
        self.control_evidence.event_count()
    }

    #[must_use]
    pub fn current_publication_revision(&self) -> Option<u64> {
        self.fixture
            .repo
            .publication_revision(self.fixture.publication_draft.procedure_id())
    }

    /// Publishes the fixed reviewed ChangeRadar event through M10 admission,
    /// durable M00 control evidence, and the direct owning M70 application port.
    #[must_use]
    pub fn publish_change_demo_as_administrator(&mut self) -> ChangePublicationOutcome {
        let Some(published_at) = self.change.as_ref().map(|change| change.published_at) else {
            return ChangePublicationOutcome::InternalInvariant;
        };
        self.publish_change_demo_at(published_at)
    }

    /// Test-only identity-conflict seam. Production callers use the fixed command above.
    #[doc(hidden)]
    pub fn publish_change_demo_at_for_test(
        &mut self,
        published_at: ustc_campus_agent_core::source_revision::RevisionTimestamp,
    ) -> ChangePublicationOutcome {
        self.publish_change_demo_at(published_at)
    }

    fn publish_change_demo_at(
        &mut self,
        published_at: ustc_campus_agent_core::source_revision::RevisionTimestamp,
    ) -> ChangePublicationOutcome {
        let Some(change) = self.change.as_mut() else {
            return ChangePublicationOutcome::InternalInvariant;
        };
        let event_id = change.candidate.event_id().clone();
        let review_receipt_id = change.review.receipt_id().clone();
        let payload_digest = change_publication_payload_digest(
            &event_id,
            review_receipt_id.as_str(),
            change.review.reviewed_at(),
            published_at,
        );
        let request_id = match RequestId::parse("request:change-publication-demo") {
            Ok(value) => value,
            Err(_) => return ChangePublicationOutcome::InternalInvariant,
        };
        let correlation_id = match CorrelationId::parse("correlation:change-publication-demo") {
            Ok(value) => value,
            Err(_) => return ChangePublicationOutcome::InternalInvariant,
        };
        let idempotency_key = match IdempotencyKey::parse("idempotency:change-publication-demo") {
            Ok(value) => value,
            Err(_) => return ChangePublicationOutcome::InternalInvariant,
        };
        let provenance = match ClientProvenance::new(
            "ustc-agentd",
            "internal-change-administrator",
            "rust-v1",
        ) {
            Ok(value) => value,
            Err(_) => return ChangePublicationOutcome::InternalInvariant,
        };
        let command = ChangePublicationCommand::new(
            request_id,
            ActorReference::Authenticated {
                session_id: self.fixture.session.session_id().clone(),
            },
            correlation_id,
            None,
            Some(idempotency_key),
            provenance,
            payload_digest,
            event_id,
            &review_receipt_id,
            change.review.reviewed_at(),
            published_at,
        );
        let expected_tenant = self.current_tenant_id.clone();
        let expected_user = self.current_user_id.clone();
        let expected_session = self.fixture.session.session_id().clone();
        let mut ports = FixturePorts::new(
            self.idempotency.clone(),
            Arc::clone(&change.publication_descriptor),
            self.fixture.now,
            self.fixture.policy_snapshot_id.clone(),
            self.sessions.clone(),
        )
        .with_capability(self.change_publication_capability);
        let mut publication = FixtureChangePublicationPort::new(
            &mut change.repository,
            &change.m60,
            &change.candidate,
            &change.review,
            &change.feed_policy,
            change.published_at,
            &expected_tenant,
            &expected_user,
            &expected_session,
            self.change_publication_counters.clone(),
        );
        let mut service =
            M10ChangePublicationService::new(&mut publication, &mut self.control_evidence);
        service.submit(&command, &mut ports)
    }

    /// Configures only the M00 capability fact for the ChangeRadar publication command.
    pub fn set_change_publication_capability(&mut self, capability: CapabilityDisposition) {
        self.change_publication_capability = capability;
    }

    /// Injects one bounded ChangeRadar persistence failure after durable M00 evidence.
    #[doc(hidden)]
    pub fn fail_next_change_publication_persistence_for_test(&mut self) {
        if let Some(change) = self.change.as_mut() {
            change.repository.fail_next_persist();
        }
    }

    /// Injects a post-review publication persistence failure after one durable review commit.
    #[doc(hidden)]
    pub fn fail_change_publication_final_persistence_for_test(&mut self) {
        if let Some(change) = self.change.as_mut() {
            change.repository.fail_publication_persist_after_review();
        }
    }

    /// Injects a post-rename parent-directory sync failure for reconciliation tests.
    #[doc(hidden)]
    pub fn fail_next_change_publication_parent_sync_after_rename_for_test(&mut self) {
        if let Some(change) = self.change.as_mut() {
            change.repository.fail_next_parent_sync_after_rename();
        }
    }

    /// Injects parent-sync uncertainty on the final publication commit, after durable review.
    #[doc(hidden)]
    pub fn fail_change_publication_final_parent_sync_for_test(&mut self) {
        if let Some(change) = self.change.as_mut() {
            change
                .repository
                .fail_publication_parent_sync_after_review();
        }
    }

    #[must_use]
    pub fn change_publication_application_call_count(&self) -> u64 {
        self.change_publication_counters.applications()
    }

    pub fn change_publication_counts(&self) -> Result<(usize, usize), String> {
        let Some(change) = self.change.as_ref() else {
            return Err("ChangeRadar fixture is unavailable".to_owned());
        };
        Ok((
            change
                .repository
                .review_count()
                .map_err(|error| format!("{error:?}"))?,
            change
                .repository
                .publication_count()
                .map_err(|error| format!("{error:?}"))?,
        ))
    }

    pub fn change_publication_receipt_id(&self) -> Result<Option<&str>, String> {
        let Some(change) = self.change.as_ref() else {
            return Err("ChangeRadar fixture is unavailable".to_owned());
        };
        change
            .repository
            .publication_receipt_id()
            .map_err(|error| format!("{error:?}"))
    }

    #[must_use]
    pub fn change_publication_m60_call_count(&self) -> u64 {
        self.change.as_ref().map_or(0, |change| {
            change.m60_calls.load(std::sync::atomic::Ordering::SeqCst)
        })
    }

    /// Handles one public ChangeRadar board query through M00, M10 and the
    /// bounded Market/Agent/ToolGateway/owning-plugin spine.
    #[must_use]
    pub fn handle_change_submit(&self, request: &SubmitChangeFeedDto) -> ClientResponseDto {
        let Some(change) = &self.change else {
            return ClientResponseDto::Unavailable;
        };
        let invocation = ChangeInvocationSpine::new(
            &change.repository,
            &change.feed_policy,
            change.market_enabled,
            change.market_grant_active,
            &change.source_evidence_digest,
            change.invocation_counters.clone(),
        );
        let m10 = M10ChangeFeedService::new(&invocation);
        let mut ports = FixturePorts::new(
            self.idempotency.clone(),
            Arc::clone(&change.descriptor),
            self.fixture.now,
            self.fixture.policy_snapshot_id.clone(),
            self.sessions.clone(),
        );
        m10.submit(request, &mut ports)
    }

    /// Handles one consent-bound tenant-private Opportunity operation through
    /// M00/M10, current M20 authority, and the statically composed M72 use case.
    #[must_use]
    pub fn handle_opportunity_submit(&self, request: &SubmitOpportunityDto) -> ClientResponseDto {
        let Some(opportunity) = &self.opportunity else {
            return ClientResponseDto::Unavailable;
        };
        let descriptor = match opportunity.fixture.descriptor(&request.command) {
            Ok(descriptor) => descriptor,
            Err(_) => return ClientResponseDto::Unavailable,
        };
        let application = OpportunityApplicationUseCase::new(
            &opportunity.profiles,
            &opportunity.fixture.source,
            &opportunity.fixture.catalog,
            &opportunity.authority,
            opportunity.fixture.application_failure,
            opportunity.fixture.application_counters.clone(),
        );
        let m10 = M10OpportunityService::new(&application);
        let mut ports = FixturePorts::new(
            self.idempotency.clone(),
            descriptor,
            self.fixture.now,
            self.fixture.policy_snapshot_id.clone(),
            self.sessions.clone(),
        );
        m10.submit(request, &mut ports)
    }

    /// Returns ChangeRadar `(effect_intents, plugin_executions,
    /// effect_receipts)` observed by the bounded invocation spine.
    #[must_use]
    pub fn change_invocation_counts(&self) -> (u64, u64, u64) {
        self.change.as_ref().map_or((0, 0, 0), |change| {
            (
                change.invocation_counters.intents(),
                change.invocation_counters.executions(),
                change.invocation_counters.receipts(),
            )
        })
    }

    /// Returns Opportunity `(M20 authorizations, M72 dispatches,
    /// application terminals)` observed by the static application use case.
    #[must_use]
    pub fn opportunity_application_counts(&self) -> (u64, u64, u64) {
        self.opportunity.as_ref().map_or((0, 0, 0), |opportunity| {
            (
                opportunity.fixture.application_counters.authorizations(),
                opportunity.fixture.application_counters.dispatches(),
                opportunity.fixture.application_counters.terminals(),
            )
        })
    }

    /// Returns Opportunity M60 source-currentness checks.
    #[must_use]
    pub fn opportunity_m60_call_count(&self) -> u64 {
        self.opportunity
            .as_ref()
            .map_or(0, |opportunity| opportunity.fixture.source.calls())
    }

    /// Returns `(active_private_payloads, durable_tombstones)` without exposing
    /// tenant-private profile values.
    #[must_use]
    pub fn opportunity_private_state_counts(&self) -> (usize, usize) {
        let Some(opportunity) = &self.opportunity else {
            return (0, 0);
        };
        opportunity.profiles.lock().map_or((0, 0), |profiles| {
            (profiles.private_payload_count(), profiles.tombstone_count())
        })
    }

    /// Injects one post-rename Opportunity parent-sync failure. The next
    /// mutation must reconcile exact canonical bytes before publishing memory.
    #[doc(hidden)]
    pub fn fail_next_opportunity_parent_sync_after_rename_for_test(&self) {
        if let Some(opportunity) = &self.opportunity
            && let Ok(mut profiles) = opportunity.profiles.lock()
        {
            profiles.fail_next_parent_sync_after_rename_for_test();
        }
    }

    /// Handles one `Lookup` intent through the M10 record store. The M71
    /// service is constructed but not called — lookup reads only the durable
    /// record store.
    #[must_use]
    pub fn handle_lookup(
        &self,
        command_id: &str,
        viewer: &ViewerAuthorizationDto,
    ) -> ClientResponseDto {
        let m71 =
            AffairsGetService::new(&self.fixture.repo, &self.fixture.m60, &self.fixture.clock);
        let affairs = AffairsInvocationSpine::new(
            m71,
            self.fixture.market_enabled,
            self.fixture.market_grant_active,
            self.fixture.source_evidence_digest.clone(),
            self.fixture.invocation_counters.clone(),
        );
        let m10 = M10Service::new(
            self.store.clone(),
            self.fixture.capabilities.clone(),
            &affairs,
            self.fixture.operator_grant_id.clone(),
        );
        m10.lookup(command_id, viewer)
    }

    /// Returns the number of M60 `verify_retained` calls observed since the
    /// composition was opened. Used by tests to prove "one M71 call" and
    /// "zero M71 call" invariants.
    #[must_use]
    pub fn m60_call_count(&self) -> u64 {
        self.fixture
            .m60_call_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Returns `(effect_intents, plugin_executions, effect_receipts)` observed
    /// by the bounded invocation spine.
    #[must_use]
    pub fn invocation_counts(&self) -> (u64, u64, u64) {
        (
            self.fixture.invocation_counters.intents(),
            self.fixture.invocation_counters.executions(),
            self.fixture.invocation_counters.receipts(),
        )
    }

    /// Returns the immutable receipt identity minted while the fixture's exact
    /// `DemoReviewed` revision was published. The M10 query path below reads
    /// the same repository state committed by that publication.
    #[must_use]
    pub fn publication_receipt_id(&self) -> &str {
        self.fixture.publication_receipt.receipt_id().as_str()
    }

    /// Binds a loopback TCP listener, prints `listening <addr>` to stdout, and
    /// serves connections sequentially. Each connection reads one
    /// `ClientIntentDto` frame and writes one `ClientResponseDto` frame.
    ///
    /// # Errors
    ///
    /// Returns a descriptive string when binding or address resolution fails.
    pub fn serve(&self, bind_addr: &str) -> Result<(), String> {
        let listener = bind_loopback(bind_addr)?;
        let local_addr = listener
            .local_addr()
            .map_err(|e| format!("local_addr failed: {e}"))?;
        println!("listening {local_addr}");
        std::io::stdout()
            .flush()
            .map_err(|e| format!("stdout flush failed: {e}"))?;

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("accept failed: {e}");
                    continue;
                }
            };
            if let Err(e) = self.handle_connection(stream) {
                eprintln!("connection error: {e}");
            }
        }
        Ok(())
    }

    fn handle_connection(&self, stream: TcpStream) -> Result<(), String> {
        let intent = read_intent_with_timeout(&stream, FRAMED_CONNECTION_READ_TIMEOUT)?;
        let response = match intent {
            ClientIntentDto::SubmitAffairsGet { request } => self.handle_submit(&request),
            ClientIntentDto::SubmitChangeFeed { request } => self.handle_change_submit(&request),
            ClientIntentDto::SubmitOpportunity { request } => {
                self.handle_opportunity_submit(&request)
            }
            ClientIntentDto::Lookup { command_id, viewer } => {
                self.handle_lookup(command_id.as_str(), &viewer)
            }
        };
        write_frame(&stream, &response).map_err(|e| format!("write response: {e}"))?;
        Ok(())
    }
}

fn read_intent_with_timeout(
    stream: &TcpStream,
    timeout: Duration,
) -> Result<ClientIntentDto, String> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| "read intent deadline overflow".to_owned())?;
    let mut reader = DeadlineReader { stream, deadline };
    let intent = read_frame(&mut reader).map_err(|error| format!("read intent: {error}"))?;
    if Instant::now() >= deadline {
        return Err("read intent: absolute frame deadline exceeded".to_owned());
    }
    Ok(intent)
}

struct DeadlineReader<'a> {
    stream: &'a TcpStream,
    deadline: Instant,
}

impl Read for DeadlineReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let Some(remaining) = self.deadline.checked_duration_since(Instant::now()) else {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "absolute frame deadline exceeded",
            ));
        };
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "absolute frame deadline exceeded",
            ));
        }
        self.stream.set_read_timeout(Some(remaining))?;
        let mut stream = self.stream;
        stream.read(buffer)
    }
}

pub(crate) fn parse_loopback_socket_addr(bind_addr: &str) -> Result<SocketAddr, String> {
    let socket_addr: SocketAddr = bind_addr
        .parse()
        .map_err(|e| format!("bind_addr parse failed: {e}"))?;
    if !socket_addr.ip().is_loopback() {
        return Err(format!(
            "bind addr {socket_addr} rejected: only loopback (127.0.0.0/8 or ::1) permitted"
        ));
    }
    Ok(socket_addr)
}

pub fn bind_loopback(bind_addr: &str) -> Result<TcpListener, String> {
    let socket_addr = parse_loopback_socket_addr(bind_addr)?;
    TcpListener::bind(socket_addr).map_err(|e| format!("bind failed: {e}"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::io::Write as _;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{bind_loopback, read_intent_with_timeout};

    #[test]
    fn incomplete_framed_connection_times_out() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback test listener");
        let endpoint = listener.local_addr().expect("test listener address");
        let _incomplete_client = TcpStream::connect(endpoint).expect("connect incomplete client");
        let (server_stream, _) = listener.accept().expect("accept incomplete client");
        let started = Instant::now();
        let result = read_intent_with_timeout(&server_stream, Duration::from_millis(50));

        assert!(result.is_err(), "an incomplete frame must time out");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "the bounded read must not stall the sequential server"
        );
    }

    #[test]
    fn drip_fed_incomplete_frame_hits_absolute_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback test listener");
        let endpoint = listener.local_addr().expect("test listener address");
        let mut client = TcpStream::connect(endpoint).expect("connect drip client");
        let (server_stream, _) = listener.accept().expect("accept drip client");
        let writer = thread::spawn(move || {
            let bytes = [0_u8, 0, 0, 100, 1, 2, 3, 4, 5, 6];
            for byte in bytes {
                if client.write_all(&[byte]).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        });

        let started = Instant::now();
        let result = read_intent_with_timeout(&server_stream, Duration::from_millis(80));

        assert!(result.is_err(), "a drip-fed incomplete frame must time out");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "progress on individual reads must not extend the absolute deadline"
        );
        writer.join().expect("join drip writer");
    }

    #[test]
    fn bind_loopback_ipv4_zero_port_succeeds() {
        let listener = bind_loopback("127.0.0.1:0");
        assert!(listener.is_ok(), "IPv4 loopback bind must succeed");
        let listener = listener.unwrap();
        let local = listener.local_addr().unwrap();
        assert!(
            local.ip().is_loopback(),
            "bound IPv4 address must be loopback"
        );
    }

    #[test]
    fn bind_loopback_ipv6_zero_port_succeeds_or_env_unsupported() {
        let listener = bind_loopback("[::1]:0");
        match listener {
            Ok(listener) => {
                let local = listener.local_addr().unwrap();
                assert!(
                    local.ip().is_loopback(),
                    "bound IPv6 address must be loopback"
                );
            }
            Err(msg) => {
                let lowered = msg.to_ascii_lowercase();
                let recognized_unsupported = lowered.contains("address family")
                    || lowered.contains("address not available")
                    || lowered.contains("not supported")
                    || lowered.contains("protocol not supported")
                    || lowered.contains("no such device")
                    || lowered.contains("eafnosupport")
                    || lowered.contains("enodev")
                    || lowered.contains("eprotonosupport");
                assert!(
                    recognized_unsupported,
                    "IPv6 bind failure must be a recognized unsupported/address-family error, got: {msg}"
                );
            }
        }
    }

    #[test]
    fn bind_loopback_rejects_non_loopback() {
        let result = bind_loopback("0.0.0.0:0");
        assert!(result.is_err(), "non-loopback bind must be rejected");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("rejected"),
            "non-loopback rejection message must explain rejection, got: {msg}"
        );
    }

    #[test]
    fn bind_loopback_rejects_unparseable() {
        let result = bind_loopback("not-an-address");
        assert!(result.is_err(), "unparseable bind must be rejected");
    }
}
