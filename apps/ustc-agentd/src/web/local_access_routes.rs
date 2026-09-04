//! plan_ref: docs/plan/modules/20-application-api-host.md#bounded-loopback-local-access-gate
//! HTTP projection for the bounded deployment-local access account.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use super::{WebState, hardened, has_application_json_content_type};
use crate::chat_provider::ProviderIdentity;
use crate::local_access::LoginError;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LocalAccessLoginBody {
    schema: String,
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LocalAccessAccount {
    username: String,
    boundary: &'static str,
}

#[derive(Serialize)]
struct LocalAccessEnvelope {
    schema: &'static str,
    authenticated: bool,
    account: Option<LocalAccessAccount>,
    provider: Option<ProviderIdentity>,
}

#[derive(Serialize)]
struct LocalAccessErrorEnvelope {
    schema: &'static str,
    error: &'static str,
}

pub(super) async fn require(
    State(state): State<WebState>,
    request: Request,
    next: Next,
) -> Response {
    if !is_authenticated(&state, request.headers()) {
        return error(StatusCode::UNAUTHORIZED, "authentication_required");
    }
    next.run(request).await
}

fn cookie_values(headers: &HeaderMap) -> Option<Vec<&str>> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .map(|value| value.to_str().ok())
        .collect()
}

fn is_authenticated(state: &WebState, headers: &HeaderMap) -> bool {
    cookie_values(headers).is_some_and(|values| state.local_access.authenticate_cookie(&values))
}

fn envelope(state: &WebState, authenticated: bool) -> LocalAccessEnvelope {
    LocalAccessEnvelope {
        schema: "ustc-local-access/v1",
        authenticated,
        account: authenticated.then(|| LocalAccessAccount {
            username: state.local_access.username().to_owned(),
            boundary: "local_deployment_access_only",
        }),
        provider: authenticated.then(|| state.chat_provider.identity()),
    }
}

pub(super) async fn session(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let authenticated = is_authenticated(&state, &headers);
    hardened(Json(envelope(&state, authenticated)).into_response())
}

pub(super) async fn login(
    State(state): State<WebState>,
    headers: HeaderMap,
    body: Result<Json<LocalAccessLoginBody>, JsonRejection>,
) -> Response {
    if !has_application_json_content_type(&headers) {
        return error(StatusCode::BAD_REQUEST, "invalid_login_request");
    }
    let Json(body) = match body {
        Ok(body) => body,
        Err(_) => return error(StatusCode::BAD_REQUEST, "invalid_login_request"),
    };
    if body.schema != "ustc-local-access-login/v1" {
        return error(StatusCode::BAD_REQUEST, "invalid_login_request");
    }
    match state.local_access.login(&body.username, &body.password) {
        Ok(token) => {
            let Ok(cookie) = HeaderValue::from_str(&format!(
                "uca_session={token}; Path=/; HttpOnly; SameSite=Strict; Max-Age=43200"
            )) else {
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "local_access_unavailable",
                );
            };
            let mut response = hardened(Json(envelope(&state, true)).into_response());
            response.headers_mut().insert(header::SET_COOKIE, cookie);
            response
        }
        Err(LoginError::InvalidRequest) => error(StatusCode::BAD_REQUEST, "invalid_login_request"),
        Err(LoginError::InvalidCredentials) => {
            error(StatusCode::UNAUTHORIZED, "invalid_credentials")
        }
        Err(LoginError::RateLimited) => {
            let mut response = error(StatusCode::TOO_MANY_REQUESTS, "rate_limited");
            response
                .headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
            response
        }
        Err(LoginError::Internal) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "local_access_unavailable",
        ),
    }
}

pub(super) async fn logout(State(state): State<WebState>, headers: HeaderMap) -> Response {
    if let Some(cookie_values) = cookie_values(&headers) {
        state.local_access.logout_cookie(&cookie_values);
    }
    let mut response = hardened(Json(envelope(&state, false)).into_response());
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "uca_session=deleted; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        ),
    );
    response
}

fn error(status: StatusCode, code: &'static str) -> Response {
    hardened(
        (
            status,
            Json(LocalAccessErrorEnvelope {
                schema: "ustc-local-access/v1",
                error: code,
            }),
        )
            .into_response(),
    )
}
