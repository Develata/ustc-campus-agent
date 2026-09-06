use super::{McpClient, McpError, PROTOCOL_VERSION};
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

impl McpClient {
    pub(super) async fn rpc(&mut self, method: &str, params: Value) -> Result<Value, McpError> {
        self.next_id = self.next_id.checked_add(1).ok_or(McpError::LimitExceeded)?;
        let id = self.next_id;
        let payload = json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params});
        let deadline = self.limits.request_timeout;
        let result = tokio::time::timeout(
            deadline,
            self.exchange(
                Method::POST,
                Some(payload),
                Some(id),
                method == "initialize",
            ),
        )
        .await
        .unwrap_or(Err(McpError::Timeout));
        self.observe_error(&result);
        result
    }

    pub(super) async fn initialized_notification(&mut self) -> Result<(), McpError> {
        let deadline = self.limits.request_timeout;
        let result = tokio::time::timeout(
            deadline,
            self.exchange(
                Method::POST,
                Some(json!({"jsonrpc":"2.0", "method":"notifications/initialized"})),
                None,
                false,
            ),
        )
        .await
        .unwrap_or(Err(McpError::Timeout));
        self.observe_error(&result);
        result.map(|_| ())
    }

    pub(super) async fn delete_session(&mut self) -> Result<(), McpError> {
        let deadline = self.limits.request_timeout;
        tokio::time::timeout(deadline, self.exchange(Method::DELETE, None, None, false))
            .await
            .unwrap_or(Err(McpError::Timeout))
            .map(|_| ())
    }

    fn observe_error(&mut self, result: &Result<Value, McpError>) {
        if let Err(error) = result {
            if *error == McpError::SessionExpired {
                self.session_id = None;
                self.initialized = false;
            }
            if matches!(
                error,
                McpError::Protocol
                    | McpError::SchemaDrift
                    | McpError::SessionExpired
                    | McpError::LimitExceeded
            ) {
                self.state = super::BindingState::Quarantined;
                self.reviewable = false;
            }
        }
    }

    async fn exchange(
        &mut self,
        method: Method,
        payload: Option<Value>,
        id: Option<u64>,
        initializing: bool,
    ) -> Result<Value, McpError> {
        let client = super::endpoint::pinned_client(
            &self.endpoint,
            self.config.endpoint_policy,
            &self.limits,
        )
        .await?;
        let mut request = client
            .request(method.clone(), self.endpoint.clone())
            .header("Accept", "application/json, text/event-stream");
        if !initializing {
            request = request.header("MCP-Protocol-Version", PROTOCOL_VERSION);
            if let Some(session) = &self.session_id {
                request = request.header("MCP-Session-Id", session);
            }
        }
        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }
        if let Some(payload) = payload {
            let body = serde_json::to_vec(&payload).map_err(|_| McpError::Protocol)?;
            if body.len() > self.limits.max_response_bytes {
                return Err(McpError::LimitExceeded);
            }
            request = request
                .header("Content-Type", "application/json")
                .body(body);
        }
        let mut response = request.send().await.map_err(network_error)?;
        let status = response.status();
        if status == StatusCode::NOT_FOUND && self.session_id.is_some() {
            return Err(McpError::SessionExpired);
        }
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(McpError::Authentication);
        }
        if status.is_redirection() {
            return Err(McpError::EndpointDenied);
        }
        if method == Method::DELETE {
            return if status.is_success() || status == StatusCode::METHOD_NOT_ALLOWED {
                Ok(Value::Null)
            } else {
                Err(McpError::Transport)
            };
        }
        if !status.is_success() {
            return Err(McpError::Transport);
        }
        if response
            .headers()
            .get("MCP-Protocol-Version")
            .is_some_and(|version| version.to_str().ok() != Some(PROTOCOL_VERSION))
        {
            return Err(McpError::Protocol);
        }
        if let Some(value) = response.headers().get("MCP-Session-Id") {
            let session = value.to_str().map_err(|_| McpError::Protocol)?;
            if session.is_empty()
                || session.len() > 256
                || !session.bytes().all(|b| (0x21..=0x7e).contains(&b))
            {
                return Err(McpError::Protocol);
            }
            if initializing {
                self.session_id = Some(session.to_owned());
            } else if self.session_id.as_deref() != Some(session) {
                return Err(McpError::Protocol);
            }
        }
        if response
            .content_length()
            .is_some_and(|size| size > self.limits.max_response_bytes as u64)
        {
            return Err(McpError::LimitExceeded);
        }
        if id.is_none() {
            if status != StatusCode::ACCEPTED
                || response
                    .chunk()
                    .await
                    .map_err(network_error)?
                    .is_some_and(|bytes| !bytes.is_empty())
            {
                return Err(McpError::Protocol);
            }
            return Ok(Value::Null);
        }
        let id = id.ok_or(McpError::Protocol)?;
        let content_type = response
            .headers()
            .get("Content-Type")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if content_type != "application/json" && content_type != "text/event-stream" {
            return Err(McpError::Protocol);
        }
        let sse = content_type == "text/event-stream";
        let mut bytes = Vec::new();
        let mut received = 0usize;
        let mut event_count = 0usize;
        while let Some(chunk) = response.chunk().await.map_err(network_error)? {
            received = received
                .checked_add(chunk.len())
                .ok_or(McpError::LimitExceeded)?;
            if received > self.limits.max_response_bytes {
                return Err(McpError::LimitExceeded);
            }
            bytes.extend_from_slice(&chunk);
            if sse {
                while let Some((end, delimiter)) = event_boundary(&bytes) {
                    event_count += 1;
                    if event_count > self.limits.max_sse_events {
                        return Err(McpError::LimitExceeded);
                    }
                    let result = parse_event(&bytes[..end], id)?;
                    bytes.drain(..end + delimiter);
                    if let Some(result) = result {
                        return Ok(result);
                    }
                }
            }
        }
        if sse {
            return Err(McpError::Protocol);
        }
        let envelope: Value = serde_json::from_slice(&bytes).map_err(|_| McpError::Protocol)?;
        decode_envelope(envelope, id, false)?.ok_or(McpError::Protocol)
    }
}

fn network_error(error: reqwest::Error) -> McpError {
    if error.is_timeout() {
        McpError::Timeout
    } else {
        McpError::Transport
    }
}

fn event_boundary(bytes: &[u8]) -> Option<(usize, usize)> {
    fn ending(bytes: &[u8], index: usize) -> usize {
        match bytes.get(index) {
            Some(b'\r') if bytes.get(index + 1) == Some(&b'\n') => 2,
            Some(b'\r' | b'\n') => 1,
            _ => 0,
        }
    }
    let mut index = 0;
    while index < bytes.len() {
        let first = ending(bytes, index);
        if first != 0 {
            let second = ending(bytes, index + first);
            if second != 0 {
                return Some((index, first + second));
            }
            index += first;
        } else {
            index += 1;
        }
    }
    None
}

fn parse_event(bytes: &[u8], id: u64) -> Result<Option<Value>, McpError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| McpError::Protocol)?
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let mut data = Vec::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        if let Some(value) = line.strip_prefix("data:") {
            data.push(value.strip_prefix(' ').unwrap_or(value));
        } else if line == "data" {
            data.push("");
        }
    }
    let data = data.join("\n");
    if data.trim().is_empty() {
        return Ok(None);
    }
    decode_envelope(
        serde_json::from_str(&data).map_err(|_| McpError::Protocol)?,
        id,
        true,
    )
}

fn decode_envelope(
    envelope: Value,
    id: u64,
    allow_notification: bool,
) -> Result<Option<Value>, McpError> {
    let object = envelope.as_object().ok_or(McpError::Protocol)?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(McpError::Protocol);
    }
    if let Some(method) = object.get("method") {
        if object.contains_key("id")
            || !allow_notification
            || object.contains_key("result")
            || object.contains_key("error")
        {
            return Err(McpError::Protocol);
        }
        return match method.as_str() {
            Some("notifications/tools/list_changed") => Err(McpError::SchemaDrift),
            Some("notifications/progress" | "notifications/message") => Ok(None),
            _ => Err(McpError::Protocol),
        };
    }
    if object.get("id").and_then(Value::as_u64) != Some(id)
        || object.contains_key("result") == object.contains_key("error")
    {
        return Err(McpError::Protocol);
    }
    if let Some(error) = object.get("error") {
        if error.get("code").and_then(Value::as_i64).is_none()
            || error.get("message").and_then(Value::as_str).is_none()
        {
            return Err(McpError::Protocol);
        }
        return Err(McpError::RemoteError);
    }
    let result = object
        .get("result")
        .filter(|result| result.is_object())
        .ok_or(McpError::Protocol)?;
    Ok(Some(result.clone()))
}

#[cfg(test)]
mod sse_tests {
    use super::*;
    #[test]
    fn standard_sse_line_endings_and_multiline_data_are_supported() {
        for ending in ["\n", "\r\n", "\r"] {
            let event = format!(
                "data: {{\"jsonrpc\":\"2.0\",{ending}data: \"id\":7,\"result\":{{}}}}{ending}{ending}"
            );
            let (end, _) = event_boundary(event.as_bytes()).expect("complete event");
            assert_eq!(
                parse_event(&event.as_bytes()[..end], 7).expect("decode"),
                Some(json!({}))
            );
        }
    }
}
