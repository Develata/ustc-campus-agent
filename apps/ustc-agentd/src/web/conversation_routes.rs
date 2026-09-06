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

// Preserve one already-built HTTP rejection at this thin adapter boundary.
#[allow(clippy::result_large_err)]
fn owner(state: &WebState, headers: &HeaderMap) -> Result<(TenantId, UserId), Response> {
    account_routes::owner(state, headers)
}

fn owned(
    state: &WebState,
    headers: &HeaderMap,
    operation: impl FnOnce(TenantId, UserId) -> Response,
) -> Response {
    match owner(state, headers) {
        Ok((tenant, user)) => operation(tenant, user),
        Err(response) => response,
    }
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
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.list(&tenant, &user)))
    })
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
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.create(&tenant, &user, &intent.request_id)))
    })
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
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.get(&tenant, &user, &id)))
    })
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
    let owner = match owner(&state, &headers) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let application = match application(&state) {
        Ok(application) => application,
        Err(error) => return failure(error),
    };
    let confirmed = matches!(
        opportunity_confirmation(&headers),
        OpportunityConfirmationDto::Confirmed
    );
    let mut executor_state = state.clone();
    executor_state.course_consent = confirmed;
    executor_state.execution_headers = Some(headers.clone());
    let executor_owner = owner.clone();
    respond(
        application
            .submit_with_executor_factory(
                owner,
                id,
                intent,
                confirmed,
                move |tool_calling| async move {
                    plugin_routes::WebChatExecutor::new(
                        executor_state,
                        tool_calling,
                        executor_owner,
                    )
                    .await
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
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.activity(&tenant, &user, &id)))
    })
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
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.manage(&tenant, &user, &id, intent)))
    })
}

pub(super) async fn root_prompt(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.root_prompt(&tenant, &user)))
    })
}

pub(super) async fn update_root_prompt(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<crate::chat_conversations::RootPromptUpdateDto>, JsonRejection>,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let Ok(Json(intent)) = body else {
        return failure(ConversationError::InvalidIntent);
    };
    if !has_application_json_content_type(&headers) {
        return failure(ConversationError::InvalidIntent);
    }
    owned(&state, &headers, |tenant, user| {
        respond(application(&state).and_then(|app| app.update_root_prompt(&tenant, &user, intent)))
    })
}
