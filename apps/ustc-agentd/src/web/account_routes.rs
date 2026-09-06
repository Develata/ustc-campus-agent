//! HTTP adapter for local operator-configured accounts. No provisioning route.
use super::*;
use crate::accounts::{AccountError, AccountService};
use ustc_campus_agent_core::identity::{TenantId, UserId};

const COOKIE: &str = "uca_local_session";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoginIntent {
    schema: String,
    login_name: String,
    password: String,
}

fn service(state: &WebState) -> Result<Option<&AccountService>, AccountError> {
    state
        .accounts
        .as_ref()
        .as_ref()
        .map(Option::as_ref)
        .map_err(|error| *error)
}
fn bearer(headers: &HeaderMap) -> Result<String, AccountError> {
    let mut tokens = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|part| part.trim().split_once('='))
        .filter(|(name, _)| *name == COOKIE)
        .map(|(_, value)| value);
    let token = tokens.next().ok_or(AccountError::Unauthenticated)?;
    if tokens.next().is_some() {
        return Err(AccountError::Unauthenticated);
    }
    Ok(token.to_owned())
}
pub(super) fn failure(error: AccountError) -> Response {
    let (status, code) = match error {
        AccountError::InvalidRequest => (StatusCode::BAD_REQUEST, "invalid_request"),
        AccountError::AuthenticationFailed => (StatusCode::UNAUTHORIZED, "authentication_failed"),
        AccountError::Unauthenticated => (StatusCode::UNAUTHORIZED, "unauthenticated"),
        AccountError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
        AccountError::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
    };
    let mut response = typed_json_response(
        status,
        json!({"schema":"platform-account-error/v1", "error":code}),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if error == AccountError::RateLimited {
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static("900"));
    }
    response
}
// Preserve one already-built HTTP rejection at this thin adapter boundary.
#[allow(clippy::result_large_err)]
pub(super) fn owner(state: &WebState, headers: &HeaderMap) -> Result<(TenantId, UserId), Response> {
    match service(state).map_err(failure)? {
        Some(service) => {
            let subject = service
                .admit(&bearer(headers).map_err(failure)?)
                .map_err(failure)?;
            Ok((subject.tenant().clone(), subject.user().clone()))
        }
        None => {
            let composition = state
                .lock()
                .map_err(|_| failure(AccountError::Unavailable))?;
            Ok((
                composition.current_tenant_id.clone(),
                composition.current_user_id.clone(),
            ))
        }
    }
}
// Preserve one already-built HTTP rejection at this thin adapter boundary.
#[allow(clippy::result_large_err)]
pub(super) fn administrator(state: &WebState, headers: &HeaderMap) -> Result<bool, Response> {
    match service(state).map_err(failure)? {
        Some(service) => Ok(service
            .admit(&bearer(headers).map_err(failure)?)
            .map_err(failure)?
            .is_administrator()),
        None => Ok(false),
    }
}
pub(super) async fn mode(State(state): State<WebState>) -> Response {
    match service(&state) {
        Ok(value) => typed_json_response(
            StatusCode::OK,
            json!({"schema":"platform-account-mode/v1", "mode":if value.is_some() {"local-accounts"} else {"loopback-demo"}, "sso_available":false}),
        ),
        Err(error) => failure(error),
    }
}
pub(super) async fn me(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let result = service(&state)
        .and_then(|service| service.ok_or(AccountError::Unauthenticated))
        .and_then(|service| service.admit(&bearer(&headers)?));
    match result {
        Ok(subject) => typed_json_response(
            StatusCode::OK,
            json!({"schema":"platform-account/v1", "account":subject}),
        ),
        Err(error) => failure(error),
    }
}
pub(super) async fn login(
    State(state): State<WebState>,
    body: Result<Json<LoginIntent>, JsonRejection>,
) -> Response {
    let Ok(Json(intent)) = body else {
        return failure(AccountError::InvalidRequest);
    };
    if intent.schema != "platform-account-login/v1" {
        return failure(AccountError::InvalidRequest);
    }
    let service =
        match service(&state).and_then(|s| s.cloned().ok_or(AccountError::Unauthenticated)) {
            Ok(service) => service,
            Err(error) => return failure(error),
        };
    let result =
        tokio::task::spawn_blocking(move || service.login(&intent.login_name, &intent.password))
            .await;
    match result {
        Ok(Ok((bearer, subject))) => {
            let mut response = typed_json_response(
                StatusCode::OK,
                json!({"schema":"platform-account/v1", "account":subject}),
            );
            let Ok(cookie) = HeaderValue::from_str(&format!(
                "{COOKIE}={bearer}; Path=/; HttpOnly; SameSite=Strict; Max-Age=28800"
            )) else {
                return failure(AccountError::Unavailable);
            };
            response.headers_mut().insert(header::SET_COOKIE, cookie);
            response
        }
        Ok(Err(error)) => failure(error),
        Err(_) => failure(AccountError::Unavailable),
    }
}
pub(super) async fn logout(State(state): State<WebState>, headers: HeaderMap) -> Response {
    let result = service(&state)
        .and_then(|s| s.ok_or(AccountError::Unauthenticated))
        .and_then(|service| service.logout(&bearer(&headers)?));
    match result {
        Ok(()) => {
            let mut response = typed_json_response(
                StatusCode::OK,
                json!({"schema":"platform-account-logout/v1", "logged_out":true}),
            );
            response.headers_mut().insert(
                header::SET_COOKIE,
                HeaderValue::from_static(
                    "uca_local_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
                ),
            );
            response
        }
        Err(error) => failure(error),
    }
}
pub(super) async fn admit_request(
    State(state): State<WebState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if matches!(service(&state), Ok(Some(_))) {
        if path.starts_with("/api/v1/opportunity/")
            || path.starts_with("/api/v1/demo/administrator/")
        {
            return web_error(
                StatusCode::FORBIDDEN,
                "demo_resource_unavailable_in_account_mode",
            );
        }
        let public = path == "/api/v1/account/login"
            || path == "/api/v1/account/mode"
            || path == "/api/v1/server/info"
            || path == "/api/v1/client/capabilities"
            || path.starts_with("/api/v1/market/")
            || path.starts_with("/api/v1/affairs/")
            || path.starts_with("/api/v1/changes/");
        if path.starts_with("/api/") && !public {
            let admitted = match owner(&state, request.headers()) {
                Ok(value) => value,
                Err(response) => return response,
            };
            if let Some(expected) = request.headers().get("x-uca-account-subject") {
                let expected = expected
                    .to_str()
                    .ok()
                    .and_then(|value| serde_json::from_str::<(String, String)>(value).ok());
                if expected.as_ref().is_none_or(|(tenant, user)| {
                    tenant != admitted.0.as_str() || user != admitted.1.as_str()
                }) {
                    return failure(AccountError::Unauthenticated);
                }
            }
        }
    }
    match service(&state) {
        Err(error) => return failure(error),
        Ok(Some(_))
            if !matches!(
                *request.method(),
                axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
            ) =>
        {
            let host = request
                .headers()
                .get(header::HOST)
                .and_then(|v| v.to_str().ok());
            let origin = request
                .headers()
                .get(header::ORIGIN)
                .and_then(|v| v.to_str().ok());
            if !host
                .zip(origin)
                .is_some_and(|(host, origin)| origin_matches_host(origin, host))
            {
                return web_error(StatusCode::FORBIDDEN, "account_origin_required");
            }
        }
        _ => {}
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
