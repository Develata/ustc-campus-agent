//! Thin, loopback-only saved-dialogue request projection.
use super::*;
use crate::chat_conversations::{ConversationManageIntentDto, ConversationTurnIntentDto};
use ustc_campus_agent_core::identity::{TenantId, UserId};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreateIntent {
    schema: String,
    request_id: String,
}

fn owner(state: &WebState) -> Result<(TenantId, UserId), ConversationError> {
    let composition = state.lock().map_err(|_| ConversationError::Unavailable)?;
    // Explicit demo composition identity. No client field/header can select a peer.
    Ok((
        composition.current_tenant_id.clone(),
        composition.current_user_id.clone(),
    ))
}

fn application(state: &WebState) -> Result<&ConversationApplication, ConversationError> {
    state
        .conversations
        .as_ref()
        .as_ref()
        .map_err(|_| ConversationError::Unavailable)
}

fn failure(error: ConversationError) -> Response {
    let code = error.code();
    let status = match error {
        ConversationError::InvalidIntent | ConversationError::InvalidChat(_) => {
            StatusCode::BAD_REQUEST
        }
        ConversationError::NotFound => StatusCode::NOT_FOUND,
        ConversationError::RequestConflict
        | ConversationError::RevisionConflict
        | ConversationError::InProgress => StatusCode::CONFLICT,
        ConversationError::Capacity => StatusCode::TOO_MANY_REQUESTS,
        ConversationError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
    };
    typed_json_response(
        status,
        json!({"schema":"chat-conversation-error/v1", "error":code}),
    )
}

fn respond<T: Serialize>(result: Result<T, ConversationError>) -> Response {
    match result {
        Ok(value) => typed_json_response(StatusCode::OK, value),
        Err(error) => failure(error),
    }
}

pub(super) async fn list(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    respond(owner(&state).and_then(|(tenant, user)| application(&state)?.list(&tenant, &user)))
}

pub(super) async fn create(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<CreateIntent>, JsonRejection>,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let Ok(Json(intent)) = body else {
        return failure(ConversationError::InvalidIntent);
    };
    if !has_application_json_content_type(&headers)
        || intent.schema != "chat-conversation-create/v1"
    {
        return failure(ConversationError::InvalidIntent);
    }
    respond(
        owner(&state).and_then(|(tenant, user)| {
            application(&state)?.create(&tenant, &user, &intent.request_id)
        }),
    )
}

pub(super) async fn get_one(
    State(state): State<WebState>,
    path: Result<AxumPath<String>, PathRejection>,
    headers: HeaderMap,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let Ok(AxumPath(id)) = path else {
        return failure(ConversationError::InvalidIntent);
    };
    respond(owner(&state).and_then(|(tenant, user)| application(&state)?.get(&tenant, &user, &id)))
}

pub(super) async fn submit(
    State(state): State<WebState>,
    path: Result<AxumPath<String>, PathRejection>,
    headers: HeaderMap,
    body: Result<Json<ConversationTurnIntentDto>, JsonRejection>,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (path, body) else {
        return failure(ConversationError::InvalidIntent);
    };
    if !has_application_json_content_type(&headers) {
        return failure(ConversationError::InvalidIntent);
    }
    let owner = match owner(&state) {
        Ok(owner) => owner,
        Err(error) => return failure(error),
    };
    let application = match application(&state) {
        Ok(application) => application,
        Err(error) => return failure(error),
    };
    let confirmed = matches!(
        opportunity_confirmation(&headers),
        OpportunityConfirmationDto::Confirmed
    );
    let executor_state = state.clone();
    respond(
        application
            .submit_with_executor_factory(
                owner,
                id,
                intent,
                confirmed,
                move |tool_calling| async move {
                    plugin_routes::WebChatExecutor::new(executor_state, tool_calling).await
                },
            )
            .await,
    )
}

pub(super) async fn activity(
    State(state): State<WebState>,
    path: Result<AxumPath<String>, PathRejection>,
    headers: HeaderMap,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let Ok(AxumPath(id)) = path else {
        return failure(ConversationError::InvalidIntent);
    };
    respond(
        owner(&state).and_then(|(tenant, user)| application(&state)?.activity(&tenant, &user, &id)),
    )
}

pub(super) async fn manage(
    State(state): State<WebState>,
    path: Result<AxumPath<String>, PathRejection>,
    headers: HeaderMap,
    body: Result<Json<ConversationManageIntentDto>, JsonRejection>,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (path, body) else {
        return failure(ConversationError::InvalidIntent);
    };
    if !has_application_json_content_type(&headers) {
        return failure(ConversationError::InvalidIntent);
    }
    respond(
        owner(&state)
            .and_then(|(tenant, user)| application(&state)?.manage(&tenant, &user, &id, intent)),
    )
}
