use ustc_campus_agent_change_radar::ChangeFeedReceipt;
use ustc_campus_agent_client_protocol::{
    ChangeFeedEntryDto, ChangeFeedFieldDto, ChangeFeedViewDto, ChangeSourceHealthDto,
    M70ChangeFeedOutcomeDto, M70ChangeFeedTerminalDto, UnixMillis, WireText,
};
use ustc_campus_agent_core::source_revision::{
    RevisionTimestamp, SourceRevisionHealth, SourceRevisionProvenance,
};

pub fn project_change_feed(
    receipt: &ChangeFeedReceipt,
) -> Result<M70ChangeFeedTerminalDto, M70ProjectionError> {
    let policy = receipt.policy();
    let entries = receipt
        .items()
        .iter()
        .map(project_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let view = ChangeFeedViewDto::try_new(
        wire(policy.board_id().as_str())?,
        policy.board_policy_revision(),
        wire(policy.feed_id())?,
        wire(policy.title())?,
        wire(policy.author_name())?,
        wire(policy.public_base_url())?,
        receipt.atom().to_owned(),
        entries,
    )
    .map_err(|_| M70ProjectionError::InvalidView)?;
    Ok(M70ChangeFeedTerminalDto::new(
        M70ChangeFeedOutcomeDto::Found {
            view: Box::new(view),
        },
    ))
}

fn project_entry(
    event: &ustc_campus_agent_change_radar::PublishedChangeEvent,
) -> Result<ChangeFeedEntryDto, M70ProjectionError> {
    let candidate = event.candidate();
    let changed_fields = candidate
        .changed_fields()
        .iter()
        .map(|field| {
            Ok(ChangeFeedFieldDto::new(
                wire(field.field().as_str())?,
                field
                    .before()
                    .map(|value| wire(value.as_str()))
                    .transpose()?,
                field
                    .after()
                    .map(|value| wire(value.as_str()))
                    .transpose()?,
            ))
        })
        .collect::<Result<Vec<_>, M70ProjectionError>>()?;
    let (old_reviewer, old_evidence) = provenance(candidate.old_revision().provenance())?;
    let (new_reviewer, new_evidence) = provenance(candidate.new_revision().provenance())?;
    let source_health = match candidate.health() {
        SourceRevisionHealth::Current => ChangeSourceHealthDto::Current,
        SourceRevisionHealth::Stale => ChangeSourceHealthDto::Stale,
        SourceRevisionHealth::Conflicting => ChangeSourceHealthDto::Conflicting,
    };
    ChangeFeedEntryDto::try_new(
        wire(candidate.event_id().as_str())?,
        wire(event.stable_guid().as_str())?,
        wire(candidate.affected_scope())?,
        changed_fields,
        candidate
            .effective_interval()
            .from()
            .map(timestamp)
            .transpose()?,
        candidate
            .effective_interval()
            .to()
            .map(timestamp)
            .transpose()?,
        timestamp(candidate.observed_at())?,
        timestamp(event.published_at())?,
        source_health,
        wire(candidate.source_id().as_str())?,
        wire(candidate.new_revision().source_url().as_str())?,
        wire(candidate.old_revision().revision_id().as_str())?,
        wire(candidate.old_revision().raw_sha256().as_str())?,
        wire(candidate.old_revision().normalized_sha256().as_str())?,
        wire(old_reviewer)?,
        wire(old_evidence)?,
        wire(candidate.new_revision().revision_id().as_str())?,
        wire(candidate.new_revision().raw_sha256().as_str())?,
        wire(candidate.new_revision().normalized_sha256().as_str())?,
        wire(new_reviewer)?,
        wire(new_evidence)?,
        wire(event.evidence_set_digest().as_str())?,
    )
    .map_err(|_| M70ProjectionError::InvalidView)
}

fn provenance(provenance: &SourceRevisionProvenance) -> Result<(&str, &str), M70ProjectionError> {
    match provenance {
        SourceRevisionProvenance::DemoReviewed { reviewer, evidence } => {
            Ok((reviewer.as_str(), evidence.as_str()))
        }
        _ => Err(M70ProjectionError::UnsupportedProvenance),
    }
}

fn timestamp(value: RevisionTimestamp) -> Result<UnixMillis, M70ProjectionError> {
    value
        .unix_seconds()
        .checked_mul(1_000)
        .map(UnixMillis::new)
        .ok_or(M70ProjectionError::TimestampOverflow)
}

fn wire(value: impl Into<String>) -> Result<WireText, M70ProjectionError> {
    WireText::parse(value).map_err(|_| M70ProjectionError::WireValue)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M70ProjectionError {
    WireValue,
    TimestampOverflow,
    UnsupportedProvenance,
    InvalidView,
}
