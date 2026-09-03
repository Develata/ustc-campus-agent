//! plan_ref: docs/plan/modules/60-model-provider-integration.md#121-bounded-chat--first-party-plugin-implementation-slice
//!
//! Composition adapter for the bounded chat slice. Model/tool-loop mechanics
//! remain owned by M30, provider HTTP remains in the M50 adapter, and this
//! module reaches Affairs and M72 only through the existing typed `WebState`
//! application paths.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde::de::DeserializeOwned;
use ustc_campus_agent_adapters::OpenAiResponsesProvider;
use ustc_campus_agent_client_protocol::{
    ClientErrorDto, ClientResponseDto, M72OpportunityTerminalDto, OpportunityCommandDto,
    OpportunityConfirmationDto, OpportunityConsentFieldDto, OpportunityPreferenceDto,
    OpportunityRejectionDto, RedactionDto, UnixMillis, WireText,
};
use ustc_campus_agent_runtime::chat::{
    BoundedChatEngine, ChatError, ChatRequest, ChatResult, ChatToolError, ChatToolPort,
    ModelToolCall, ToolExecutionOutput, USTC_AFFAIRS_LOOKUP_TOOL, USTC_COURSE_ADVICE_TOOL,
};

use crate::web::WebState;

const COURSE_PLAN_MAX_RESULTS: u16 = 3;
const COURSE_PLAN_BEAM_WIDTH: u16 = 1_024;

pub(crate) enum ChatService {
    Ready(OpenAiResponsesProvider),
    Unavailable,
    Misconfigured,
}

impl ChatService {
    pub(crate) fn from_env() -> Self {
        match OpenAiResponsesProvider::from_env() {
            Ok(Some(provider)) => Self::Ready(provider),
            Ok(None) => Self::Unavailable,
            Err(_) => Self::Misconfigured,
        }
    }

    pub(crate) async fn run(
        &self,
        state: WebState,
        message: String,
        course_profile_consent: bool,
    ) -> Result<ChatResult, ChatServiceError> {
        let provider = match self {
            Self::Ready(provider) => provider,
            Self::Unavailable => return Err(ChatServiceError::Unavailable),
            Self::Misconfigured => return Err(ChatServiceError::Misconfigured),
        };
        let tools = RequestChatTools {
            applications: ApplicationChatTools::new(state),
            course_profile_consent,
        };
        BoundedChatEngine::new(provider, &tools)
            .chat(ChatRequest { message })
            .await
            .map_err(ChatServiceError::Chat)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatServiceError {
    Unavailable,
    Misconfigured,
    Chat(ChatError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatToolAdapterError {
    InvalidArguments,
    CourseConsentRequired,
    TypedDenial,
    ApplicationUnavailable,
    CleanupFailed,
    UnexpectedTerminal,
    SerializationFailed,
}

impl ChatToolAdapterError {
    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::InvalidArguments => "chat_tool_invalid_arguments",
            Self::CourseConsentRequired => "chat_course_consent_required",
            Self::TypedDenial => "chat_tool_denied",
            Self::ApplicationUnavailable => "chat_tool_unavailable",
            Self::CleanupFailed => "chat_course_cleanup_failed",
            Self::UnexpectedTerminal => "chat_tool_unexpected_terminal",
            Self::SerializationFailed => "chat_tool_serialization_failed",
        }
    }
}

#[derive(Clone)]
pub(crate) struct ApplicationChatTools {
    state: WebState,
    course_sequence: Arc<Mutex<()>>,
}

impl ApplicationChatTools {
    pub(crate) fn new(state: WebState) -> Self {
        Self {
            state,
            course_sequence: Arc::new(Mutex::new(())),
        }
    }

    fn affairs(&self, raw_arguments: &str) -> Result<String, ChatToolAdapterError> {
        let arguments: AffairsToolArguments = parse_exact_arguments(raw_arguments)?;
        let response = self
            .state
            .submit(arguments.procedure_id, None)
            .map_err(|_| ChatToolAdapterError::ApplicationUnavailable)?;
        match response {
            ClientResponseDto::Available {
                terminal,
                redaction: RedactionDto::Public,
                ..
            } => serde_json::to_string(terminal.as_ref())
                .map_err(|_| ChatToolAdapterError::SerializationFailed),
            other => Err(classify_non_success(&other)),
        }
    }

    fn course(
        &self,
        raw_arguments: &str,
        course_profile_consent: bool,
    ) -> Result<String, ChatToolAdapterError> {
        if !course_profile_consent {
            return Err(ChatToolAdapterError::CourseConsentRequired);
        }
        let arguments: CourseToolArguments = parse_exact_arguments(raw_arguments)?;
        let completed_courses = arguments
            .completed_courses
            .into_iter()
            .map(parse_private_wire_text)
            .collect::<Result<Vec<_>, _>>()?;
        let preference_weights = arguments
            .preference_weights
            .into_iter()
            .map(|preference| {
                Ok(OpportunityPreferenceDto {
                    course_code: parse_private_wire_text(preference.course_code)?,
                    weight: preference.weight,
                })
            })
            .collect::<Result<Vec<_>, ChatToolAdapterError>>()?;
        let consented_at = current_unix_millis()?;
        let consent_purpose = parse_private_wire_text("opportunity_planning".to_owned())?;
        let create = OpportunityCommandDto::CreateProfile {
            consent_purpose,
            consent_fields: vec![
                OpportunityConsentFieldDto::CompletedCourses,
                OpportunityConsentFieldDto::CreditBounds,
                OpportunityConsentFieldDto::PreferenceWeights,
            ],
            consented_at,
            completed_courses,
            min_credits: arguments.min_credits,
            max_credits: arguments.max_credits,
            preference_weights,
        };
        create
            .validate()
            .map_err(|_| ChatToolAdapterError::InvalidArguments)?;

        // M72 currently owns one active profile per admitted principal. Keep its
        // create -> plan -> revoke/delete sequence indivisible relative to other
        // chat requests while each typed application operation retains its own
        // internal composition lock.
        let _sequence = self
            .course_sequence
            .lock()
            .map_err(|_| ChatToolAdapterError::ApplicationUnavailable)?;
        let created = self
            .state
            .submit_opportunity(create, OpportunityConfirmationDto::Confirmed)
            .map_err(|_| ChatToolAdapterError::ApplicationUnavailable)?;
        let profile_snapshot_id = match created {
            ClientResponseDto::OpportunityAccepted { terminal, .. } => match *terminal {
                M72OpportunityTerminalDto::ProfileCreated { profile } => {
                    profile.profile_snapshot_id
                }
                _ => return Err(ChatToolAdapterError::UnexpectedTerminal),
            },
            other => return Err(classify_non_success(&other)),
        };

        let planned = self.state.submit_opportunity(
            OpportunityCommandDto::GeneratePlan {
                profile_snapshot_id: profile_snapshot_id.clone(),
                max_results: COURSE_PLAN_MAX_RESULTS,
                beam_width: COURSE_PLAN_BEAM_WIDTH,
            },
            OpportunityConfirmationDto::Confirmed,
        );
        let projected_plan = match planned {
            Ok(ClientResponseDto::OpportunityAccepted { terminal, .. }) => {
                match terminal.as_ref() {
                    M72OpportunityTerminalDto::PlanGenerated { plan }
                        if plan.profile_snapshot_id == profile_snapshot_id =>
                    {
                        serde_json::to_string(terminal.as_ref())
                            .map_err(|_| ChatToolAdapterError::SerializationFailed)
                    }
                    _ => Err(ChatToolAdapterError::UnexpectedTerminal),
                }
            }
            Ok(other) => Err(classify_non_success(&other)),
            Err(_) => Err(ChatToolAdapterError::ApplicationUnavailable),
        };

        // Once creation succeeds, cleanup is mandatory even when planning or
        // projection failed. A cleanup failure dominates the earlier outcome.
        let cleanup = self.state.submit_opportunity(
            OpportunityCommandDto::RevokeConsentAndDeleteProfile {
                profile_snapshot_id: profile_snapshot_id.clone(),
                revoked_at: current_unix_millis_at_least(consented_at)?,
            },
            OpportunityConfirmationDto::Confirmed,
        );
        match cleanup {
            Ok(ClientResponseDto::OpportunityAccepted { terminal, .. }) => {
                match terminal.as_ref() {
                    M72OpportunityTerminalDto::ProfileDeleted { deletion }
                        if deletion.profile_snapshot_id == profile_snapshot_id => {}
                    _ => return Err(ChatToolAdapterError::CleanupFailed),
                }
            }
            Ok(_) | Err(_) => return Err(ChatToolAdapterError::CleanupFailed),
        }
        projected_plan
    }
}

struct RequestChatTools {
    applications: ApplicationChatTools,
    course_profile_consent: bool,
}

impl ChatToolPort for RequestChatTools {
    fn execute(&self, call: &ModelToolCall) -> Result<ToolExecutionOutput, ChatToolError> {
        let output_json = match call.name.as_str() {
            USTC_AFFAIRS_LOOKUP_TOOL => self.applications.affairs(&call.arguments_json),
            USTC_COURSE_ADVICE_TOOL => self
                .applications
                .course(&call.arguments_json, self.course_profile_consent),
            _ => return Err(ChatToolError::Denied),
        }
        .map_err(map_tool_error)?;
        Ok(ToolExecutionOutput { output_json })
    }
}

fn map_tool_error(error: ChatToolAdapterError) -> ChatToolError {
    match error {
        ChatToolAdapterError::InvalidArguments => ChatToolError::MalformedArguments,
        ChatToolAdapterError::CourseConsentRequired | ChatToolAdapterError::TypedDenial => {
            ChatToolError::Denied
        }
        ChatToolAdapterError::ApplicationUnavailable => ChatToolError::Unavailable,
        ChatToolAdapterError::CleanupFailed
        | ChatToolAdapterError::UnexpectedTerminal
        | ChatToolAdapterError::SerializationFailed => ChatToolError::Failed,
    }
}

fn parse_exact_arguments<T: DeserializeOwned>(
    raw_arguments: &str,
) -> Result<T, ChatToolAdapterError> {
    let mut deserializer = serde_json::Deserializer::from_str(raw_arguments);
    let value =
        T::deserialize(&mut deserializer).map_err(|_| ChatToolAdapterError::InvalidArguments)?;
    deserializer
        .end()
        .map_err(|_| ChatToolAdapterError::InvalidArguments)?;
    Ok(value)
}

fn parse_private_wire_text(value: String) -> Result<WireText, ChatToolAdapterError> {
    WireText::parse(value).map_err(|_| ChatToolAdapterError::InvalidArguments)
}

fn current_unix_millis() -> Result<UnixMillis, ChatToolAdapterError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ChatToolAdapterError::ApplicationUnavailable)?
        .as_millis();
    let millis = i64::try_from(millis).map_err(|_| ChatToolAdapterError::ApplicationUnavailable)?;
    if millis <= 0 {
        return Err(ChatToolAdapterError::ApplicationUnavailable);
    }
    Ok(UnixMillis::new(millis))
}

fn current_unix_millis_at_least(minimum: UnixMillis) -> Result<UnixMillis, ChatToolAdapterError> {
    let current = current_unix_millis()?;
    Ok(UnixMillis::new(current.get().max(minimum.get())))
}

fn classify_non_success(response: &ClientResponseDto) -> ChatToolAdapterError {
    match response {
        ClientResponseDto::OpportunityRejected {
            rejection:
                OpportunityRejectionDto::AuthenticationRequired
                | OpportunityRejectionDto::AccessDenied
                | OpportunityRejectionDto::MissingProfile
                | OpportunityRejectionDto::ProfileDeleted
                | OpportunityRejectionDto::ProfileAlreadyExists
                | OpportunityRejectionDto::DeleteBeforeConsent
                | OpportunityRejectionDto::InvalidProfileFacts
                | OpportunityRejectionDto::SourceNotCurrent { .. },
            ..
        }
        | ClientResponseDto::Error {
            error: ClientErrorDto::Admission { .. },
        } => ChatToolAdapterError::TypedDenial,
        ClientResponseDto::OpportunityRejected {
            rejection: OpportunityRejectionDto::SourceUnavailable,
            ..
        }
        | ClientResponseDto::Error {
            error: ClientErrorDto::Infrastructure { .. },
        }
        | ClientResponseDto::Unavailable => ChatToolAdapterError::ApplicationUnavailable,
        ClientResponseDto::Error {
            error: ClientErrorDto::InternalInvariant { .. },
        }
        | ClientResponseDto::Incomplete { .. } => ChatToolAdapterError::ApplicationUnavailable,
        ClientResponseDto::ServerInfo { .. }
        | ClientResponseDto::Capabilities { .. }
        | ClientResponseDto::Compatibility { .. }
        | ClientResponseDto::Accepted { .. }
        | ClientResponseDto::ChangeFeedAccepted { .. }
        | ClientResponseDto::OpportunityAccepted { .. }
        | ClientResponseDto::Available { .. } => ChatToolAdapterError::UnexpectedTerminal,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AffairsToolArguments {
    procedure_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CourseToolArguments {
    completed_courses: Vec<String>,
    min_credits: u16,
    max_credits: u16,
    preference_weights: Vec<CoursePreferenceArgument>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoursePreferenceArgument {
    course_code: String,
    weight: i32,
}
