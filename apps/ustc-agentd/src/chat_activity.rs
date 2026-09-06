//! Bounded, discardable observation cache; persisted conversation outcomes win.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::agent_chat::{ChatActivityEvent, ChatActivityObserver, ChatActivityTool};
use crate::chat_conversations::{ConversationTurnDto, TurnPhase};
use crate::chat_tools::ChatToolStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActivityPhase {
    Idle,
    Running,
    Completed,
    Failed,
    Interrupted,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActivityKind {
    Model,
    Tool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActivityStatus {
    Interrupted,
    Running,
    Succeeded,
    Denied,
    Failed,
}
impl From<ChatToolStatus> for ActivityStatus {
    fn from(status: ChatToolStatus) -> Self {
        match status {
            ChatToolStatus::Succeeded => Self::Succeeded,
            ChatToolStatus::Denied => Self::Denied,
            ChatToolStatus::Failed => Self::Failed,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActivityStepDto {
    pub(crate) id: String,
    pub(crate) kind: ActivityKind,
    pub(crate) tool: Option<ChatActivityTool>,
    pub(crate) status: ActivityStatus,
}
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChatActivityDto {
    pub(crate) schema: &'static str,
    pub(crate) conversation_id: String,
    pub(crate) request_id: Option<String>,
    pub(crate) phase: ActivityPhase,
    pub(crate) sequence: u32,
    pub(crate) steps: Vec<ActivityStepDto>,
    pub(crate) partial_answer: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChatProgress {
    pub(crate) tool_results: Vec<serde_json::Value>,
    pub(crate) steps: Vec<ActivityStepDto>,
    pub(crate) partial_answer: String,
}
impl ChatProgress {
    pub(crate) fn valid(&self) -> bool {
        self.tool_results.len() <= 4
            && self.tool_results.iter().enumerate().all(|(index, v)| {
                v.as_object().is_some_and(|o| {
                    o.len() == 4
                        && ["schema", "trust", "status", "data"]
                            .iter()
                            .all(|k| o.contains_key(*k))
                }) && v["schema"] == "ustc-agent-chat-tool-result/v1"
                    && v["trust"] == "untrusted_data"
                    && matches!(
                        v["status"].as_str(),
                        Some("succeeded" | "denied" | "failed")
                    )
                    && self.steps.iter().any(|s| {
                        s.id == format!("call-{}", index + 1)
                            && s.kind == ActivityKind::Tool
                            && s.status != ActivityStatus::Running
                    })
                    && serde_json::to_vec(v).is_ok_and(|v| v.len() <= 64 * 1024)
            })
            && self.partial_answer.len() <= 16 * 1024
            && !self.partial_answer.contains('\0')
            && self.steps.len() <= 7
            && self.steps.iter().enumerate().all(|(i, step)| {
                !self.steps[..i].iter().any(|old| old.id == step.id)
                    && match step.kind {
                        ActivityKind::Model => {
                            step.tool.is_none()
                                && (1..=3).any(|n| step.id == format!("model-{n}"))
                                && step.status != ActivityStatus::Denied
                        }
                        ActivityKind::Tool => {
                            step.tool.is_some() && (1..=4).any(|n| step.id == format!("call-{n}"))
                        }
                    }
            })
    }
}
#[derive(Default)]
struct Observation {
    request_id: String,
    sequence: u32,
    steps: Vec<ActivityStepDto>,
    partial_answer: String,
    tool_results: Vec<serde_json::Value>,
    cancellation: Option<tokio::sync::watch::Sender<bool>>,
}
#[derive(Default)]
pub(crate) struct ActivityRegistry {
    active: Mutex<BTreeMap<String, Arc<Mutex<Observation>>>>,
}
impl ActivityRegistry {
    pub(crate) fn register(self: &Arc<Self>, id: &str, request_id: &str) -> ActivityGuard {
        let (cancellation, _) = tokio::sync::watch::channel(false);
        let observation = Arc::new(Mutex::new(Observation {
            cancellation: Some(cancellation),
            request_id: request_id.to_owned(),
            ..Observation::default()
        }));
        if let Ok(mut active) = self.active.lock()
            && (active.len() < 4 || active.contains_key(id))
        {
            active.insert(id.to_owned(), Arc::clone(&observation));
        }
        ActivityGuard {
            registry: Arc::clone(self),
            id: id.to_owned(),
            observation,
            checkpoint: None,
        }
    }

    pub(crate) fn cancel(&self, id: &str, request_id: &str) -> bool {
        if let Ok(active) = self.active.lock()
            && let Some(observation) = active.get(id)
            && let Ok(observation) = observation.lock()
            && observation.request_id == request_id
            && let Some(sender) = &observation.cancellation
        {
            sender.send_replace(true);
            return true;
        }
        false
    }

    /// Caller must first obtain this exact turn through an owner-admitted store query.
    pub(crate) fn project(&self, id: &str, turn: Option<&ConversationTurnDto>) -> ChatActivityDto {
        let mut dto = ChatActivityDto {
            schema: "chat-conversation-activity/v2",
            conversation_id: id.to_owned(),
            request_id: turn.map(|t| t.request_id.clone()),
            phase: ActivityPhase::Idle,
            sequence: 0,
            steps: Vec::new(),
            partial_answer: String::new(),
        };
        let Some(turn) = turn else {
            return dto;
        };
        dto.phase = match turn.phase {
            TurnPhase::Running => ActivityPhase::Running,
            TurnPhase::Completed => ActivityPhase::Completed,
            TurnPhase::Failed => ActivityPhase::Failed,
            TurnPhase::Interrupted => ActivityPhase::Interrupted,
        };
        if turn.phase == TurnPhase::Running {
            if let Ok(active) = self.active.lock()
                && let Some(observation) = active.get(id)
                && let Ok(observation) = observation.lock()
                && observation.request_id == turn.request_id
            {
                dto.sequence = observation.sequence;
                dto.steps.clone_from(&observation.steps);
                dto.partial_answer.clone_from(&observation.partial_answer);
            }
        } else {
            dto.sequence = u32::MAX;
            // Rebuild only allowlisted fields of the canonical final trace. Do not
            // recover model phases or project arbitrary persisted JSON strings.
            if let Some(trace) = turn
                .response
                .as_ref()
                .and_then(|response| response.get("tool_trace"))
                .and_then(serde_json::Value::as_array)
            {
                for (index, entry) in trace.iter().take(4).enumerate() {
                    let Some(tool) = entry
                        .get("tool")
                        .and_then(serde_json::Value::as_str)
                        .and_then(ChatActivityTool::from_name)
                    else {
                        continue;
                    };
                    let status = match entry.get("status").and_then(serde_json::Value::as_str) {
                        Some("succeeded") => ActivityStatus::Succeeded,
                        Some("denied") => ActivityStatus::Denied,
                        Some("failed") => ActivityStatus::Failed,
                        _ => continue,
                    };
                    dto.steps.push(ActivityStepDto {
                        id: format!("call-{}", index + 1),
                        kind: ActivityKind::Tool,
                        tool: Some(tool),
                        status,
                    });
                }
            }
        }
        dto
    }
}
type CheckpointSink = Box<dyn Fn(&ChatProgress) -> Result<(), crate::agent_chat::ChatError> + Send>;
pub(crate) struct ActivityGuard {
    registry: Arc<ActivityRegistry>,
    id: String,
    observation: Arc<Mutex<Observation>>,
    checkpoint: Option<CheckpointSink>,
}
impl ActivityGuard {
    pub(crate) fn cancellation(&self) -> tokio::sync::watch::Receiver<bool> {
        self.observation
            .lock()
            .expect("activity lock")
            .cancellation
            .as_ref()
            .expect("cancellation")
            .subscribe()
    }
    pub(crate) fn on_checkpoint(
        &mut self,
        checkpoint: impl Fn(&ChatProgress) -> Result<(), crate::agent_chat::ChatError> + Send + 'static,
    ) {
        self.checkpoint = Some(Box::new(checkpoint));
    }
    pub(crate) fn snapshot(&self) -> ChatProgress {
        self.observation
            .lock()
            .map(|o| ChatProgress {
                steps: o.steps.clone(),
                partial_answer: o.partial_answer.clone(),
                tool_results: o.tool_results.clone(),
            })
            .unwrap_or_default()
    }
}
impl ChatActivityObserver for ActivityGuard {
    fn check_cancelled(&self) -> Result<(), crate::agent_chat::ChatError> {
        if self
            .observation
            .lock()
            .map_err(|_| crate::agent_chat::ChatError::Internal)?
            .cancellation
            .as_ref()
            .is_some_and(|s| *s.borrow())
        {
            return Err(crate::agent_chat::ChatError::Cancelled);
        }
        Ok(())
    }
    fn tool_result(&mut self, call: u8, result: &str) {
        if let Ok(mut o) = self.observation.lock()
            && usize::from(call) == o.tool_results.len() + 1
            && result.len() <= 64 * 1024
            && let Ok(value) = serde_json::from_str(result)
        {
            o.tool_results.push(value);
        }
    }
    fn text_delta(&mut self, text: &str) {
        if let Ok(mut o) = self.observation.lock()
            && o.partial_answer.len() + text.len() <= 16 * 1024
        {
            o.partial_answer.push_str(text);
            o.sequence = o.sequence.saturating_add(1).min(u32::MAX - 1);
        }
    }
    fn checkpoint(&mut self) -> Result<(), crate::agent_chat::ChatError> {
        if let Some(checkpoint) = &self.checkpoint {
            checkpoint(&self.snapshot())?;
        }
        Ok(())
    }
    fn observe(&mut self, event: ChatActivityEvent) {
        if matches!(event, ChatActivityEvent::ModelStarted { .. })
            && let Ok(mut observation) = self.observation.lock()
        {
            observation.partial_answer.clear();
        }
        let (id, kind, tool, status) = match event {
            ChatActivityEvent::ModelStarted { turn } if (1..=3).contains(&turn) => (
                format!("model-{turn}"),
                ActivityKind::Model,
                None,
                ActivityStatus::Running,
            ),
            ChatActivityEvent::ModelFinished { turn, succeeded } if (1..=3).contains(&turn) => (
                format!("model-{turn}"),
                ActivityKind::Model,
                None,
                if succeeded {
                    ActivityStatus::Succeeded
                } else {
                    ActivityStatus::Failed
                },
            ),
            ChatActivityEvent::ToolStarted { call, tool } if (1..=4).contains(&call) => (
                format!("call-{call}"),
                ActivityKind::Tool,
                Some(tool),
                ActivityStatus::Running,
            ),
            ChatActivityEvent::ToolFinished { call, tool, status } if (1..=4).contains(&call) => (
                format!("call-{call}"),
                ActivityKind::Tool,
                Some(tool),
                status.into(),
            ),
            _ => return,
        };
        if let Ok(mut observation) = self.observation.lock() {
            if let Some(step) = observation.steps.iter_mut().find(|step| step.id == id) {
                // Observations are monotonic: no late start can reopen a terminal step.
                if step.status != ActivityStatus::Running {
                    return;
                }
                step.status = status;
            } else if observation.steps.len() < 7 {
                observation.steps.push(ActivityStepDto {
                    id,
                    kind,
                    tool,
                    status,
                });
            } else {
                return;
            }
            observation.sequence = observation.sequence.saturating_add(1).min(u32::MAX - 1);
        }
    }
}
impl Drop for ActivityGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = self.registry.active.lock()
            && active
                .get(&self.id)
                .is_some_and(|value| Arc::ptr_eq(value, &self.observation))
        {
            active.remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn turn(request: &str, phase: TurnPhase) -> ConversationTurnDto {
        ConversationTurnDto {
            request_id: request.to_owned(),
            phase,
            user: "private".to_owned(),
            response: None,
            error: None,
        }
    }
    #[test]
    fn activity_registry_request_binding_bounds_and_raii() {
        let registry = Arc::new(ActivityRegistry::default());
        let mut guard = registry.register("conversation", "request");
        guard.observe(ChatActivityEvent::ModelStarted { turn: 1 });
        assert_eq!(
            registry
                .project("conversation", Some(&turn("request", TurnPhase::Running)))
                .steps
                .len(),
            1
        );
        assert!(
            registry
                .project("conversation", Some(&turn("other", TurnPhase::Running)))
                .steps
                .is_empty()
        );
        let replacement = registry.register("conversation", "other");
        drop(guard);
        assert_eq!(
            registry
                .active
                .lock()
                .expect("valid activity test fixture")
                .len(),
            1
        );
        drop(replacement);
        assert!(
            registry
                .active
                .lock()
                .expect("valid activity test fixture")
                .is_empty()
        );
        let guards: Vec<_> = (0..5)
            .map(|n| registry.register(&n.to_string(), "r"))
            .collect();
        assert_eq!(
            registry
                .active
                .lock()
                .expect("valid activity test fixture")
                .len(),
            4
        );
        drop(guards);
        assert!(
            registry
                .active
                .lock()
                .expect("valid activity test fixture")
                .is_empty()
        );
    }
    #[test]
    fn activity_terminal_rebuild_uses_only_known_canonical_trace() {
        let registry = ActivityRegistry::default();
        let mut saved = turn("request", TurnPhase::Completed);
        saved.response = Some(serde_json::json!({"answer":"private", "tool_trace":[
            {"call_id":"private-id", "tool":"simple_calendar_items", "status":"succeeded", "args":"private"},
            {"tool":"unknown-private", "status":"succeeded"},
            {"tool":"change_radar_get", "status":"denied"}
        ]}));
        let dto = registry.project("conversation", Some(&saved));
        assert_eq!(dto.phase, ActivityPhase::Completed);
        assert_eq!(dto.sequence, u32::MAX);
        assert_eq!(dto.steps.len(), 2);
        assert_eq!(dto.steps[0].id, "call-1");
        assert_eq!(dto.steps[0].tool, Some(ChatActivityTool::CalendarItems));
        assert!(
            !serde_json::to_string(&dto)
                .expect("valid activity test fixture")
                .contains("private")
        );
        for phase in [
            TurnPhase::Running,
            TurnPhase::Failed,
            TurnPhase::Interrupted,
        ] {
            let dto = registry.project("conversation", Some(&turn("request", phase)));
            assert!(dto.steps.is_empty());
        }
        assert_eq!(
            registry.project("conversation", None).phase,
            ActivityPhase::Idle
        );
    }
    #[test]
    fn activity_observation_is_bounded_and_terminal_steps_do_not_reopen() {
        let registry = Arc::new(ActivityRegistry::default());
        let mut guard = registry.register("c", "r");
        for n in 1..=3 {
            guard.observe(ChatActivityEvent::ModelStarted { turn: n });
            guard.observe(ChatActivityEvent::ModelFinished {
                turn: n,
                succeeded: true,
            });
        }
        for n in 1..=4 {
            guard.observe(ChatActivityEvent::ToolStarted {
                call: n,
                tool: ChatActivityTool::CalendarItems,
            });
            guard.observe(ChatActivityEvent::ToolFinished {
                call: n,
                tool: ChatActivityTool::CalendarItems,
                status: ChatToolStatus::Succeeded,
            });
        }
        guard.observe(ChatActivityEvent::ModelStarted { turn: 1 });
        guard.observe(ChatActivityEvent::ModelStarted { turn: 4 });
        let dto = registry.project("c", Some(&turn("r", TurnPhase::Running)));
        assert_eq!(dto.sequence, 14);
        assert_eq!(dto.steps.len(), 7);
        assert!(
            dto.steps
                .iter()
                .all(|step| step.status == ActivityStatus::Succeeded)
        );
    }
}
