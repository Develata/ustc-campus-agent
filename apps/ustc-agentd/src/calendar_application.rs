//! Calendar application commands bind the admitted subject and server time.
//! Item/proposal transitions and persistence remain in the Calendar owner.
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use ustc_campus_agent_simple_calendar::{CalendarError, CalendarMutation};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProposeCalendarIntent {
    pub schema: String,
    pub request_id: String,
    pub mutation: CalendarMutation,
}
pub(crate) fn now() -> Result<u64, CalendarError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| CalendarError::ClockUnavailable)
}
/// Framework-neutral owner-private Calendar query projection.
pub(crate) fn workspace_list(
    store: &mut ustc_campus_agent_simple_calendar::CalendarStore,
    subject: &str,
) -> Result<serde_json::Value, CalendarError> {
    let timestamp = now()?;
    let reminders = store.dispatch_reminders(timestamp)?;
    Ok(
        serde_json::json!({"schema":"calendar-proposals/v1","now_unix_secs":timestamp,"timezone":"UTC+08:00","proposals":store.proposals(subject)?,"items":store.items()?.to_vec(),"batches":store.batches(subject)?,"reminders":reminders,"reminder_channel":"station_inbox"}),
    )
}

pub(crate) fn batch_chat_definition()
-> Result<crate::chat_tools::ChatDynamicToolDefinition, crate::agent_chat::ChatError> {
    use ustc_agent_tool_protocol::{
        UnvalidatedSchemaNodeV0 as N, UnvalidatedToolInputSchemaV0, ValidatedToolInputSchemaV0,
    };
    let schema = ValidatedToolInputSchemaV0::try_from(UnvalidatedToolInputSchemaV0 {
        dialect: "tool-input-schema/v0".into(),
        root: N::Object {
            properties: vec![(
                "items".into(),
                N::Array {
                    items: Box::new(N::Object {
                        properties: vec![
                            ("title".into(), N::String { enum_values: None }),
                            ("scheduled_for".into(), N::String { enum_values: None }),
                        ],
                        required: vec!["title".into(), "scheduled_for".into()],
                    }),
                },
            )],
            required: vec!["items".into()],
        },
    })
    .map_err(|_| crate::agent_chat::ChatError::Internal)?;
    crate::chat_tools::ChatDynamicToolDefinition::new("plugin_calendar_batch_propose".into(),"提出批量日历提案，1到32个事项，每项title与scheduled_for完整RFC3339时间含UTC偏移。仅用于用户要求加入的明确课程或事项；有歧义先询问。只生成可审阅提案，不执行。用户在日历面板确认全部内容后原子写入并安排站内提醒，模型无权确认。".into(),schema).map_err(|_|crate::agent_chat::ChatError::Internal)
}
