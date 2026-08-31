use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;
use ustc_campus_agent_adapters::adapter_health;
use ustc_campus_agent_core::{
    COURSE_PLANNING_SLICE, DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES, OPPORTUNITY_GRAPH_PLUGIN_ID,
    PRODUCT_NAME,
};
use ustc_campus_agent_course_planning::{PlanningConfig, load_fixture, plan_fixture};
use ustc_campus_agent_runtime::RUN_SPEC_SCHEMA_VERSION;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(message) = run(&args) {
        eprintln!("error: {message}");
        std::process::exit(2);
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [] => {
            print_help();
            Ok(())
        }
        [cmd] if cmd == "--help" || cmd == "help" => {
            print_help();
            Ok(())
        }
        [cmd] if cmd == "--version" || cmd == "version" => {
            println!("ustc-agentctl {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        [cmd] if cmd == "doctor" => {
            println!("product={PRODUCT_NAME}");
            for plugin in DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES {
                println!(
                    "default_first_party_plugin={}@{}",
                    plugin.id, plugin.version
                );
            }
            println!("bounded_spike_plugin={OPPORTUNITY_GRAPH_PLUGIN_ID}");
            println!("bounded_spike_slice={COURSE_PLANNING_SLICE}");
            println!("agent_runtime_kernel_schema={RUN_SPEC_SCHEMA_VERSION}");
            println!("{}", adapter_health());
            Ok(())
        }
        [cmd, sub] if cmd == "market" && sub == "validate" => {
            println!("market validation is implemented by scripts/check_repo_contracts.py");
            for plugin in DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES {
                println!("first_party_package={}@{}", plugin.id, plugin.version);
            }
            Ok(())
        }
        [cmd, sub, rest @ ..] if cmd == "course" && sub == "plan" => run_course_plan(rest),
        [cmd, sub, rest @ ..] if cmd == "affairs" && sub == "publication-status" => {
            run_affairs_publication(rest, AffairsPublicationAction::Status)
        }
        [cmd, sub, rest @ ..] if cmd == "affairs" && sub == "publish-demo" => {
            run_affairs_publication(rest, AffairsPublicationAction::Publish)
        }
        [cmd, sub, rest @ ..]
            if matches!(cmd.as_str(), "change" | "changes") && sub == "publication-status" =>
        {
            run_change_publication(rest, ChangePublicationAction::Status)
        }
        [cmd, sub, rest @ ..]
            if matches!(cmd.as_str(), "change" | "changes") && sub == "publish-demo" =>
        {
            run_change_publication(rest, ChangePublicationAction::Publish)
        }
        _ => Err("unknown command; run `ustc-agentctl help`".to_owned()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AffairsPublicationAction {
    Status,
    Publish,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AffairsPublicationOptions {
    server: SocketAddr,
}

fn run_affairs_publication(
    args: &[String],
    action: AffairsPublicationAction,
) -> Result<(), String> {
    let options = parse_affairs_publication_options(args, action)?;
    let (method, body) = match action {
        AffairsPublicationAction::Status => ("GET", ""),
        AffairsPublicationAction::Publish => ("POST", r#"{"confirm_publish":true}"#),
    };
    let response = loopback_http_request(
        options.server,
        "/api/v1/demo/administrator/affairs/publication",
        method,
        body,
    )?;
    let value: serde_json::Value = serde_json::from_str(&response.body)
        .map_err(|error| format!("server returned invalid JSON: {error}"))?;
    let expected_schema = match action {
        AffairsPublicationAction::Status => "ustc-affairs-publication-status/v1",
        AffairsPublicationAction::Publish => "ustc-affairs-publication-response/v1",
    };
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(expected_schema) {
        return Err(format!(
            "server returned an unexpected Affairs publication schema (HTTP {})",
            response.status
        ));
    }
    let rendered = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to render server response: {error}"))?;
    println!("{rendered}");
    if !(200..300).contains(&response.status) {
        return Err(format!(
            "Affairs publication request failed with HTTP {}",
            response.status
        ));
    }
    Ok(())
}

fn parse_affairs_publication_options(
    args: &[String],
    action: AffairsPublicationAction,
) -> Result<AffairsPublicationOptions, String> {
    let mut server = None;
    let mut confirm = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--server" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--server requires a loopback socket address".to_owned());
                };
                let parsed = value.parse::<SocketAddr>().map_err(|_| {
                    "--server must be a socket address such as 127.0.0.1:8080".to_owned()
                })?;
                if !parsed.ip().is_loopback() {
                    return Err("--server must resolve to a loopback address".to_owned());
                }
                server = Some(parsed);
                index += 2;
            }
            "--confirm" if action == AffairsPublicationAction::Publish => {
                confirm = true;
                index += 1;
            }
            unknown => return Err(format!("unknown Affairs publication option: {unknown}")),
        }
    }
    let Some(server) = server else {
        return Err("Affairs publication commands require --server <loopback:port>".to_owned());
    };
    if action == AffairsPublicationAction::Publish && !confirm {
        return Err("publish-demo requires the explicit --confirm flag".to_owned());
    }
    Ok(AffairsPublicationOptions { server })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangePublicationAction {
    Status,
    Publish,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChangePublicationOptions {
    server: SocketAddr,
}

fn run_change_publication(args: &[String], action: ChangePublicationAction) -> Result<(), String> {
    let options = parse_change_publication_options(args, action)?;
    let (method, body) = match action {
        ChangePublicationAction::Status => ("GET", ""),
        ChangePublicationAction::Publish => ("POST", r#"{"confirm_publish":true}"#),
    };
    let response = loopback_http_request(
        options.server,
        "/api/v1/demo/administrator/changes/publication",
        method,
        body,
    )?;
    let value: serde_json::Value = serde_json::from_str(&response.body)
        .map_err(|error| format!("server returned invalid JSON: {error}"))?;
    let expected_schema = match action {
        ChangePublicationAction::Status => "ustc-change-publication-status/v1",
        ChangePublicationAction::Publish => "ustc-change-publication-response/v1",
    };
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(expected_schema) {
        return Err(format!(
            "server returned an unexpected ChangeRadar publication schema (HTTP {})",
            response.status
        ));
    }
    let rendered = serde_json::to_string_pretty(&value)
        .map_err(|error| format!("failed to render server response: {error}"))?;
    println!("{rendered}");
    if !(200..300).contains(&response.status) {
        return Err(format!(
            "ChangeRadar publication request failed with HTTP {}",
            response.status
        ));
    }
    Ok(())
}

fn parse_change_publication_options(
    args: &[String],
    action: ChangePublicationAction,
) -> Result<ChangePublicationOptions, String> {
    let mut server = None;
    let mut confirm = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--server" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--server requires a loopback socket address".to_owned());
                };
                let parsed = value.parse::<SocketAddr>().map_err(|_| {
                    "--server must be a socket address such as 127.0.0.1:8080".to_owned()
                })?;
                if !parsed.ip().is_loopback() {
                    return Err("--server must resolve to a loopback address".to_owned());
                }
                server = Some(parsed);
                index += 2;
            }
            "--confirm" if action == ChangePublicationAction::Publish => {
                confirm = true;
                index += 1;
            }
            unknown => {
                return Err(format!("unknown ChangeRadar publication option: {unknown}"));
            }
        }
    }
    let Some(server) = server else {
        return Err("ChangeRadar publication commands require --server <loopback:port>".to_owned());
    };
    if action == ChangePublicationAction::Publish && !confirm {
        return Err("publish-demo requires the explicit --confirm flag".to_owned());
    }
    Ok(ChangePublicationOptions { server })
}

struct HttpResponse {
    status: u16,
    body: String,
}

fn loopback_http_request(
    server: SocketAddr,
    path: &str,
    method: &str,
    body: &str,
) -> Result<HttpResponse, String> {
    let mut stream = TcpStream::connect_timeout(&server, Duration::from_secs(5))
        .map_err(|error| format!("failed to connect to loopback server: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|error| format!("failed to configure loopback read timeout: {error}"))?;
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: {server}\r\nAccept: application/json\r\nX-USTC-Agent-Administrator-Demo: confirm-v1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .map_err(|error| format!("failed to write loopback request: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("failed to flush loopback request: {error}"))?;
    let mut bytes = Vec::new();
    stream
        .take(65_537)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read loopback response: {error}"))?;
    if bytes.len() > 65_536 {
        return Err("loopback response exceeded 64 KiB".to_owned());
    }
    parse_http_response(&bytes)
}

fn parse_http_response(bytes: &[u8]) -> Result<HttpResponse, String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| "loopback response was not valid UTF-8".to_owned())?;
    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| "loopback response omitted the HTTP separator".to_owned())?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| "loopback response had an invalid HTTP status".to_owned())?;
    Ok(HttpResponse {
        status,
        body: body.to_owned(),
    })
}

fn run_course_plan(args: &[String]) -> Result<(), String> {
    let options = parse_course_plan_options(args)?;
    let fixture = load_fixture(&options.fixture).map_err(|error| error.to_string())?;
    let result =
        plan_fixture(&fixture, PlanningConfig::default()).map_err(|error| error.to_string())?;
    let output = serde_json::to_string_pretty(&result)
        .map_err(|error| format!("failed to encode plan result: {error}"))?;
    println!("{output}");
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CoursePlanOptions {
    fixture: PathBuf,
}

fn parse_course_plan_options(args: &[String]) -> Result<CoursePlanOptions, String> {
    let mut fixture = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--fixture" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--fixture requires a path".to_owned());
                };
                fixture = Some(PathBuf::from(value));
                index += 2;
            }
            "--format" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("--format requires a value".to_owned());
                };
                if value != "json" {
                    return Err(format!("unsupported format {value:?}; expected \"json\""));
                }
                index += 2;
            }
            unknown => return Err(format!("unknown course plan option: {unknown}")),
        }
    }
    let Some(fixture) = fixture else {
        return Err("course plan requires --fixture <path>".to_owned());
    };
    Ok(CoursePlanOptions { fixture })
}

fn print_help() {
    println!(
        "{PRODUCT_NAME} operator CLI\n\nCommands:\n  doctor                         print repository/product invariants\n  market validate                point to the market contract validator\n  course plan --fixture PATH     produce deterministic Course Planning JSON\n              [--format json]\n  affairs publication-status     read bounded durable Affairs publication status\n              --server LOOPBACK:PORT\n  affairs publish-demo           run the fixed M10 → M00/evidence → M71 demo command\n              --server LOOPBACK:PORT --confirm\n  change publication-status      read bounded durable ChangeRadar publication status\n              --server LOOPBACK:PORT\n  change publish-demo            run the fixed M10 → M00/evidence → M70 demo command\n              --server LOOPBACK:PORT --confirm\n  changes ...                    accepted alias for the ChangeRadar commands\n  --version                      show binary version\n  help                           show this message"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn serve_once(schema: &'static str) -> (SocketAddr, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback fixture server");
        let address = listener.local_addr().expect("fixture server address");
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept CLI request");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 1024];
                let read = stream.read(&mut chunk).expect("read CLI request");
                assert!(read > 0, "CLI closed before sending a complete request");
                request.extend_from_slice(&chunk[..read]);
                let Some(header_end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n")
                else {
                    continue;
                };
                let headers = std::str::from_utf8(&request[..header_end])
                    .expect("ASCII fixture request headers");
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().expect("content length"))
                    })
                    .unwrap_or(0);
                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
            let body = format!(r#"{{"schema":"{schema}"}}"#);
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("write CLI response");
        });
        (address, handle)
    }

    #[test]
    fn singular_change_commands_dispatch_to_the_bounded_handlers() {
        let (status_server, status_thread) = serve_once("ustc-change-publication-status/v1");
        run(&strings(&[
            "change",
            "publication-status",
            "--server",
            &status_server.to_string(),
        ]))
        .expect("singular status command dispatch");
        status_thread.join().expect("status fixture server");

        let (publish_server, publish_thread) = serve_once("ustc-change-publication-response/v1");
        run(&strings(&[
            "change",
            "publish-demo",
            "--server",
            &publish_server.to_string(),
            "--confirm",
        ]))
        .expect("singular publish command dispatch");
        publish_thread.join().expect("publish fixture server");
    }

    #[test]
    fn course_plan_options_require_fixture() {
        let result = parse_course_plan_options(&strings(&["--format", "json"]));
        assert!(result.is_err());
    }

    #[test]
    fn course_plan_options_accept_json() {
        let result = parse_course_plan_options(&strings(&[
            "--fixture",
            "market/fixtures/course-planning/minimal-v0.json",
            "--format",
            "json",
        ]));
        let Ok(result) = result else {
            panic!("valid course plan options must parse");
        };
        assert_eq!(
            result.fixture,
            PathBuf::from("market/fixtures/course-planning/minimal-v0.json")
        );
    }

    #[test]
    fn course_plan_options_reject_unknown_format() {
        let result =
            parse_course_plan_options(&strings(&["--fixture", "fixture.json", "--format", "yaml"]));
        assert!(result.is_err());
    }

    #[test]
    fn affairs_publication_options_are_loopback_and_confirmation_bounded() {
        assert!(
            parse_affairs_publication_options(
                &strings(&["--server", "127.0.0.1:8080"]),
                AffairsPublicationAction::Status,
            )
            .is_ok()
        );
        assert!(
            parse_affairs_publication_options(
                &strings(&["--server", "127.0.0.1:8080"]),
                AffairsPublicationAction::Publish,
            )
            .is_err()
        );
        assert!(
            parse_affairs_publication_options(
                &strings(&["--server", "127.0.0.1:8080", "--confirm"]),
                AffairsPublicationAction::Publish,
            )
            .is_ok()
        );
        assert!(
            parse_affairs_publication_options(
                &strings(&["--server", "192.0.2.1:8080"]),
                AffairsPublicationAction::Status,
            )
            .is_err()
        );
    }

    #[test]
    fn change_publication_options_are_loopback_and_confirmation_bounded() {
        assert!(
            parse_change_publication_options(
                &strings(&["--server", "127.0.0.1:8080"]),
                ChangePublicationAction::Status,
            )
            .is_ok()
        );
        assert!(
            parse_change_publication_options(
                &strings(&["--server", "127.0.0.1:8080"]),
                ChangePublicationAction::Publish,
            )
            .is_err()
        );
        assert!(
            parse_change_publication_options(
                &strings(&["--server", "127.0.0.1:8080", "--confirm"]),
                ChangePublicationAction::Publish,
            )
            .is_ok()
        );
        assert!(
            parse_change_publication_options(
                &strings(&["--server", "192.0.2.1:8080"]),
                ChangePublicationAction::Status,
            )
            .is_err()
        );
    }

    #[test]
    fn affairs_http_response_parser_is_bounded_to_status_and_body() {
        let parsed = parse_http_response(
            b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"schema\":\"v1\"}",
        )
        .expect("valid response");
        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.body, "{\"schema\":\"v1\"}");
        assert!(parse_http_response(b"not-http").is_err());
    }
}
