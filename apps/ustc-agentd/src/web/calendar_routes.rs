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
pub(super) async fn list(State(state): State<WebState>) -> Response {
    let Ok(mut application) = state.lock() else {
        return error(CalendarError::PersistenceUnavailable);
    };
    match application.calendar_proposals() {
        Ok(list) => Json(list).into_response(),
        Err(e) => error(e),
    }
}
pub(super) async fn propose(
    State(state): State<WebState>,
    body: Result<Json<ProposeCalendarIntent>, JsonRejection>,
) -> Response {
    let Ok(Json(intent)) = body else {
        return error(CalendarError::InvalidProposal);
    };
    let Ok(mut application) = state.lock() else {
        return error(CalendarError::PersistenceUnavailable);
    };
    result(application.propose_calendar(intent))
}
pub(super) async fn confirm(
    State(state): State<WebState>,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (id, body) else {
        return error(CalendarError::InvalidProposal);
    };
    if intent.schema != "calendar-proposal-confirm/v1" {
        return error(CalendarError::InvalidProposal);
    }
    let Ok(mut application) = state.lock() else {
        return error(CalendarError::PersistenceUnavailable);
    };
    result(application.confirm_calendar_proposal(&id))
}
pub(super) async fn cancel(
    State(state): State<WebState>,
    id: Result<AxumPath<String>, PathRejection>,
    body: Result<Json<ConfirmIntent>, JsonRejection>,
) -> Response {
    let (Ok(AxumPath(id)), Ok(Json(intent))) = (id, body) else {
        return error(CalendarError::InvalidProposal);
    };
    if intent.schema != "calendar-proposal-cancel/v1" {
        return error(CalendarError::InvalidProposal);
    }
    let Ok(mut application) = state.lock() else {
        return error(CalendarError::PersistenceUnavailable);
    };
    result(application.cancel_calendar_proposal(&id))
}
