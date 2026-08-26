use std::path::PathBuf;
use std::process::ExitCode;

use ustc_agentd::AffairsComposition;
use ustc_campus_agent_core::{DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES, PRODUCT_NAME};
use ustc_campus_agent_runtime::RUN_SPEC_SCHEMA_VERSION;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("version") => {
            println!("ustc-agentd {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("serve") => {
            let mut bind: Option<String> = None;
            let mut fixture: Option<String> = None;
            let mut store: Option<String> = None;
            let mut idempotency: Option<String> = None;
            while let Some(flag) = args.next() {
                match flag.as_str() {
                    "--bind" => match take_value(&mut args, "--bind") {
                        Ok(v) => bind = Some(v),
                        Err(code) => return code,
                    },
                    "--fixture" => match take_value(&mut args, "--fixture") {
                        Ok(v) => fixture = Some(v),
                        Err(code) => return code,
                    },
                    "--store" => match take_value(&mut args, "--store") {
                        Ok(v) => store = Some(v),
                        Err(code) => return code,
                    },
                    "--idempotency" => match take_value(&mut args, "--idempotency") {
                        Ok(v) => idempotency = Some(v),
                        Err(code) => return code,
                    },
                    other => {
                        eprintln!("unknown serve flag: {other}");
                        return ExitCode::from(2);
                    }
                }
            }
            let bind = match require(bind, "--bind") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let fixture = match require(fixture, "--fixture") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let store = match require(store, "--store") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let idempotency = match require(idempotency, "--idempotency") {
                Ok(v) => v,
                Err(code) => return code,
            };
            let composition = match AffairsComposition::open(
                &PathBuf::from(fixture),
                &PathBuf::from(store),
                &PathBuf::from(idempotency),
            ) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("composition open failed: {e}");
                    return ExitCode::from(1);
                }
            };
            if let Err(e) = composition.serve(&bind) {
                eprintln!("serve failed: {e}");
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
        Some("--help") | Some("help") | None => {
            println!("{PRODUCT_NAME} daemon skeleton");
            println!("agent_runtime_kernel_schema={RUN_SPEC_SCHEMA_VERSION}");
            for plugin in DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES {
                println!(
                    "default_first_party_plugin={}@{}",
                    plugin.id, plugin.version
                );
            }
            println!(
                "\nCommands:\n  --help      show this message\n  --version   show binary version\n  serve       run the bounded affairs composition over loopback TCP"
            );
            println!(
                "\nserve flags:\n  --bind <addr>         loopback bind address (e.g. 127.0.0.1:0)\n  --fixture <path>      durable fixture JSON path\n  --store <path>        durable record store path\n  --idempotency <path>  durable idempotency store path"
            );
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            ExitCode::from(2)
        }
    }
}

fn take_value(iter: &mut std::iter::Skip<std::env::Args>, flag: &str) -> Result<String, ExitCode> {
    iter.next().ok_or_else(|| {
        eprintln!("missing value for {flag}");
        ExitCode::from(2)
    })
}

fn require(value: Option<String>, flag: &str) -> Result<String, ExitCode> {
    value.ok_or_else(|| {
        eprintln!("missing required flag {flag}");
        ExitCode::from(2)
    })
}
