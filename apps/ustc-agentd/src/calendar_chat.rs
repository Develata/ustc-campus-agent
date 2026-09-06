//! Compact read-only Calendar projection within the complete Chat result budget.
use crate::chat_tools::ChatToolExecution;
use serde_json::{Value, json};
use ustc_campus_agent_simple_calendar::CalendarItem;

pub(super) fn list_result(items: &[CalendarItem]) -> ChatToolExecution {
    let clock = time::OffsetDateTime::now_utc();
    let server_now = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        clock.year(),
        u8::from(clock.month()),
        clock.day(),
        clock.hour(),
        clock.minute(),
        clock.second()
    );
    let items: Vec<Value> = items
        .iter()
        .map(|item| {
            let mut value = json!({"id":item.id,"title":item.title});
            if let Some(scheduled_for) = &item.scheduled_for {
                value["scheduled_for"] = json!(scheduled_for);
            }
            value
        })
        .collect();
    ChatToolExecution::succeeded(json!({
        "schema":"ustc-simple-calendar-result/v1",
        "package_id":"ustc.simple-calendar",
        "action":"list",
        "server_now": server_now,
        "timezone":"UTC+08:00",
        "reminder_delivery":false,
        "items":items,
    }))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::agent_chat::{ChatRequestDto, run_bounded_chat};
    use crate::chat_provider::ChatProvider;
    use crate::chat_tools::{
        CalendarAction, ChatToolRequest, ChatToolStatus, MAX_TOOL_RESULT_BYTES,
    };
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    use ustc_campus_agent_simple_calendar::CalendarStore;

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct TempRoot(PathBuf);
    impl TempRoot {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "uca-calendar-chat-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("new fixture root");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("private root");
            Self(path)
        }
    }
    impl Drop for TempRoot {
        fn drop(&mut self) {
            if self.0.parent() == Some(std::env::temp_dir().as_path())
                && self.0.file_name().is_some_and(|name| {
                    name.to_string_lossy()
                        .starts_with("uca-calendar-chat-test-")
                })
            {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }

    #[tokio::test]
    async fn calendar_chat_near_capacity_store_lists_every_item_through_real_mock() {
        for scheduled_for in [None, Some("2026-09-05T12:00:00Z")] {
            let temp = TempRoot::new();
            let path = temp.0.join("calendar.json");
            let mut store = CalendarStore::open(&path).expect("isolated real store");
            while store.record(&"\\".repeat(256), scheduled_for).is_ok() {}
            for length in (1..=256).rev() {
                if store.record(&"\\".repeat(length), scheduled_for).is_ok() {
                    break;
                }
            }
            let before = fs::read(&path).expect("durable data");
            assert!(
                before.len() >= 65_534,
                "fixture reaches actual store byte boundary"
            );
            let items = store.items().expect("items").to_vec();
            let previous = ChatToolExecution::succeeded(
                json!({"schema":"ustc-simple-calendar-result/v1","package_id":"ustc.simple-calendar","action":"list","items":items}),
            );
            assert!(
                previous.serialize_for_provider().is_err(),
                "old full-record projection exceeds Chat budget"
            );
            let serialized = list_result(&items)
                .serialize_for_provider()
                .expect("bounded complete list");
            assert!(serialized.len() <= MAX_TOOL_RESULT_BYTES);
            let result: Value = serde_json::from_str(&serialized).expect("projection");
            let projected = result["data"]["items"].as_array().expect("all items");
            assert_eq!(projected.len(), items.len());
            for (value, item) in projected.iter().zip(&items) {
                assert_eq!(value["id"], item.id);
                assert_eq!(value["title"], item.title);
                assert_eq!(
                    value.get("scheduled_for").and_then(Value::as_str),
                    scheduled_for
                );
                assert!(value.get("created_at_unix_secs").is_none());
            }
            let request: ChatRequestDto = serde_json::from_value(json!({"schema":"ustc-agent-chat-request/v1","messages":[{"role":"user","content":"查看我的事项"}]})).expect("request");
            let mut calls = 0;
            let response = run_bounded_chat(
                "chat-run:calendar-boundary".to_owned(),
                request,
                false,
                &ChatProvider::deterministic_mock(),
                &mut |request| {
                    assert!(matches!(
                        request,
                        ChatToolRequest::CalendarItems {
                            action: CalendarAction::List,
                            ..
                        }
                    ));
                    calls += 1;
                    list_result(&items)
                },
            )
            .await
            .expect("real coordinator and mock can read near-capacity store");
            assert_eq!(calls, 1);
            assert_eq!(response.tool_trace.len(), 1);
            assert_eq!(response.tool_trace[0].status, ChatToolStatus::Succeeded);
            assert!(
                response
                    .answer
                    .contains(&format!("当前共 {} 项", items.len()))
            );
            assert!(response.answer.contains("calendar:item:1"));
            if let Some(scheduled_for) = scheduled_for {
                assert!(response.answer.contains(scheduled_for));
            }
            assert_eq!(fs::read(&path).expect("unchanged store"), before);
            drop(store);
            let mut reopened = CalendarStore::open(&path).expect("restart");
            assert_eq!(reopened.items().expect("retained items"), items);
        }
    }
}
