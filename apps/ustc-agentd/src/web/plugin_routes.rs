//! Thin protocol intents over the package application service.
use super::*;
use crate::plugin_runtime::{PluginError, PluginRuntime};
use ustc_campus_agent_client_protocol::plugins::{PluginCommandDto, PluginProbeDto};
use ustc_campus_agent_core::identity::{TenantId, UserId};
fn application(state: &WebState) -> Result<&PluginRuntime, PluginError> {
    state
        .plugins
        .as_ref()
        .as_ref()
        .map_err(|_| PluginError::Unavailable)
}
fn owner(state: &WebState) -> Result<(TenantId, UserId), PluginError> {
    let composition = state.lock().map_err(|_| PluginError::Unavailable)?;
    Ok((
        composition.current_tenant_id.clone(),
        composition.current_user_id.clone(),
    ))
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
        let (tenant, user) = owner(&state)?;
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
        let (tenant, user) = owner(&state)?;
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
        let (tenant, user) = owner(&state)?;
        application(&state)?.probe(&tenant, &user, request).await
    }
    .await;
    respond(result)
}

pub(super) struct WebChatExecutor {
    state: WebState,
    session: Option<crate::plugin_runtime::PluginToolSession>,
}
impl WebChatExecutor {
    pub(super) async fn new(state: WebState, tool_calling: bool) -> Result<Self, ChatError> {
        if !tool_calling {
            return Ok(Self {
                state,
                session: None,
            });
        }
        let owner = owner(&state).map_err(|_| ChatError::Internal)?;
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
        self.session
            .as_ref()
            .map(|session| session.definitions())
            .unwrap_or_default()
    }
    async fn execute(&mut self, request: ChatToolRequest) -> ChatToolExecution {
        match request {
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
