use super::*;

fn object(node: Value) -> Value {
    json!({"type":"object","properties":{"value":node},"required":["value"],"additionalProperties":false})
}

#[test]
fn scalar_constraints_compile_and_validate_input_and_output() {
    for (node, accepted, rejected) in [
        (
            json!({"type":"string","minLength":1,"maxLength":1}),
            json!("😀"),
            json!("e\u{301}"),
        ),
        (
            json!({"type":"integer","minimum":-2,"maximum":2}),
            json!(2),
            json!(3),
        ),
        (
            json!({"type":"number","minimum":0.5,"maximum":2.5}),
            json!(2.5),
            json!(2.6),
        ),
    ] {
        let schema = object(node);
        let compiled = compile_schema(&schema).expect("bounded schema compiles");
        assert!(validate_arguments(&compiled, &json!({"value":accepted})).is_ok());
        assert_eq!(
            validate_arguments(&compiled, &json!({"value":rejected})),
            Err(McpError::InvalidArguments)
        );
        assert!(validate_output(&schema, &json!({"value":accepted})).is_ok());
        assert_eq!(
            validate_output(&schema, &json!({"value":rejected})),
            Err(McpError::InvalidSchema)
        );
    }
    let number = object(json!({"type":"number","maximum":9007199254740992_u64}));
    assert!(validate_output(&number, &json!({"value":9007199254740992_u64})).is_ok());
    assert!(validate_output(&number, &json!({"value":9007199254740993_u64})).is_err());
    let integer = object(json!({"type":"integer","maximum":i64::MAX}));
    assert!(validate_output(&integer, &json!({"value":i64::MAX})).is_ok());
    assert!(validate_output(&integer, &json!({"value":10.0})).is_ok());
    assert!(validate_output(&integer, &json!({"value":9223372036854775808.0})).is_err());
    assert!(validate_output(&integer, &json!({"value":u64::MAX})).is_err());
    assert_eq!(
        validate_arguments(
            &compile_schema(&integer).expect("integer"),
            &json!({"value":10.0})
        ),
        Err(McpError::InvalidArguments)
    );
    assert_eq!(
        validate_arguments(
            &compile_schema(&number).expect("number"),
            &json!({"value":10})
        ),
        Err(McpError::InvalidArguments)
    );
}

#[test]
fn scalar_constraints_invalid_keywords_types_and_lossy_thresholds_reject() {
    for node in [
        json!({"type":"integer","minimum":1.5}),
        json!({"type":"integer","minimum":1.0}),
        json!({"type":"integer","minimum":u64::MAX}),
        json!({"type":"integer","minimum":2,"maximum":1}),
        json!({"type":"number","minimum":9007199254740993_u64}),
        json!({"type":"number","maximum":i64::MAX}),
        json!({"type":"number","minimum":null}),
        json!({"type":"number","maximum":"3"}),
        json!({"type":"string","minLength":-1}),
        json!({"type":"string","maxLength":2.0}),
        json!({"type":"string","minLength":3,"maxLength":2}),
        json!({"type":"string","minimum":1}),
        json!({"type":"number","exclusiveMinimum":1}),
        json!({"type":"number","multipleOf":0.5}),
        json!({"type":"string","pattern":"x"}),
        json!({"type":"integer","minLength":1}),
    ] {
        assert_eq!(
            compile_schema(&object(node)).err(),
            Some(McpError::InvalidSchema)
        );
    }
    assert!(compile_schema(&object(json!({"type":"string","maxLength":u64::MAX}))).is_ok());
    assert!(compile_schema(&object(json!({"type":"number","minimum":i64::MIN}))).is_err());
    for raw in [
        r#"{"type":"number","minimum":18446744073709551617}"#,
        r#"{"type":"number","maximum":-18446744073709551617}"#,
        r#"{"type":"number","maximum":-9223372036854775809}"#,
    ] {
        let parsed: Value = serde_json::from_str(raw).expect("wire JSON");
        assert_eq!(
            compile_schema(&object(parsed)).err(),
            Some(McpError::InvalidSchema)
        );
    }
}

#[test]
fn scalar_constraints_wire_reject_before_call_and_require_review_after_drift() {
    let mut definition = tool("bounded");
    definition["inputSchema"] = object(json!({"type":"string","minLength":1,"maxLength":2}));
    definition["outputSchema"] = object(json!({"type":"integer","minimum":1,"maximum":3}));
    let mut changed = definition.clone();
    changed["inputSchema"]["properties"]["value"]["maxLength"] = json!(1);
    let peer = Peer::start(vec![
        ("initialize", init()),
        ("notifications/initialized", Reply::accepted()),
        ("tools/list", Reply::rpc(json!({"tools":[definition]}))),
        (
            "tools/call",
            Reply::rpc(json!({"content":[],"structuredContent":{"value":3.0}})),
        ),
        ("tools/list", Reply::rpc(json!({"tools":[changed]}))),
        (
            "tools/call",
            Reply::rpc(json!({"content":[],"structuredContent":{"value":4}})).sse(),
        ),
    ]);
    runtime().block_on(async {
        let mut client = McpClient::new(config(&peer.endpoint), Limits::default()).expect("client");
        let old = client.discover().await.expect("bounded inventory");
        client.activate_reviewed(old.digest()).expect("review");
        assert_eq!(
            client
                .call_tool(old.digest(), "bounded", &json!({"value":"abc"}))
                .await,
            Err(McpError::InvalidArguments)
        );
        client
            .call_tool(old.digest(), "bounded", &json!({"value":"😀"}))
            .await
            .expect("valid scalar bounds");
        assert_eq!(client.discover().await.err(), Some(McpError::SchemaDrift));
        assert_eq!(client.state(), BindingState::Quarantined);
        assert_eq!(
            client
                .call_tool(old.digest(), "bounded", &json!({"value":"x"}))
                .await,
            Err(McpError::NotActive)
        );
        let new = client.inventory().expect("changed inventory").clone();
        assert_ne!(old.digest(), new.digest());
        assert_ne!(
            old.tools()[0].compiled_schema().digest(),
            new.tools()[0].compiled_schema().digest()
        );
        client.activate_reviewed(new.digest()).expect("new review");
        assert_eq!(
            client
                .call_tool(new.digest(), "bounded", &json!({"value":"x"}))
                .await,
            Err(McpError::InvalidSchema)
        );
        assert_eq!(client.state(), BindingState::Quarantined);
    });
    let requests = peer.finish();
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.body["method"] == "tools/call")
            .count(),
        2,
        "invalid input and old readiness perform no business call"
    );
}
