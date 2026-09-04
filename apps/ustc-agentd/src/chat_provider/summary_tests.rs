#![allow(clippy::unwrap_used)]

use super::{AFFAIRS_TOOL, CALENDAR_TOOL, CHANGE_TOOL, OPPORTUNITY_TOOL, render_mock_tool_result};
use serde_json::{Value, json};

fn opportunity_candidate(code: &str, rationale_count: usize) -> Value {
    json!({
        "course_codes": [code],
        "total_credits": 4,
        "soft_score": 97,
        "hard_constraint_violations": [],
        "requirement_credits": [],
        "provenance": [],
        "rationale": (0..rationale_count)
            .map(|index| format!("reason-{index}"))
            .collect::<Vec<_>>()
    })
}

#[test]
fn deterministic_mock_summarizes_each_known_tool_shape_for_people() {
    let affairs = render_mock_tool_result(
        Some(AFFAIRS_TOOL),
        &json!({
            "kind": "available",
            "terminal": {"outcome": {
                "kind": "found",
                "view": {
                    "title": "成绩单证明办理",
                    "procedure_id": "proc:ustc:undergraduate:transcript-certificate",
                    "ordered_steps": [
                        {"ordinal": 1, "instruction": "登录综合教务系统"},
                        {"ordinal": 2, "instruction": "选择材料并下载"}
                    ],
                    "entry_points": [
                        {"label": "综合教务系统", "url": "https://jw.ustc.edu.cn/"}
                    ]
                }
            }}
        }),
    )
    .unwrap();
    assert!(affairs.contains("办事流程：成绩单证明办理"));
    assert!(affairs.contains("1. 登录综合教务系统"));
    assert!(affairs.contains("https://jw.ustc.edu.cn/"));
    assert!(!affairs.contains("ordered_steps"));

    let change = render_mock_tool_result(
        Some(CHANGE_TOOL),
        &json!({
            "kind": "change_feed_accepted",
            "terminal": {"outcome": {
                "kind": "found",
                "view": {
                    "title": "USTC Academic Calendar Changes",
                    "board_id": "board:ustc:academic-calendar",
                    "entries": [{
                        "affected_scope": "2026 秋季本科生选课",
                        "changed_fields": [{
                            "field": "registration.deadline",
                            "before": "2026-09-01T17:00:00+08:00",
                            "after": "2026-09-03T17:00:00+08:00"
                        }],
                        "source_url": "https://www.teach.ustc.edu.cn/calendar/2026-fall"
                    }]
                }
            }}
        }),
    )
    .unwrap();
    assert!(change.contains("校历变更：USTC Academic Calendar Changes"));
    assert!(change.contains("registration.deadline"));
    assert!(change.contains("2026-09-01T17:00:00+08:00 → 2026-09-03T17:00:00+08:00"));
    assert!(!change.contains("changed_fields"));

    let opportunity = render_mock_tool_result(
        Some(OPPORTUNITY_TOOL),
        &json!({
            "kind": "opportunity_accepted",
            "terminal": {
                "kind": "plan_generated",
                "plan": {"decision": {
                    "kind": "planned",
                    "hard_constraint_violations": 0,
                    "warnings": [],
                    "candidates": [{
                        "course_codes": ["MATH2001", "MATH2003"],
                        "total_credits": 10,
                        "soft_score": 97,
                        "hard_constraint_violations": [],
                        "requirement_credits": [],
                        "provenance": [],
                        "rationale": [
                            "MATH2001 community signal 97/100; verify the linked iCourse page before deciding: https://icourse.club/course/2059/"
                        ]
                    }]
                }}
            }
        }),
    )
    .unwrap();
    assert!(opportunity.contains("课程建议"));
    assert!(opportunity.contains("MATH2001、MATH2003（10 学分，soft score 97）"));
    assert!(opportunity.contains("hard constraints 通过"));
    assert!(opportunity.contains("https://icourse.club/course/2059/"));
    assert!(!opportunity.contains("course_codes"));

    let calendar = render_mock_tool_result(
        Some(CALENDAR_TOOL),
        &json!({
            "schema": "ustc-simple-calendar-result/v1",
            "package_id": "ustc.simple-calendar",
            "action": "record",
            "item": {
                "id": "calendar:item:1",
                "title": "提交开题报告",
                "scheduled_for": "2026-09-10T09:00:00+08:00",
                "created_at_unix_secs": 1
            }
        }),
    )
    .unwrap();
    assert!(calendar.contains("已记录「提交开题报告」"));
    assert!(calendar.contains("calendar:item:1"));
    assert!(calendar.contains("2026-09-10T09:00:00+08:00"));
    assert!(!calendar.contains("scheduled_for"));
}

#[test]
fn known_tool_shape_drift_is_explicit_instead_of_dumping_transport_json() {
    for tool_name in [AFFAIRS_TOOL, CHANGE_TOOL, OPPORTUNITY_TOOL, CALENDAR_TOOL] {
        let answer = render_mock_tool_result(
            Some(tool_name),
            &json!({"unexpected": "transport-internal-marker"}),
        )
        .unwrap();
        assert!(answer.contains("结果结构"), "tool={tool_name}");
        assert!(answer.contains("功能面板"), "tool={tool_name}");
        assert!(
            !answer.contains("transport-internal-marker"),
            "tool={tool_name}"
        );
    }
}

#[test]
fn deterministic_tool_summaries_disclose_internal_omissions() {
    let steps = (1_u64..=7)
        .map(|ordinal| json!({"ordinal": ordinal, "instruction": format!("step-{ordinal}")}))
        .collect::<Vec<_>>();
    let entry_points = (0..4)
        .map(|index| json!({"label": format!("entry-{index}"), "url": null}))
        .collect::<Vec<_>>();
    let affairs = render_mock_tool_result(
        Some(AFFAIRS_TOOL),
        &json!({
            "kind": "available",
            "terminal": {"outcome": {"kind": "found", "view": {
                "title": "Procedure",
                "procedure_id": "procedure:1",
                "ordered_steps": steps,
                "entry_points": entry_points
            }}}
        }),
    )
    .unwrap();
    assert!(affairs.contains("另有 1 个步骤未展开"));
    assert!(affairs.contains("另有 1 个入口未展开"));

    let fields = (0..7)
        .map(|index| {
            json!({
                "field": format!("field-{index}"),
                "before": null,
                "after": format!("after-{index}")
            })
        })
        .collect::<Vec<_>>();
    let entries = (0..4)
        .map(|index| {
            json!({
                "affected_scope": format!("scope-{index}"),
                "changed_fields": fields,
                "source_url": format!("https://example.invalid/{index}")
            })
        })
        .collect::<Vec<_>>();
    let change = render_mock_tool_result(
        Some(CHANGE_TOOL),
        &json!({
            "kind": "change_feed_accepted",
            "terminal": {"outcome": {"kind": "found", "view": {
                "title": "Calendar",
                "board_id": "board:1",
                "entries": entries
            }}}
        }),
    )
    .unwrap();
    assert!(change.contains("另有 1 个字段变更未展开"));
    assert!(change.contains("另有 1 条变更未展开"));

    let candidates = (0..4)
        .map(|index| opportunity_candidate(&format!("COURSE{index}"), 7))
        .collect::<Vec<_>>();
    let opportunity = render_mock_tool_result(
        Some(OPPORTUNITY_TOOL),
        &json!({
            "kind": "opportunity_accepted",
            "terminal": {"kind": "plan_generated", "plan": {"decision": {
                "kind": "planned",
                "hard_constraint_violations": 0,
                "warnings": ["warning-0", "warning-1", "warning-2", "warning-3"],
                "candidates": candidates
            }}}
        }),
    )
    .unwrap();
    assert!(opportunity.contains("另有 1 条理由未展开"));
    assert!(opportunity.contains("另有 1 个候选未展开"));
    assert!(opportunity.contains("另有 1 条规划提示未展开"));

    let items = (0..11)
        .map(|index| {
            json!({
                "id": format!("calendar:item:{index}"),
                "title": format!("item-{index}"),
                "scheduled_for": null,
                "created_at_unix_secs": index
            })
        })
        .collect::<Vec<_>>();
    let calendar = render_mock_tool_result(
        Some(CALENDAR_TOOL),
        &json!({
            "schema": "ustc-simple-calendar-result/v1",
            "package_id": "ustc.simple-calendar",
            "action": "list",
            "items": items
        }),
    )
    .unwrap();
    assert!(calendar.contains("另有 1 项未展开"));
}
