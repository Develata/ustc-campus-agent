use std::path::PathBuf;
use std::process::ExitCode;

use ustc_agentd::AffairsComposition;
use ustc_campus_agent_core::{DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES, PRODUCT_NAME};
use ustc_campus_agent_runtime::RUN_SPEC_SCHEMA_VERSION;

#[derive(Clone, Copy)]
enum ServeMode {
    Framed,
    Web,
}

struct ServeOptions {
    bind: String,
    fixture: PathBuf,
    change_fixture: Option<PathBuf>,
    store: PathBuf,
    idempotency: PathBuf,
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("version") => {
            println!("ustc-agentd {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("serve") => run_serve(args, ServeMode::Framed),
        Some("serve-web") => run_serve(args, ServeMode::Web),
        Some("--help") | Some("help") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            ExitCode::from(2)
        }
    }
}

fn run_serve(mut args: std::iter::Skip<std::env::Args>, mode: ServeMode) -> ExitCode {
    let mut bind: Option<String> = None;
    let mut fixture: Option<String> = None;
    let mut change_fixture: Option<String> = None;
    let mut store: Option<String> = None;
    let mut idempotency: Option<String> = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--bind" => match take_value(&mut args, "--bind") {
                Ok(value) => bind = Some(value),
                Err(code) => return code,
            },
            "--fixture" => match take_value(&mut args, "--fixture") {
                Ok(value) => fixture = Some(value),
                Err(code) => return code,
            },
            "--change-fixture" => match take_value(&mut args, "--change-fixture") {
                Ok(value) => change_fixture = Some(value),
                Err(code) => return code,
            },
            "--store" => match take_value(&mut args, "--store") {
                Ok(value) => store = Some(value),
                Err(code) => return code,
            },
            "--idempotency" => match take_value(&mut args, "--idempotency") {
                Ok(value) => idempotency = Some(value),
                Err(code) => return code,
            },
            other => {
                eprintln!("unknown server flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let options = match collect_options(bind, fixture, change_fixture, store, idempotency) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let composition_result = match options.change_fixture.as_deref() {
        Some(change_fixture) => AffairsComposition::open_with_change(
            &options.fixture,
            change_fixture,
            &options.store,
            &options.idempotency,
        ),
        None => AffairsComposition::open(&options.fixture, &options.store, &options.idempotency),
    };
    let composition = match composition_result {
        Ok(composition) => composition,
        Err(error) => {
            eprintln!("composition open failed: {error}");
            return ExitCode::from(1);
        }
    };

    let result = match mode {
        ServeMode::Framed => composition.serve(&options.bind),
        ServeMode::Web => {
            let runtime = match tokio::runtime::Builder::new_multi_thread()
                .enable_io()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    eprintln!("web runtime initialization failed: {error}");
                    return ExitCode::from(1);
                }
            };
            runtime.block_on(composition.serve_web(&options.bind))
        }
    };
    if let Err(error) = result {
        eprintln!("serve failed: {error}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn collect_options(
    bind: Option<String>,
    fixture: Option<String>,
    change_fixture: Option<String>,
    store: Option<String>,
    idempotency: Option<String>,
) -> Result<ServeOptions, ExitCode> {
    Ok(ServeOptions {
        bind: require(bind, "--bind")?,
        fixture: PathBuf::from(require(fixture, "--fixture")?),
        change_fixture: change_fixture.map(PathBuf::from),
        store: PathBuf::from(require(store, "--store")?),
        idempotency: PathBuf::from(require(idempotency, "--idempotency")?),
    })
}

fn print_help() {
    println!("{PRODUCT_NAME} daemon skeleton");
    println!("agent_runtime_kernel_schema={RUN_SPEC_SCHEMA_VERSION}");
    for plugin in DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES {
        println!(
            "default_first_party_plugin={}@{}",
            plugin.id, plugin.version
        );
    }
    println!(
        "\nCommands:\n  --help      show this message\n  --version   show binary version\n  serve       run bounded Affairs, with optional ChangeRadar, over loopback framed TCP\n  serve-web   run the loopback Affairs Web demo, with optional ChangeRadar"
    );
    println!(
        "\nserver flags:\n  --bind <addr>            loopback bind address (e.g. 127.0.0.1:0)\n  --fixture <path>         reviewed Affairs fixture JSON path\n  --change-fixture <path> optional reviewed ChangeRadar fixture JSON path\n  --store <path>           durable record store path\n  --idempotency <path>     durable idempotency store path"
    );
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
