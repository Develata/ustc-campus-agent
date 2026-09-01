//! Loopback-only HTTP/Web adapter for bounded Affairs, ChangeRadar and Opportunity journeys.
//!
//! The adapter owns no procedure, change event, source, freshness, conflict,
//! authorization or eligibility decisions. It constructs bounded public M10
//! requests and admits only the matching typed public result as JSON or Atom.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::rejection::JsonRejection;
use axum::extract::{DefaultBodyLimit, Path as AxumPath, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use ustc_campus_agent_application_ingress::{
    AffairsPublicationApplicationError, AffairsPublicationOutcome,
    ChangePublicationApplicationError, ChangePublicationOutcome, dispatch_with_protocol_major,
};
use ustc_campus_agent_client_protocol::{
    ActorIntentDto, CLIENT_PROTOCOL_MAJOR_HEADER, CapabilityListDto, ClientErrorDto,
    ClientProtocolMajor, ClientProvenanceDto, ClientResponseDto, OpportunityCommandDto,
    OpportunityConsentFieldDto, OpportunityPreferenceDto, OpportunityRejectionDto,
    ProtocolCompatibilityDto, RedactionDto, ServerInfoDto, SubmitAffairsGetDto,
    SubmitChangeFeedDto, SubmitOpportunityDto, UnixMillis, ViewerAuthorizationDto,
    WireErrorClassDto, WireText, affairs_get_payload_digest, change_feed_payload_digest,
    opportunity_payload_digest,
};

use super::{AffairsComposition, parse_loopback_socket_addr};

const INDEX_HTML: &str = include_str!("web/index.html");
const APP_JS: &str = include_str!("web/app.js");
const STYLES_CSS: &str = include_str!("web/styles.css");

const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'";
const ADMINISTRATOR_DEMO_HEADER: &str = "x-ustc-agent-administrator-demo";
const ADMINISTRATOR_DEMO_CONFIRMATION: &str = "confirm-v1";

#[derive(Clone)]
struct WebState {
    composition: Arc<Mutex<AffairsComposition>>,
    next_request: Arc<AtomicU64>,
}

impl WebState {
    fn new(composition: Arc<Mutex<AffairsComposition>>) -> Self {
        Self {
            composition,
            next_request: Arc::new(AtomicU64::new(1)),
        }
    }

    fn lock(&self) -> Result<MutexGuard<'_, AffairsComposition>, WebRequestError> {
        self.composition
            .lock()
            .map_err(|_| WebRequestError::CompositionUnavailable)
    }

    fn submit(
        &self,
        procedure_id: String,
        as_of: Option<UnixMillis>,
    ) -> Result<ClientResponseDto, WebRequestError> {
        let procedure_id =
            WireText::parse(procedure_id).map_err(|_| WebRequestError::InvalidProcedureId)?;
        let sequence = self
            .next_request
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| WebRequestError::CounterExhausted)?;
        let request_nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| WebRequestError::InternalIdentity)?
            .as_nanos();
        let payload_digest = affairs_get_payload_digest(&procedure_id, as_of)
            .map_err(|_| WebRequestError::InternalIdentity)?;
        let request = SubmitAffairsGetDto {
            request_id: checked_text(format!("req:web:{request_nonce}:{sequence}"))?,
            correlation_id: checked_text(format!("corr:web:{request_nonce}:{sequence}"))?,
            causation_id: None,
            idempotency_key: None,
            actor: ActorIntentDto::Public,
            provenance: ClientProvenanceDto {
                build: checked_text(concat!("ustc-agentd/", env!("CARGO_PKG_VERSION")))?,
                target: checked_text("web-loopback")?,
                protocol: checked_text("http-json-v1")?,
            },
            payload_digest,
            procedure_id,
            as_of,
        };
        let composition = self.lock()?;
        let submitted = composition.handle_submit(&request);
        self.resolve_public_available(&composition, submitted)
    }

    fn submit_change(&self, board_id: String) -> Result<ClientResponseDto, WebRequestError> {
        let board_id = WireText::parse(board_id).map_err(|_| WebRequestError::InvalidBoardId)?;
        let sequence = self
            .next_request
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| WebRequestError::CounterExhausted)?;
        let request_nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| WebRequestError::InternalIdentity)?
            .as_nanos();
        let payload_digest =
            change_feed_payload_digest(&board_id).map_err(|_| WebRequestError::InternalIdentity)?;
        let request = SubmitChangeFeedDto {
            request_id: checked_text(format!("req:web:change:{request_nonce}:{sequence}"))?,
            correlation_id: checked_text(format!("corr:web:change:{request_nonce}:{sequence}"))?,
            causation_id: None,
            idempotency_key: None,
            actor: ActorIntentDto::Public,
            provenance: ClientProvenanceDto {
                build: checked_text(concat!("ustc-agentd/", env!("CARGO_PKG_VERSION")))?,
                target: checked_text("web-loopback")?,
                protocol: checked_text("http-json-v1")?,
            },
            payload_digest,
            board_id,
        };
        Ok(self.lock()?.handle_change_submit(&request))
    }

    fn submit_opportunity(
        &self,
        command: OpportunityCommandDto,
    ) -> Result<ClientResponseDto, WebRequestError> {
        self.submit_opportunity_with_identity(command, None)
    }

    fn submit_opportunity_with_identity(
        &self,
        command: OpportunityCommandDto,
        identity: Option<OpportunityCallerIdentity>,
    ) -> Result<ClientResponseDto, WebRequestError> {
        command
            .validate()
            .map_err(|_| WebRequestError::InvalidOpportunityRequest)?;
        let (request_id, correlation_id, idempotency_key) = match identity {
            Some(identity) => (
                identity.request_id,
                identity.correlation_id,
                identity.idempotency_key,
            ),
            None => {
                let sequence = self
                    .next_request
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                        current.checked_add(1)
                    })
                    .map_err(|_| WebRequestError::CounterExhausted)?;
                let request_nonce = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| WebRequestError::InternalIdentity)?
                    .as_nanos();
                (
                    checked_text(format!("req:web:opportunity:{request_nonce}:{sequence}"))?,
                    checked_text(format!("corr:web:opportunity:{request_nonce}:{sequence}"))?,
                    checked_text(format!("idem:web:opportunity:{request_nonce}:{sequence}"))?,
                )
            }
        };
        let payload_digest =
            opportunity_payload_digest(&command).map_err(|_| WebRequestError::InternalIdentity)?;
        let composition = self.lock()?;
        let request = SubmitOpportunityDto {
            request_id,
            correlation_id,
            causation_id: None,
            idempotency_key: Some(idempotency_key),
            actor: ActorIntentDto::Authenticated {
                session_id: checked_text(composition.fixture.session.session_id().as_str())?,
            },
            provenance: ClientProvenanceDto {
                build: checked_text(concat!("ustc-agentd/", env!("CARGO_PKG_VERSION")))?,
                target: checked_text("web-loopback-private-demo")?,
                protocol: checked_text("http-json-v1")?,
            },
            payload_digest,
            command,
        };
        Ok(composition.handle_opportunity_submit(&request))
    }

    fn resolve_public_available(
        &self,
        composition: &AffairsComposition,
        submitted: ClientResponseDto,
    ) -> Result<ClientResponseDto, WebRequestError> {
        match submitted {
            ClientResponseDto::Accepted {
                command_id,
                public_capability: Some(capability),
                ..
            } => {
                let viewer = ViewerAuthorizationDto::PublicCapability { capability };
                let lookup = composition.handle_lookup(command_id.as_str(), &viewer);
                Self::admit_public_available(lookup)
            }
            ClientResponseDto::Accepted {
                public_capability: None,
                ..
            } => Err(WebRequestError::MissingPublicCapability),
            response @ (ClientResponseDto::Error { .. } | ClientResponseDto::Unavailable) => {
                Ok(response)
            }
            _ => Err(WebRequestError::UnexpectedSubmitResponse),
        }
    }

    fn admit_public_available(
        response: ClientResponseDto,
    ) -> Result<ClientResponseDto, WebRequestError> {
        match response {
            response @ ClientResponseDto::Available {
                redaction: RedactionDto::Public,
                ..
            } => Ok(response),
            _ => Err(WebRequestError::UnexpectedLookupResponse),
        }
    }

    fn publication_status(&self) -> Result<AffairsPublicationStatusEnvelope, WebRequestError> {
        let composition = self.lock()?;
        Ok(AffairsPublicationStatusEnvelope {
            schema: "ustc-affairs-publication-status/v1",
            publication_revision: composition.current_publication_revision(),
            publication_receipt_id: composition.publication_receipt_id().to_owned(),
            control_evidence_event_count: composition.control_evidence_event_count(),
        })
    }

    fn publish_demo(
        &self,
    ) -> Result<(StatusCode, AffairsPublicationResponseEnvelope), WebRequestError> {
        let mut composition = self.lock()?;
        let (status, outcome) = match composition.publish_demo_as_administrator() {
            AffairsPublicationOutcome::Published(receipt) => (
                StatusCode::OK,
                AffairsPublicationResponseKind::Published {
                    receipt_id: receipt.receipt_id().as_str().to_owned(),
                    expected_publication_revision: receipt.expected_publication_revision(),
                    publication_revision: receipt.publication_revision(),
                },
            ),
            AffairsPublicationOutcome::Rejected(_) => (
                StatusCode::FORBIDDEN,
                AffairsPublicationResponseKind::Rejected {
                    error: "m00_admission_denied",
                },
            ),
            AffairsPublicationOutcome::Incomplete { .. } => (
                StatusCode::CONFLICT,
                AffairsPublicationResponseKind::Rejected {
                    error: "m00_session_incomplete",
                },
            ),
            AffairsPublicationOutcome::MalformedCommand => (
                StatusCode::BAD_REQUEST,
                AffairsPublicationResponseKind::Rejected {
                    error: "malformed_publication_command",
                },
            ),
            AffairsPublicationOutcome::EvidenceRejected(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                AffairsPublicationResponseKind::Rejected {
                    error: "control_evidence_unavailable",
                },
            ),
            AffairsPublicationOutcome::PublicationRejected(
                AffairsPublicationApplicationError::Denied,
            ) => (
                StatusCode::CONFLICT,
                AffairsPublicationResponseKind::Rejected {
                    error: "publication_denied",
                },
            ),
            AffairsPublicationOutcome::PublicationRejected(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                AffairsPublicationResponseKind::Rejected {
                    error: "publication_unavailable",
                },
            ),
            AffairsPublicationOutcome::InternalInvariant => (
                StatusCode::INTERNAL_SERVER_ERROR,
                AffairsPublicationResponseKind::Rejected {
                    error: "internal_publication_invariant",
                },
            ),
        };
        Ok((
            status,
            AffairsPublicationResponseEnvelope {
                schema: "ustc-affairs-publication-response/v1",
                outcome,
            },
        ))
    }

    fn change_publication_status(
        &self,
    ) -> Result<ChangePublicationStatusEnvelope, WebRequestError> {
        let composition = self.lock()?;
        let (review_count, publication_count) = composition
            .change_publication_counts()
            .map_err(|_| WebRequestError::CompositionUnavailable)?;
        Ok(ChangePublicationStatusEnvelope {
            schema: "ustc-change-publication-status/v1",
            review_count,
            publication_count,
            publication_receipt_id: composition
                .change_publication_receipt_id()
                .map_err(|_| WebRequestError::CompositionUnavailable)?
                .map(str::to_owned),
            control_evidence_event_count: composition.control_evidence_event_count(),
        })
    }

    fn publish_change_demo(
        &self,
    ) -> Result<(StatusCode, ChangePublicationResponseEnvelope), WebRequestError> {
        let mut composition = self.lock()?;
        let (status, outcome) = match composition.publish_change_demo_as_administrator() {
            ChangePublicationOutcome::Published(publication) => (
                StatusCode::OK,
                ChangePublicationResponseKind::Published {
                    receipt_id: publication.receipt_id().as_str().to_owned(),
                    stable_guid: publication.stable_guid().as_str().to_owned(),
                    event_id: publication.event_id().as_str().to_owned(),
                },
            ),
            ChangePublicationOutcome::Rejected(_) => (
                StatusCode::FORBIDDEN,
                ChangePublicationResponseKind::Rejected {
                    error: "m00_admission_denied",
                },
            ),
            ChangePublicationOutcome::Incomplete { .. } => (
                StatusCode::CONFLICT,
                ChangePublicationResponseKind::Rejected {
                    error: "m00_session_incomplete",
                },
            ),
            ChangePublicationOutcome::MalformedCommand => (
                StatusCode::BAD_REQUEST,
                ChangePublicationResponseKind::Rejected {
                    error: "malformed_change_publication_command",
                },
            ),
            ChangePublicationOutcome::EvidenceRejected(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ChangePublicationResponseKind::Rejected {
                    error: "control_evidence_unavailable",
                },
            ),
            ChangePublicationOutcome::PublicationRejected(
                ChangePublicationApplicationError::Denied,
            ) => (
                StatusCode::CONFLICT,
                ChangePublicationResponseKind::Rejected {
                    error: "change_publication_denied",
                },
            ),
            ChangePublicationOutcome::PublicationRejected(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ChangePublicationResponseKind::Rejected {
                    error: "change_publication_unavailable",
                },
            ),
            ChangePublicationOutcome::InternalInvariant => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ChangePublicationResponseKind::Rejected {
                    error: "internal_change_publication_invariant",
                },
            ),
        };
        Ok((
            status,
            ChangePublicationResponseEnvelope {
                schema: "ustc-change-publication-response/v1",
                outcome,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebRequestError {
    InvalidProcedureId,
    InvalidBoardId,
    InvalidOpportunityRequest,
    InvalidOpportunityIdentity,
    CounterExhausted,
    InternalIdentity,
    MissingPublicCapability,
    UnexpectedSubmitResponse,
    UnexpectedLookupResponse,
    CompositionUnavailable,
}

fn checked_text(value: impl Into<String>) -> Result<WireText, WebRequestError> {
    WireText::parse(value).map_err(|_| WebRequestError::InternalIdentity)
}

/// Caller-stable bounded identity for one create/revoke-delete intent. Passed
/// through unchanged so a byte-identical retry keeps the same M00 command
/// digest and recovers the committed terminal instead of minting a conflict.
struct OpportunityCallerIdentity {
    request_id: WireText,
    correlation_id: WireText,
    idempotency_key: WireText,
}

fn parse_caller_identity(
    request_id: &str,
    correlation_id: &str,
    idempotency_key: &str,
) -> Result<OpportunityCallerIdentity, WebRequestError> {
    let parse = |value: &str| {
        WireText::parse(value).map_err(|_| WebRequestError::InvalidOpportunityIdentity)
    };
    Ok(OpportunityCallerIdentity {
        request_id: parse(request_id)?,
        correlation_id: parse(correlation_id)?,
        idempotency_key: parse(idempotency_key)?,
    })
}

/// Caller-supplied timestamp bound: the server never replaces it, so retries
/// keep the command digest stable.
fn bounded_operation_timestamp(value: i64) -> Result<UnixMillis, WebRequestError> {
    if value <= 0 {
        return Err(WebRequestError::InvalidOpportunityIdentity);
    }
    Ok(UnixMillis::new(value))
}

#[derive(Serialize)]
struct WebErrorEnvelope {
    schema: &'static str,
    error: &'static str,
}

#[derive(Serialize)]
struct HealthEnvelope {
    schema: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
struct AffairsPublicationStatusEnvelope {
    schema: &'static str,
    publication_revision: Option<u64>,
    publication_receipt_id: String,
    control_evidence_event_count: usize,
}

#[derive(Serialize)]
struct AffairsPublicationResponseEnvelope {
    schema: &'static str,
    outcome: AffairsPublicationResponseKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum AffairsPublicationResponseKind {
    Published {
        receipt_id: String,
        expected_publication_revision: Option<u64>,
        publication_revision: u64,
    },
    Rejected {
        error: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishAffairsDemoBody {
    confirm_publish: bool,
}

#[derive(Serialize)]
struct ChangePublicationStatusEnvelope {
    schema: &'static str,
    review_count: usize,
    publication_count: usize,
    publication_receipt_id: Option<String>,
    control_evidence_event_count: usize,
}

#[derive(Serialize)]
struct ChangePublicationResponseEnvelope {
    schema: &'static str,
    outcome: ChangePublicationResponseKind,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ChangePublicationResponseKind {
    Published {
        receipt_id: String,
        stable_guid: String,
        event_id: String,
    },
    Rejected {
        error: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublishChangeDemoBody {
    confirm_publish: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateOpportunityProfileBody {
    consent: bool,
    request_id: String,
    correlation_id: String,
    idempotency_key: String,
    consented_at: i64,
    completed_courses: Vec<String>,
    min_credits: u16,
    max_credits: u16,
    preference_weights: Vec<OpportunityPreferenceBody>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpportunityPreferenceBody {
    course_code: String,
    weight: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerateOpportunityPlanBody {
    profile_snapshot_id: String,
    max_results: u16,
    beam_width: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteOpportunityProfileBody {
    confirm_delete: bool,
    request_id: String,
    correlation_id: String,
    idempotency_key: String,
    revoked_at: i64,
}

/// Builds the bounded same-origin Web router over one composition.
pub fn web_router(composition: Arc<Mutex<AffairsComposition>>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/app.js", get(app_js))
        .route("/assets/styles.css", get(styles_css))
        .route("/healthz", get(healthz))
        .route("/api/v1/server/info", get(server_info))
        .route("/api/v1/client/capabilities", get(capability_list))
        .route("/api/v1/affairs/{procedure_id}", get(affairs_get))
        .route(
            "/api/v1/demo/administrator/affairs/publication",
            get(affairs_publication_status).post(affairs_publication_publish),
        )
        .route(
            "/api/v1/demo/administrator/changes/publication",
            get(change_publication_status).post(change_publication_publish),
        )
        .route("/api/v1/changes/{board_id}", get(change_feed_get))
        .route("/api/v1/changes/{board_id}/atom", get(change_feed_atom))
        .route(
            "/api/v1/opportunity/profiles",
            post(opportunity_profile_create),
        )
        .route(
            "/api/v1/opportunity/profiles/{profile_id}",
            get(opportunity_profile_view),
        )
        .route("/api/v1/opportunity/plans", post(opportunity_plan_generate))
        .route(
            "/api/v1/opportunity/profiles/{profile_id}/revoke-delete",
            post(opportunity_profile_delete),
        )
        .layer(DefaultBodyLimit::max(16 * 1024))
        .with_state(WebState::new(composition))
}

impl AffairsComposition {
    /// Serves the bounded three-plugin HTTP/Web demonstration on a loopback address.
    ///
    /// # Errors
    ///
    /// Rejects non-loopback addresses and reports listener/server failures.
    pub async fn serve_web(self, bind_addr: &str) -> Result<(), String> {
        let socket_addr = parse_loopback_socket_addr(bind_addr)?;
        let listener = tokio::net::TcpListener::bind(socket_addr)
            .await
            .map_err(|error| format!("web bind failed: {error}"))?;
        let local_addr = listener
            .local_addr()
            .map_err(|error| format!("web local_addr failed: {error}"))?;
        println!("web listening http://{local_addr}");
        use std::io::Write as _;
        std::io::stdout()
            .flush()
            .map_err(|error| format!("stdout flush failed: {error}"))?;
        axum::serve(listener, web_router(Arc::new(Mutex::new(self))))
            .await
            .map_err(|error| format!("web serve failed: {error}"))
    }
}

async fn index() -> Response {
    static_response(INDEX_HTML, "text/html; charset=utf-8")
}

async fn app_js() -> Response {
    static_response(APP_JS, "text/javascript; charset=utf-8")
}

async fn styles_css() -> Response {
    static_response(STYLES_CSS, "text/css; charset=utf-8")
}

async fn healthz() -> Response {
    hardened(
        Json(HealthEnvelope {
            schema: "ustc-agentd-health/v1",
            status: "ok",
        })
        .into_response(),
    )
}

async fn server_info() -> Response {
    typed_json_response(
        StatusCode::OK,
        ClientResponseDto::ServerInfo {
            info: ServerInfoDto::new(
                checked_text(concat!("ustc-agentd/", env!("CARGO_PKG_VERSION")))
                    .expect("static server build is valid wire text"),
            ),
        },
    )
}

async fn capability_list(headers: HeaderMap) -> Response {
    match dispatch_with_protocol_major(presented_protocol_major(&headers), || {
        ClientResponseDto::Capabilities {
            capabilities: CapabilityListDto::affairs_first(),
        }
    }) {
        Ok(response) => typed_json_response(StatusCode::OK, response),
        Err(compatibility) => compatibility_response(compatibility),
    }
}

async fn affairs_get(
    AxumPath(procedure_id): AxumPath<String>,
    State(state): State<WebState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let as_of = match parse_affairs_as_of(&uri) {
        Ok(as_of) => as_of,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_affairs_query"),
    };
    match dispatch_with_protocol_major(presented_protocol_major(&headers), || {
        state.submit(procedure_id, as_of)
    }) {
        Err(compatibility) => compatibility_response(compatibility),
        Ok(Ok(response)) => typed_json_response(affairs_response_status(&response), response),
        Ok(Err(WebRequestError::InvalidProcedureId)) => {
            web_error(StatusCode::BAD_REQUEST, "invalid_procedure_id")
        }
        Ok(Err(WebRequestError::CounterExhausted | WebRequestError::InternalIdentity)) => {
            web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error")
        }
        Ok(Err(WebRequestError::UnexpectedSubmitResponse)) => {
            web_error(StatusCode::BAD_GATEWAY, "public_submit_unavailable")
        }
        Ok(Err(
            WebRequestError::MissingPublicCapability | WebRequestError::UnexpectedLookupResponse,
        )) => web_error(StatusCode::BAD_GATEWAY, "public_lookup_unavailable"),
        Ok(Err(WebRequestError::InvalidBoardId)) => {
            web_error(StatusCode::BAD_REQUEST, "invalid_board_id")
        }
        Ok(Err(WebRequestError::InvalidOpportunityRequest)) => {
            web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error")
        }
        Ok(Err(WebRequestError::InvalidOpportunityIdentity)) => {
            web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_identity")
        }
        Ok(Err(WebRequestError::CompositionUnavailable)) => {
            web_error(StatusCode::SERVICE_UNAVAILABLE, "composition_unavailable")
        }
    }
}

fn parse_affairs_as_of(uri: &Uri) -> Result<Option<UnixMillis>, ()> {
    let Some(query) = uri.query() else {
        return Ok(None);
    };
    let mut fields = query.split('&');
    let field = fields.next().ok_or(())?;
    if fields.next().is_some() {
        return Err(());
    }
    let (key, value) = field.split_once('=').ok_or(())?;
    if key != "as_of" || value.is_empty() || value.contains('=') {
        return Err(());
    }
    value
        .parse::<i64>()
        .map(UnixMillis::new)
        .map(Some)
        .map_err(|_| ())
}

fn presented_protocol_major(headers: &HeaderMap) -> Option<ClientProtocolMajor> {
    let mut values = headers.get_all(CLIENT_PROTOCOL_MAJOR_HEADER).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }
    let value = value.to_str().ok()?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse::<u16>().ok().map(ClientProtocolMajor::new)
}

fn compatibility_response(compatibility: ProtocolCompatibilityDto) -> Response {
    let status = match compatibility {
        ProtocolCompatibilityDto::UpgradeRequired { .. } => StatusCode::UPGRADE_REQUIRED,
        ProtocolCompatibilityDto::IncompatibleProtocol { .. } => StatusCode::CONFLICT,
    };
    typed_json_response(status, ClientResponseDto::Compatibility { compatibility })
}

fn affairs_response_status(response: &ClientResponseDto) -> StatusCode {
    match response {
        ClientResponseDto::Available { .. } => StatusCode::OK,
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } if error.class == WireErrorClassDto::PolicyDenied => StatusCode::FORBIDDEN,
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } if error.class == WireErrorClassDto::MalformedCommand => StatusCode::BAD_REQUEST,
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { .. },
        }
        | ClientResponseDto::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::BAD_GATEWAY,
    }
}

async fn affairs_publication_status(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !administrator_demo_header_authorized(&headers) {
        return web_error(
            StatusCode::FORBIDDEN,
            "administrator_demo_confirmation_required",
        );
    }
    match state.publication_status() {
        Ok(status) => typed_json_response(StatusCode::OK, status),
        Err(_) => web_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "publication_status_unavailable",
        ),
    }
}

async fn affairs_publication_publish(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<PublishAffairsDemoBody>, JsonRejection>,
) -> Response {
    if !administrator_demo_header_authorized(&headers) {
        return web_error(
            StatusCode::FORBIDDEN,
            "administrator_demo_confirmation_required",
        );
    }
    let Json(body) = match body {
        Ok(body) => body,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_publication_json"),
    };
    if !body.confirm_publish {
        return web_error(
            StatusCode::BAD_REQUEST,
            "explicit_publish_confirmation_required",
        );
    }
    match state.publish_demo() {
        Ok((status, envelope)) => typed_json_response(status, envelope),
        Err(_) => web_error(StatusCode::SERVICE_UNAVAILABLE, "publication_unavailable"),
    }
}

fn administrator_demo_header_authorized(headers: &HeaderMap) -> bool {
    headers
        .get(ADMINISTRATOR_DEMO_HEADER)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == ADMINISTRATOR_DEMO_CONFIRMATION)
}

async fn change_publication_status(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if !administrator_demo_header_authorized(&headers) {
        return web_error(
            StatusCode::FORBIDDEN,
            "administrator_demo_confirmation_required",
        );
    }
    match state.change_publication_status() {
        Ok(status) => typed_json_response(StatusCode::OK, status),
        Err(_) => web_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "change_publication_status_unavailable",
        ),
    }
}

async fn change_publication_publish(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<PublishChangeDemoBody>, JsonRejection>,
) -> Response {
    if !administrator_demo_header_authorized(&headers) {
        return web_error(
            StatusCode::FORBIDDEN,
            "administrator_demo_confirmation_required",
        );
    }
    let Json(body) = match body {
        Ok(body) => body,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_change_publication_json"),
    };
    if !body.confirm_publish {
        return web_error(
            StatusCode::BAD_REQUEST,
            "explicit_publish_confirmation_required",
        );
    }
    match state.publish_change_demo() {
        Ok((status, envelope)) => typed_json_response(status, envelope),
        Err(_) => web_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "change_publication_unavailable",
        ),
    }
}

async fn change_feed_get(
    AxumPath(board_id): AxumPath<String>,
    State(state): State<WebState>,
) -> Response {
    match state.submit_change(board_id) {
        Ok(response) => {
            let status = match &response {
                ClientResponseDto::ChangeFeedAccepted { .. } => StatusCode::OK,
                ClientResponseDto::Error {
                    error: ClientErrorDto::Admission { error },
                } if error.class == WireErrorClassDto::PolicyDenied => StatusCode::FORBIDDEN,
                ClientResponseDto::Error {
                    error: ClientErrorDto::Admission { error },
                } if error.class == WireErrorClassDto::MalformedCommand => StatusCode::BAD_REQUEST,
                ClientResponseDto::Error {
                    error: ClientErrorDto::Infrastructure { .. },
                }
                | ClientResponseDto::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::BAD_GATEWAY,
            };
            typed_json_response(status, response)
        }
        Err(WebRequestError::InvalidBoardId) => {
            web_error(StatusCode::BAD_REQUEST, "invalid_board_id")
        }
        Err(WebRequestError::CounterExhausted | WebRequestError::InternalIdentity) => {
            web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error")
        }
        Err(_) => web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error"),
    }
}

async fn change_feed_atom(
    AxumPath(board_id): AxumPath<String>,
    State(state): State<WebState>,
) -> Response {
    let response = match state.submit_change(board_id) {
        Ok(value) => value,
        Err(WebRequestError::InvalidBoardId) => {
            return web_error(StatusCode::BAD_REQUEST, "invalid_board_id");
        }
        Err(_) => return web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error"),
    };
    let terminal = match response {
        ClientResponseDto::ChangeFeedAccepted { terminal, .. } => terminal,
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } if error.class == WireErrorClassDto::PolicyDenied => {
            return web_error(StatusCode::FORBIDDEN, "change_feed_policy_denied");
        }
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } if error.class == WireErrorClassDto::MalformedCommand => {
            return web_error(StatusCode::BAD_REQUEST, "change_feed_malformed");
        }
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { .. },
        }
        | ClientResponseDto::Unavailable => {
            return web_error(StatusCode::SERVICE_UNAVAILABLE, "change_feed_unavailable");
        }
        _ => return web_error(StatusCode::BAD_GATEWAY, "change_feed_unavailable"),
    };
    match terminal.outcome() {
        ustc_campus_agent_client_protocol::M70ChangeFeedOutcomeDto::Found { view } => {
            dynamic_response(
                view.atom().to_owned(),
                "application/atom+xml; charset=utf-8",
            )
        }
        ustc_campus_agent_client_protocol::M70ChangeFeedOutcomeDto::NotFound { .. } => {
            web_error(StatusCode::NOT_FOUND, "change_board_not_found")
        }
    }
}

async fn opportunity_profile_create(
    State(state): State<WebState>,
    body: Result<Json<CreateOpportunityProfileBody>, JsonRejection>,
) -> Response {
    let Json(body) = match body {
        Ok(body) => body,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_json"),
    };
    if !body.consent {
        return web_error(StatusCode::BAD_REQUEST, "explicit_consent_required");
    }
    let completed_courses = match body
        .completed_courses
        .into_iter()
        .map(checked_text)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(courses) => courses,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_profile"),
    };
    let preference_weights = match body
        .preference_weights
        .into_iter()
        .map(|preference| {
            checked_text(preference.course_code).map(|course_code| OpportunityPreferenceDto {
                course_code,
                weight: preference.weight,
            })
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(preferences) => preferences,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_profile"),
    };
    let consented_at = match bounded_operation_timestamp(body.consented_at) {
        Ok(value) => value,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_identity"),
    };
    let consent_purpose = match checked_text("opportunity_planning") {
        Ok(value) => value,
        Err(_) => return web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error"),
    };
    let result = parse_caller_identity(
        &body.request_id,
        &body.correlation_id,
        &body.idempotency_key,
    )
    .and_then(|identity| {
        state.submit_opportunity_with_identity(
            OpportunityCommandDto::CreateProfile {
                consent_purpose,
                consent_fields: vec![
                    OpportunityConsentFieldDto::CompletedCourses,
                    OpportunityConsentFieldDto::CreditBounds,
                    OpportunityConsentFieldDto::PreferenceWeights,
                ],
                consented_at,
                completed_courses,
                min_credits: body.min_credits,
                max_credits: body.max_credits,
                preference_weights,
            },
            Some(identity),
        )
    });
    opportunity_response(result)
}

async fn opportunity_profile_view(
    AxumPath(profile_id): AxumPath<String>,
    State(state): State<WebState>,
) -> Response {
    let profile_snapshot_id = match checked_text(profile_id) {
        Ok(value) => value,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_profile_snapshot_id"),
    };
    opportunity_response(
        state.submit_opportunity(OpportunityCommandDto::ViewProfile {
            profile_snapshot_id,
        }),
    )
}

async fn opportunity_plan_generate(
    State(state): State<WebState>,
    body: Result<Json<GenerateOpportunityPlanBody>, JsonRejection>,
) -> Response {
    let Json(body) = match body {
        Ok(body) => body,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_json"),
    };
    let profile_snapshot_id = match checked_text(body.profile_snapshot_id) {
        Ok(value) => value,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_profile_snapshot_id"),
    };
    opportunity_response(
        state.submit_opportunity(OpportunityCommandDto::GeneratePlan {
            profile_snapshot_id,
            max_results: body.max_results,
            beam_width: body.beam_width,
        }),
    )
}

async fn opportunity_profile_delete(
    AxumPath(profile_id): AxumPath<String>,
    State(state): State<WebState>,
    body: Result<Json<DeleteOpportunityProfileBody>, JsonRejection>,
) -> Response {
    let Json(body) = match body {
        Ok(body) => body,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_json"),
    };
    if !body.confirm_delete {
        return web_error(
            StatusCode::BAD_REQUEST,
            "explicit_delete_confirmation_required",
        );
    }
    let profile_snapshot_id = match checked_text(profile_id) {
        Ok(value) => value,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_profile_snapshot_id"),
    };
    let revoked_at = match bounded_operation_timestamp(body.revoked_at) {
        Ok(value) => value,
        Err(_) => return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_identity"),
    };
    let result = parse_caller_identity(
        &body.request_id,
        &body.correlation_id,
        &body.idempotency_key,
    )
    .and_then(|identity| {
        state.submit_opportunity_with_identity(
            OpportunityCommandDto::RevokeConsentAndDeleteProfile {
                profile_snapshot_id,
                revoked_at,
            },
            Some(identity),
        )
    });
    opportunity_response(result)
}

fn opportunity_response(response: Result<ClientResponseDto, WebRequestError>) -> Response {
    let response = match response {
        Ok(response) => response,
        Err(WebRequestError::InvalidOpportunityRequest) => {
            return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_request");
        }
        Err(WebRequestError::InvalidOpportunityIdentity) => {
            return web_error(StatusCode::BAD_REQUEST, "invalid_opportunity_identity");
        }
        Err(_) => return web_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_request_error"),
    };
    let status = match &response {
        ClientResponseDto::OpportunityAccepted { terminal, .. } => match terminal.as_ref() {
            ustc_campus_agent_client_protocol::M72OpportunityTerminalDto::ProfileCreated {
                ..
            } => StatusCode::CREATED,
            _ => StatusCode::OK,
        },
        ClientResponseDto::OpportunityRejected { rejection, .. } => match rejection {
            OpportunityRejectionDto::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            OpportunityRejectionDto::AccessDenied => StatusCode::FORBIDDEN,
            OpportunityRejectionDto::MissingProfile => StatusCode::NOT_FOUND,
            OpportunityRejectionDto::ProfileDeleted => StatusCode::GONE,
            OpportunityRejectionDto::ProfileAlreadyExists => StatusCode::CONFLICT,
            OpportunityRejectionDto::DeleteBeforeConsent => StatusCode::UNPROCESSABLE_ENTITY,
            OpportunityRejectionDto::InvalidProfileFacts => StatusCode::UNPROCESSABLE_ENTITY,
            OpportunityRejectionDto::SourceNotCurrent { .. } => StatusCode::CONFLICT,
            OpportunityRejectionDto::SourceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        },
        ClientResponseDto::Incomplete { .. } => StatusCode::ACCEPTED,
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } if error.class == WireErrorClassDto::PolicyDenied => StatusCode::FORBIDDEN,
        ClientResponseDto::Error {
            error: ClientErrorDto::Admission { error },
        } if error.class == WireErrorClassDto::MalformedCommand => StatusCode::BAD_REQUEST,
        ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { .. },
        }
        | ClientResponseDto::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::BAD_GATEWAY,
    };
    typed_json_response(status, response)
}

fn typed_json_response<T: Serialize>(status: StatusCode, response: T) -> Response {
    let mut response = Json(response).into_response();
    *response.status_mut() = status;
    hardened(response)
}

fn web_error(status: StatusCode, error: &'static str) -> Response {
    hardened(
        (
            status,
            Json(WebErrorEnvelope {
                schema: "ustc-web-error/v1",
                error,
            }),
        )
            .into_response(),
    )
}

fn static_response(body: &'static str, content_type: &'static str) -> Response {
    dynamic_response(body.to_owned(), content_type)
}

fn dynamic_response(body: String, content_type: &'static str) -> Response {
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    hardened(response)
}

fn hardened(mut response: Response) -> Response {
    let headers: &mut HeaderMap = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::{
        CONTENT_SECURITY_POLICY, WebRequestError, WebState, affairs_response_status,
        static_response,
    };
    use axum::http::{StatusCode, header};
    use ustc_campus_agent_client_protocol::{
        ClientErrorDto, ClientResponseDto, EchoPayloadDto, M10WireErrorDto, RetryabilityDto,
        WireErrorClassDto, WireText,
    };

    fn wire(value: &str) -> WireText {
        WireText::parse(value).expect("valid wire text")
    }

    #[test]
    fn web_boundary_rejects_every_non_available_lookup_result() {
        let result = WebState::admit_public_available(ClientResponseDto::Unavailable);
        assert!(matches!(
            result,
            Err(WebRequestError::UnexpectedLookupResponse)
        ));
    }

    #[test]
    fn affairs_http_status_mapping_covers_malformed_unavailable_and_unexpected() {
        let malformed = ClientResponseDto::Error {
            error: ClientErrorDto::Admission {
                error: M10WireErrorDto::try_new(
                    WireErrorClassDto::MalformedCommand,
                    RetryabilityDto::RetryableAfterChange,
                    wire("malformed_command"),
                    EchoPayloadDto::None,
                )
                .expect("valid malformed-command relation"),
            },
        };
        let infrastructure = ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure {
                retryable: true,
                wire_code: wire("fixture_unavailable"),
            },
        };
        let unexpected = ClientResponseDto::Error {
            error: ClientErrorDto::InternalInvariant {
                wire_code: wire("fixture_internal"),
            },
        };

        assert_eq!(affairs_response_status(&malformed), StatusCode::BAD_REQUEST);
        assert_eq!(
            affairs_response_status(&ClientResponseDto::Unavailable),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            affairs_response_status(&infrastructure),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            affairs_response_status(&unexpected),
            StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn static_response_has_no_store_and_security_headers() {
        let response = static_response("ok", "text/plain; charset=utf-8");
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("no-store")
        );
        assert_eq!(
            response
                .headers()
                .get("content-security-policy")
                .and_then(|value| value.to_str().ok()),
            Some(CONTENT_SECURITY_POLICY)
        );
        assert_eq!(
            response
                .headers()
                .get("x-content-type-options")
                .and_then(|value| value.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            response
                .headers()
                .get("x-frame-options")
                .and_then(|value| value.to_str().ok()),
            Some("DENY")
        );
    }
}
