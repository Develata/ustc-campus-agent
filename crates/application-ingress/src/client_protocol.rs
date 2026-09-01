//! M10-owned framework-neutral client protocol admission.
//!
//! This module decides only compatibility for the retained protocol major. It
//! owns no HTTP parsing and calls the supplied application closure exactly once
//! only after the major is admitted.

use ustc_campus_agent_client_protocol::{
    ClientProtocolMajor, ProtocolCompatibilityDto, admit_protocol_major,
};

/// Runs one application dispatch only when the presented client major is
/// admitted by the M10 protocol contract.
///
/// Missing or unsupported majors return the server-owned typed compatibility
/// outcome without invoking `dispatch`.
pub fn dispatch_with_protocol_major<T>(
    client_major: Option<ClientProtocolMajor>,
    dispatch: impl FnOnce() -> T,
) -> Result<T, ProtocolCompatibilityDto> {
    admit_protocol_major(client_major)?;
    Ok(dispatch())
}
