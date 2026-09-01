#![allow(clippy::unwrap_used)]

use ustc_campus_agent_client_core::wire::{
    CapabilityListDto, ClientProtocolMajor, ClientResponseDto, ProtocolCompatibilityDto,
    ServerInfoDto, WireText,
};
use ustc_campus_agent_client_core::{
    ClientState, Origin, exit_class, provenance, reduce_response, render_result,
};

fn text(value: &str) -> WireText {
    WireText::parse(value).unwrap()
}

fn render(response: ClientResponseDto) -> (ClientState, String) {
    let state = reduce_response(response);
    let provenance = provenance("ustc-agent/test", "cli", "client-protocol/v1").unwrap();
    let rendered = render_result(&state, Origin::Server, &provenance);
    (state, rendered)
}

#[test]
fn reduces_server_info_and_capabilities_under_v1_result_envelope() {
    let (info, info_json) = render(ClientResponseDto::ServerInfo {
        info: ServerInfoDto::new(text("ustc-agentd/test")),
    });
    assert!(matches!(info, ClientState::ServerInfo { .. }));
    assert_eq!(exit_class(&info).code(), 0);
    assert!(info_json.contains(r#""schema":"ustc-client-result/v1""#));
    assert!(info_json.contains(r#""kind":"server_info""#));

    let (capabilities, capabilities_json) = render(ClientResponseDto::Capabilities {
        capabilities: CapabilityListDto::affairs_first(),
    });
    assert!(matches!(capabilities, ClientState::Capabilities { .. }));
    assert_eq!(exit_class(&capabilities).code(), 0);
    assert!(capabilities_json.contains(r#""kind":"capabilities""#));
    assert!(capabilities_json.contains(r#""operation_id":"affairs.get""#));
}

#[test]
fn preserves_server_owned_compatibility_relations_without_recalculation() {
    let (upgrade, upgrade_json) = render(ClientResponseDto::Compatibility {
        compatibility: ProtocolCompatibilityDto::try_upgrade_required(ClientProtocolMajor::new(0))
            .unwrap(),
    });
    assert!(matches!(upgrade, ClientState::UpgradeRequired { .. }));
    assert_eq!(exit_class(&upgrade).code(), 5);
    assert!(upgrade_json.contains(r#""kind":"upgrade_required""#));
    assert!(upgrade_json.contains(r#""minimum_client_major":1"#));

    let (incompatible, incompatible_json) = render(ClientResponseDto::Compatibility {
        compatibility: ProtocolCompatibilityDto::try_incompatible_protocol(None).unwrap(),
    });
    assert!(matches!(
        incompatible,
        ClientState::IncompatibleProtocol {
            client_major: None,
            ..
        }
    ));
    assert_eq!(exit_class(&incompatible).code(), 5);
    assert!(incompatible_json.contains(r#""kind":"incompatible_protocol""#));
    assert!(incompatible_json.contains(r#""supported_majors":[1]"#));
}
