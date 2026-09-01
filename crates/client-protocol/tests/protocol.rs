#![allow(clippy::unwrap_used)]

//! Golden and mutation tests for the retained Affairs-first client protocol.

use ustc_campus_agent_client_protocol::*;

fn text(value: &str) -> WireText {
    WireText::parse(value).unwrap()
}

#[test]
fn golden_server_info_response_json() {
    let response = ClientResponseDto::ServerInfo {
        info: ServerInfoDto::new(text("ustc-agentd/test")),
    };
    assert_eq!(
        serde_json::to_string(&response).unwrap(),
        r#"{"kind":"server_info","info":{"protocol_schema":"ustc-client-protocol/v1","protocol_major":1,"supported_protocol_majors":[1],"minimum_client_protocol_major":1,"result_schema":"ustc-client-result/v1","server_build":"ustc-agentd/test","capabilities_route":"/api/v1/client/capabilities"}}"#
    );
}

#[test]
fn golden_capability_registry_response_json() {
    let response = ClientResponseDto::Capabilities {
        capabilities: CapabilityListDto::affairs_first(),
    };
    assert_eq!(
        serde_json::to_string(&response).unwrap(),
        r#"{"kind":"capabilities","capabilities":{"registry_revision":"m10-affairs-first/v1","protocol_major":1,"operations":[{"operation_id":"server.info","protocol_major":1,"request_schema":"none","result_schema":"server-info/v1","permission_class":"public_read","effect_class":"read","method":"GET","route":"/api/v1/server/info","requires_protocol_major":false,"adapters":["web","cli"]},{"operation_id":"capability.list","protocol_major":1,"request_schema":"none","result_schema":"capability-list/v1","permission_class":"public_read","effect_class":"read","method":"GET","route":"/api/v1/client/capabilities","requires_protocol_major":true,"adapters":["web","cli"]},{"operation_id":"affairs.get","protocol_major":1,"request_schema":"affairs-get-query/v1","result_schema":"client-response/v1","permission_class":"public_read","effect_class":"read","method":"GET","route":"/api/v1/affairs/{procedure_id}?as_of=<unix-ms>","requires_protocol_major":true,"adapters":["web","cli"]}]}}"#
    );
}

#[test]
fn golden_protocol_compatibility_response_json() {
    let old = ClientResponseDto::Compatibility {
        compatibility: admit_protocol_major(Some(ClientProtocolMajor::new(0))).unwrap_err(),
    };
    assert_eq!(
        serde_json::to_string(&old).unwrap(),
        r#"{"kind":"compatibility","compatibility":{"kind":"upgrade_required","client_major":0,"minimum_client_major":1,"server_major":1}}"#
    );

    let newer = ClientResponseDto::Compatibility {
        compatibility: admit_protocol_major(Some(ClientProtocolMajor::new(2))).unwrap_err(),
    };
    assert_eq!(
        serde_json::to_string(&newer).unwrap(),
        r#"{"kind":"compatibility","compatibility":{"kind":"incompatible_protocol","client_major":2,"supported_majors":[1],"server_major":1}}"#
    );

    let unknown = ClientResponseDto::Compatibility {
        compatibility: admit_protocol_major(None).unwrap_err(),
    };
    assert_eq!(
        serde_json::to_string(&unknown).unwrap(),
        r#"{"kind":"compatibility","compatibility":{"kind":"incompatible_protocol","client_major":null,"supported_majors":[1],"server_major":1}}"#
    );
}

#[test]
fn protocol_serde_rejects_unknown_and_incoherent_values() {
    assert!(serde_json::from_str::<OperationIdDto>(r#""other.operation""#).is_err());
    assert!(serde_json::from_str::<OperationSchemaDto>(r#""affairs-get-query/v2""#).is_err());
    assert!(serde_json::from_str::<PublicPermissionClassDto>(r#""tenant_private""#).is_err());
    assert!(serde_json::from_str::<OperationEffectClassDto>(r#""write""#).is_err());
    assert!(serde_json::from_str::<HttpRouteDto>(r#""/api/v1/affairs/search""#).is_err());
    assert!(serde_json::from_str::<ClientAdapterDto>(r#""android""#).is_err());

    let mut server_info =
        serde_json::to_value(ServerInfoDto::new(text("ustc-agentd/test"))).unwrap();
    server_info["protocol_major"] = serde_json::json!(2);
    assert!(serde_json::from_value::<ServerInfoDto>(server_info).is_err());

    let mut capabilities = serde_json::to_value(CapabilityListDto::affairs_first()).unwrap();
    capabilities["operations"][0]["requires_protocol_major"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<CapabilityListDto>(capabilities).is_err());

    let invalid_upgrade = serde_json::json!({
        "kind": "upgrade_required",
        "client_major": 1,
        "minimum_client_major": 1,
        "server_major": 1
    });
    assert!(serde_json::from_value::<ProtocolCompatibilityDto>(invalid_upgrade).is_err());

    let invalid_supported = serde_json::json!({
        "kind": "incompatible_protocol",
        "client_major": 2,
        "supported_majors": [2],
        "server_major": 1
    });
    assert!(serde_json::from_value::<ProtocolCompatibilityDto>(invalid_supported).is_err());
}
