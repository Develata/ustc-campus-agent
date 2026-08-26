//! Real bounded loopback transport over the M10 length-delimited framing API.
//!
//! `send_intent` opens a fresh TCP connection to a loopback endpoint, writes
//! exactly one [`ClientIntentDto`] frame via [`ustc_campus_agent_client_protocol::write_frame`],
//! reads one [`ClientResponseDto`] frame via [`ustc_campus_agent_client_protocol::read_frame`],
//! and closes. Connect, read and write are each bounded by a caller-supplied
//! timeout so a silent server cannot hang an automation client.
//!
//! Transport outcomes are mapped to three stable classes:
//!
//! - [`TransportError::Unavailable`] — the request could not be sent because
//!   the TCP connect failed or a pre-write socket option failed. The server is
//!   not known to have processed anything; this reduces to the same opaque
//!   `Unavailable` state as a server-typed denial.
//! - [`TransportError::OutcomeUnknown`] — the request may have reached the
//!   server but no valid response arrived. This covers every `write_frame`
//!   error after connect (the framing API exposes no committed-byte count, so
//!   some or all request bytes may have been delivered before the error) and
//!   every read timeout/EOF/reset after the write. The server may have
//!   processed the operation; reconciliation is required.
//! - [`TransportError::Malformed`] — a frame arrived but failed framing/DTO
//!   validation; a protocol error.

use std::io;
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

use ustc_campus_agent_client_protocol::{
    ClientIntentDto, ClientResponseDto, read_frame, write_frame,
};

/// A validated loopback `SocketAddr` endpoint.
///
/// The address is checked at parse time: the host must be a numeric IPv4 or
/// IPv6 literal and [`std::net::IpAddr::is_loopback`] must return true. DNS
/// names and non-loopback addresses are rejected so authenticated owner IDs
/// and response-only capabilities are never sent over plaintext TCP outside
/// agentd.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Endpoint {
    addr: SocketAddr,
}

/// Error returned when an [`Endpoint`] string cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointError(pub String);

impl std::fmt::Display for EndpointError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid endpoint: ")?;
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for EndpointError {}

impl Endpoint {
    /// Parses a numeric `host:port` endpoint and rejects non-loopback
    /// addresses.
    ///
    /// The input must parse directly as a [`SocketAddr`] (numeric IPv4
    /// `127.0.0.0/8` or IPv6 `[::1]`). DNS names are rejected because they
    /// cannot be deterministically proven to resolve to loopback.
    ///
    /// # Errors
    /// Returns [`EndpointError`] if the string is not a valid numeric
    /// `SocketAddr` or if the address is not loopback. Error messages carry
    /// only the endpoint literal, never a capability, capsule, or secret.
    pub fn parse(value: impl Into<String>) -> Result<Self, EndpointError> {
        let value = value.into();
        let addr: SocketAddr = value
            .parse()
            .map_err(|_| EndpointError(format!("not a numeric host:port `{value}`")))?;
        if !addr.ip().is_loopback() {
            return Err(EndpointError(format!(
                "not loopback `{value}` (required: 127.0.0.0/8 or [::1])"
            )));
        }
        Ok(Self { addr })
    }

    /// Returns the checked [`SocketAddr`].
    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

/// Stable transport failure class. Carries no payload, capability, or secret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    Unavailable,
    OutcomeUnknown,
    Malformed,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            TransportError::Unavailable => "transport unavailable",
            TransportError::OutcomeUnknown => "transport outcome unknown",
            TransportError::Malformed => "transport malformed frame",
        };
        formatter.write_str(msg)
    }
}

impl std::error::Error for TransportError {}

/// Sends one typed intent over a fresh bounded loopback connection and returns
/// the server's typed response.
///
/// # Errors
/// Returns [`TransportError::Unavailable`] only for connect failures and
/// pre-write socket-option failures. Every `write_frame` error after connect
/// returns [`TransportError::OutcomeUnknown`] because the framing API exposes
/// no committed-byte count and the server may have processed a partial or
/// complete request.
pub fn send_intent(
    endpoint: &Endpoint,
    timeout: Duration,
    intent: &ClientIntentDto,
) -> Result<ClientResponseDto, TransportError> {
    let addr = endpoint.addr();
    let stream = TcpStream::connect_timeout(&addr, timeout).map_err(connect_failure)?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| TransportError::Unavailable)?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|_| TransportError::Unavailable)?;
    // Shutdown the write half after sending so a well-behaved server sees EOF
    // once the request frame is complete; this does not cancel an accepted
    // server-side operation.
    write_frame(&stream, intent).map_err(write_failure)?;
    stream.shutdown(Shutdown::Write).ok();
    read_frame(&stream).map_err(read_failure)
}

fn connect_failure(_error: io::Error) -> TransportError {
    // Connect failure means the server is not known to have processed anything.
    TransportError::Unavailable
}

fn write_failure(_error: io::Error) -> TransportError {
    // After connect, write_frame can write some or all request bytes before
    // returning an error (including failure around flush). The framing API
    // exposes no committed-byte count, so conservatively classify every
    // write_frame error as OutcomeUnknown: the server may have processed the
    // operation and reconciliation is required.
    TransportError::OutcomeUnknown
}

fn read_failure(error: io::Error) -> TransportError {
    match error.kind() {
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => TransportError::OutcomeUnknown,
        io::ErrorKind::UnexpectedEof => TransportError::OutcomeUnknown,
        io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionAborted => {
            TransportError::OutcomeUnknown
        }
        io::ErrorKind::InvalidData => TransportError::Malformed,
        _ => TransportError::OutcomeUnknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- R1: loopback authority ---

    #[test]
    fn parses_ipv4_loopback() {
        let endpoint = Endpoint::parse("127.0.0.1:8080").expect("valid endpoint");
        assert_eq!(endpoint.addr(), "127.0.0.1:8080".parse().unwrap());
    }

    #[test]
    fn parses_ipv4_loopback_any_127_octet() {
        let endpoint = Endpoint::parse("127.255.255.254:1").expect("valid endpoint");
        assert!(endpoint.addr().ip().is_loopback());
    }

    #[test]
    fn parses_ipv6_loopback() {
        let endpoint = Endpoint::parse("[::1]:8080").expect("valid endpoint");
        assert_eq!(endpoint.addr(), "[::1]:8080".parse().unwrap());
    }

    #[test]
    fn rejects_public_ipv4() {
        assert!(Endpoint::parse("8.8.8.8:8080").is_err());
    }

    #[test]
    fn rejects_public_ipv6() {
        assert!(Endpoint::parse("[2001:4860:4860::8888]:8080").is_err());
    }

    #[test]
    fn rejects_non_loopback_private_ipv4() {
        assert!(Endpoint::parse("10.0.0.1:8080").is_err());
        assert!(Endpoint::parse("192.168.1.1:8080").is_err());
    }

    #[test]
    fn rejects_dns_name() {
        // DNS names are not numeric SocketAddrs and must be rejected because
        // they cannot be deterministically proven to resolve to loopback.
        assert!(Endpoint::parse("localhost:8080").is_err());
        assert!(Endpoint::parse("agentd.ustc.local:8080").is_err());
    }

    #[test]
    fn rejects_missing_port() {
        assert!(Endpoint::parse("127.0.0.1").is_err());
    }

    #[test]
    fn rejects_empty_host() {
        assert!(Endpoint::parse(":8080").is_err());
    }

    #[test]
    fn rejects_non_numeric_port() {
        assert!(Endpoint::parse("127.0.0.1:abc").is_err());
    }

    // --- R2: write failure classification ---

    #[test]
    fn connect_failure_is_unavailable() {
        let error = io::Error::new(io::ErrorKind::ConnectionRefused, "test");
        assert_eq!(connect_failure(error), TransportError::Unavailable);
    }

    #[test]
    fn write_failure_is_outcome_unknown() {
        // After connect, write_frame can write some/all bytes before failing.
        // The framing API exposes no committed-byte count, so conservatively
        // classify every write_frame error as OutcomeUnknown: the server may
        // have processed the request. Only connect and pre-write socket-option
        // failures may be Unavailable.
        let error = io::Error::new(io::ErrorKind::BrokenPipe, "test");
        assert_eq!(write_failure(error), TransportError::OutcomeUnknown);
    }
}
