use ustc_campus_agent_adapters::adapter_health;
use ustc_campus_agent_core::{FIRST_VERTICAL_SLICE, OPPORTUNITY_GRAPH_PLUGIN_ID, PRODUCT_NAME};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => print_help(),
        [cmd] if cmd == "--help" || cmd == "help" => print_help(),
        [cmd] if cmd == "--version" || cmd == "version" => {
            println!("ustc-agentctl {}", env!("CARGO_PKG_VERSION"));
        }
        [cmd] if cmd == "doctor" => {
            println!("product={PRODUCT_NAME}");
            println!("first_plugin={OPPORTUNITY_GRAPH_PLUGIN_ID}");
            println!("first_vertical_slice={FIRST_VERTICAL_SLICE}");
            println!("{}", adapter_health());
        }
        [cmd, sub] if cmd == "market" && sub == "validate" => {
            println!("market validation is implemented by scripts/check_repo_contracts.py");
            println!("first_party_package={OPPORTUNITY_GRAPH_PLUGIN_ID}");
        }
        _ => {
            eprintln!("unknown command; run `ustc-agentctl help`");
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!(
        "{PRODUCT_NAME} operator CLI skeleton\n\nCommands:\n  doctor             print repository/product invariants\n  market validate    point to the market contract validator\n  --version          show binary version\n  help               show this message"
    );
}
