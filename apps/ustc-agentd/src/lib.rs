//! Bounded composition root for the `ustc-agentd` product-path slice.
//!
//! Only this crate may simultaneously name M00 fixture ports, application-ingress,
//! M71 service/repository/application port and the equal-contract M60 fixture.
//! The M60 fixture is injected only through M71's `M60ProcedureEvidencePort`;
//! it never enters an M10/client seam and remains explicitly noncanonical.

#![forbid(unsafe_code)]

mod affairs_fixture;
mod web;

pub use web::web_router;

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use affairs_fixture::{AffairsFixture, DurableIdempotencyStore, FixturePorts};
use affairs_navigator::AffairsGetService;
use ustc_campus_agent_application_ingress::{FileRecordStore, M10Service};
use ustc_campus_agent_client_protocol::{
    ClientIntentDto, ClientResponseDto, SubmitAffairsGetDto, ViewerAuthorizationDto, read_frame,
    write_frame,
};

const FRAMED_CONNECTION_READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Bounded composition root owning the fixture, record store and idempotency
/// store for the product-path slice.
///
/// Per-request construction of `AffairsGetService` and `M10Service` avoids
/// self-referential lifetime issues: `AffairsGetService` borrows the repository,
/// M60 port and clock from the fixture, and `M10Service` borrows the
/// `AffairsGetService`. Both borrows are scoped to a single request.
pub struct AffairsComposition {
    fixture: AffairsFixture,
    store: FileRecordStore,
    idempotency: DurableIdempotencyStore,
}

impl AffairsComposition {
    /// Opens the composition from durable fixture, record-store and
    /// idempotency-store paths.
    ///
    /// # Errors
    ///
    /// Returns a descriptive string when any component fails to load or open.
    pub fn open(
        fixture_path: &Path,
        store_path: &Path,
        idempotency_path: &Path,
    ) -> Result<Self, String> {
        let fixture = AffairsFixture::load(fixture_path)?;
        let store =
            FileRecordStore::open(store_path).map_err(|e| format!("store open failed: {e:?}"))?;
        let now_ms = fixture.now.as_unix_millis();
        let idempotency = DurableIdempotencyStore::open(
            idempotency_path,
            now_ms,
            fixture.idempotency_deadline_ms,
        )?;
        Ok(Self {
            fixture,
            store,
            idempotency,
        })
    }

    /// Handles one `SubmitAffairsGet` intent through the real M00 admission
    /// coordinator, M10 service, M71 application service and M60 fixture port.
    #[must_use]
    pub fn handle_submit(&self, request: &SubmitAffairsGetDto) -> ClientResponseDto {
        let m71 =
            AffairsGetService::new(&self.fixture.repo, &self.fixture.m60, &self.fixture.clock);
        let m10 = M10Service::new(
            self.store.clone(),
            self.fixture.capabilities.clone(),
            &m71,
            self.fixture.operator_grant_id.clone(),
        );
        let mut ports = FixturePorts::new(
            self.idempotency.clone(),
            Arc::clone(&self.fixture.descriptor),
            self.fixture.now,
            self.fixture.policy_snapshot_id.clone(),
            Some(self.fixture.session.clone()),
        );
        let now_ms = i64::try_from(self.fixture.now.as_unix_millis()).unwrap_or(i64::MAX);
        m10.submit(request, &mut ports, now_ms)
    }

    /// Handles one `Lookup` intent through the M10 record store. The M71
    /// service is constructed but not called — lookup reads only the durable
    /// record store.
    #[must_use]
    pub fn handle_lookup(
        &self,
        command_id: &str,
        viewer: &ViewerAuthorizationDto,
    ) -> ClientResponseDto {
        let m71 =
            AffairsGetService::new(&self.fixture.repo, &self.fixture.m60, &self.fixture.clock);
        let m10 = M10Service::new(
            self.store.clone(),
            self.fixture.capabilities.clone(),
            &m71,
            self.fixture.operator_grant_id.clone(),
        );
        m10.lookup(command_id, viewer)
    }

    /// Returns the number of M60 `verify_retained` calls observed since the
    /// composition was opened. Used by tests to prove "one M71 call" and
    /// "zero M71 call" invariants.
    #[must_use]
    pub fn m60_call_count(&self) -> u64 {
        self.fixture
            .m60_call_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Binds a loopback TCP listener, prints `listening <addr>` to stdout, and
    /// serves connections sequentially. Each connection reads one
    /// `ClientIntentDto` frame and writes one `ClientResponseDto` frame.
    ///
    /// # Errors
    ///
    /// Returns a descriptive string when binding or address resolution fails.
    pub fn serve(&self, bind_addr: &str) -> Result<(), String> {
        let listener = bind_loopback(bind_addr)?;
        let local_addr = listener
            .local_addr()
            .map_err(|e| format!("local_addr failed: {e}"))?;
        println!("listening {local_addr}");
        std::io::stdout()
            .flush()
            .map_err(|e| format!("stdout flush failed: {e}"))?;

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("accept failed: {e}");
                    continue;
                }
            };
            if let Err(e) = self.handle_connection(stream) {
                eprintln!("connection error: {e}");
            }
        }
        Ok(())
    }

    fn handle_connection(&self, stream: TcpStream) -> Result<(), String> {
        let intent = read_intent_with_timeout(&stream, FRAMED_CONNECTION_READ_TIMEOUT)?;
        let response = match intent {
            ClientIntentDto::SubmitAffairsGet { request } => self.handle_submit(&request),
            ClientIntentDto::Lookup { command_id, viewer } => {
                self.handle_lookup(command_id.as_str(), &viewer)
            }
        };
        write_frame(&stream, &response).map_err(|e| format!("write response: {e}"))?;
        Ok(())
    }
}

fn read_intent_with_timeout(
    stream: &TcpStream,
    timeout: Duration,
) -> Result<ClientIntentDto, String> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| "read intent deadline overflow".to_owned())?;
    let mut reader = DeadlineReader { stream, deadline };
    let intent = read_frame(&mut reader).map_err(|error| format!("read intent: {error}"))?;
    if Instant::now() >= deadline {
        return Err("read intent: absolute frame deadline exceeded".to_owned());
    }
    Ok(intent)
}

struct DeadlineReader<'a> {
    stream: &'a TcpStream,
    deadline: Instant,
}

impl Read for DeadlineReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let Some(remaining) = self.deadline.checked_duration_since(Instant::now()) else {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "absolute frame deadline exceeded",
            ));
        };
        if remaining.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "absolute frame deadline exceeded",
            ));
        }
        self.stream.set_read_timeout(Some(remaining))?;
        let mut stream = self.stream;
        stream.read(buffer)
    }
}

pub(crate) fn parse_loopback_socket_addr(bind_addr: &str) -> Result<SocketAddr, String> {
    let socket_addr: SocketAddr = bind_addr
        .parse()
        .map_err(|e| format!("bind_addr parse failed: {e}"))?;
    if !socket_addr.ip().is_loopback() {
        return Err(format!(
            "bind addr {socket_addr} rejected: only loopback (127.0.0.0/8 or ::1) permitted"
        ));
    }
    Ok(socket_addr)
}

pub fn bind_loopback(bind_addr: &str) -> Result<TcpListener, String> {
    let socket_addr = parse_loopback_socket_addr(bind_addr)?;
    TcpListener::bind(socket_addr).map_err(|e| format!("bind failed: {e}"))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::io::Write as _;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{bind_loopback, read_intent_with_timeout};

    #[test]
    fn incomplete_framed_connection_times_out() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback test listener");
        let endpoint = listener.local_addr().expect("test listener address");
        let _incomplete_client = TcpStream::connect(endpoint).expect("connect incomplete client");
        let (server_stream, _) = listener.accept().expect("accept incomplete client");
        let started = Instant::now();
        let result = read_intent_with_timeout(&server_stream, Duration::from_millis(50));

        assert!(result.is_err(), "an incomplete frame must time out");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "the bounded read must not stall the sequential server"
        );
    }

    #[test]
    fn drip_fed_incomplete_frame_hits_absolute_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback test listener");
        let endpoint = listener.local_addr().expect("test listener address");
        let mut client = TcpStream::connect(endpoint).expect("connect drip client");
        let (server_stream, _) = listener.accept().expect("accept drip client");
        let writer = thread::spawn(move || {
            let bytes = [0_u8, 0, 0, 100, 1, 2, 3, 4, 5, 6];
            for byte in bytes {
                if client.write_all(&[byte]).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
            }
        });

        let started = Instant::now();
        let result = read_intent_with_timeout(&server_stream, Duration::from_millis(80));

        assert!(result.is_err(), "a drip-fed incomplete frame must time out");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "progress on individual reads must not extend the absolute deadline"
        );
        writer.join().expect("join drip writer");
    }

    #[test]
    fn bind_loopback_ipv4_zero_port_succeeds() {
        let listener = bind_loopback("127.0.0.1:0");
        assert!(listener.is_ok(), "IPv4 loopback bind must succeed");
        let listener = listener.unwrap();
        let local = listener.local_addr().unwrap();
        assert!(
            local.ip().is_loopback(),
            "bound IPv4 address must be loopback"
        );
    }

    #[test]
    fn bind_loopback_ipv6_zero_port_succeeds_or_env_unsupported() {
        let listener = bind_loopback("[::1]:0");
        match listener {
            Ok(listener) => {
                let local = listener.local_addr().unwrap();
                assert!(
                    local.ip().is_loopback(),
                    "bound IPv6 address must be loopback"
                );
            }
            Err(msg) => {
                let lowered = msg.to_ascii_lowercase();
                let recognized_unsupported = lowered.contains("address family")
                    || lowered.contains("address not available")
                    || lowered.contains("not supported")
                    || lowered.contains("protocol not supported")
                    || lowered.contains("no such device")
                    || lowered.contains("eafnosupport")
                    || lowered.contains("enodev")
                    || lowered.contains("eprotonosupport");
                assert!(
                    recognized_unsupported,
                    "IPv6 bind failure must be a recognized unsupported/address-family error, got: {msg}"
                );
            }
        }
    }

    #[test]
    fn bind_loopback_rejects_non_loopback() {
        let result = bind_loopback("0.0.0.0:0");
        assert!(result.is_err(), "non-loopback bind must be rejected");
        let msg = result.unwrap_err();
        assert!(
            msg.contains("rejected"),
            "non-loopback rejection message must explain rejection, got: {msg}"
        );
    }

    #[test]
    fn bind_loopback_rejects_unparseable() {
        let result = bind_loopback("not-an-address");
        assert!(result.is_err(), "unparseable bind must be rejected");
    }
}
