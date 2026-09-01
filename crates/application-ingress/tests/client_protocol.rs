#![allow(clippy::unwrap_used)]

use std::cell::Cell;

use ustc_campus_agent_application_ingress::dispatch_with_protocol_major;
use ustc_campus_agent_client_protocol::{
    CURRENT_CLIENT_PROTOCOL_MAJOR, ClientProtocolMajor, ProtocolCompatibilityDto,
};

#[test]
fn incompatible_majors_never_dispatch_application_work() {
    for major in [
        None,
        Some(ClientProtocolMajor::new(0)),
        Some(ClientProtocolMajor::new(2)),
    ] {
        let calls = Cell::new(0_u8);
        let result = dispatch_with_protocol_major(major, || calls.set(calls.get() + 1));
        assert!(result.is_err());
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn current_major_dispatches_exactly_once() {
    let calls = Cell::new(0_u8);
    let result = dispatch_with_protocol_major(Some(CURRENT_CLIENT_PROTOCOL_MAJOR), || {
        calls.set(calls.get() + 1);
        "dispatched"
    });
    assert_eq!(result.unwrap(), "dispatched");
    assert_eq!(calls.get(), 1);
}

#[test]
fn old_new_and_unknown_major_keep_distinct_typed_outcomes() {
    assert!(matches!(
        dispatch_with_protocol_major(Some(ClientProtocolMajor::new(0)), || ()),
        Err(ProtocolCompatibilityDto::UpgradeRequired { .. })
    ));
    assert!(matches!(
        dispatch_with_protocol_major(Some(ClientProtocolMajor::new(2)), || ()),
        Err(ProtocolCompatibilityDto::IncompatibleProtocol {
            client_major: Some(_),
            ..
        })
    ));
    assert!(matches!(
        dispatch_with_protocol_major(None, || ()),
        Err(ProtocolCompatibilityDto::IncompatibleProtocol {
            client_major: None,
            ..
        })
    ));
}
