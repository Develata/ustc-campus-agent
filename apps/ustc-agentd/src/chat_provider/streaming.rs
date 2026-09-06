//! Incremental Chat Completions SSE. Complete batches still pass the ordinary parser.
use super::*;
use std::collections::BTreeMap;

impl OpenAiCompatibleProvider {
    pub(super) async fn complete_streamed(
        &self,
        request: &ProviderRequest,
        observer: &mut (impl FnMut(&str) + Send),
    ) -> Result<ProviderTurn, ProviderError> {
        let mut wire = build_wire_request(&self.identity.model, request, self.profile)?;
        wire.stream = true;
        let bytes = serde_json::to_vec(&wire).map_err(|_| ProviderError::Protocol)?;
        preflight_context_budget(bytes.len(), self.context_limit_tokens, self.profile)?;
        let mut response = self
            .client
            .post(self.endpoint.clone())
            .bearer_auth(&self.api_key.0)
            .header(CONTENT_TYPE, "application/json")
            .header("Accept", "text/event-stream, application/json")
            .body(bytes)
            .send()
            .await
            .map_err(map_transport_error)?;
        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(ProviderError::Unauthorized);
            }
            StatusCode::TOO_MANY_REQUESTS => return Err(ProviderError::RateLimited),
            status if !status.is_success() => return Err(ProviderError::Unavailable),
            _ => {}
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAX_RESPONSE_BYTES as u64)
        {
            return Err(ProviderError::Protocol);
        }
        let sse = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| {
                v.split(';')
                    .next()
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case("text/event-stream"))
            });
        let mut parser = StreamParser::default();
        let mut buffer = Vec::new();
        let mut total = 0usize;
        while let Some(chunk) = response.chunk().await.map_err(map_transport_error)? {
            total = total
                .checked_add(chunk.len())
                .ok_or(ProviderError::Protocol)?;
            if total > MAX_RESPONSE_BYTES {
                return Err(ProviderError::Protocol);
            }
            buffer.extend_from_slice(&chunk);
            if sse {
                while let Some(end) = buffer.iter().position(|v| *v == b'\n') {
                    let line = buffer.drain(..=end).collect::<Vec<_>>();
                    let line = std::str::from_utf8(&line)
                        .map_err(|_| ProviderError::Protocol)?
                        .trim_end_matches(['\r', '\n']);
                    parser.line(line, observer)?;
                }
            }
        }
        if sse {
            if !buffer.is_empty() {
                let line = std::str::from_utf8(&buffer).map_err(|_| ProviderError::Protocol)?;
                parser.line(line.trim_end_matches('\r'), observer)?;
            }
            parser.line("", observer)?;
            parser.finish()
        } else {
            // Some compatible peers ignore stream=true. Their complete JSON is validated
            // once and is never presented as a synthetic stream.
            parse_wire_response(
                serde_json::from_slice(&buffer).map_err(|_| ProviderError::Protocol)?,
            )
        }
    }
}

#[derive(Default)]
struct StreamParser {
    event: String,
    content: String,
    role_seen: bool,
    calls: BTreeMap<u64, Value>,
    finish_reason: Option<String>,
    usage: Option<Value>,
    done: bool,
}
impl StreamParser {
    fn line(&mut self, line: &str, observer: &mut impl FnMut(&str)) -> Result<(), ProviderError> {
        if let Some(data) = line.strip_prefix("data:") {
            if !self.event.is_empty() {
                self.event.push('\n');
            }
            self.event.push_str(data.strip_prefix(' ').unwrap_or(data));
        } else if line.is_empty() && !self.event.is_empty() {
            let event = std::mem::take(&mut self.event);
            self.event(&event, observer)?;
        }
        Ok(())
    }
    fn event(&mut self, data: &str, observer: &mut impl FnMut(&str)) -> Result<(), ProviderError> {
        if self.done {
            return Err(ProviderError::Protocol);
        }
        if data == "[DONE]" {
            self.done = true;
            return Ok(());
        }
        let value: Value = serde_json::from_str(data).map_err(|_| ProviderError::Protocol)?;
        let choices = value
            .get("choices")
            .and_then(Value::as_array)
            .ok_or(ProviderError::Protocol)?;
        if let Some(usage) = value.get("usage").filter(|v| !v.is_null()) {
            serde_json::from_value::<OpenAiUsage>(usage.clone())
                .map_err(|_| ProviderError::Protocol)?;
            self.usage = Some(usage.clone());
        }
        if choices.is_empty() {
            return if self.finish_reason.is_some() && self.usage.is_some() {
                Ok(())
            } else {
                Err(ProviderError::Protocol)
            };
        }
        if choices.len() != 1 || self.finish_reason.is_some() || choices[0]["index"] != 0 {
            return Err(ProviderError::Protocol);
        }
        let choice = &choices[0];
        let delta = choice
            .get("delta")
            .and_then(Value::as_object)
            .ok_or(ProviderError::Protocol)?;
        if let Some(role) = delta.get("role") {
            if role != "assistant" {
                return Err(ProviderError::Protocol);
            }
            self.role_seen = true;
        }
        if let Some(text) = delta.get("content").filter(|v| !v.is_null()) {
            let text = text.as_str().ok_or(ProviderError::Protocol)?;
            if self.content.len() + text.len() > 16 * 1024 || text.contains('\0') {
                return Err(ProviderError::Protocol);
            }
            self.content.push_str(text);
            observer(text);
        }
        if let Some(calls) = delta.get("tool_calls").filter(|v| !v.is_null()) {
            for call in calls.as_array().ok_or(ProviderError::Protocol)? {
                let index = call
                    .get("index")
                    .and_then(Value::as_u64)
                    .filter(|n| *n < 4)
                    .ok_or(ProviderError::Protocol)?;
                let entry = self.calls.entry(index).or_insert_with(
                    || serde_json::json!({"id":"","type":"","function":{"name":"","arguments":""}}),
                );
                for (field, bound) in [("id", MAX_TOOL_CALL_ID_BYTES), ("type", 16)] {
                    append(&mut entry[field], call.get(field), bound)?;
                }
                if let Some(function) = call.get("function") {
                    if !function.is_object() {
                        return Err(ProviderError::Protocol);
                    }
                    append(
                        &mut entry["function"]["name"],
                        function.get("name"),
                        MAX_TOOL_NAME_BYTES,
                    )?;
                    append(
                        &mut entry["function"]["arguments"],
                        function.get("arguments"),
                        MAX_TOOL_ARGUMENT_BYTES,
                    )?;
                }
            }
        }
        if let Some(reason) = choice.get("finish_reason").filter(|v| !v.is_null()) {
            self.finish_reason = Some(reason.as_str().ok_or(ProviderError::Protocol)?.to_owned());
        }
        Ok(())
    }
    fn finish(self) -> Result<ProviderTurn, ProviderError> {
        if !self.done
            || !self.role_seen
            || self.calls.keys().copied().ne(0..self.calls.len() as u64)
        {
            return Err(ProviderError::Protocol);
        }
        let wire = serde_json::json!({
            "choices":[{"finish_reason":self.finish_reason, "message":{
                "role":"assistant", "content":self.content, "tool_calls":self.calls.into_values().collect::<Vec<_>>()
            }}], "usage":self.usage
        });
        parse_wire_response(serde_json::from_value(wire).map_err(|_| ProviderError::Protocol)?)
    }
}
fn append(target: &mut Value, fragment: Option<&Value>, bound: usize) -> Result<(), ProviderError> {
    if let Some(fragment) = fragment {
        let fragment = fragment.as_str().ok_or(ProviderError::Protocol)?;
        let current = target.as_str().ok_or(ProviderError::Protocol)?;
        if current.len() + fragment.len() > bound || fragment.contains('\0') {
            return Err(ProviderError::Protocol);
        }
        *target = Value::String(format!("{current}{fragment}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn event(
        parser: &mut StreamParser,
        value: Value,
        seen: &mut String,
    ) -> Result<(), ProviderError> {
        parser.event(&value.to_string(), &mut |text| seen.push_str(text))
    }
    #[test]
    fn streaming_text_is_incremental_and_requires_valid_terminal() {
        let mut parser = StreamParser::default();
        let mut seen = String::new();
        event(&mut parser, json!({"choices":[{"index":0,"delta":{"role":"assistant","content":"hello "},"finish_reason":null}]}), &mut seen).expect("controlled execution fixture");
        assert_eq!(seen, "hello ");
        event(
            &mut parser,
            json!({"choices":[{"index":0,"delta":{"content":"world"},"finish_reason":"stop"}]}),
            &mut seen,
        )
        .expect("controlled execution fixture");
        parser
            .event("[DONE]", &mut |_| {})
            .expect("controlled execution fixture");
        assert_eq!(
            parser
                .finish()
                .expect("controlled execution fixture")
                .content
                .as_deref(),
            Some("hello world")
        );
        let mut incomplete = StreamParser::default();
        event(&mut incomplete, json!({"choices":[{"index":0,"delta":{"role":"assistant","content":"partial"},"finish_reason":"length"}]}), &mut String::new()).expect("controlled execution fixture");
        incomplete
            .event("[DONE]", &mut |_| {})
            .expect("controlled execution fixture");
        assert_eq!(incomplete.finish(), Err(ProviderError::Protocol));
    }
    #[test]
    fn streaming_tool_fragments_are_not_executable_until_complete() {
        let mut parser = StreamParser::default();
        let mut seen = String::new();
        event(&mut parser,json!({"choices":[{"index":0,"delta":{"role":"assistant","tool_calls":[{"index":0,"id":"call-a","type":"function","function":{"name":"simple_calendar_","arguments":"{\"action\":"}}]},"finish_reason":null}]}), &mut seen).expect("controlled execution fixture");
        event(&mut parser,json!({"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"name":"items","arguments":"\"list\"}"}}]},"finish_reason":"tool_calls"}]}), &mut seen).expect("controlled execution fixture");
        parser
            .event("[DONE]", &mut |_| {})
            .expect("controlled execution fixture");
        let turn = parser.finish().expect("controlled execution fixture");
        assert_eq!(turn.tool_calls[0].name, "simple_calendar_items");
        assert_eq!(turn.tool_calls[0].arguments, "{\"action\":\"list\"}");
        assert!(seen.is_empty());
        let mut missing = StreamParser::default();
        event(&mut missing,json!({"choices":[{"index":0,"delta":{"role":"assistant","content":"a"},"finish_reason":null}]}), &mut seen).expect("controlled execution fixture");
        assert_eq!(missing.finish(), Err(ProviderError::Protocol));
    }
    #[test]
    fn streaming_sse_handles_crlf_multiline_and_rejects_role_choice_and_oversize() {
        let mut parser = StreamParser::default();
        let mut seen = String::new();
        for line in [
            "data: {\"choices\":[",
            "data: {\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"ok\"},\"finish_reason\":\"stop\"}]}",
            "",
            "data: [DONE]",
            "",
        ] {
            parser
                .line(line, &mut |s| seen.push_str(s))
                .expect("controlled execution fixture");
        }
        assert_eq!(
            parser
                .finish()
                .expect("controlled execution fixture")
                .content
                .as_deref(),
            Some("ok")
        );
        for value in [
            json!({"choices":[{"index":1,"delta":{"role":"assistant"},"finish_reason":null}]}),
            json!({"choices":[{"index":0,"delta":{"role":"user"},"finish_reason":null}]}),
            json!({"choices":[{"index":0,"delta":{"role":"assistant","content":"x".repeat(16385)},"finish_reason":null}]}),
        ] {
            assert_eq!(
                event(&mut StreamParser::default(), value, &mut String::new()),
                Err(ProviderError::Protocol)
            );
        }
    }
}

#[cfg(all(test, unix))]
mod transport_tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        os::unix::fs::PermissionsExt,
    };
    #[tokio::test]
    async fn streaming_real_http_delivers_delta_before_peer_releases_final_frame() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("controlled execution fixture")
            .as_nanos();
        let key =
            std::env::temp_dir().join(format!("uca-stream-key-{}-{nonce}", std::process::id()));
        std::fs::write(&key, "synthetic-test-only").expect("controlled execution fixture");
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600))
            .expect("controlled execution fixture");
        let listener = TcpListener::bind("127.0.0.1:0").expect("controlled execution fixture");
        let address = listener.local_addr().expect("controlled execution fixture");
        let (ack, received) = std::sync::mpsc::channel();
        let peer = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("controlled execution fixture");
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("controlled execution fixture");
            let mut input = Vec::new();
            let mut chunk = [0; 4096];
            loop {
                let n = stream
                    .read(&mut chunk)
                    .expect("controlled execution fixture");
                assert!(n > 0);
                input.extend_from_slice(&chunk[..n]);
                if let Some(end) = input.windows(4).position(|v| v == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&input[..end]);
                    let length = header
                        .lines()
                        .filter_map(|s| s.split_once(':'))
                        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .expect("controlled execution fixture")
                        .1
                        .trim()
                        .parse::<usize>()
                        .expect("controlled execution fixture");
                    if input.len() >= end + 4 + length {
                        let wire: Value = serde_json::from_slice(&input[end + 4..end + 4 + length])
                            .expect("controlled execution fixture");
                        assert_eq!(wire["stream"], true);
                        break;
                    }
                }
            }
            write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {{\"choices\":[{{\"index\":0,\"delta\":{{\"role\":\"assistant\",\"content\":\"first \"}},\"finish_reason\":null}}]}}\r\n\r\n").expect("controlled execution fixture");
            stream.flush().expect("controlled execution fixture");
            received
                .recv_timeout(Duration::from_secs(3))
                .expect("client saw real first delta before final exists");
            write!(stream,"data: {{\"choices\":[{{\"index\":0,\"delta\":{{\"content\":\"second\"}},\"finish_reason\":\"stop\"}}]}}\n\ndata: [DONE]\n\n").expect("controlled execution fixture");
        });
        let provider = ChatProvider::openai_compatible_for_test(
            &format!("http://{address}/v1"),
            "synthetic-model",
            &key,
            5000,
        )
        .expect("controlled execution fixture");
        let mut seen = String::new();
        let answer = provider
            .complete_observed(
                &ProviderRequest {
                    messages: vec![ProviderMessage::User {
                        content: "hello".into(),
                    }],
                    tools: vec![],
                },
                &mut |text| {
                    seen.push_str(text);
                    if text == "first " {
                        ack.send(()).expect("controlled execution fixture");
                    }
                },
            )
            .await
            .expect("controlled execution fixture");
        assert_eq!(seen, "first second");
        assert_eq!(answer.content.as_deref(), Some("first second"));
        peer.join().expect("controlled execution fixture");
        std::fs::remove_file(key).expect("controlled execution fixture");
    }
}
