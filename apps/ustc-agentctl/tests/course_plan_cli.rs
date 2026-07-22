use std::path::PathBuf;
use std::process::Command;
use ustc_campus_agent_course_planning::PlanResult;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../market/fixtures/course-planning/minimal-v0.json")
}

#[test]
fn course_plan_cli_emits_valid_zero_violation_json() {
    let output = Command::new(env!("CARGO_BIN_EXE_ustc-agentctl"))
        .arg("course")
        .arg("plan")
        .arg("--fixture")
        .arg(fixture_path())
        .arg("--format")
        .arg("json")
        .output();
    let Ok(output) = output else {
        panic!("course plan CLI must execute");
    };
    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let result = serde_json::from_slice::<PlanResult>(&output.stdout);
    let Ok(result) = result else {
        panic!("course plan CLI stdout must be PlanResult JSON");
    };
    assert_eq!(result.schema_version, "course-plan-result/v0");
    assert!(result.candidates.len() >= 2);
    assert_eq!(result.hard_constraint_violations, 0);
}

#[test]
fn course_plan_cli_rejects_non_json_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_ustc-agentctl"))
        .arg("course")
        .arg("plan")
        .arg("--fixture")
        .arg(fixture_path())
        .arg("--format")
        .arg("yaml")
        .output();
    let Ok(output) = output else {
        panic!("course plan CLI must execute");
    };
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported format"));
}
