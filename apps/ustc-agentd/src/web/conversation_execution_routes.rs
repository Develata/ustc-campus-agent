//! Owner-admitted execution control, independent of model/tool authority.
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CancelIntent {
    schema: String,
    request_id: String,
}
pub(super) async fn cancel(
    State(state): State<WebState>,
    path: Result<AxumPath<String>, PathRejection>,
    headers: HeaderMap,
    body: Result<Json<CancelIntent>, JsonRejection>,
) -> Response {
    if let Err(compatibility) =
        dispatch_with_protocol_major(presented_protocol_major(&headers), || ())
    {
        return compatibility_response(compatibility);
    }
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (path, body) else {
        return failure(ConversationError::InvalidIntent);
    };
    if !has_application_json_content_type(&headers)
        || intent.schema != "chat-conversation-cancel/v1"
        || intent.request_id.is_empty()
        || intent.request_id.len() > 128
        || !intent
            .request_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
    {
        return failure(ConversationError::InvalidIntent);
    }
    let owner = match account_routes::owner(&state, &headers) {
        Ok(owner) => owner,
        Err(response) => return response,
    };
    let result = state
        .conversations
        .as_ref()
        .as_ref()
        .map_err(|_| ConversationError::Unavailable)
        .and_then(|app| app.cancel(&owner.0, &owner.1, &id, &intent.request_id));
    match result {
        Ok(value) => typed_json_response(StatusCode::OK, value),
        Err(error) => failure(error),
    }
}
fn failure(error: ConversationError) -> Response {
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
        json!({"schema":"chat-conversation-error/v1","error":error.code()}),
    )
}
