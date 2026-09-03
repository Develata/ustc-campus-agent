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
    opportunity_fixture: Option<PathBuf>,
    opportunity_catalog: Option<PathBuf>,
    opportunity_profile_store: Option<PathBuf>,
    store: PathBuf,
    idempotency: PathBuf,
    session_store: PathBuf,
}

#[derive(Default)]
struct RawServeOptions {
    bind: Option<String>,
    fixture: Option<String>,
    change_fixture: Option<String>,
    opportunity_fixture: Option<String>,
    opportunity_catalog: Option<String>,
    opportunity_profile_store: Option<String>,
    store: Option<String>,
    idempotency: Option<String>,
    session_store: Option<String>,
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
    let mut raw = RawServeOptions::default();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--bind" => match take_value(&mut args, "--bind") {
                Ok(value) => raw.bind = Some(value),
                Err(code) => return code,
            },
            "--fixture" => match take_value(&mut args, "--fixture") {
                Ok(value) => raw.fixture = Some(value),
                Err(code) => return code,
            },
            "--change-fixture" => match take_value(&mut args, "--change-fixture") {
                Ok(value) => raw.change_fixture = Some(value),
                Err(code) => return code,
            },
            "--opportunity-fixture" => match take_value(&mut args, "--opportunity-fixture") {
                Ok(value) => raw.opportunity_fixture = Some(value),
                Err(code) => return code,
            },
            "--opportunity-catalog" => match take_value(&mut args, "--opportunity-catalog") {
                Ok(value) => raw.opportunity_catalog = Some(value),
                Err(code) => return code,
            },
            "--opportunity-profile-store" => {
                match take_value(&mut args, "--opportunity-profile-store") {
                    Ok(value) => raw.opportunity_profile_store = Some(value),
                    Err(code) => return code,
                }
            }
            "--store" => match take_value(&mut args, "--store") {
                Ok(value) => raw.store = Some(value),
                Err(code) => return code,
            },
            "--idempotency" => match take_value(&mut args, "--idempotency") {
                Ok(value) => raw.idempotency = Some(value),
                Err(code) => return code,
            },
            "--session-store" => match take_value(&mut args, "--session-store") {
                Ok(value) => raw.session_store = Some(value),
                Err(code) => return code,
            },
            other => {
                eprintln!("unknown server flag: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let options = match collect_options(raw) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let opportunity_paths = options
        .opportunity_fixture
        .as_deref()
        .zip(options.opportunity_catalog.as_deref())
        .zip(options.opportunity_profile_store.as_deref())
        .map(|((fixture, catalog), profile_store)| (fixture, catalog, profile_store));
    let composition_result = match (options.change_fixture.as_deref(), opportunity_paths) {
        (Some(change_fixture), Some((opportunity_fixture, catalog, profile_store))) => {
            AffairsComposition::open_with_change_and_opportunity(
                &options.fixture,
                change_fixture,
                opportunity_fixture,
                catalog,
                profile_store,
                &options.store,
                &options.idempotency,
                &options.session_store,
            )
        }
        (None, Some((opportunity_fixture, catalog, profile_store))) => {
            AffairsComposition::open_with_opportunity(
                &options.fixture,
                opportunity_fixture,
                catalog,
                profile_store,
                &options.store,
                &options.idempotency,
                &options.session_store,
            )
        }
        (Some(change_fixture), None) => AffairsComposition::open_with_change(
            &options.fixture,
            change_fixture,
            &options.store,
            &options.idempotency,
            &options.session_store,
        ),
        (None, None) => AffairsComposition::open(
            &options.fixture,
            &options.store,
            &options.idempotency,
            &options.session_store,
        ),
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
                .enable_time()
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

fn collect_options(raw: RawServeOptions) -> Result<ServeOptions, ExitCode> {
    let RawServeOptions {
        bind,
        fixture,
        change_fixture,
        opportunity_fixture,
        opportunity_catalog,
        opportunity_profile_store,
        store,
        idempotency,
        session_store,
    } = raw;
    match (
        opportunity_fixture.as_ref(),
        opportunity_catalog.as_ref(),
        opportunity_profile_store.as_ref(),
    ) {
        (None, None, None) | (Some(_), Some(_), Some(_)) => {}
        _ => {
            eprintln!(
                "--opportunity-fixture, --opportunity-catalog and --opportunity-profile-store must be supplied together"
            );
            return Err(ExitCode::from(2));
        }
    }
    Ok(ServeOptions {
        bind: require(bind, "--bind")?,
        fixture: PathBuf::from(require(fixture, "--fixture")?),
        change_fixture: change_fixture.map(PathBuf::from),
        opportunity_fixture: opportunity_fixture.map(PathBuf::from),
        opportunity_catalog: opportunity_catalog.map(PathBuf::from),
        opportunity_profile_store: opportunity_profile_store.map(PathBuf::from),
        store: PathBuf::from(require(store, "--store")?),
        idempotency: PathBuf::from(require(idempotency, "--idempotency")?),
        session_store: PathBuf::from(require(session_store, "--session-store")?),
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
        "\nCommands:\n  --help      show this message\n  --version   show binary version\n  serve       run bounded Affairs, with optional independent ChangeRadar and Opportunity Graph, over loopback framed TCP\n  serve-web   run the loopback three-Plugin Web demo; each optional Plugin fails closed when not configured"
    );
    println!(
        "\nserver flags:\n  --bind <addr>                      loopback bind address (e.g. 127.0.0.1:0)\n  --fixture <path>                   reviewed Affairs fixture JSON path\n  --change-fixture <path>            optional reviewed ChangeRadar fixture JSON path\n  --opportunity-fixture <path>       optional DemoReviewed Opportunity metadata JSON\n  --opportunity-catalog <path>       retained Course Planning catalog bytes (required with Opportunity)\n  --opportunity-profile-store <path> tenant-private durable profile/tombstone store (required with Opportunity)\n  --store <path>                     durable Affairs record store path\n  --idempotency <path>               common durable M10 idempotency store path\n  --session-store <path>             durable current-session event store path"
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
