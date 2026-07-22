//! Platform-owned, framework-neutral Agent run state machine.
//!
//! This crate owns immutable run identity, legal transitions, deterministic replay,
//! effect intent/receipt ordering, and replay-stable budgets. It deliberately contains
//! no provider SDK, MCP transport, database, HTTP server, or framework checkpoint type.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Exact schema version accepted by the R0 runtime kernel.
pub const RUN_SPEC_SCHEMA_VERSION: &str = "agent-run/v0";

/// Immutable limits pinned when a run is created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunBudgets {
    /// Maximum number of provider/model turns.
    pub max_turns: u32,
    /// Maximum number of proposed tool calls.
    pub max_tool_calls: u32,
    /// Maximum provider-reported input tokens.
    pub max_input_tokens: u64,
    /// Maximum provider-reported output tokens.
    pub max_output_tokens: u64,
    /// Maximum provider-reported cost in platform-defined millionth currency units.
    pub max_cost_microunits: u64,
    /// Maximum number of explicitly recorded retries.
    pub max_retries: u32,
    /// Maximum monotone elapsed time observed by the platform clock.
    pub max_elapsed_ms: u64,
}

/// Immutable platform authority for one bounded Agent run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunSpec {
    /// Contract version. Must equal [`RUN_SPEC_SCHEMA_VERSION`].
    pub schema_version: String,
    /// Stable platform run identifier.
    pub run_id: String,
    /// Owning tenant/user scope.
    pub tenant_id: String,
    /// Exact resolved installation identity.
    pub installation_id: String,
    /// Exact package identity.
    pub package_id: String,
    /// Exact package version.
    pub package_version: String,
    /// Exact package component identity.
    pub component_id: String,
    /// Exact model/provider profile identity.
    pub provider_profile_id: String,
    /// Exact capability-grant snapshot identity.
    pub grant_snapshot_id: String,
    /// Digest of the complete approved tool-schema set.
    pub tool_schema_set_digest: String,
    /// Immutable run budgets.
    pub budgets: RunBudgets,
}

impl RunSpec {
    fn validate(&self) -> Result<(), RuntimeError> {
        let mut problems = Vec::new();
        if self.schema_version != RUN_SPEC_SCHEMA_VERSION {
            problems.push(format!(
                "schema_version must equal {RUN_SPEC_SCHEMA_VERSION:?}"
            ));
        }
        for (name, value) in [
            ("run_id", self.run_id.as_str()),
            ("tenant_id", self.tenant_id.as_str()),
            ("installation_id", self.installation_id.as_str()),
            ("package_id", self.package_id.as_str()),
            ("package_version", self.package_version.as_str()),
            ("component_id", self.component_id.as_str()),
            ("provider_profile_id", self.provider_profile_id.as_str()),
            ("grant_snapshot_id", self.grant_snapshot_id.as_str()),
        ] {
            if !is_valid_identity(value) {
                problems.push(format!("{name} must be a non-empty bounded identity"));
            }
        }
        if !is_sha256_digest(&self.tool_schema_set_digest) {
            problems.push("tool_schema_set_digest must be lowercase sha256:<64 hex>".to_owned());
        }
        if self.budgets.max_turns == 0 {
            problems.push("max_turns must be non-zero".to_owned());
        }
        if self.budgets.max_tool_calls == 0 {
            problems.push("max_tool_calls must be non-zero".to_owned());
        }
        if self.budgets.max_input_tokens == 0 {
            problems.push("max_input_tokens must be non-zero".to_owned());
        }
        if self.budgets.max_output_tokens == 0 {
            problems.push("max_output_tokens must be non-zero".to_owned());
        }
        if self.budgets.max_cost_microunits == 0 {
            problems.push("max_cost_microunits must be non-zero".to_owned());
        }
        if self.budgets.max_retries == 0 {
            problems.push("max_retries must be non-zero".to_owned());
        }
        if self.budgets.max_elapsed_ms == 0 {
            problems.push("max_elapsed_ms must be non-zero".to_owned());
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(RuntimeError::InvalidRunSpec { problems })
        }
    }
}

/// Coarse phase of one platform-owned run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunPhase {
    /// Spec accepted; no preparation event has been persisted.
    Created,
    /// Ready to begin the next model turn.
    Preparing,
    /// One model turn is in progress or has produced a final response.
    ModelTurn,
    /// A tool call is awaiting platform approval and intent creation.
    AwaitingToolApproval,
    /// An effect intent is persisted and awaits an exact receipt.
    ExecutingTools,
    /// Successful terminal state.
    Completed,
    /// Failed terminal state.
    Failed,
    /// Explicitly cancelled terminal state.
    Cancelled,
    /// Time-expired terminal state.
    Expired,
}

impl RunPhase {
    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Expired
        )
    }
}

/// Exact provider-reported usage for one completed model turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelUsage {
    /// Input tokens charged by the provider profile.
    pub input_tokens: u64,
    /// Output tokens charged by the provider profile.
    pub output_tokens: u64,
    /// Cost in platform-defined millionth currency units.
    pub cost_microunits: u64,
}

/// Stable proposal emitted by a future model/provider adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCallProposal {
    /// Stable call identity within the platform run.
    pub call_id: String,
    /// Exact approved tool name.
    pub tool_name: String,
    /// Digest of the canonical validated argument payload.
    pub arguments_digest: String,
}

/// Persisted authorization intent that must precede external execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectIntent {
    /// Stable effect identity.
    pub effect_id: String,
    /// Stable duplicate-suppression key.
    pub idempotency_key: String,
    /// Proposal call identity.
    pub call_id: String,
    /// Exact approved tool name.
    pub tool_name: String,
    /// Digest of the canonical validated argument payload.
    pub arguments_digest: String,
    /// Stable capability ID checked by the future resolver.
    pub capability_id: String,
    /// Exact grant snapshot pinned by the run spec.
    pub grant_snapshot_id: String,
    /// Exact approved tool-schema-set digest pinned by the run spec.
    pub tool_schema_set_digest: String,
}

/// Typed terminal outcome of one bounded effect execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectOutcome {
    /// Successful effect with a canonical output digest.
    Succeeded { output_digest: String },
    /// Failed effect with a stable non-secret error code.
    Failed { error_code: String },
}

/// Receipt persisted before the run can leave `ExecutingTools`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectReceipt {
    /// Exact pending effect identity.
    pub effect_id: String,
    /// Exact pending idempotency key.
    pub idempotency_key: String,
    /// Typed execution result.
    pub outcome: EffectOutcome,
}

/// Terminal detail retained in the replayed checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TerminalOutcome {
    /// Run completed with a canonical final-output digest.
    Completed { output_digest: String },
    /// Run failed with a stable non-secret error code.
    Failed { error_code: String },
    /// Run was cancelled with a stable reason code.
    Cancelled { reason_code: String },
    /// Run expired under platform time policy.
    Expired,
}

/// User/orchestrator intent submitted to the pure kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunCommand {
    /// Enter preparation.
    Prepare,
    /// Begin and charge the next model turn.
    StartModelTurn,
    /// Persist exact usage before accepting the model turn outcome.
    RecordModelUsage(ModelUsage),
    /// Persist one proposed tool call.
    ProposeToolCall(ToolCallProposal),
    /// Persist an approved effect intent before execution.
    ApproveToolCall(EffectIntent),
    /// Persist an exact receipt before advancing.
    RecordEffectReceipt(EffectReceipt),
    /// Charge one explicit retry while preparing.
    RecordRetry { reason_code: String },
    /// Record a monotone elapsed-time observation.
    ObserveElapsed { total_elapsed_ms: u64 },
    /// Complete from a model turn.
    Complete { output_digest: String },
    /// Fail from a non-executing phase.
    Fail { error_code: String },
    /// Cancel from a non-executing phase.
    Cancel { reason_code: String },
    /// Expire from a non-executing phase.
    Expire,
}

impl RunCommand {
    fn name(&self) -> &'static str {
        match self {
            Self::Prepare => "prepare",
            Self::StartModelTurn => "start_model_turn",
            Self::RecordModelUsage(_) => "record_model_usage",
            Self::ProposeToolCall(_) => "propose_tool_call",
            Self::ApproveToolCall(_) => "approve_tool_call",
            Self::RecordEffectReceipt(_) => "record_effect_receipt",
            Self::RecordRetry { .. } => "record_retry",
            Self::ObserveElapsed { .. } => "observe_elapsed",
            Self::Complete { .. } => "complete",
            Self::Fail { .. } => "fail",
            Self::Cancel { .. } => "cancel",
            Self::Expire => "expire",
        }
    }

    fn into_event_kind(self) -> RunEventKind {
        match self {
            Self::Prepare => RunEventKind::Prepared,
            Self::StartModelTurn => RunEventKind::ModelTurnStarted,
            Self::RecordModelUsage(usage) => RunEventKind::ModelUsageRecorded { usage },
            Self::ProposeToolCall(proposal) => RunEventKind::ToolCallProposed { proposal },
            Self::ApproveToolCall(intent) => RunEventKind::EffectIntentPersisted { intent },
            Self::RecordEffectReceipt(receipt) => RunEventKind::EffectReceiptPersisted { receipt },
            Self::RecordRetry { reason_code } => RunEventKind::RetryRecorded { reason_code },
            Self::ObserveElapsed { total_elapsed_ms } => {
                RunEventKind::ElapsedObserved { total_elapsed_ms }
            }
            Self::Complete { output_digest } => RunEventKind::Completed { output_digest },
            Self::Fail { error_code } => RunEventKind::Failed { error_code },
            Self::Cancel { reason_code } => RunEventKind::Cancelled { reason_code },
            Self::Expire => RunEventKind::Expired,
        }
    }
}

/// Immutable payload stored in a run journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RunEventKind {
    /// Preparation started.
    Prepared,
    /// One model turn started and consumed budget.
    ModelTurnStarted,
    /// Exact provider-reported usage was persisted for the current turn.
    ModelUsageRecorded { usage: ModelUsage },
    /// Tool call proposal persisted.
    ToolCallProposed { proposal: ToolCallProposal },
    /// Effect intent persisted before adapter execution.
    EffectIntentPersisted { intent: EffectIntent },
    /// Effect receipt persisted after adapter execution.
    EffectReceiptPersisted { receipt: EffectReceipt },
    /// Retry counter advanced.
    RetryRecorded { reason_code: String },
    /// Monotone elapsed-time observation persisted.
    ElapsedObserved { total_elapsed_ms: u64 },
    /// Run completed.
    Completed { output_digest: String },
    /// Run failed.
    Failed { error_code: String },
    /// Run cancelled.
    Cancelled { reason_code: String },
    /// Run expired.
    Expired,
}

impl RunEventKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::ModelTurnStarted => "model_turn_started",
            Self::ModelUsageRecorded { .. } => "model_usage_recorded",
            Self::ToolCallProposed { .. } => "tool_call_proposed",
            Self::EffectIntentPersisted { .. } => "effect_intent_persisted",
            Self::EffectReceiptPersisted { .. } => "effect_receipt_persisted",
            Self::RetryRecorded { .. } => "retry_recorded",
            Self::ElapsedObserved { .. } => "elapsed_observed",
            Self::Completed { .. } => "completed",
            Self::Failed { .. } => "failed",
            Self::Cancelled { .. } => "cancelled",
            Self::Expired => "expired",
        }
    }
}

/// One exact, ordered run event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunEvent {
    /// One-based event sequence.
    pub sequence: u64,
    /// Typed event payload.
    pub kind: RunEventKind,
}

/// Result of deciding one command without mutating the checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Persist this event under the checkpoint's expected revision, then apply it.
    Append(RunEvent),
    /// The exact receipt/observation is already represented; append nothing.
    AlreadyApplied,
}

/// Replayable checkpoint derived from one immutable spec and ordered events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun {
    spec: RunSpec,
    phase: RunPhase,
    revision: u64,
    turns_used: u32,
    tool_calls_used: u32,
    input_tokens_used: u64,
    output_tokens_used: u64,
    cost_microunits_used: u64,
    retries_used: u32,
    elapsed_ms: u64,
    model_usage_recorded: bool,
    seen_call_ids: BTreeSet<String>,
    seen_effect_ids: BTreeSet<String>,
    seen_idempotency_keys: BTreeSet<String>,
    proposed_call: Option<ToolCallProposal>,
    pending_effect: Option<EffectIntent>,
    receipts: BTreeMap<String, EffectReceipt>,
    terminal: Option<TerminalOutcome>,
}

impl AgentRun {
    /// Validate a spec and create a new run at revision zero.
    pub fn new(spec: RunSpec) -> Result<Self, RuntimeError> {
        spec.validate()?;
        Ok(Self {
            spec,
            phase: RunPhase::Created,
            revision: 0,
            turns_used: 0,
            tool_calls_used: 0,
            input_tokens_used: 0,
            output_tokens_used: 0,
            cost_microunits_used: 0,
            retries_used: 0,
            elapsed_ms: 0,
            model_usage_recorded: false,
            seen_call_ids: BTreeSet::new(),
            seen_effect_ids: BTreeSet::new(),
            seen_idempotency_keys: BTreeSet::new(),
            proposed_call: None,
            pending_effect: None,
            receipts: BTreeMap::new(),
            terminal: None,
        })
    }

    /// Reconstruct a checkpoint from the immutable spec and exact ordered events.
    pub fn replay(spec: RunSpec, events: &[RunEvent]) -> Result<Self, RuntimeError> {
        let mut run = Self::new(spec)?;
        for event in events {
            run.apply(event.clone())?;
        }
        Ok(run)
    }

    /// Immutable run specification.
    #[must_use]
    pub fn spec(&self) -> &RunSpec {
        &self.spec
    }

    /// Current phase.
    #[must_use]
    pub const fn phase(&self) -> RunPhase {
        self.phase
    }

    /// Last applied event sequence.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Consumed model turns.
    #[must_use]
    pub const fn turns_used(&self) -> u32 {
        self.turns_used
    }

    /// Consumed tool-call proposals.
    #[must_use]
    pub const fn tool_calls_used(&self) -> u32 {
        self.tool_calls_used
    }

    /// Provider-reported input tokens consumed across recorded turns.
    #[must_use]
    pub const fn input_tokens_used(&self) -> u64 {
        self.input_tokens_used
    }

    /// Provider-reported output tokens consumed across recorded turns.
    #[must_use]
    pub const fn output_tokens_used(&self) -> u64 {
        self.output_tokens_used
    }

    /// Provider-reported cost consumed across recorded turns.
    #[must_use]
    pub const fn cost_microunits_used(&self) -> u64 {
        self.cost_microunits_used
    }

    /// Consumed retries.
    #[must_use]
    pub const fn retries_used(&self) -> u32 {
        self.retries_used
    }

    /// Latest monotone elapsed observation.
    #[must_use]
    pub const fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }

    /// Exact pending effect, if adapter execution is authorized and unreceipted.
    #[must_use]
    pub fn pending_effect(&self) -> Option<&EffectIntent> {
        self.pending_effect.as_ref()
    }

    /// Receipt committed for an effect identity.
    #[must_use]
    pub fn receipt(&self, effect_id: &str) -> Option<&EffectReceipt> {
        self.receipts.get(effect_id)
    }

    /// Terminal detail, if the run has ended.
    #[must_use]
    pub fn terminal(&self) -> Option<&TerminalOutcome> {
        self.terminal.as_ref()
    }

    /// Decide one command without mutating the run.
    pub fn decide(&self, command: RunCommand) -> Result<Decision, RuntimeError> {
        match &command {
            RunCommand::RecordEffectReceipt(receipt) => {
                if let Some(existing) = self.receipts.get(&receipt.effect_id) {
                    return if existing == receipt {
                        Ok(Decision::AlreadyApplied)
                    } else {
                        Err(RuntimeError::ConflictingReceipt {
                            effect_id: receipt.effect_id.clone(),
                        })
                    };
                }
            }
            RunCommand::ObserveElapsed { total_elapsed_ms }
                if *total_elapsed_ms == self.elapsed_ms =>
            {
                return Ok(Decision::AlreadyApplied);
            }
            _ => {}
        }
        self.validate_command(&command)?;
        let Some(sequence) = self.revision.checked_add(1) else {
            return Err(RuntimeError::RevisionExhausted);
        };
        Ok(Decision::Append(RunEvent {
            sequence,
            kind: command.into_event_kind(),
        }))
    }

    /// Apply one already-persisted event to this checkpoint.
    pub fn apply(&mut self, event: RunEvent) -> Result<(), RuntimeError> {
        let Some(expected) = self.revision.checked_add(1) else {
            return Err(RuntimeError::RevisionExhausted);
        };
        if event.sequence != expected {
            return Err(RuntimeError::EventSequenceMismatch {
                expected,
                actual: event.sequence,
            });
        }
        self.validate_event(&event.kind)?;
        self.evolve(event.kind);
        self.revision = event.sequence;
        Ok(())
    }

    fn validate_command(&self, command: &RunCommand) -> Result<(), RuntimeError> {
        if self.phase.is_terminal() {
            return Err(RuntimeError::TerminalRunMutation {
                phase: self.phase,
                operation: command.name(),
            });
        }
        match command {
            RunCommand::Prepare => self.require_phase(RunPhase::Created, command.name()),
            RunCommand::StartModelTurn => {
                self.require_phase(RunPhase::Preparing, command.name())?;
                self.require_elapsed_budget_remaining()?;
                self.require_budget(
                    "turns",
                    u64::from(self.turns_used),
                    u64::from(self.spec.budgets.max_turns),
                )
            }
            RunCommand::RecordModelUsage(usage) => {
                self.require_phase(RunPhase::ModelTurn, command.name())?;
                if self.model_usage_recorded {
                    return Err(RuntimeError::ModelUsageAlreadyRecorded);
                }
                self.validate_model_usage(*usage)
            }
            RunCommand::ProposeToolCall(proposal) => {
                self.require_phase(RunPhase::ModelTurn, command.name())?;
                self.require_model_usage()?;
                validate_proposal(proposal)?;
                if self.seen_call_ids.contains(&proposal.call_id) {
                    return Err(RuntimeError::DuplicateCallId {
                        call_id: proposal.call_id.clone(),
                    });
                }
                self.require_budget(
                    "tool_calls",
                    u64::from(self.tool_calls_used),
                    u64::from(self.spec.budgets.max_tool_calls),
                )
            }
            RunCommand::ApproveToolCall(intent) => {
                self.require_phase(RunPhase::AwaitingToolApproval, command.name())?;
                self.require_elapsed_budget_remaining()?;
                self.validate_intent(intent)
            }
            RunCommand::RecordEffectReceipt(receipt) => {
                self.require_phase(RunPhase::ExecutingTools, command.name())?;
                self.validate_receipt(receipt)
            }
            RunCommand::RecordRetry { reason_code } => {
                if !matches!(self.phase, RunPhase::Preparing | RunPhase::ModelTurn) {
                    return Err(RuntimeError::IllegalTransition {
                        phase: self.phase,
                        operation: command.name(),
                    });
                }
                validate_identity("reason_code", reason_code)?;
                self.require_budget(
                    "retries",
                    u64::from(self.retries_used),
                    u64::from(self.spec.budgets.max_retries),
                )
            }
            RunCommand::ObserveElapsed { total_elapsed_ms } => {
                if *total_elapsed_ms < self.elapsed_ms {
                    return Err(RuntimeError::ElapsedTimeRegressed {
                        previous: self.elapsed_ms,
                        proposed: *total_elapsed_ms,
                    });
                }
                if *total_elapsed_ms > self.spec.budgets.max_elapsed_ms {
                    return Err(RuntimeError::BudgetExceeded {
                        budget: "elapsed_ms",
                        used: *total_elapsed_ms,
                        limit: self.spec.budgets.max_elapsed_ms,
                    });
                }
                Ok(())
            }
            RunCommand::Complete { output_digest } => {
                self.require_phase(RunPhase::ModelTurn, command.name())?;
                self.require_model_usage()?;
                validate_digest("output_digest", output_digest)
            }
            RunCommand::Fail { error_code } => {
                self.require_non_executing(command.name())?;
                validate_identity("error_code", error_code)
            }
            RunCommand::Cancel { reason_code } => {
                self.require_non_executing(command.name())?;
                validate_identity("reason_code", reason_code)
            }
            RunCommand::Expire => self.require_non_executing(command.name()),
        }
    }

    fn validate_event(&self, event: &RunEventKind) -> Result<(), RuntimeError> {
        let command = match event {
            RunEventKind::Prepared => RunCommand::Prepare,
            RunEventKind::ModelTurnStarted => RunCommand::StartModelTurn,
            RunEventKind::ModelUsageRecorded { usage } => RunCommand::RecordModelUsage(*usage),
            RunEventKind::ToolCallProposed { proposal } => {
                RunCommand::ProposeToolCall(proposal.clone())
            }
            RunEventKind::EffectIntentPersisted { intent } => {
                RunCommand::ApproveToolCall(intent.clone())
            }
            RunEventKind::EffectReceiptPersisted { receipt } => {
                RunCommand::RecordEffectReceipt(receipt.clone())
            }
            RunEventKind::RetryRecorded { reason_code } => RunCommand::RecordRetry {
                reason_code: reason_code.clone(),
            },
            RunEventKind::ElapsedObserved { total_elapsed_ms } => RunCommand::ObserveElapsed {
                total_elapsed_ms: *total_elapsed_ms,
            },
            RunEventKind::Completed { output_digest } => RunCommand::Complete {
                output_digest: output_digest.clone(),
            },
            RunEventKind::Failed { error_code } => RunCommand::Fail {
                error_code: error_code.clone(),
            },
            RunEventKind::Cancelled { reason_code } => RunCommand::Cancel {
                reason_code: reason_code.clone(),
            },
            RunEventKind::Expired => RunCommand::Expire,
        };
        self.validate_command(&command).map_err(|error| {
            if matches!(error, RuntimeError::IllegalTransition { .. }) {
                RuntimeError::IllegalTransition {
                    phase: self.phase,
                    operation: event.name(),
                }
            } else {
                error
            }
        })
    }

    fn evolve(&mut self, event: RunEventKind) {
        match event {
            RunEventKind::Prepared => self.phase = RunPhase::Preparing,
            RunEventKind::ModelTurnStarted => {
                self.turns_used += 1;
                self.model_usage_recorded = false;
                self.phase = RunPhase::ModelTurn;
            }
            RunEventKind::ModelUsageRecorded { usage } => {
                self.input_tokens_used += usage.input_tokens;
                self.output_tokens_used += usage.output_tokens;
                self.cost_microunits_used += usage.cost_microunits;
                self.model_usage_recorded = true;
            }
            RunEventKind::ToolCallProposed { proposal } => {
                self.tool_calls_used += 1;
                self.seen_call_ids.insert(proposal.call_id.clone());
                self.proposed_call = Some(proposal);
                self.phase = RunPhase::AwaitingToolApproval;
            }
            RunEventKind::EffectIntentPersisted { intent } => {
                self.proposed_call = None;
                self.seen_effect_ids.insert(intent.effect_id.clone());
                self.seen_idempotency_keys
                    .insert(intent.idempotency_key.clone());
                self.pending_effect = Some(intent);
                self.phase = RunPhase::ExecutingTools;
            }
            RunEventKind::EffectReceiptPersisted { receipt } => {
                self.receipts.insert(receipt.effect_id.clone(), receipt);
                self.pending_effect = None;
                self.model_usage_recorded = false;
                self.phase = RunPhase::Preparing;
            }
            RunEventKind::RetryRecorded { .. } => {
                self.retries_used += 1;
                self.model_usage_recorded = false;
                self.phase = RunPhase::Preparing;
            }
            RunEventKind::ElapsedObserved { total_elapsed_ms } => {
                self.elapsed_ms = total_elapsed_ms;
            }
            RunEventKind::Completed { output_digest } => {
                self.phase = RunPhase::Completed;
                self.terminal = Some(TerminalOutcome::Completed { output_digest });
            }
            RunEventKind::Failed { error_code } => {
                self.phase = RunPhase::Failed;
                self.terminal = Some(TerminalOutcome::Failed { error_code });
            }
            RunEventKind::Cancelled { reason_code } => {
                self.phase = RunPhase::Cancelled;
                self.terminal = Some(TerminalOutcome::Cancelled { reason_code });
            }
            RunEventKind::Expired => {
                self.phase = RunPhase::Expired;
                self.terminal = Some(TerminalOutcome::Expired);
            }
        }
    }

    fn validate_model_usage(&self, usage: ModelUsage) -> Result<(), RuntimeError> {
        if usage.input_tokens == 0 && usage.output_tokens == 0 {
            return Err(RuntimeError::InvalidPayload {
                field: "model_usage",
                requirement: "at least one input or output token",
            });
        }
        self.require_additive_budget(
            "input_tokens",
            self.input_tokens_used,
            usage.input_tokens,
            self.spec.budgets.max_input_tokens,
        )?;
        self.require_additive_budget(
            "output_tokens",
            self.output_tokens_used,
            usage.output_tokens,
            self.spec.budgets.max_output_tokens,
        )?;
        self.require_additive_budget(
            "cost_microunits",
            self.cost_microunits_used,
            usage.cost_microunits,
            self.spec.budgets.max_cost_microunits,
        )
    }

    fn require_model_usage(&self) -> Result<(), RuntimeError> {
        if self.model_usage_recorded {
            Ok(())
        } else {
            Err(RuntimeError::ModelUsageMissing)
        }
    }

    fn require_additive_budget(
        &self,
        budget: &'static str,
        used: u64,
        added: u64,
        limit: u64,
    ) -> Result<(), RuntimeError> {
        let Some(proposed) = used.checked_add(added) else {
            return Err(RuntimeError::BudgetCounterOverflow { budget });
        };
        if proposed <= limit {
            Ok(())
        } else {
            Err(RuntimeError::BudgetExceeded {
                budget,
                used: proposed,
                limit,
            })
        }
    }

    fn validate_intent(&self, intent: &EffectIntent) -> Result<(), RuntimeError> {
        for (name, value) in [
            ("effect_id", intent.effect_id.as_str()),
            ("idempotency_key", intent.idempotency_key.as_str()),
            ("capability_id", intent.capability_id.as_str()),
        ] {
            validate_identity(name, value)?;
        }
        if self.seen_effect_ids.contains(&intent.effect_id) {
            return Err(RuntimeError::DuplicateEffectId {
                effect_id: intent.effect_id.clone(),
            });
        }
        if self.seen_idempotency_keys.contains(&intent.idempotency_key) {
            return Err(RuntimeError::DuplicateIdempotencyKey {
                idempotency_key: intent.idempotency_key.clone(),
            });
        }
        validate_digest("arguments_digest", &intent.arguments_digest)?;
        validate_digest("tool_schema_set_digest", &intent.tool_schema_set_digest)?;
        let Some(proposal) = self.proposed_call.as_ref() else {
            return Err(RuntimeError::MissingToolProposal);
        };
        if intent.call_id != proposal.call_id
            || intent.tool_name != proposal.tool_name
            || intent.arguments_digest != proposal.arguments_digest
        {
            return Err(RuntimeError::EffectIntentMismatch);
        }
        if intent.grant_snapshot_id != self.spec.grant_snapshot_id {
            return Err(RuntimeError::GrantSnapshotMismatch);
        }
        if intent.tool_schema_set_digest != self.spec.tool_schema_set_digest {
            return Err(RuntimeError::ToolSchemaSnapshotMismatch);
        }
        Ok(())
    }

    fn validate_receipt(&self, receipt: &EffectReceipt) -> Result<(), RuntimeError> {
        let Some(intent) = self.pending_effect.as_ref() else {
            return Err(RuntimeError::MissingPendingEffect);
        };
        if receipt.effect_id != intent.effect_id
            || receipt.idempotency_key != intent.idempotency_key
        {
            return Err(RuntimeError::EffectReceiptMismatch);
        }
        match &receipt.outcome {
            EffectOutcome::Succeeded { output_digest } => {
                validate_digest("output_digest", output_digest)
            }
            EffectOutcome::Failed { error_code } => validate_identity("error_code", error_code),
        }
    }

    fn require_phase(
        &self,
        expected: RunPhase,
        operation: &'static str,
    ) -> Result<(), RuntimeError> {
        if self.phase == expected {
            Ok(())
        } else {
            Err(RuntimeError::IllegalTransition {
                phase: self.phase,
                operation,
            })
        }
    }

    fn require_non_executing(&self, operation: &'static str) -> Result<(), RuntimeError> {
        if self.phase == RunPhase::ExecutingTools {
            Err(RuntimeError::InFlightEffectCannotTerminate { operation })
        } else {
            Ok(())
        }
    }

    fn require_budget(
        &self,
        budget: &'static str,
        used: u64,
        limit: u64,
    ) -> Result<(), RuntimeError> {
        if used < limit {
            Ok(())
        } else {
            Err(RuntimeError::BudgetExceeded {
                budget,
                used,
                limit,
            })
        }
    }

    fn require_elapsed_budget_remaining(&self) -> Result<(), RuntimeError> {
        self.require_budget(
            "elapsed_ms",
            self.elapsed_ms,
            self.spec.budgets.max_elapsed_ms,
        )
    }
}

/// Typed fail-closed errors emitted by the R0 kernel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Immutable specification is malformed.
    InvalidRunSpec { problems: Vec<String> },
    /// Command/event is illegal in the current phase.
    IllegalTransition {
        phase: RunPhase,
        operation: &'static str,
    },
    /// A terminal run received a mutating command/event.
    TerminalRunMutation {
        phase: RunPhase,
        operation: &'static str,
    },
    /// Event sequence is duplicated, missing, or out of order.
    EventSequenceMismatch { expected: u64, actual: u64 },
    /// No further event sequence can be represented.
    RevisionExhausted,
    /// One immutable budget would be exceeded.
    BudgetExceeded {
        budget: &'static str,
        used: u64,
        limit: u64,
    },
    /// A cumulative budget counter cannot represent the proposed value.
    BudgetCounterOverflow { budget: &'static str },
    /// The current model turn already persisted usage.
    ModelUsageAlreadyRecorded,
    /// A successful model turn outcome was proposed before exact usage was persisted.
    ModelUsageMissing,
    /// Elapsed time observation moved backwards.
    ElapsedTimeRegressed { previous: u64, proposed: u64 },
    /// Command payload identity or digest is malformed.
    InvalidPayload {
        field: &'static str,
        requirement: &'static str,
    },
    /// Effect approval had no stored proposal.
    MissingToolProposal,
    /// Effect intent did not exactly match the stored proposal.
    EffectIntentMismatch,
    /// Effect intent attempted to use a different grant snapshot.
    GrantSnapshotMismatch,
    /// Effect intent attempted to use a different tool-schema snapshot.
    ToolSchemaSnapshotMismatch,
    /// Receipt had no exact pending effect.
    MissingPendingEffect,
    /// Receipt identity did not match the pending effect.
    EffectReceiptMismatch,
    /// A committed effect received a different second receipt.
    ConflictingReceipt { effect_id: String },
    /// A call identity was reused within one run.
    DuplicateCallId { call_id: String },
    /// An effect identity was reused within one run.
    DuplicateEffectId { effect_id: String },
    /// An idempotency identity was reused within one run.
    DuplicateIdempotencyKey { idempotency_key: String },
    /// Terminal mutation was attempted while an effect lacked a receipt.
    InFlightEffectCannotTerminate { operation: &'static str },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRunSpec { problems } => {
                write!(formatter, "invalid run spec: {}", problems.join("; "))
            }
            Self::IllegalTransition { phase, operation } => {
                write!(formatter, "illegal transition: {operation} from {phase:?}")
            }
            Self::TerminalRunMutation { phase, operation } => {
                write!(formatter, "terminal run {phase:?} rejects {operation}")
            }
            Self::EventSequenceMismatch { expected, actual } => {
                write!(
                    formatter,
                    "event sequence mismatch: expected {expected}, got {actual}"
                )
            }
            Self::RevisionExhausted => formatter.write_str("run event revision exhausted"),
            Self::BudgetExceeded {
                budget,
                used,
                limit,
            } => write!(
                formatter,
                "budget exceeded: {budget} used/proposed {used}, limit {limit}"
            ),
            Self::BudgetCounterOverflow { budget } => {
                write!(formatter, "budget counter overflow: {budget}")
            }
            Self::ModelUsageAlreadyRecorded => {
                formatter.write_str("model usage already recorded for current turn")
            }
            Self::ModelUsageMissing => {
                formatter.write_str("model usage must be recorded before accepting turn outcome")
            }
            Self::ElapsedTimeRegressed { previous, proposed } => write!(
                formatter,
                "elapsed time regressed from {previous} to {proposed}"
            ),
            Self::InvalidPayload { field, requirement } => {
                write!(formatter, "invalid {field}: {requirement}")
            }
            Self::MissingToolProposal => formatter.write_str("effect intent has no tool proposal"),
            Self::EffectIntentMismatch => {
                formatter.write_str("effect intent does not match tool proposal")
            }
            Self::GrantSnapshotMismatch => {
                formatter.write_str("effect intent grant snapshot differs from run spec")
            }
            Self::ToolSchemaSnapshotMismatch => {
                formatter.write_str("effect intent tool schema differs from run spec")
            }
            Self::MissingPendingEffect => formatter.write_str("receipt has no pending effect"),
            Self::EffectReceiptMismatch => {
                formatter.write_str("receipt does not match pending effect")
            }
            Self::ConflictingReceipt { effect_id } => {
                write!(formatter, "conflicting receipt for effect {effect_id}")
            }
            Self::DuplicateCallId { call_id } => {
                write!(formatter, "duplicate call identity {call_id}")
            }
            Self::DuplicateEffectId { effect_id } => {
                write!(formatter, "duplicate effect identity {effect_id}")
            }
            Self::DuplicateIdempotencyKey { idempotency_key } => {
                write!(
                    formatter,
                    "duplicate idempotency identity {idempotency_key}"
                )
            }
            Self::InFlightEffectCannotTerminate { operation } => write!(
                formatter,
                "cannot {operation} while an effect awaits its exact receipt"
            ),
        }
    }
}

impl Error for RuntimeError {}

fn is_valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .chars()
            .all(|character| !character.is_control() && !character.is_whitespace())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_identity(field: &'static str, value: &str) -> Result<(), RuntimeError> {
    if is_valid_identity(value) {
        Ok(())
    } else {
        Err(RuntimeError::InvalidPayload {
            field,
            requirement: "non-empty identity without whitespace/control characters",
        })
    }
}

fn validate_digest(field: &'static str, value: &str) -> Result<(), RuntimeError> {
    if is_sha256_digest(value) {
        Ok(())
    } else {
        Err(RuntimeError::InvalidPayload {
            field,
            requirement: "lowercase sha256:<64 hex>",
        })
    }
}

fn validate_proposal(proposal: &ToolCallProposal) -> Result<(), RuntimeError> {
    validate_identity("call_id", &proposal.call_id)?;
    validate_identity("tool_name", &proposal.tool_name)?;
    validate_digest("arguments_digest", &proposal.arguments_digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_DIGEST: &str =
        "sha256:0000000000000000000000000000000000000000000000000000000000000000";
    const ONE_DIGEST: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";

    fn spec() -> RunSpec {
        RunSpec {
            schema_version: RUN_SPEC_SCHEMA_VERSION.to_owned(),
            run_id: "run:demo-001".to_owned(),
            tenant_id: "tenant:alice".to_owned(),
            installation_id: "installation:opportunity-graph@0.1.0".to_owned(),
            package_id: "ustc.opportunity-graph".to_owned(),
            package_version: "0.1.0".to_owned(),
            component_id: "native:course-planning".to_owned(),
            provider_profile_id: "provider:deterministic-test".to_owned(),
            grant_snapshot_id: "grant:demo-v1".to_owned(),
            tool_schema_set_digest: ZERO_DIGEST.to_owned(),
            budgets: RunBudgets {
                max_turns: 3,
                max_tool_calls: 2,
                max_input_tokens: 1_000,
                max_output_tokens: 1_000,
                max_cost_microunits: 10_000,
                max_retries: 2,
                max_elapsed_ms: 10_000,
            },
        }
    }

    fn proposal() -> ToolCallProposal {
        ToolCallProposal {
            call_id: "call:001".to_owned(),
            tool_name: "planner.generate".to_owned(),
            arguments_digest: ONE_DIGEST.to_owned(),
        }
    }

    const fn usage() -> ModelUsage {
        ModelUsage {
            input_tokens: 3,
            output_tokens: 2,
            cost_microunits: 10,
        }
    }

    fn intent() -> EffectIntent {
        EffectIntent {
            effect_id: "effect:001".to_owned(),
            idempotency_key: "run:demo-001/call:001".to_owned(),
            call_id: "call:001".to_owned(),
            tool_name: "planner.generate".to_owned(),
            arguments_digest: ONE_DIGEST.to_owned(),
            capability_id: "campus.public_plan.read".to_owned(),
            grant_snapshot_id: "grant:demo-v1".to_owned(),
            tool_schema_set_digest: ZERO_DIGEST.to_owned(),
        }
    }

    fn receipt(output_digest: &str) -> EffectReceipt {
        EffectReceipt {
            effect_id: "effect:001".to_owned(),
            idempotency_key: "run:demo-001/call:001".to_owned(),
            outcome: EffectOutcome::Succeeded {
                output_digest: output_digest.to_owned(),
            },
        }
    }

    fn decide_and_apply(run: &mut AgentRun, events: &mut Vec<RunEvent>, command: RunCommand) {
        let Ok(Decision::Append(event)) = run.decide(command) else {
            panic!("command must produce an append decision");
        };
        let result = run.apply(event.clone());
        assert!(result.is_ok());
        events.push(event);
    }

    #[test]
    fn run_spec_round_trip_preserves_exact_identity() {
        let original = spec();
        let encoded = serde_json::to_string(&original);
        let Ok(encoded) = encoded else {
            panic!("valid run spec must encode");
        };
        let decoded = serde_json::from_str::<RunSpec>(&encoded);
        let Ok(decoded) = decoded else {
            panic!("encoded run spec must decode");
        };
        assert_eq!(decoded, original);
        let run = AgentRun::new(decoded);
        let Ok(run) = run else {
            panic!("valid run spec must create a run");
        };
        assert_eq!(run.spec(), &original);
        assert_eq!(run.phase(), RunPhase::Created);
    }

    #[test]
    fn invalid_run_specs_fail_closed() {
        let mut invalid = spec();
        invalid.run_id.clear();
        invalid.tool_schema_set_digest = "sha256:ABC".to_owned();
        invalid.budgets.max_turns = 0;
        let result = AgentRun::new(invalid);
        let Err(RuntimeError::InvalidRunSpec { problems }) = result else {
            panic!("invalid spec must fail with typed problems");
        };
        assert!(problems.len() >= 3);
    }

    #[test]
    fn run_spec_rejects_unknown_fields_and_zero_budgets() {
        let encoded = serde_json::to_value(spec());
        let Ok(mut encoded) = encoded else {
            panic!("fixture spec must encode");
        };
        let Some(object) = encoded.as_object_mut() else {
            panic!("run spec must encode as an object");
        };
        object.insert("framework_session".to_owned(), serde_json::json!("opaque"));
        assert!(serde_json::from_value::<RunSpec>(encoded).is_err());

        let mut invalid = spec();
        invalid.budgets = RunBudgets {
            max_turns: 0,
            max_tool_calls: 0,
            max_input_tokens: 0,
            max_output_tokens: 0,
            max_cost_microunits: 0,
            max_retries: 0,
            max_elapsed_ms: 0,
        };
        let Err(RuntimeError::InvalidRunSpec { problems }) = AgentRun::new(invalid) else {
            panic!("zero budgets must fail with typed problems");
        };
        for budget in [
            "max_turns",
            "max_tool_calls",
            "max_input_tokens",
            "max_output_tokens",
            "max_cost_microunits",
            "max_retries",
            "max_elapsed_ms",
        ] {
            assert!(problems.iter().any(|problem| problem.contains(budget)));
        }
    }

    #[test]
    fn legal_run_replays_deterministically() {
        let immutable_spec = spec();
        let run = AgentRun::new(immutable_spec.clone());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ProposeToolCall(proposal()),
        );
        decide_and_apply(&mut run, &mut events, RunCommand::ApproveToolCall(intent()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::RecordEffectReceipt(receipt(ZERO_DIGEST)),
        );
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::Complete {
                output_digest: ONE_DIGEST.to_owned(),
            },
        );

        let encoded_events = serde_json::to_string(&events);
        let Ok(encoded_events) = encoded_events else {
            panic!("legal events must encode");
        };
        let decoded_events = serde_json::from_str::<Vec<RunEvent>>(&encoded_events);
        let Ok(decoded_events) = decoded_events else {
            panic!("encoded legal events must decode");
        };
        assert_eq!(decoded_events, events);
        let replayed = AgentRun::replay(immutable_spec, &decoded_events);
        let Ok(replayed) = replayed else {
            panic!("legal event stream must replay");
        };
        assert_eq!(replayed, run);
        assert_eq!(replayed.phase(), RunPhase::Completed);
        assert_eq!(replayed.turns_used(), 2);
        assert_eq!(replayed.tool_calls_used(), 1);
        assert!(matches!(
            replayed.terminal(),
            Some(TerminalOutcome::Completed { output_digest }) if output_digest == ONE_DIGEST
        ));
    }

    #[test]
    fn illegal_transitions_fail_closed() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let before = run.clone();
        let result = run.decide(RunCommand::Complete {
            output_digest: ZERO_DIGEST.to_owned(),
        });
        assert!(matches!(
            result,
            Err(RuntimeError::IllegalTransition {
                phase: RunPhase::Created,
                ..
            })
        ));
        assert_eq!(run, before);

        let gap = RunEvent {
            sequence: 2,
            kind: RunEventKind::Prepared,
        };
        let result = run.apply(gap);
        assert!(matches!(
            result,
            Err(RuntimeError::EventSequenceMismatch {
                expected: 1,
                actual: 2
            })
        ));
        assert_eq!(run, before);
    }

    #[test]
    fn effect_identity_and_in_flight_termination_fail_closed() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ProposeToolCall(proposal()),
        );

        let mut mismatched = intent();
        mismatched.grant_snapshot_id = "grant:other".to_owned();
        assert!(matches!(
            run.decide(RunCommand::ApproveToolCall(mismatched)),
            Err(RuntimeError::GrantSnapshotMismatch)
        ));

        decide_and_apply(&mut run, &mut events, RunCommand::ApproveToolCall(intent()));
        assert!(matches!(
            run.decide(RunCommand::Cancel {
                reason_code: "user_cancelled".to_owned()
            }),
            Err(RuntimeError::InFlightEffectCannotTerminate { .. })
        ));
    }

    #[test]
    fn identical_receipt_is_idempotent_but_conflict_fails() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ProposeToolCall(proposal()),
        );
        decide_and_apply(&mut run, &mut events, RunCommand::ApproveToolCall(intent()));
        let committed = receipt(ZERO_DIGEST);
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::RecordEffectReceipt(committed.clone()),
        );

        assert_eq!(
            run.decide(RunCommand::RecordEffectReceipt(committed)),
            Ok(Decision::AlreadyApplied)
        );
        assert!(matches!(
            run.decide(RunCommand::RecordEffectReceipt(receipt(ONE_DIGEST))),
            Err(RuntimeError::ConflictingReceipt { .. })
        ));

        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        assert!(matches!(
            run.decide(RunCommand::ProposeToolCall(proposal())),
            Err(RuntimeError::DuplicateCallId { .. })
        ));

        let mut second_proposal = proposal();
        second_proposal.call_id = "call:002".to_owned();
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ProposeToolCall(second_proposal),
        );
        let mut reused_effect = intent();
        reused_effect.call_id = "call:002".to_owned();
        assert!(matches!(
            run.decide(RunCommand::ApproveToolCall(reused_effect.clone())),
            Err(RuntimeError::DuplicateEffectId { .. })
        ));

        reused_effect.effect_id = "effect:002".to_owned();
        assert!(matches!(
            run.decide(RunCommand::ApproveToolCall(reused_effect.clone())),
            Err(RuntimeError::DuplicateIdempotencyKey { .. })
        ));

        reused_effect.idempotency_key = "run:demo-001/call:002".to_owned();
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ApproveToolCall(reused_effect),
        );
    }

    #[test]
    fn budgets_fail_closed_and_survive_replay() {
        let mut bounded = spec();
        bounded.budgets = RunBudgets {
            max_turns: 1,
            max_tool_calls: 1,
            max_input_tokens: 3,
            max_output_tokens: 2,
            max_cost_microunits: 10,
            max_retries: 1,
            max_elapsed_ms: 100,
        };
        let run = AgentRun::new(bounded.clone());
        let Ok(mut run) = run else {
            panic!("bounded spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::RecordRetry {
                reason_code: "provider_unavailable".to_owned(),
            },
        );
        assert!(matches!(
            run.decide(RunCommand::RecordRetry {
                reason_code: "provider_unavailable".to_owned()
            }),
            Err(RuntimeError::BudgetExceeded {
                budget: "retries",
                ..
            })
        ));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ObserveElapsed {
                total_elapsed_ms: 99,
            },
        );
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ProposeToolCall(proposal()),
        );

        let before = run.clone();
        assert!(matches!(
            run.decide(RunCommand::ObserveElapsed {
                total_elapsed_ms: 101
            }),
            Err(RuntimeError::BudgetExceeded {
                budget: "elapsed_ms",
                ..
            })
        ));
        assert_eq!(run, before);
        assert!(matches!(
            run.decide(RunCommand::ObserveElapsed {
                total_elapsed_ms: 98
            }),
            Err(RuntimeError::ElapsedTimeRegressed {
                previous: 99,
                proposed: 98
            })
        ));

        decide_and_apply(&mut run, &mut events, RunCommand::ApproveToolCall(intent()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::RecordEffectReceipt(receipt(ZERO_DIGEST)),
        );
        assert!(matches!(
            run.decide(RunCommand::StartModelTurn),
            Err(RuntimeError::BudgetExceeded {
                budget: "turns",
                ..
            })
        ));

        let replayed = AgentRun::replay(bounded, &events);
        let Ok(replayed) = replayed else {
            panic!("bounded events must replay");
        };
        assert_eq!(replayed.retries_used(), 1);
        assert_eq!(replayed.elapsed_ms(), 99);
        assert_eq!(replayed.turns_used(), 1);
        assert_eq!(replayed.tool_calls_used(), 1);
    }

    #[test]
    fn model_usage_is_required_once_and_bounded() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        assert_eq!(
            run.decide(RunCommand::ProposeToolCall(proposal())),
            Err(RuntimeError::ModelUsageMissing)
        );
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        assert_eq!(run.input_tokens_used(), 3);
        assert_eq!(run.output_tokens_used(), 2);
        assert_eq!(run.cost_microunits_used(), 10);
        assert_eq!(
            run.decide(RunCommand::RecordModelUsage(usage())),
            Err(RuntimeError::ModelUsageAlreadyRecorded)
        );

        let mut too_small = spec();
        too_small.budgets.max_input_tokens = 2;
        let bounded = AgentRun::new(too_small);
        let Ok(mut bounded) = bounded else {
            panic!("bounded spec must be valid");
        };
        let mut bounded_events = Vec::new();
        decide_and_apply(&mut bounded, &mut bounded_events, RunCommand::Prepare);
        decide_and_apply(
            &mut bounded,
            &mut bounded_events,
            RunCommand::StartModelTurn,
        );
        assert!(matches!(
            bounded.decide(RunCommand::RecordModelUsage(usage())),
            Err(RuntimeError::BudgetExceeded {
                budget: "input_tokens",
                used: 3,
                limit: 2
            })
        ));
    }

    #[test]
    fn elapsed_budget_blocks_new_external_work() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ObserveElapsed {
                total_elapsed_ms: 10_000,
            },
        );
        assert!(matches!(
            run.decide(RunCommand::StartModelTurn),
            Err(RuntimeError::BudgetExceeded {
                budget: "elapsed_ms",
                used: 10_000,
                limit: 10_000
            })
        ));

        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ProposeToolCall(proposal()),
        );
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::ObserveElapsed {
                total_elapsed_ms: 10_000,
            },
        );
        assert!(matches!(
            run.decide(RunCommand::ApproveToolCall(intent())),
            Err(RuntimeError::BudgetExceeded {
                budget: "elapsed_ms",
                used: 10_000,
                limit: 10_000
            })
        ));
    }

    #[test]
    fn retry_from_model_turn_returns_to_preparing() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::RecordRetry {
                reason_code: "provider_timeout".to_owned(),
            },
        );
        assert_eq!(run.phase(), RunPhase::Preparing);
        assert_eq!(run.retries_used(), 1);
    }

    #[test]
    fn revision_exhaustion_fails_closed() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        run.revision = u64::MAX;
        assert_eq!(
            run.decide(RunCommand::Prepare),
            Err(RuntimeError::RevisionExhausted)
        );
        assert_eq!(
            run.apply(RunEvent {
                sequence: 0,
                kind: RunEventKind::Prepared,
            }),
            Err(RuntimeError::RevisionExhausted)
        );
    }

    #[test]
    fn terminal_phases_reject_state_changes() {
        let terminal_commands = [
            RunCommand::Fail {
                error_code: "provider_failed".to_owned(),
            },
            RunCommand::Cancel {
                reason_code: "user_cancelled".to_owned(),
            },
            RunCommand::Expire,
        ];
        for terminal_command in terminal_commands {
            let run = AgentRun::new(spec());
            let Ok(mut run) = run else {
                panic!("fixture spec must be valid");
            };
            let mut events = Vec::new();
            decide_and_apply(&mut run, &mut events, terminal_command);
            let terminal_phase = run.phase();
            let before = run.clone();
            assert!(matches!(
                run.decide(RunCommand::Prepare),
                Err(RuntimeError::TerminalRunMutation { phase, .. }) if phase == terminal_phase
            ));
            assert_eq!(run, before);
        }

        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let mut events = Vec::new();
        decide_and_apply(&mut run, &mut events, RunCommand::Prepare);
        decide_and_apply(&mut run, &mut events, RunCommand::StartModelTurn);
        decide_and_apply(&mut run, &mut events, RunCommand::RecordModelUsage(usage()));
        decide_and_apply(
            &mut run,
            &mut events,
            RunCommand::Complete {
                output_digest: ZERO_DIGEST.to_owned(),
            },
        );
        let before = run.clone();
        assert!(matches!(
            run.decide(RunCommand::ObserveElapsed {
                total_elapsed_ms: 1
            }),
            Err(RuntimeError::TerminalRunMutation {
                phase: RunPhase::Completed,
                ..
            })
        ));
        assert_eq!(run, before);
    }

    #[test]
    fn event_sequence_duplicates_fail_closed() {
        let run = AgentRun::new(spec());
        let Ok(mut run) = run else {
            panic!("fixture spec must be valid");
        };
        let event = RunEvent {
            sequence: 1,
            kind: RunEventKind::Prepared,
        };
        assert!(run.apply(event.clone()).is_ok());
        assert!(matches!(
            run.apply(event),
            Err(RuntimeError::EventSequenceMismatch {
                expected: 2,
                actual: 1
            })
        ));
    }
}
