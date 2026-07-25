use ustc_campus_agent_core::{DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES, PRODUCT_NAME};
use ustc_campus_agent_runtime::RUN_SPEC_SCHEMA_VERSION;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("version") => {
            println!("ustc-agentd {}", env!("CARGO_PKG_VERSION"));
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
                "\nCommands:\n  --help      show this message\n  --version   show binary version"
            );
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
}
