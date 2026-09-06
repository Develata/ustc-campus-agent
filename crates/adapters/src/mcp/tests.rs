use super::*;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

#[derive(Clone, Debug)]
struct Captured {
    method: String,
    headers: String,
    body: Value,
}
struct Reply {
    result: Value,
    status: u16,
    headers: Vec<(String, String)>,
    sse: bool,
    raw: Option<String>,
    id_delta: u64,
    delay: Duration,
}
impl Reply {
    fn rpc(result: Value) -> Self {
        Self {
            result,
            status: 200,
            headers: vec![],
            sse: false,
            raw: None,
            id_delta: 0,
            delay: Duration::ZERO,
        }
    }
    fn accepted() -> Self {
        Self {
            status: 202,
            raw: Some(String::new()),
            ..Self::rpc(Value::Null)
        }
    }
    fn status(status: u16) -> Self {
        Self {
            status,
            raw: Some(String::new()),
            ..Self::rpc(Value::Null)
        }
    }
    fn session(mut self, value: &str) -> Self {
        self.headers.push(("MCP-Session-Id".into(), value.into()));
        self
    }
    fn sse(mut self) -> Self {
        self.sse = true;
        self
    }
}
struct Peer {
    endpoint: String,
    requests: Arc<Mutex<Vec<Captured>>>,
    worker: thread::JoinHandle<()>,
}
impl Peer {
    fn start(steps: Vec<(&'static str, Reply)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind controlled peer");
        listener.set_nonblocking(true).expect("nonblocking accept");
        let endpoint = format!(
            "http://{}/mcp",
            listener.local_addr().expect("local address")
        );
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&requests);
        let worker = thread::spawn(move || {
            for (expected, reply) in steps {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && Instant::now() < deadline =>
                        {
                            thread::sleep(Duration::from_millis(2))
                        }
                        Err(error) => panic!("controlled peer missing request {expected}: {error}"),
                    }
                };
                let captured = read_request(&mut stream);
                if expected == "DELETE" {
                    assert_eq!(captured.method, "DELETE");
                } else {
                    assert_eq!(captured.method, "POST");
                    assert_eq!(captured.body["method"], expected);
                }
                let id = captured.body["id"].as_u64().unwrap_or(0) + reply.id_delta;
                observed.lock().expect("requests").push(captured);
                if !reply.delay.is_zero() {
                    thread::sleep(reply.delay);
                }
                let envelope = json!({"jsonrpc":"2.0", "id":id, "result":reply.result});
                let body = reply.raw.unwrap_or_else(|| if reply.sse {
                    format!("id: prime\r\ndata:\r\n\r\n: keepalive\n\ndata: {}\n\ndata: {envelope}\n\n",
                        json!({"jsonrpc":"2.0", "method":"notifications/progress", "params":{"progress":1,"progressToken":"fixture"}}))
                } else { envelope.to_string() });
                let extra = reply
                    .headers
                    .into_iter()
                    .map(|(key, value)| format!("{key}: {value}\r\n"))
                    .collect::<String>();
                let response = format!(
                    "HTTP/1.1 {} Synthetic\r\nContent-Type: {}\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{body}",
                    reply.status,
                    if reply.sse {
                        "text/event-stream"
                    } else {
                        "application/json"
                    },
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        Self {
            endpoint,
            requests,
            worker,
        }
    }
    fn finish(self) -> Vec<Captured> {
        self.worker
            .join()
            .expect("controlled peer completed all expected requests");
        self.requests.lock().expect("requests").clone()
    }
}
fn read_request(stream: &mut TcpStream) -> Captured {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("read timeout");
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    let end = loop {
        let count = stream.read(&mut buffer).expect("request bytes");
        assert!(count > 0, "request header complete");
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(index) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
            break index + 4;
        }
        assert!(bytes.len() < 32 * 1024);
    };
    let headers = String::from_utf8(bytes[..end].to_vec())
        .expect("request headers")
        .to_ascii_lowercase();
    let length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length:").map(str::trim))
        .unwrap_or("0")
        .parse::<usize>()
        .expect("content length");
    while bytes.len() < end + length {
        let count = stream.read(&mut buffer).expect("body");
        assert!(count > 0);
        bytes.extend_from_slice(&buffer[..count]);
    }
    Captured {
        method: headers
            .split_whitespace()
            .next()
            .expect("method")
            .to_ascii_uppercase(),
        headers,
        body: if length == 0 {
            Value::Null
        } else {
            serde_json::from_slice(&bytes[end..end + length]).expect("JSON request")
        },
    }
}
fn config(endpoint: &str) -> BindingConfig {
    BindingConfig {
        binding_id: "binding:synthetic".into(),
        owner_id: "tenant:synthetic/user:synthetic".into(),
        installation_id: "installation:synthetic".into(),
        component_id: "component:synthetic".into(),
        endpoint: endpoint.into(),
        endpoint_policy: EndpointPolicy::LoopbackDevelopment,
        bearer_token: None,
    }
}
fn init() -> Reply {
    Reply::rpc(
        json!({"protocolVersion":PROTOCOL_VERSION, "capabilities":{"tools":{}}, "serverInfo":{"name":"synthetic-peer", "version":"1"}}),
    )
}
fn tool(name: &str) -> Value {
    json!({"name":name, "description":"Synthetic read-only tool", "inputSchema":{"type":"object", "properties":{"query":{"type":"string"}}, "required":["query"],"additionalProperties":false}})
}
fn list(name: &str) -> Reply {
    Reply::rpc(json!({"tools":[tool(name)]}))
}
fn successful() -> Reply {
    Reply::rpc(
        json!({"content":[{"type":"text", "text":"synthetic result"}], "structuredContent":{"value":"synthetic"}, "isError":false}),
    )
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("runtime")
}

#[test]
fn real_http_discovery_paging_sse_call_and_close_preserve_protocol() {
    let peer = Peer::start(vec![
        ("initialize", init().session("session-a")),
        ("notifications/initialized", Reply::accepted()),
        (
            "tools/list",
            Reply::rpc(json!({"tools":[tool("lookup")],"nextCursor":"page-2"})),
        ),
        (
            "tools/list",
            Reply::rpc(json!({"tools":[tool("search")]})).sse(),
        ),
        ("tools/call", successful().sse()),
        ("DELETE", Reply::status(204)),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
        let inventory = client.discover().await.expect("complete discovery");
        assert_eq!(inventory.tools().len(), 2);
        assert_eq!(client.state(), BindingState::ToolsDiscovered);
        assert_eq!(
            client
                .call_tool(inventory.digest(), "lookup", &json!({"query":"hi"}))
                .await,
            Err(McpError::NotActive)
        );
        client
            .activate_reviewed(inventory.digest())
            .expect("explicit review");
        assert_eq!(
            client
                .call_tool(inventory.digest(), "missing", &json!({}))
                .await,
            Err(McpError::UnknownTool)
        );
        assert_eq!(
            client
                .call_tool(inventory.digest(), "lookup", &json!({"query":1}))
                .await,
            Err(McpError::InvalidArguments)
        );
        let output = client
            .call_tool(inventory.digest(), "lookup", &json!({"query":"hi"}))
            .await
            .expect("business call");
        assert_eq!(output.text, vec!["synthetic result"]);
        assert_eq!(
            output.structured_content,
            Some(json!({"value":"synthetic"}))
        );
        client.close().await.expect("close");
        assert_eq!(client.state(), BindingState::Retired);
        assert_eq!(client.discover().await.err(), Some(McpError::Retired));
    });
    let requests = peer.finish();
    assert_eq!(requests.len(), 6);
    assert!(!requests[0].headers.contains("mcp-session-id"));
    assert_eq!(requests[0].body["params"]["capabilities"], json!({}));
    for request in &requests[1..] {
        assert!(request.headers.contains("mcp-session-id: session-a"));
        assert!(request.headers.contains("mcp-protocol-version: 2025-11-25"));
    }
    assert_eq!(requests[3].body["params"]["cursor"], "page-2");
    assert_eq!(requests[4].body["params"]["name"], "lookup");
}

#[test]
fn discovery_is_not_activation_and_complete_drift_requires_new_review() {
    let peer = Peer::start(vec![
        ("initialize", init()),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("old")),
        ("tools/list", list("new")),
        ("tools/call", successful()),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
        let old = client.discover().await.expect("inventory");
        client.activate_reviewed(old.digest()).expect("review");
        assert_eq!(client.discover().await.err(), Some(McpError::SchemaDrift));
        assert_eq!(client.state(), BindingState::Quarantined);
        assert_eq!(
            client
                .call_tool(old.digest(), "old", &json!({"query":"hi"}))
                .await,
            Err(McpError::NotActive)
        );
        assert_eq!(
            client.activate_reviewed(old.digest()),
            Err(McpError::SnapshotMismatch)
        );
        let new = client.inventory().expect("new inventory").clone();
        client.activate_reviewed(new.digest()).expect("new review");
        client
            .call_tool(new.digest(), "new", &json!({"query":"hi"}))
            .await
            .expect("new exact tool");
    });
    peer.finish();
}

#[test]
fn incomplete_rediscovery_cannot_reactivate_stale_snapshot() {
    let peer = Peer::start(vec![
        ("initialize", init()),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("lookup")),
        (
            "tools/list",
            Reply::rpc(json!({"tools":[],"nextCursor":"same"})),
        ),
        (
            "tools/list",
            Reply::rpc(json!({"tools":[],"nextCursor":"same"})),
        ),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
        let old = client.discover().await.expect("inventory");
        client.activate_reviewed(old.digest()).expect("review");
        assert_eq!(client.discover().await.err(), Some(McpError::Protocol));
        assert_eq!(
            client.activate_reviewed(old.digest()),
            Err(McpError::NotActive)
        );
    });
    peer.finish();
}

#[test]
fn session_expiry_has_no_implicit_business_retry_and_requires_rediscovery() {
    let peer = Peer::start(vec![
        ("initialize", init().session("first")),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("lookup")),
        ("tools/call", Reply::status(404)),
        ("initialize", init().session("second")),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("lookup")),
        ("tools/call", successful()),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
        let inventory = client.discover().await.expect("inventory");
        client
            .activate_reviewed(inventory.digest())
            .expect("review");
        assert_eq!(
            client
                .call_tool(inventory.digest(), "lookup", &json!({"query":"x"}))
                .await,
            Err(McpError::SessionExpired)
        );
        assert_eq!(
            client
                .call_tool(inventory.digest(), "lookup", &json!({"query":"x"}))
                .await,
            Err(McpError::NotActive)
        );
        let inventory = client.discover().await.expect("new session");
        client
            .activate_reviewed(inventory.digest())
            .expect("review");
        client
            .call_tool(
                inventory.digest(),
                "lookup",
                &json!({"query":"new explicit call"}),
            )
            .await
            .expect("explicit call");
    });
    let requests = peer.finish();
    assert_eq!(requests.len(), 8);
    assert!(!requests[4].headers.contains("mcp-session-id"));
    assert!(requests[7].headers.contains("mcp-session-id: second"));
}

#[test]
fn separate_owner_clients_do_not_share_sessions_or_credentials() {
    let peer = Peer::start(vec![
        ("initialize", init().session("owner-a")),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("lookup")),
        ("initialize", init().session("owner-b")),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("lookup")),
        ("tools/call", successful()),
        ("tools/call", successful()),
    ]);
    runtime().block_on(async {
        let mut a = config(&peer.endpoint);
        a.bearer_token = Some("synthetic-a".into());
        let mut b = config(&peer.endpoint);
        b.owner_id = "tenant:b/user:b".into();
        b.bearer_token = Some("synthetic-b".into());
        let mut a = McpClient::new(a, Limits::default()).expect("a");
        let mut b = McpClient::new(b, Limits::default()).expect("b");
        let inventory_a = a.discover().await.expect("a inventory");
        let inventory_b = b.discover().await.expect("b inventory");
        a.activate_reviewed(inventory_a.digest()).expect("a review");
        b.activate_reviewed(inventory_b.digest()).expect("b review");
        a.call_tool(inventory_a.digest(), "lookup", &json!({"query":"a"}))
            .await
            .expect("a call");
        b.call_tool(inventory_b.digest(), "lookup", &json!({"query":"b"}))
            .await
            .expect("b call");
    });
    let requests = peer.finish();
    assert!(!requests[3].headers.contains("mcp-session-id"));
    assert!(requests[6].headers.contains("mcp-session-id: owner-a"));
    assert!(
        requests[6]
            .headers
            .contains("authorization: bearer synthetic-a")
    );
    assert!(requests[7].headers.contains("mcp-session-id: owner-b"));
    assert!(
        requests[7]
            .headers
            .contains("authorization: bearer synthetic-b")
    );
}

#[test]
fn hostile_initialization_is_bounded_and_redacted() {
    let mut wrong_version = init();
    wrong_version.result["protocolVersion"] = json!("other");
    let mut wrong_id = init();
    wrong_id.id_delta = 1;
    let mut malformed = init();
    malformed.raw = Some("synthetic-secret malformed".into());
    let mut oversized = init();
    oversized.raw = Some("x".repeat(1024 * 1024 + 1));
    let mut redirect = Reply::status(302);
    redirect
        .headers
        .push(("Location".into(), "http://127.0.0.1/secret".into()));
    let mut timeout = init();
    timeout.delay = Duration::from_millis(100);
    for (reply, expected) in [
        (wrong_version, McpError::Protocol),
        (wrong_id, McpError::Protocol),
        (malformed, McpError::Protocol),
        (oversized, McpError::LimitExceeded),
        (redirect, McpError::EndpointDenied),
        (Reply::status(401), McpError::Authentication),
        (timeout, McpError::Timeout),
    ] {
        let peer = Peer::start(vec![("initialize", reply)]);
        runtime().block_on(async {
            let limits = Limits {
                request_timeout: if expected == McpError::Timeout {
                    Duration::from_millis(30)
                } else {
                    Duration::from_secs(2)
                },
                ..Limits::default()
            };
            let mut client = McpClient::new(config(&peer.endpoint), limits).expect("client");
            let error = client.discover().await.expect_err("hostile init rejected");
            assert_eq!(error, expected);
            assert!(!error.to_string().contains("secret"));
            assert_eq!(client.state(), BindingState::Quarantined);
        });
        peer.finish();
    }
}

#[test]
fn duplicate_names_page_tool_schema_and_sse_limits_reject_whole_inventory() {
    for (page, limits, expected) in [
        (
            Reply::rpc(json!({"tools":[tool("lookup"),tool("lookup")]})),
            Limits::default(),
            McpError::Protocol,
        ),
        (
            Reply::rpc(json!({"tools":[tool("a"),tool("b")]})),
            Limits {
                max_tools: 1,
                ..Limits::default()
            },
            McpError::LimitExceeded,
        ),
        (
            Reply::rpc(json!({"tools":[],"nextCursor":"more"})),
            Limits {
                max_pages: 1,
                ..Limits::default()
            },
            McpError::LimitExceeded,
        ),
        (
            list("lookup"),
            Limits {
                max_schema_bytes: 8,
                ..Limits::default()
            },
            McpError::LimitExceeded,
        ),
        (
            list("lookup").sse(),
            Limits {
                max_sse_events: 1,
                ..Limits::default()
            },
            McpError::LimitExceeded,
        ),
    ] {
        let peer = Peer::start(vec![
            ("initialize", init()),
            ("notifications/initialized", Reply::accepted()),
            ("tools/list", page),
        ]);
        runtime().block_on(async {
            let mut client = McpClient::new(config(&peer.endpoint), limits).expect("client");
            assert_eq!(client.discover().await.err(), Some(expected));
            assert!(client.inventory().is_none());
        });
        peer.finish();
    }
}

#[test]
fn strict_schema_and_untrusted_output_semantics() {
    let schema = tool("lookup")["inputSchema"].clone();
    let compiled = compile_schema(&schema).expect("admitted schema");
    assert_eq!(
        validate_arguments(&compiled, &json!({"query":"ok", "extra":1})),
        Err(McpError::InvalidArguments)
    );
    for changed in [
        json!({"type":"object","properties":{}}),
        json!({"type":"object","additionalProperties":false,"properties":{"x":{"type":"string","pattern":".*"}}}),
        json!({"type":"object","additionalProperties":false,"$ref":"https://example.com/schema"}),
    ] {
        assert_eq!(
            compile_schema(&changed).err(),
            Some(McpError::InvalidSchema)
        );
    }
    assert_eq!(
        parse_result(
            json!({"content":[{"type":"resource_link","uri":"file:///secret"}]}),
            None
        ),
        Err(McpError::UnsupportedContent)
    );
    assert_eq!(
        parse_result(
            json!({"content":[],"structuredContent":{"query":1}}),
            Some(&schema)
        ),
        Err(McpError::InvalidSchema)
    );
    assert!(
        parse_result(
            json!({"content":[],"structuredContent":{"query":"ok"}}),
            Some(&schema)
        )
        .is_ok()
    );
    assert!(
        parse_result(
            json!({"content":[{"type":"text","text":"untrusted instruction"}],"isError":true}),
            None
        )
        .expect("tool failure data")
        .is_error
    );
}

#[test]
fn schema_digests_ignore_inventory_order_but_include_annotations_and_output_schema() {
    let a = json!({"tools":[tool("a"),tool("b")]});
    let b = json!({"tools":[tool("b"),tool("a")]});
    let mut drift = a.clone();
    drift["tools"][0]["annotations"] = json!({"readOnlyHint":true});
    let peer = Peer::start(vec![
        ("initialize", init()),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", Reply::rpc(a)),
        ("tools/list", Reply::rpc(b)),
        ("tools/list", Reply::rpc(drift)),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
        let first = client.discover().await.expect("first");
        client.activate_reviewed(first.digest()).expect("review");
        let second = client.discover().await.expect("same inventory reordered");
        assert_eq!(first.digest(), second.digest());
        client.activate_reviewed(second.digest()).expect("review");
        assert_eq!(client.discover().await.err(), Some(McpError::SchemaDrift));
    });
    peer.finish();
}

#[test]
fn hostile_business_responses_quarantine_without_replaying_the_call() {
    let mut wrong_id = successful();
    wrong_id.id_delta = 1;
    let mut server_request = successful().sse();
    server_request.raw = Some(format!(
        "data: {}\n\n",
        json!({"jsonrpc":"2.0","id":88,"method":"sampling/createMessage","params":{}})
    ));
    let mut list_changed = successful().sse();
    list_changed.raw = Some(format!(
        "data: {}\n\n",
        json!({"jsonrpc":"2.0","method":"notifications/tools/list_changed"})
    ));
    let mut wrong_header = successful();
    wrong_header
        .headers
        .push(("MCP-Protocol-Version".into(), "other".into()));
    let mut changed_session = successful();
    changed_session
        .headers
        .push(("MCP-Session-Id".into(), "other-owner-session".into()));
    let unsupported =
        Reply::rpc(json!({"content":[{"type":"image","data":"synthetic","mimeType":"image/png"}]}));
    for (reply, expected) in [
        (wrong_id, McpError::Protocol),
        (server_request, McpError::Protocol),
        (list_changed, McpError::SchemaDrift),
        (wrong_header, McpError::Protocol),
        (changed_session, McpError::Protocol),
        (unsupported, McpError::UnsupportedContent),
    ] {
        let peer = Peer::start(vec![
            ("initialize", init().session("session")),
            ("notifications/initialized", Reply::accepted()),
            ("tools/list", list("lookup")),
            ("tools/call", reply),
        ]);
        runtime().block_on(async {
            let mut client =
                McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
            let inventory = client.discover().await.expect("discovery");
            client
                .activate_reviewed(inventory.digest())
                .expect("review");
            assert_eq!(
                client
                    .call_tool(inventory.digest(), "lookup", &json!({"query":"x"}))
                    .await,
                Err(expected)
            );
            assert_eq!(client.state(), BindingState::Quarantined);
            assert_eq!(
                client.activate_reviewed(inventory.digest()),
                Err(McpError::NotActive)
            );
        });
        assert_eq!(peer.finish().len(), 4);
    }
}

#[test]
fn business_timeout_is_reported_once_without_automatic_retry() {
    let mut delayed = successful();
    delayed.delay = Duration::from_millis(100);
    let peer = Peer::start(vec![
        ("initialize", init()),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", list("lookup")),
        ("tools/call", delayed),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(
            config(&peer.endpoint),
            Limits {
                request_timeout: Duration::from_millis(30),
                ..Limits::default()
            },
        )
        .expect("client");
        let inventory = client.discover().await.expect("discovery");
        client
            .activate_reviewed(inventory.digest())
            .expect("review");
        assert_eq!(
            client
                .call_tool(inventory.digest(), "lookup", &json!({"query":"x"}))
                .await,
            Err(McpError::Timeout)
        );
    });
    assert_eq!(peer.finish().len(), 4);
}

#[test]
fn mcp_output_numeric_schema_accepts_json_and_sse_without_weakening_inputs() {
    for sse in [false, true] {
        for (kind, value) in [
            ("number", json!(10)),
            ("integer", json!(10.0)),
            ("number", json!(u64::MAX)),
        ] {
            let output_schema = json!({"type":"object","properties":{"value":{"type":kind}},"required":["value"],"additionalProperties":false});
            let mut definition = tool("lookup");
            definition["outputSchema"] = output_schema;
            let mut reply = Reply::rpc(json!({"content":[],"structuredContent":{"value":value}}));
            reply.sse = sse;
            let peer = Peer::start(vec![
                ("initialize", init()),
                ("notifications/initialized", Reply::accepted()),
                ("tools/list", Reply::rpc(json!({"tools":[definition]}))),
                ("tools/call", reply),
            ]);
            runtime().block_on(async {
                let mut client =
                    McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
                let inventory = client.discover().await.expect("discovery");
                client
                    .activate_reviewed(inventory.digest())
                    .expect("activate");
                let result = client
                    .call_tool(inventory.digest(), "lookup", &json!({"query":"synthetic"}))
                    .await
                    .expect("valid numeric output");
                assert_eq!(result.structured_content, Some(json!({"value":value})));
                assert_eq!(client.state(), BindingState::Active);
            });
            peer.finish();
        }
    }
    for (kind, value) in [
        ("integer", json!(10.5)),
        ("number", json!("10")),
        ("number", Value::Null),
    ] {
        let schema = json!({"type":"object","properties":{"value":{"type":kind}},"required":["value"],"additionalProperties":false});
        assert!(schema::validate_output(&schema, &json!({"value":value})).is_err());
    }
    let input = json!({"type":"object","properties":{"value":{"type":"number"}},"required":["value"],"additionalProperties":false});
    assert!(
        validate_arguments(
            &compile_schema(&input).expect("schema"),
            &json!({"value":10})
        )
        .is_err(),
        "input retains exact platform numeric tags"
    );
    assert!(schema::validate_output(&input, &json!({"value":10,"extra":1})).is_err());
    assert!(schema::validate_output(&input, &json!({})).is_err());
}

#[test]
fn mcp_failed_initial_discovery_releases_session_and_preserves_original_error() {
    for bad_initialize in [false, true] {
        let mut initial = init().session("failed-discovery");
        if bad_initialize {
            initial.result["protocolVersion"] = json!("wrong");
        }
        let mut steps = vec![("initialize", initial)];
        if !bad_initialize {
            steps.push(("notifications/initialized", Reply::accepted()));
            steps.push(("tools/list", Reply::rpc(json!({"tools":false}))));
        }
        // Cleanup errors do not replace the protocol error or revive the session.
        steps.push(("DELETE", Reply::status(500)));
        steps.extend([
            ("initialize", init()),
            ("notifications/initialized", Reply::accepted()),
            ("tools/list", list("lookup")),
        ]);
        let peer = Peer::start(steps);
        runtime().block_on(async {
            let mut client =
                McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
            assert_eq!(client.discover().await.err(), Some(McpError::Protocol));
            assert_eq!(client.state(), BindingState::Quarantined);
            assert!(client.session_id.is_none());
            client.discover().await.expect("explicit rediscovery");
        });
        let requests = peer.finish();
        assert_eq!(requests.iter().filter(|r| r.method == "DELETE").count(), 1);
        assert!(!requests.iter().any(|r| r.body["method"] == "tools/call"));
    }
}
