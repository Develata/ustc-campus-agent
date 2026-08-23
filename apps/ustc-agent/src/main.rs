//! `ustc-agent` — ordinary-user / headless-automation client.
//!
//! Sends exactly one typed intent through `client-core` over the bounded
//! loopback transport, prints exactly one canonical `ustc-client-result/v1`
//! JSON envelope on stdout, and exits with a stable class code. Diagnostics
//! go to stderr; no capability/capsule/secret is ever logged or dumped.
//!
//! Privilege boundary (`cli/v2.1` §1, `client-shell/v2.1` §14): this binary
//! depends only on `client-core` (and `serde_json` for stdout). It has no
//! dependency on `ustc-agentctl`, platform-core, affairs-navigator,
//! application-ingress, M60, agentd, a file store, or any operator surface.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use std::process::ExitCode;
use std::time::Duration;

use ustc_campus_agent_client_core::{
    self as core, ClientIntentDto, ClientProvenanceDto, ClientState, DEFAULT_CALL_TIMEOUT,
    Endpoint, Origin, RESULT_SCHEMA, TransportError, UnixMillis, authenticated_affairs_get,
    exit_class, lookup_as_operator, lookup_as_owner, lookup_by_capability, public_affairs_get,
    reduce_response, reduce_transport_failure, render_result,
};

const PROTOCOL: &str = "client-protocol/v0.1";
const TARGET: &str = "cli";
/// Maximum accepted `--timeout` value in seconds. An automation client must
/// not block indefinitely; 300 seconds (5 minutes) is the documented ceiling.
const MAX_TIMEOUT_SECS: u64 = 300;

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let outcome = dispatch(args);
    if !outcome.stdout.is_empty() {
        print!("{}", outcome.stdout);
    }
    if !outcome.stderr.is_empty() {
        eprint!("{}", outcome.stderr);
    }
    ExitCode::from(code_to_u8(outcome.code))
}

fn code_to_u8(code: i32) -> u8 {
    // Out-of-range codes must never fall back to 0 (success). u8::MAX (255)
    // signals an internal invariant violation visibly rather than silently
    // masquerading as success.
    u8::try_from(code).unwrap_or(u8::MAX)
}

fn dispatch(args: Vec<String>) -> Outcome {
    let mut args = args.into_iter().skip(1);
    match args.next().as_deref() {
        Some("--version") => Outcome {
            code: 0,
            stdout: format!("ustc-agent {}\n", env!("CARGO_PKG_VERSION")),
            stderr: String::new(),
        },
        Some("--help") | None => Outcome {
            code: 0,
            stdout: help_text(),
            stderr: String::new(),
        },
        Some("affairs") => match args.next().as_deref() {
            Some("get") => dispatch_affairs_get(args.collect()),
            Some("lookup") => dispatch_affairs_lookup(args.collect()),
            Some(other) => usage_err(format!("unknown affairs subcommand `{other}`")),
            None => usage_err("missing affairs subcommand (get|lookup)".into()),
        },
        Some(other) => usage_err(format!("unknown command `{other}`")),
    }
}

fn dispatch_affairs_get(args: Vec<String>) -> Outcome {
    let mut endpoint: Option<String> = None;
    let mut procedure_id: Option<String> = None;
    let mut as_of: Option<i64> = None;
    let mut request_id: Option<String> = None;
    let mut correlation_id: Option<String> = None;
    let mut causation_id: Option<String> = None;
    let mut idempotency_key: Option<String> = None;
    let mut payload_digest: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut timeout = DEFAULT_CALL_TIMEOUT;
    let mut non_interactive = false;
    let mut format_json = true;

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--endpoint" => {
                endpoint = match take_value(&mut iter, "--endpoint") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--procedure-id" => {
                procedure_id = match take_value(&mut iter, "--procedure-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--as-of" => {
                as_of = match parse_i64(&mut iter, "--as-of") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--request-id" => {
                request_id = match take_value(&mut iter, "--request-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--correlation-id" => {
                correlation_id = match take_value(&mut iter, "--correlation-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--causation-id" => {
                causation_id = match take_value(&mut iter, "--causation-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--idempotency-key" => {
                idempotency_key = match take_value(&mut iter, "--idempotency-key") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--payload-digest" => {
                payload_digest = match take_value(&mut iter, "--payload-digest") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--session-id" => {
                session_id = match take_value(&mut iter, "--session-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--timeout" => {
                timeout = match parse_timeout(&mut iter, "--timeout") {
                    Ok(v) => v,
                    Err(o) => return o,
                }
            }
            "--non-interactive" => non_interactive = true,
            "--format" => {
                let value = match take_value(&mut iter, "--format") {
                    Ok(v) => v,
                    Err(o) => return o,
                };
                if value != "json" {
                    return usage_err("only --format json is supported".into());
                }
                format_json = true;
            }
            other => return usage_err(format!("unknown flag `{other}`")),
        }
    }
    let _ = non_interactive;
    let _ = format_json;

    let endpoint = match require(endpoint, "--endpoint") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let procedure_id = match require(procedure_id, "--procedure-id") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let request_id = match require(request_id, "--request-id") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let correlation_id = match require(correlation_id, "--correlation-id") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let payload_digest = match require(payload_digest, "--payload-digest") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let endpoint = match Endpoint::parse(&endpoint) {
        Ok(value) => value,
        Err(error) => return usage_err(error.to_string()),
    };
    let provenance = match core::provenance(
        format!("ustc-agent/{}", env!("CARGO_PKG_VERSION")),
        TARGET,
        PROTOCOL,
    ) {
        Ok(value) => value,
        Err(error) => return usage_err(error.to_string()),
    };
    let as_of = as_of.map(UnixMillis::new);
    let intent = if let Some(session_id) = session_id {
        authenticated_affairs_get(
            request_id,
            correlation_id,
            causation_id,
            idempotency_key,
            provenance.clone(),
            payload_digest,
            procedure_id,
            as_of,
            session_id,
        )
    } else {
        public_affairs_get(
            request_id,
            correlation_id,
            causation_id,
            idempotency_key,
            provenance.clone(),
            payload_digest,
            procedure_id,
            as_of,
        )
    };
    let intent = match intent {
        Ok(value) => value,
        Err(error) => return usage_err(error.to_string()),
    };
    send_and_render(&endpoint, timeout, &intent, &provenance)
}

fn dispatch_affairs_lookup(args: Vec<String>) -> Outcome {
    let mut endpoint: Option<String> = None;
    let mut command_id: Option<String> = None;
    let mut capability: Option<String> = None;
    let mut tenant_id: Option<String> = None;
    let mut user_id: Option<String> = None;
    let mut grant_id: Option<String> = None;
    let mut timeout = DEFAULT_CALL_TIMEOUT;
    let mut non_interactive = false;

    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--endpoint" => {
                endpoint = match take_value(&mut iter, "--endpoint") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--command-id" => {
                command_id = match take_value(&mut iter, "--command-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--capability" => {
                capability = match take_value(&mut iter, "--capability") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--tenant-id" => {
                tenant_id = match take_value(&mut iter, "--tenant-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--user-id" => {
                user_id = match take_value(&mut iter, "--user-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--grant-id" => {
                grant_id = match take_value(&mut iter, "--grant-id") {
                    Ok(v) => Some(v),
                    Err(o) => return o,
                }
            }
            "--timeout" => {
                timeout = match parse_timeout(&mut iter, "--timeout") {
                    Ok(v) => v,
                    Err(o) => return o,
                }
            }
            "--non-interactive" => non_interactive = true,
            "--format" => {
                let value = match take_value(&mut iter, "--format") {
                    Ok(v) => v,
                    Err(o) => return o,
                };
                if value != "json" {
                    return usage_err("only --format json is supported".into());
                }
            }
            other => return usage_err(format!("unknown flag `{other}`")),
        }
    }
    let _ = non_interactive;

    let endpoint = match require(endpoint, "--endpoint") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let command_id = match require(command_id, "--command-id") {
        Ok(value) => value,
        Err(outcome) => return outcome,
    };
    let endpoint = match Endpoint::parse(&endpoint) {
        Ok(value) => value,
        Err(error) => return usage_err(error.to_string()),
    };
    let provenance = match core::provenance(
        format!("ustc-agent/{}", env!("CARGO_PKG_VERSION")),
        TARGET,
        PROTOCOL,
    ) {
        Ok(value) => value,
        Err(error) => return usage_err(error.to_string()),
    };
    let viewers = [
        capability.is_some(),
        tenant_id.is_some() || user_id.is_some(),
        grant_id.is_some(),
    ];
    let selected = viewers.iter().filter(|flag| **flag).count();
    if selected != 1 {
        return usage_err(
            "exactly one of --capability, (--tenant-id and --user-id), or --grant-id is required"
                .into(),
        );
    }
    let intent = if let Some(capability) = capability {
        lookup_by_capability(command_id, capability)
    } else if let (Some(tenant_id), Some(user_id)) = (tenant_id, user_id) {
        lookup_as_owner(command_id, tenant_id, user_id)
    } else if let Some(grant_id) = grant_id {
        lookup_as_operator(command_id, grant_id)
    } else {
        return usage_err("--tenant-id and --user-id must be supplied together".into());
    };
    let intent = match intent {
        Ok(value) => value,
        Err(error) => return usage_err(error.to_string()),
    };
    send_and_render(&endpoint, timeout, &intent, &provenance)
}

fn send_and_render(
    endpoint: &Endpoint,
    timeout: Duration,
    intent: &ClientIntentDto,
    provenance: &ClientProvenanceDto,
) -> Outcome {
    match core::send_intent(endpoint, timeout, intent) {
        Ok(response) => {
            let state = reduce_response(response);
            render_outcome(&state, Origin::Server, provenance, String::new())
        }
        Err(error) => {
            let state = reduce_transport_failure(error);
            let diagnostic = transport_diagnostic(error);
            render_outcome(&state, Origin::Transport, provenance, diagnostic)
        }
    }
}

fn render_outcome(
    state: &ClientState,
    origin: Origin,
    provenance: &ClientProvenanceDto,
    stderr: String,
) -> Outcome {
    let stdout = format!("{}\n", render_result(state, origin, provenance));
    Outcome {
        code: exit_class(state).code(),
        stdout,
        stderr,
    }
}

fn transport_diagnostic(error: TransportError) -> String {
    format!("error: {error}\n")
}

fn take_value(iter: &mut std::vec::IntoIter<String>, flag: &str) -> Result<String, Outcome> {
    iter.next()
        .ok_or_else(|| usage_err(format!("missing value for {flag}")))
}

fn parse_i64(iter: &mut std::vec::IntoIter<String>, flag: &str) -> Result<i64, Outcome> {
    let raw = take_value(iter, flag)?;
    raw.parse::<i64>()
        .map_err(|_| usage_err(format!("invalid value for {flag}: `{raw}`")))
}

fn parse_timeout(iter: &mut std::vec::IntoIter<String>, flag: &str) -> Result<Duration, Outcome> {
    let raw = take_value(iter, flag)?;
    let secs: u64 = raw
        .parse()
        .map_err(|_| usage_err(format!("invalid value for {flag}: `{raw}`")))?;
    if secs == 0 {
        return Err(usage_err(format!(
            "invalid value for {flag}: must be nonzero"
        )));
    }
    if secs > MAX_TIMEOUT_SECS {
        return Err(usage_err(format!(
            "invalid value for {flag}: must be at most {MAX_TIMEOUT_SECS} seconds"
        )));
    }
    Ok(Duration::from_secs(secs))
}

fn require(value: Option<String>, flag: &str) -> Result<String, Outcome> {
    value.ok_or_else(|| usage_err(format!("missing required flag {flag}")))
}

fn usage_err(message: String) -> Outcome {
    Outcome {
        code: 2,
        stdout: String::new(),
        stderr: format!("error: {message}\n"),
    }
}

fn help_text() -> String {
    format!(
        "ustc-agent {version} — ordinary-user / automation client\n\n\
         Usage:\n  \
         ustc-agent affairs get --endpoint <loopback-host:port> --procedure-id <id> \\\n    \
         --request-id <id> --correlation-id <id> --payload-digest <hex> \\\n    \
         [--as-of <unix-millis>] [--session-id <id>] [--causation-id <id>] \\\n    \
         [--idempotency-key <key>] [--timeout <1..=300 secs>] [--non-interactive] [--format json]\n  \
         ustc-agent affairs lookup --endpoint <loopback-host:port> --command-id <id> \\\n    \
         (--capability <cap> | --tenant-id <t> --user-id <u> | --grant-id <g>) \\\n    \
         [--timeout <1..=300 secs>] [--non-interactive] [--format json]\n  \
         ustc-agent --version\n  \
         ustc-agent --help\n\n\
         Output: one `{schema}` JSON envelope on stdout per request; diagnostics on stderr.\n\
         Exit classes: 0 success, 2 usage, 3 auth, 4 policy, 5 compat, 6 unavailable, \\\
         7 conflict, 8 outcome-unknown, 9 protocol.\n\n\
         Note: --endpoint must be a numeric loopback address (127.0.0.0/8 or [::1]).\n\
         Note: --payload-digest is caller-supplied; the client propagates it for server-side \\\
         idempotency verification and does not authoritatively compute it.\n",
        version = env!("CARGO_PKG_VERSION"),
        schema = RESULT_SCHEMA,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ustc_campus_agent_client_core::wire::{
        ClientResponseDto, M71LineageDto, M71OutcomeDto, M71TerminalDto, WireText,
    };

    fn provenance() -> ClientProvenanceDto {
        core::provenance("ustc-agent/test", "cli", PROTOCOL).expect("valid provenance")
    }

    #[test]
    fn version_prints_zero_exit() {
        let outcome = dispatch(vec!["ustc-agent".into(), "--version".into()]);
        assert_eq!(outcome.code, 0);
        assert!(outcome.stdout.contains("ustc-agent"));
        assert!(outcome.stdout.is_empty() || outcome.stdout.contains('\n'));
    }

    #[test]
    fn help_prints_zero_exit() {
        let outcome = dispatch(vec!["ustc-agent".into(), "--help".into()]);
        assert_eq!(outcome.code, 0);
        assert!(outcome.stdout.contains("affairs get"));
    }

    #[test]
    fn no_args_prints_help() {
        let outcome = dispatch(vec!["ustc-agent".into()]);
        assert_eq!(outcome.code, 0);
        assert!(outcome.stdout.contains("affairs"));
    }

    #[test]
    fn unknown_command_is_usage_error() {
        let outcome = dispatch(vec!["ustc-agent".into(), "frobnicate".into()]);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("unknown command"));
    }

    #[test]
    fn missing_endpoint_is_usage_error() {
        let outcome = dispatch(vec![
            "ustc-agent".into(),
            "affairs".into(),
            "get".into(),
            "--procedure-id".into(),
            "p1".into(),
            "--request-id".into(),
            "r1".into(),
            "--correlation-id".into(),
            "c1".into(),
            "--payload-digest".into(),
            "d1".into(),
        ]);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("--endpoint"));
    }

    #[test]
    fn missing_payload_digest_is_usage_error() {
        let outcome = dispatch(vec![
            "ustc-agent".into(),
            "affairs".into(),
            "get".into(),
            "--endpoint".into(),
            "127.0.0.1:8080".into(),
            "--procedure-id".into(),
            "p1".into(),
            "--request-id".into(),
            "r1".into(),
            "--correlation-id".into(),
            "c1".into(),
        ]);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("--payload-digest"));
    }

    #[test]
    fn lookup_requires_exactly_one_viewer() {
        let outcome = dispatch(vec![
            "ustc-agent".into(),
            "affairs".into(),
            "lookup".into(),
            "--endpoint".into(),
            "127.0.0.1:8080".into(),
            "--command-id".into(),
            "cmd1".into(),
            "--capability".into(),
            "cap1".into(),
            "--grant-id".into(),
            "g1".into(),
        ]);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("exactly one"));
    }

    #[test]
    fn exit_class_maps_each_state() {
        let prov = provenance();
        let state = ClientState::Unavailable;
        assert_eq!(exit_class(&state).code(), 6);

        let procedure_id = WireText::parse("proc-1").unwrap();
        let terminal_dto = M71TerminalDto::try_new(
            M71OutcomeDto::NotFound {
                procedure_id: procedure_id.clone(),
            },
            M71LineageDto::NotRequired {
                materialization_receipt_id: WireText::parse("mr-1").unwrap(),
                reason: WireText::parse("no_visible_artifact").unwrap(),
            },
        )
        .expect("valid pairing");
        let response = ClientResponseDto::Accepted {
            command_id: WireText::parse("cmd-1").unwrap(),
            terminal: Box::new(terminal_dto),
            public_capability: None,
        };
        let state = reduce_response(response);
        assert_eq!(exit_class(&state).code(), 0);
        let rendered = render_result(&state, Origin::Server, &prov);
        assert!(rendered.contains(RESULT_SCHEMA));
        assert!(rendered.contains("\"exit_code\":0"));
    }

    #[test]
    fn canonical_result_envelope_is_stable() {
        let prov = provenance();
        let state = ClientState::Unavailable;
        let rendered = render_result(&state, Origin::Server, &prov);
        assert_eq!(
            rendered,
            format!(
                "{{\"schema\":\"{schema}\",\"exit_class\":\"unavailable\",\"exit_code\":6,\"origin\":\"server\",\"state\":{{\"kind\":\"unavailable\"}},\"provenance\":{{\"build\":\"ustc-agent/test\",\"target\":\"cli\",\"protocol\":\"client-protocol/v0.1\"}}}}",
                schema = RESULT_SCHEMA,
            )
        );
    }

    fn base_get_args() -> Vec<String> {
        vec![
            "ustc-agent".into(),
            "affairs".into(),
            "get".into(),
            "--endpoint".into(),
            "127.0.0.1:8080".into(),
            "--procedure-id".into(),
            "p1".into(),
            "--request-id".into(),
            "r1".into(),
            "--correlation-id".into(),
            "c1".into(),
            "--payload-digest".into(),
            "d1".into(),
        ]
    }

    #[test]
    fn missing_as_of_value_is_usage_error() {
        let mut args = base_get_args();
        args.push("--as-of".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --as-of"));
    }

    #[test]
    fn malformed_as_of_is_usage_error() {
        let mut args = base_get_args();
        args.push("--as-of".into());
        args.push("nope".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("invalid value for --as-of"));
    }

    #[test]
    fn missing_session_id_value_is_usage_error() {
        let mut args = base_get_args();
        args.push("--session-id".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --session-id"));
    }

    #[test]
    fn missing_causation_id_value_is_usage_error() {
        let mut args = base_get_args();
        args.push("--causation-id".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --causation-id"));
    }

    #[test]
    fn missing_idempotency_key_value_is_usage_error() {
        let mut args = base_get_args();
        args.push("--idempotency-key".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(
            outcome
                .stderr
                .contains("missing value for --idempotency-key")
        );
    }

    #[test]
    fn missing_timeout_value_is_usage_error() {
        let mut args = base_get_args();
        args.push("--timeout".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --timeout"));
    }

    #[test]
    fn malformed_timeout_is_usage_error() {
        let mut args = base_get_args();
        args.push("--timeout".into());
        args.push("abc".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("invalid value for --timeout"));
    }

    #[test]
    fn zero_timeout_is_usage_error() {
        let mut args = base_get_args();
        args.push("--timeout".into());
        args.push("0".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("must be nonzero"));
    }

    #[test]
    fn overflow_timeout_is_usage_error() {
        let mut args = base_get_args();
        args.push("--timeout".into());
        args.push("301".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("must be at most 300"));
    }

    #[test]
    fn timeout_boundary_one_second_is_accepted() {
        let mut args = base_get_args();
        args.push("--timeout".into());
        args.push("1".into());
        let outcome = dispatch(args);
        assert_ne!(
            outcome.code, 2,
            "timeout=1 must be accepted (not a usage error)"
        );
    }

    #[test]
    fn timeout_boundary_300_seconds_is_accepted() {
        let mut args = base_get_args();
        args.push("--timeout".into());
        args.push("300".into());
        let outcome = dispatch(args);
        assert_ne!(
            outcome.code, 2,
            "timeout=300 must be accepted (not a usage error)"
        );
    }

    #[test]
    fn missing_format_value_is_usage_error() {
        let mut args = base_get_args();
        args.push("--format".into());
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --format"));
    }

    #[test]
    fn missing_endpoint_value_is_usage_error() {
        let outcome = dispatch(vec![
            "ustc-agent".into(),
            "affairs".into(),
            "get".into(),
            "--procedure-id".into(),
            "p1".into(),
            "--request-id".into(),
            "r1".into(),
            "--correlation-id".into(),
            "c1".into(),
            "--payload-digest".into(),
            "d1".into(),
            "--endpoint".into(),
        ]);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --endpoint"));
    }

    #[test]
    fn missing_lookup_capability_value_is_usage_error() {
        let outcome = dispatch(vec![
            "ustc-agent".into(),
            "affairs".into(),
            "lookup".into(),
            "--endpoint".into(),
            "127.0.0.1:8080".into(),
            "--command-id".into(),
            "cmd1".into(),
            "--capability".into(),
        ]);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("missing value for --capability"));
    }

    #[test]
    fn non_loopback_endpoint_is_usage_error() {
        let mut args = base_get_args();
        args.iter_mut().for_each(|a| {
            if a == "127.0.0.1:8080" {
                *a = "8.8.8.8:8080".into();
            }
        });
        let outcome = dispatch(args);
        assert_eq!(outcome.code, 2);
        assert!(outcome.stderr.contains("not loopback"));
    }

    #[test]
    fn out_of_range_code_never_becomes_zero() {
        assert_eq!(code_to_u8(0), 0);
        assert_eq!(code_to_u8(2), 2);
        assert_eq!(code_to_u8(9), 9);
        assert_eq!(code_to_u8(255), 255);
        assert_eq!(code_to_u8(-1), u8::MAX);
        assert_eq!(code_to_u8(256), u8::MAX);
        assert_eq!(code_to_u8(i32::MAX), u8::MAX);
        assert_eq!(code_to_u8(i32::MIN), u8::MAX);
        assert_ne!(code_to_u8(-1), 0);
        assert_ne!(code_to_u8(256), 0);
        assert_ne!(code_to_u8(i32::MAX), 0);
        assert_ne!(code_to_u8(i32::MIN), 0);
    }
}
