use ustc_campus_agent_core::{OPPORTUNITY_GRAPH_PLUGIN_ID, PRODUCT_NAME};

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--version") | Some("version") => {
            println!("ustc-agentd {}", env!("CARGO_PKG_VERSION"));
        }
        Some("--help") | Some("help") | None => {
            println!(
                "{PRODUCT_NAME} daemon skeleton\n\nfirst_plugin={OPPORTUNITY_GRAPH_PLUGIN_ID}\n\nCommands:\n  --help      show this message\n  --version   show binary version"
            );
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
}
