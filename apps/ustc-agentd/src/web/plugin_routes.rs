//! Thin protocol intents over the package application service.
use super::*;
use crate::plugin_runtime::{PluginError, PluginRuntime};
use ustc_campus_agent_client_protocol::plugins::{PluginCommandDto, PluginProbeDto};
use ustc_campus_agent_core::identity::{TenantId, UserId};
// Accommodate a 64 KiB decoded Skill plus JSON escapes and the review envelope.
pub(super) const IMPORT_PREVIEW_BODY_LIMIT: usize = 1024 * 1024;
fn application(state: &WebState) -> Result<&PluginRuntime, PluginError> {
    state
        .plugins
        .as_ref()
        .as_ref()
        .map_err(|_| PluginError::Unavailable)
}
fn owner(state: &WebState, headers: &HeaderMap) -> Result<(TenantId, UserId), PluginError> {
    account_routes::owner(state, headers).map_err(|_| PluginError::Denied)
}

fn respond<T: Serialize>(value: Result<T, PluginError>) -> Response {
    match value {
        Ok(value) => typed_json_response(StatusCode::OK, value),
        Err(error) => {
            let (status, code) = match error {
                PluginError::InvalidRequest => (StatusCode::BAD_REQUEST, "invalid_plugin_request"),
                PluginError::NotFound => (StatusCode::NOT_FOUND, "plugin_not_found"),
                PluginError::Conflict => (StatusCode::CONFLICT, "plugin_revision_conflict"),
                PluginError::Denied => (StatusCode::FORBIDDEN, "plugin_permission_denied"),
                PluginError::Unsupported => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "plugin_profile_unsupported",
                ),
                PluginError::NotReady => (StatusCode::CONFLICT, "plugin_review_required"),
                PluginError::Capacity => {
                    (StatusCode::TOO_MANY_REQUESTS, "plugin_capacity_exceeded")
                }
                PluginError::Unavailable => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "plugin_runtime_unavailable",
                ),
            };
            typed_json_response(status, json!({"schema":"plugin-error/v1","error":code}))
        }
    }
}
pub(super) async fn list(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if let Err(error) = dispatch_with_protocol_major(presented_protocol_major(&headers), || ()) {
        return compatibility_response(error);
    }
    let result = async {
        let (tenant, user) = owner(&state, &headers)?;
        application(&state)?.list(&tenant, &user).await
    }
    .await;
    respond(result)
}
pub(super) async fn command(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<PluginCommandDto>, JsonRejection>,
) -> Response {
    if let Err(error) = dispatch_with_protocol_major(presented_protocol_major(&headers), || ()) {
        return compatibility_response(error);
    }
    let Ok(Json(request)) = body else {
        return respond::<()>(Err(PluginError::InvalidRequest));
    };
    if !has_application_json_content_type(&headers) {
        return respond::<()>(Err(PluginError::InvalidRequest));
    }
    let result = async {
        let (tenant, user) = owner(&state, &headers)?;
        application(&state)?.command(&tenant, &user, request).await
    }
    .await;
    respond(result)
}
pub(super) async fn probe(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<PluginProbeDto>, JsonRejection>,
) -> Response {
    if let Err(error) = dispatch_with_protocol_major(presented_protocol_major(&headers), || ()) {
        return compatibility_response(error);
    }
    let Ok(Json(request)) = body else {
        return respond::<()>(Err(PluginError::InvalidRequest));
    };
    if !has_application_json_content_type(&headers) {
        return respond::<()>(Err(PluginError::InvalidRequest));
    }
    let result = async {
        let (tenant, user) = owner(&state, &headers)?;
        application(&state)?.probe(&tenant, &user, request).await
    }
    .await;
    respond(result)
}

pub(super) async fn preview_import(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<
        Json<ustc_campus_agent_client_protocol::plugins::PluginImportPreviewDto>,
        JsonRejection,
    >,
) -> Response {
    if let Err(error) = dispatch_with_protocol_major(presented_protocol_major(&headers), || ()) {
        return compatibility_response(error);
    }
    let Ok(Json(request)) = body else {
        return respond::<()>(Err(PluginError::InvalidRequest));
    };
    if !has_application_json_content_type(&headers) {
        return respond::<()>(Err(PluginError::InvalidRequest));
    }
    if let Err(response) = account_routes::owner(&state, &headers) {
        return response;
    }
    respond(application(&state).and_then(|application| application.preview_import(request)))
}

pub(super) async fn update(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<ustc_campus_agent_client_protocol::plugins::PluginUpdateDto>, JsonRejection>,
) -> Response {
    if let Err(error) = dispatch_with_protocol_major(presented_protocol_major(&headers), || ()) {
        return compatibility_response(error);
    }
    let Ok(Json(request)) = body else {
        return respond::<()>(Err(PluginError::InvalidRequest));
    };
    if !has_application_json_content_type(&headers) {
        return respond::<()>(Err(PluginError::InvalidRequest));
    }
    let (tenant, user) = match owner(&state, &headers) {
        Ok(owner) => owner,
        Err(error) => return respond::<()>(Err(error)),
    };
    let result = async { application(&state)?.update(&tenant, &user, request).await }.await;
    respond(result)
}

pub(super) struct WebChatExecutor {
    state: WebState,
    session: Option<crate::plugin_runtime::PluginToolSession>,
}
impl WebChatExecutor {
    pub(super) async fn new(
        mut state: WebState,
        tool_calling: bool,
        owner: (TenantId, UserId),
    ) -> Result<Self, ChatError> {
        state.request_owner = Some(owner.clone());
        if !tool_calling {
            return Ok(Self {
                state,
                session: None,
            });
        }
        let session = match application(&state) {
            Ok(runtime) => match runtime.session(&owner.0, &owner.1).await {
                Ok(session) => Some(session),
                Err(PluginError::Capacity) => return Err(ChatError::CompositionUnavailable),
                Err(_) => None,
            },
            Err(_) => None,
        };
        Ok(Self { state, session })
    }
}
impl crate::chat_tools::ChatToolExecutor for WebChatExecutor {
    fn definitions(&self) -> Vec<crate::chat_tools::ChatDynamicToolDefinition> {
        let mut definitions = self
            .session
            .as_ref()
            .map(|session| session.definitions())
            .unwrap_or_default();
        if let Ok(tools) = crate::source_search::chat_definitions() {
            definitions.extend(tools);
        }
        if let Ok(tool) = crate::source_search::course_chat_definition() {
            definitions.push(tool);
        }
        if let Ok(tool) = crate::calendar_application::batch_chat_definition() {
            definitions.push(tool);
        }
        definitions
    }
    async fn execute(&mut self, request: ChatToolRequest) -> ChatToolExecution {
        if matches!(self.state.accounts.as_ref(), Ok(Some(_))) {
            let admitted = self
                .state
                .execution_headers
                .as_ref()
                .and_then(|headers| account_routes::owner(&self.state, headers).ok());
            if admitted.as_ref() != self.state.request_owner.as_ref() || admitted.is_none() {
                return ChatToolExecution::denied(json!({"code":"account_session_expired"}));
            }
        }
        match request {
            ChatToolRequest::Plugin {
                tool_name,
                arguments,
            } if matches!(
                tool_name.as_str(),
                "plugin_official_source_search" | "plugin_official_source_fetch"
            ) =>
            {
                crate::source_search::execute_chat(&self.state.sources, &tool_name, &arguments)
                    .await
            }
            ChatToolRequest::Plugin {
                tool_name,
                arguments,
            } if tool_name == "plugin_course_plan" => {
                crate::source_search::execute_course_chat(&arguments, self.state.course_consent)
            }
            ChatToolRequest::Plugin {
                tool_name,
                arguments,
            } if tool_name == "plugin_calendar_batch_propose" => {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Drafts {
                    items: Vec<ustc_campus_agent_simple_calendar::CalendarDraft>,
                }
                let drafts = match serde_json::from_value::<Drafts>(arguments) {
                    Ok(v) => v,
                    Err(_) => return calendar_error_execution(CalendarError::InvalidProposal),
                };
                let owner = match self.state.tool_owner() {
                    Ok(v) => v,
                    Err(e) => return calendar_error_execution(e),
                };
                let subject = format!("{}/{}", owner.0.as_str(), owner.1.as_str());
                let request = match self.state.next_chat_run_id() {
                    Ok(v) => v,
                    Err(_) => return calendar_error_execution(CalendarError::CounterExhausted),
                };
                match crate::calendar_application::now().and_then(|now| {
                    self.state.with_calendar(&owner, |store| {
                        store.propose_batch(&subject, &request, drafts.items, now)
                    })
                }) {
                    Ok(batch) => ChatToolExecution::succeeded(
                        json!({"schema":"calendar-batch-result/v1","batch":batch,"message":"Pending explicit user confirmation. No items have been recorded."}),
                    ),
                    Err(e) => calendar_error_execution(e),
                }
            }
            ChatToolRequest::Plugin {
                tool_name,
                arguments,
            } => match (application(&self.state), &self.session) {
                (Ok(runtime), Some(session)) => {
                    runtime.execute_frozen(session, &tool_name, arguments).await
                }
                _ => ChatToolExecution::denied(json!({"code":"plugin_runtime_unavailable"})),
            },
            other => execute_chat_tool(&self.state, other),
        }
    }
}
