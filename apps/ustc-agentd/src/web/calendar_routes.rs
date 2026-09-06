//! Thin versioned Calendar intent/result projection; the application owns commands.
use super::*;
use crate::calendar_application::ProposeCalendarIntent;
use ustc_campus_agent_simple_calendar::CalendarProposal;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ConfirmIntent {
    schema: String,
}

pub(super) fn error(error: CalendarError) -> Response {
    let (status, code) = match error {
        CalendarError::InvalidProposal
        | CalendarError::InvalidTitle
        | CalendarError::InvalidScheduledFor => {
            (StatusCode::BAD_REQUEST, "invalid_calendar_proposal")
        }
        CalendarError::ProposalNotFound | CalendarError::ItemNotFound => {
            (StatusCode::NOT_FOUND, "calendar_proposal_not_found")
        }
        CalendarError::ProposalConflict => (StatusCode::CONFLICT, "calendar_proposal_conflict"),
        CalendarError::ProposalExpired => (StatusCode::CONFLICT, "calendar_proposal_expired"),
        CalendarError::ProposalLimitExceeded | CalendarError::ItemLimitExceeded => (
            StatusCode::TOO_MANY_REQUESTS,
            "calendar_proposal_capacity_exceeded",
        ),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            "calendar_store_unavailable",
        ),
    };
    (
        status,
        Json(json!({"schema":"calendar-proposal-error/v1","error":code})),
    )
        .into_response()
}
fn result(value: Result<CalendarProposal, CalendarError>) -> Response {
    match value {
        Ok(proposal) => Json(json!({"schema":"calendar-proposal-result/v1","proposal":proposal}))
            .into_response(),
        Err(e) => error(e),
    }
}

// Preserve one already-built HTTP rejection at this thin adapter boundary.
#[allow(clippy::result_large_err)]
fn scoped<T>(
    state: &WebState,
    headers: &HeaderMap,
    f: impl FnOnce(
        &mut ustc_campus_agent_simple_calendar::CalendarStore,
        &str,
        u64,
    ) -> Result<T, CalendarError>,
) -> Result<T, Response> {
    let owner = account_routes::owner(state, headers)?;
    let subject = format!("{}/{}", owner.0.as_str(), owner.1.as_str());
    let now = crate::calendar_application::now().map_err(error)?;
    state
        .with_calendar(&owner, |store| f(store, &subject, now))
        .map_err(error)
}
pub(super) async fn list(State(state): State<WebState>, headers: HeaderMap) -> Response {
    match scoped(&state, &headers, |store, subject, _| {
        crate::calendar_application::workspace_list(store, subject)
    }) {
        Ok(v) => Json(v).into_response(),
        Err(e) => e,
    }
}
pub(super) async fn propose(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<ProposeCalendarIntent>, JsonRejection>,
) -> Response {
    let Ok(Json(intent)) = body else {
        return error(CalendarError::InvalidProposal);
    };
    if intent.schema != "calendar-proposal/v1" {
        return error(CalendarError::InvalidProposal);
    }
    match scoped(&state, &headers, |store, subject, now| {
        store.propose(subject, &intent.request_id, intent.mutation, now)
    }) {
        Ok(v) => result(Ok(v)),
        Err(e) => e,
    }
}
pub(super) async fn confirm(
    State(state): State<WebState>,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    finish(state, headers, id, body, true)
}
pub(super) async fn cancel(
    State(state): State<WebState>,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    finish(state, headers, id, body, false)
}
fn finish(
    state: WebState,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
    confirm: bool,
) -> Response {
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (id, body) else {
        return error(CalendarError::InvalidProposal);
    };
    let schema = if confirm {
        "calendar-proposal-confirm/v1"
    } else {
        "calendar-proposal-cancel/v1"
    };
    if intent.schema != schema {
        return error(CalendarError::InvalidProposal);
    }
    match scoped(&state, &headers, |store, subject, now| {
        if confirm {
            store.confirm_proposal(subject, &id, now)
        } else {
            store.cancel_proposal(subject, &id, now)
        }
    }) {
        Ok(v) => result(Ok(v)),
        Err(e) => e,
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BatchIntent {
    schema: String,
    request_id: String,
    items: Vec<ustc_campus_agent_simple_calendar::CalendarDraft>,
}
pub(super) async fn propose_batch(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<BatchIntent>, JsonRejection>,
) -> Response {
    let Ok(Json(intent)) = body else {
        return error(CalendarError::InvalidProposal);
    };
    if intent.schema != "calendar-batch-proposal/v1" {
        return error(CalendarError::InvalidProposal);
    }
    batch_result(scoped(&state, &headers, |store, subject, now| {
        store.propose_batch(subject, &intent.request_id, intent.items, now)
    }))
}
fn batch_result(
    value: Result<ustc_campus_agent_simple_calendar::CalendarBatch, Response>,
) -> Response {
    match value {
        Ok(batch) => {
            Json(json!({"schema":"calendar-batch-result/v1","batch":batch})).into_response()
        }
        Err(e) => e,
    }
}
pub(super) async fn confirm_batch(
    State(state): State<WebState>,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    batch_finish(state, headers, id, body, true)
}
pub(super) async fn cancel_batch(
    State(state): State<WebState>,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    batch_finish(state, headers, id, body, false)
}
fn batch_finish(
    state: WebState,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
    confirm: bool,
) -> Response {
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (id, body) else {
        return error(CalendarError::InvalidProposal);
    };
    if intent.schema
        != if confirm {
            "calendar-batch-confirm/v1"
        } else {
            "calendar-batch-cancel/v1"
        }
    {
        return error(CalendarError::InvalidProposal);
    }
    batch_result(scoped(&state, &headers, |store, subject, now| {
        store.finish_batch(subject, &id, confirm, now)
    }))
}
pub(super) async fn read_reminder(
    State(state): State<WebState>,
    headers: HeaderMap,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (id, body) else {
        return error(CalendarError::InvalidProposal);
    };
    if intent.schema != "calendar-reminder-read/v1" {
        return error(CalendarError::InvalidProposal);
    }
    match scoped(&state, &headers, |store, _, now| {
        store.read_reminder(&id, now)
    }) {
        Ok(r) => {
            Json(json!({"schema":"calendar-reminder-receipt/v1","reminder":r})).into_response()
        }
        Err(e) => e,
    }
}
