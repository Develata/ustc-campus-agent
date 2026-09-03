//! Unit tests for the bounded OpenAI-compatible Responses adapter.

use super::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::{self, JoinHandle};

struct FakeReply {
    status: &'static str,
    body: String,
}

struct CapturedRequest {
    request_line: String,
    headers: String,
    body: Value,
}

fn response_with(output: Value) -> String {
    json!({
        "id": "resp-test",
        "object": "response",
        "status": "completed",
        "model": "model-test",
        "output": [output],
    })
    .to_string()
}

fn text_output(text: &str) -> Value {
    json!({
        "type": "message",
        "id": "msg-test",
        "status": "completed",
        "role": "assistant",
        "content": [{
            "type": "output_text",
            "text": text,
            "annotations": [],
        }],
    })
}

fn function_output() -> Value {
    json!({
        "type": "function_call",
        "id": "fc-test",
        "call_id": "call-test",
        "status": "completed",
        "name": USTC_AFFAIRS_LOOKUP_TOOL,
        "arguments": r#"{"procedure_id":"proc-011"}"#,
    })
}

fn initial_request() -> ModelInvocationRequest {
    ModelInvocationRequest::Initial {
        developer_instruction: "Bounded developer instruction".to_owned(),
        user_message: "Where is the procedure?".to_owned(),
        tools: vec![
            ustc_campus_agent_runtime::chat::ModelToolDefinition {
                name: USTC_AFFAIRS_LOOKUP_TOOL.to_owned(),
                description: "Lookup".to_owned(),
                parameters_json: r#"{"type":"object"}"#.to_owned(),
                strict: true,
            },
            ustc_campus_agent_runtime::chat::ModelToolDefinition {
                name: USTC_COURSE_ADVICE_TOOL.to_owned(),
                description: "Advice".to_owned(),
                parameters_json: r#"{"type":"object"}"#.to_owned(),
                strict: true,
            },
        ],
        max_output_tokens: MAX_CHAT_OUTPUT_TOKENS,
    }
}

fn continuation_request(
    prior_response_id: String,
    tool_call: ModelToolCall,
    output_json: &str,
) -> ModelInvocationRequest {
    let ModelInvocationRequest::Initial {
        developer_instruction,
        user_message,
        tools,
        ..
    } = initial_request()
    else {
        unreachable!("initial_request always builds the initial variant")
    };
    ModelInvocationRequest::ToolContinuation {
        prior_response_id,
        developer_instruction,
        user_message,
        tool_output: ustc_campus_agent_runtime::chat::ModelToolOutput {
            call_id: tool_call.call_id.clone(),
            name: tool_call.name.clone(),
            output_json: output_json.to_owned(),
        },
        tool_call,
        tools,
        max_output_tokens: MAX_CHAT_OUTPUT_TOKENS,
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("test runtime")
}

fn spawn_fake_server(replies: Vec<FakeReply>) -> (String, JoinHandle<Vec<CapturedRequest>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake provider");
    let address = listener.local_addr().expect("fake provider address");
    let handle = thread::spawn(move || {
        let mut captured = Vec::new();
        for reply in replies {
            let (mut stream, _) = listener.accept().expect("accept provider request");
            let request = read_request(&mut stream);
            let wire = format!(
                "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply.status,
                reply.body.len(),
                reply.body
            );
            stream
                .write_all(wire.as_bytes())
                .expect("write provider response");
            stream.flush().expect("flush provider response");
            captured.push(request);
        }
        captured
    });
    (format!("http://{address}"), handle)
}

fn read_request(stream: &mut TcpStream) -> CapturedRequest {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4_096];
    let header_end = loop {
        let read = stream.read(&mut buffer).expect("read provider request");
        assert!(read > 0, "provider request ended before headers");
        bytes.extend_from_slice(&buffer[..read]);
        assert!(bytes.len() <= 128 * 1_024, "provider request too large");
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = std::str::from_utf8(&bytes[..header_end])
        .expect("UTF-8 request headers")
        .to_owned();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().expect("content length"))
        })
        .expect("request content length");
    while bytes.len() < header_end + content_length {
        let read = stream
            .read(&mut buffer)
            .expect("read provider request body");
        assert!(read > 0, "provider request body ended early");
        bytes.extend_from_slice(&buffer[..read]);
    }
    let request_line = headers.lines().next().expect("request line").to_owned();
    let body = serde_json::from_slice(&bytes[header_end..header_end + content_length])
        .expect("JSON request body");
    CapturedRequest {
        request_line,
        headers,
        body,
    }
}

fn adapter(base_url: String, secret: &str) -> OpenAiResponsesAdapter {
    let config = OpenAiResponsesConfig::new(base_url, secret, "model-test", Some(5))
        .expect("valid test config");
    OpenAiResponsesAdapter::new(config).expect("test adapter")
}

#[test]
fn direct_answer_uses_bounded_initial_responses_wire() {
    let (base_url, server) = spawn_fake_server(vec![FakeReply {
        status: "200 OK",
        body: response_with(text_output("Direct answer")),
    }]);
    let adapter = adapter(base_url, "synthetic-test-key");

    let response = runtime()
        .block_on(adapter.invoke(initial_request()))
        .expect("direct provider response");
    assert_eq!(
        response.outputs,
        vec![ModelOutput::Text("Direct answer".to_owned())]
    );
    assert_eq!(response.response_id, "resp-test");
    assert_eq!(response.model, "model-test");

    let captured = server.join().expect("fake provider thread");
    let [request] = captured.as_slice() else {
        panic!("one provider request expected");
    };
    assert_eq!(request.request_line, "POST /v1/responses HTTP/1.1");
    let authorization = request
        .headers
        .lines()
        .find(|line| {
            line.to_ascii_lowercase()
                .starts_with("authorization: bearer ")
        })
        .expect("authorization header");
    assert!(authorization.len() > "authorization: bearer ".len());
    assert_eq!(request.body["model"], "model-test");
    assert_eq!(request.body["store"], false);
    assert_eq!(request.body["parallel_tool_calls"], false);
    assert_eq!(request.body["max_output_tokens"], MAX_CHAT_OUTPUT_TOKENS);
    assert_eq!(request.body["input"].as_array().map(Vec::len), Some(2));
    assert_eq!(request.body["input"][0]["role"], "developer");
    assert_eq!(request.body["input"][1]["role"], "user");
    assert_eq!(request.body["tools"].as_array().map(Vec::len), Some(2));
    assert_eq!(request.body["tools"][0]["strict"], true);
    assert_eq!(request.body["tools"][1]["strict"], true);
}

#[test]
fn tool_call_and_continuation_preserve_response_and_call_correlation() {
    let (base_url, server) = spawn_fake_server(vec![
        FakeReply {
            status: "200 OK",
            body: response_with(function_output()),
        },
        FakeReply {
            status: "200 OK",
            body: response_with(text_output("Grounded answer")),
        },
    ]);
    let adapter = adapter(base_url, "synthetic-test-key");
    let runtime = runtime();

    let first = runtime
        .block_on(adapter.invoke(initial_request()))
        .expect("function call response");
    let [ModelOutput::ToolCall(call)] = first.outputs.as_slice() else {
        panic!("one function call expected");
    };
    assert_eq!(call.call_id, "call-test");
    assert_eq!(call.name, USTC_AFFAIRS_LOOKUP_TOOL);

    let second = runtime
        .block_on(adapter.invoke(continuation_request(
            first.response_id,
            call.clone(),
            r#"{"status":"found"}"#,
        )))
        .expect("continuation response");
    assert_eq!(
        second.outputs,
        vec![ModelOutput::Text("Grounded answer".to_owned())]
    );

    let captured = server.join().expect("fake provider thread");
    let [initial, continuation] = captured.as_slice() else {
        panic!("two provider requests expected");
    };
    assert!(initial.body.get("previous_response_id").is_none());
    assert!(continuation.body.get("previous_response_id").is_none());
    assert_eq!(continuation.body["input"].as_array().map(Vec::len), Some(4));
    assert_eq!(continuation.body["input"][2]["type"], "function_call");
    assert_eq!(
        continuation.body["input"][3]["type"],
        "function_call_output"
    );
    assert_eq!(continuation.body["input"][2]["call_id"], "call-test");
    assert_eq!(continuation.body["input"][3]["call_id"], "call-test");
    assert_eq!(
        continuation.body["input"][3]["output"],
        r#"{"status":"found"}"#
    );
    assert_eq!(continuation.body["tools"].as_array().map(Vec::len), Some(2));
}

#[test]
fn invalid_origins_and_bounded_configuration_fail_closed() {
    for invalid in [
        "http://example.com",
        "http://localhost:8080",
        "http://127.0.0.2:8080",
        "https://user@example.com",
        "https://example.com/base",
        "https://example.com?query=1",
        "https://example.com#fragment",
        "relative.example.com",
    ] {
        assert!(matches!(
            OpenAiResponsesConfig::new(invalid, "key", "model", None),
            Err(OpenAiResponsesConfigError::InvalidBaseUrl)
        ));
    }
    assert!(OpenAiResponsesConfig::new("https://example.com", "key", "model", None).is_ok());
    assert!(OpenAiResponsesConfig::new("http://127.0.0.1:8080", "key", "model", None).is_ok());
    assert!(OpenAiResponsesConfig::new("http://[::1]:8080", "key", "model", None).is_ok());
    assert!(matches!(
        OpenAiResponsesConfig::new("https://example.com", "key", "model", Some(0)),
        Err(OpenAiResponsesConfigError::InvalidTimeout)
    ));
    assert!(matches!(
        OpenAiResponsesConfig::new("https://example.com", "key", "model", Some(121)),
        Err(OpenAiResponsesConfigError::InvalidTimeout)
    ));
    assert!(matches!(
        OpenAiResponsesConfig::new("https://example.com", "", "model", None),
        Err(OpenAiResponsesConfigError::InvalidApiKey)
    ));
    assert!(matches!(
        OpenAiResponsesConfig::new("https://example.com", "key", " model ", None),
        Err(OpenAiResponsesConfigError::InvalidModel)
    ));
}

#[test]
fn malformed_or_mixed_provider_response_is_rejected() {
    let malformed = json!({
        "id": "resp-test",
        "status": "completed",
        "model": "model-test",
        "output": [text_output("answer"), function_output()],
    })
    .to_string();
    let (base_url, server) = spawn_fake_server(vec![FakeReply {
        status: "200 OK",
        body: malformed,
    }]);
    let adapter = adapter(base_url, "synthetic-test-key");
    let error = runtime()
        .block_on(adapter.invoke(initial_request()))
        .expect_err("mixed output must fail");
    assert_eq!(error, ModelInvocationError::MalformedResponse);
    let captured = server.join().expect("fake provider thread");
    assert_eq!(captured.len(), 1);
}

#[test]
fn provider_model_mismatch_is_rejected() {
    let mut mismatched: Value =
        serde_json::from_str(&response_with(text_output("answer"))).expect("response JSON");
    mismatched["model"] = json!("unexpected-model");
    let (base_url, server) = spawn_fake_server(vec![FakeReply {
        status: "200 OK",
        body: mismatched.to_string(),
    }]);
    let adapter = adapter(base_url, "synthetic-test-key");
    let error = runtime()
        .block_on(adapter.invoke(initial_request()))
        .expect_err("provider model mismatch must fail");
    assert_eq!(error, ModelInvocationError::MalformedResponse);
    assert_eq!(server.join().expect("fake provider thread").len(), 1);
}

#[test]
fn secret_url_model_and_provider_body_never_render_in_errors_or_debug() {
    let secret = "synthetic-secret-never-render";
    let provider_body = "provider-body-never-render";
    let (base_url, server) = spawn_fake_server(vec![FakeReply {
        status: "500 Internal Server Error",
        body: provider_body.to_owned(),
    }]);
    let config =
        OpenAiResponsesConfig::new(base_url.clone(), secret, "model-never-render", Some(5))
            .expect("valid config");
    let config_debug = format!("{config:?}");
    let adapter = OpenAiResponsesAdapter::new(config).expect("test adapter");
    let adapter_debug = format!("{adapter:?}");
    let error = runtime()
        .block_on(adapter.invoke(initial_request()))
        .expect_err("HTTP failure");
    let rendered = format!("{error:?} {error}");
    for forbidden in [
        secret,
        base_url.as_str(),
        "model-never-render",
        provider_body,
    ] {
        assert!(!config_debug.contains(forbidden));
        assert!(!adapter_debug.contains(forbidden));
        assert!(!rendered.contains(forbidden));
    }
    let captured = server.join().expect("fake provider thread");
    assert_eq!(captured.len(), 1);
}
