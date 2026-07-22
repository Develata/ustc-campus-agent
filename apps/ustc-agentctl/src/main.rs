use std::path::PathBuf;
use ustc_campus_agent_adapters::adapter_health;
use ustc_campus_agent_core::{
    COURSE_PLANNING_SLICE, DEFAULT_FIRST_PARTY_PLUGIN_IDENTITIES, OPPORTUNITY_GRAPH_PLUGIN_ID,
    PRODUCT_NAME,
};
use ustc_campus_agent_course_planning::{PlanningConfig, load_fixture, plan_fixture};

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
        _ => Err("unknown command; run `ustc-agentctl help`".to_owned()),
    }
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
        "{PRODUCT_NAME} operator CLI\n\nCommands:\n  doctor                         print repository/product invariants\n  market validate                point to the market contract validator\n  course plan --fixture PATH     produce deterministic Course Planning JSON\n              [--format json]\n  --version                      show binary version\n  help                           show this message"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
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
}
