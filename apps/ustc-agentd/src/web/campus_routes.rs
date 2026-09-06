//! Thin source observation and request-local course planning adapters.
use super::*;
use ustc_campus_agent_course_planning::personal::{PersonalCourseRequest, plan_personal};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SearchIntent {
    schema: String,
    query: String,
    source_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ImportIntent {
    schema: String,
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReviewIntent {
    schema: String,
    revision_id: String,
    reviewer: String,
    evidence: String,
}
fn respond<T: Serialize>(result: Result<T, String>) -> Response {
    match result {
        Ok(value) => typed_json_response(StatusCode::OK, value),
        Err(error) => {
            let status = if error.contains("not_found") {
                StatusCode::NOT_FOUND
            } else if error.contains("rate_limited") {
                StatusCode::TOO_MANY_REQUESTS
            } else if error.contains("not_reviewed") || error.contains("permission") {
                StatusCode::FORBIDDEN
            } else if error.contains("unavailable")
                || error.contains("failed")
                || error.contains("timeout")
            {
                StatusCode::SERVICE_UNAVAILABLE
            } else {
                StatusCode::BAD_REQUEST
            };
            typed_json_response(
                status,
                json!({"schema":"campus-workspace-error/v1","error":error}),
            )
        }
    }
}
// Preserve one already-built HTTP rejection at this thin adapter boundary.
#[allow(clippy::result_large_err)]
fn require_administrator(state: &WebState, headers: &HeaderMap) -> Result<(), Response> {
    match state.accounts.as_ref().as_ref() {
        Ok(Some(_)) => {
            if account_routes::administrator(state, headers)? {
                Ok(())
            } else {
                Err(denied())
            }
        }
        Ok(None) => {
            if administrator_demo_header_authorized(headers) {
                Ok(())
            } else {
                Err(denied())
            }
        }
        Err(error) => Err(account_routes::failure(*error)),
    }
}
fn denied() -> Response {
    typed_json_response(
        StatusCode::FORBIDDEN,
        json!({"schema":"campus-workspace-error/v1","error":"administrator_confirmation_required"}),
    )
}
pub(super) async fn sources(State(state): State<WebState>) -> Response {
    // Review evidence is operator-only; user catalog exposes the acquisition
    // boundary and useful source identity, not internal permission documents.
    respond(state.sources.sources().map(|sources| json!({"schema":"source-catalog/v1","sources":sources.into_iter().map(|s|json!({"source_id":s.source_id,"title":s.title,"url":s.url,"minimum_interval_seconds":s.minimum_interval_seconds})).collect::<Vec<_>>() })))
}
pub(super) async fn search(
    State(state): State<WebState>,
    body: Result<Json<SearchIntent>, JsonRejection>,
) -> Response {
    let Ok(Json(intent)) = body else {
        return respond::<()>(Err("source_search_invalid".into()));
    };
    if intent.schema != "source-search/v1" {
        return respond::<()>(Err("source_search_invalid".into()));
    }
    respond(
        state
            .sources
            .search(&intent.query, intent.source_id.as_deref()),
    )
}
pub(super) async fn history(
    State(state): State<WebState>,
    AxumPath(id): AxumPath<String>,
) -> Response {
    respond(state.sources.history(&id))
}
pub(super) async fn fetch(
    State(state): State<WebState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
) -> Response {
    if let Err(response) = require_administrator(&state, &headers) {
        return response;
    }
    respond(state.sources.fetch(&id).await)
}
pub(super) async fn import(
    State(state): State<WebState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
    body: Result<Json<ImportIntent>, JsonRejection>,
) -> Response {
    if let Err(response) = require_administrator(&state, &headers) {
        return response;
    }
    let Ok(Json(intent)) = body else {
        return respond::<()>(Err("source_import_invalid".into()));
    };
    if intent.schema != "source-import-text/v1" {
        return respond::<()>(Err("source_import_invalid".into()));
    }
    respond(state.sources.import(&id, intent.text))
}
pub(super) async fn review(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<ReviewIntent>, JsonRejection>,
) -> Response {
    if let Err(response) = require_administrator(&state, &headers) {
        return response;
    }
    let Ok(Json(intent)) = body else {
        return respond::<()>(Err("source_review_invalid".into()));
    };
    if intent.schema != "source-observation-review/v1" {
        return respond::<()>(Err("source_review_invalid".into()));
    }
    respond(
        state
            .sources
            .review(&intent.revision_id, &intent.reviewer, &intent.evidence),
    )
}
pub(super) async fn courses(body: Result<Json<PersonalCourseRequest>, JsonRejection>) -> Response {
    let Ok(Json(intent)) = body else {
        return respond::<()>(Err("course_request_invalid".into()));
    };
    respond(plan_personal(&intent))
}
