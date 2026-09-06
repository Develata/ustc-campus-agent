//! Atomic, confirmed creation of a bounded course/calendar plan.
use super::*;
const MAX_BATCHES: usize = 32;
const LIFETIME: u64 = 1800;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarDraft {
    pub title: String,
    pub scheduled_for: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarBatch {
    pub id: String,
    pub subject: String,
    pub request_id: String,
    pub items: Vec<CalendarDraft>,
    pub base_revision: u64,
    pub created_at_unix_secs: u64,
    pub expires_at_unix_secs: u64,
    pub status: CalendarProposalStatus,
    pub result: Vec<CalendarItem>,
}
fn validate_drafts(items: &[CalendarDraft]) -> Result<(), CalendarError> {
    if items.is_empty() || items.len() > 32 {
        return Err(CalendarError::InvalidProposal);
    }
    let mut unique = BTreeSet::new();
    for item in items {
        validate_title(&item.title)?;
        validate_scheduled_for(Some(&item.scheduled_for))?;
        if !unique.insert((item.title.trim(), &item.scheduled_for)) {
            return Err(CalendarError::InvalidProposal);
        }
    }
    Ok(())
}
impl CalendarStore {
    pub fn propose_batch(
        &mut self,
        subject: &str,
        request_id: &str,
        items: Vec<CalendarDraft>,
        now: u64,
    ) -> Result<CalendarBatch, CalendarError> {
        self.resolve_uncertain_durability()?;
        proposals::checked_key(subject)?;
        proposals::checked_key(request_id)?;
        if let Some(old) = self
            .state
            .batches
            .iter()
            .find(|b| b.subject == subject && b.request_id == request_id)
        {
            return if old.items == items {
                Ok(old.clone())
            } else {
                Err(CalendarError::ProposalConflict)
            };
        }
        validate_drafts(&items)?;
        if self.state.batches.len() >= MAX_BATCHES {
            return Err(CalendarError::ProposalLimitExceeded);
        }
        let mut preview = self.state.clone();
        apply(&mut preview, &items, now)?;
        storage::check_items(&preview)?;
        let batch = CalendarBatch {
            id: format!("calendar:batch:{}", self.state.batches.len() + 1),
            subject: subject.to_owned(),
            request_id: request_id.to_owned(),
            items,
            base_revision: self.state.item_revision,
            created_at_unix_secs: now,
            expires_at_unix_secs: now
                .checked_add(LIFETIME)
                .ok_or(CalendarError::ClockUnavailable)?,
            status: CalendarProposalStatus::Pending,
            result: vec![],
        };
        let mut next = self.state.clone();
        next.schema = storage::V3.to_owned();
        next.batches.push(batch.clone());
        self.commit(next)?;
        Ok(batch)
    }
    pub fn batches(&mut self, subject: &str) -> Result<Vec<CalendarBatch>, CalendarError> {
        self.resolve_uncertain_durability()?;
        Ok(self
            .state
            .batches
            .iter()
            .filter(|b| b.subject == subject)
            .cloned()
            .collect())
    }
    pub fn finish_batch(
        &mut self,
        subject: &str,
        id: &str,
        confirm: bool,
        now: u64,
    ) -> Result<CalendarBatch, CalendarError> {
        self.resolve_uncertain_durability()?;
        let index = self
            .state
            .batches
            .iter()
            .position(|b| b.subject == subject && b.id == id)
            .ok_or(CalendarError::ProposalNotFound)?;
        let old = &self.state.batches[index];
        let terminal = if confirm {
            CalendarProposalStatus::Applied
        } else {
            CalendarProposalStatus::Cancelled
        };
        if old.status == terminal {
            return Ok(old.clone());
        }
        if old.status != CalendarProposalStatus::Pending {
            return Err(CalendarError::ProposalConflict);
        }
        if now < old.created_at_unix_secs {
            return Err(CalendarError::ClockUnavailable);
        }
        let mut next = self.state.clone();
        if confirm {
            if now >= old.expires_at_unix_secs {
                return Err(CalendarError::ProposalExpired);
            }
            if old.base_revision != self.state.item_revision {
                return Err(CalendarError::ProposalConflict);
            }
            next.batches[index].result = apply(&mut next, &old.items, now)?;
            proposals::advance_revision(&mut next)?;
        }
        next.batches[index].status = terminal;
        let receipt = next.batches[index].clone();
        self.commit(next)?;
        Ok(receipt)
    }
}
fn apply(
    state: &mut PersistedCalendar,
    items: &[CalendarDraft],
    now: u64,
) -> Result<Vec<CalendarItem>, CalendarError> {
    let mut out = vec![];
    for draft in items {
        let item = proposals::apply(
            state,
            &CalendarMutation::Record {
                title: draft.title.clone(),
                scheduled_for: Some(draft.scheduled_for.clone()),
            },
            now,
        )?;
        reminders::schedule(state, &item)?;
        out.push(item);
    }
    Ok(out)
}
pub(super) fn validate(state: &PersistedCalendar) -> Result<(), CalendarError> {
    if state.batches.len() > MAX_BATCHES {
        return Err(CalendarError::InvalidStore);
    }
    let mut keys = BTreeSet::new();
    let mut applied = BTreeSet::new();
    for (i, b) in state.batches.iter().enumerate() {
        validate_drafts(&b.items).map_err(|_| CalendarError::InvalidStore)?;
        proposals::checked_key(&b.subject).map_err(|_| CalendarError::InvalidStore)?;
        proposals::checked_key(&b.request_id).map_err(|_| CalendarError::InvalidStore)?;
        if b.id != format!("calendar:batch:{}", i + 1)
            || !keys.insert((&b.subject, &b.request_id))
            || b.created_at_unix_secs.checked_add(LIFETIME) != Some(b.expires_at_unix_secs)
            || b.base_revision > state.item_revision
        {
            return Err(CalendarError::InvalidStore);
        }
        if b.status == CalendarProposalStatus::Applied {
            if b.result.len() != b.items.len()
                || b.base_revision >= state.item_revision
                || !applied.insert(b.base_revision)
                || state.proposals.iter().any(|p| {
                    p.status == CalendarProposalStatus::Applied
                        && p.base_revision == b.base_revision
                })
            {
                return Err(CalendarError::InvalidStore);
            }
            let mut ids = BTreeSet::new();
            for (item, draft) in b.result.iter().zip(&b.items) {
                proposals::validate_item(item, state.next_id)?;
                if !ids.insert(&item.id)
                    || item.title != draft.title.trim()
                    || item.scheduled_for.as_ref() != Some(&draft.scheduled_for)
                    || item.created_at_unix_secs < b.created_at_unix_secs
                    || item.created_at_unix_secs >= b.expires_at_unix_secs
                {
                    return Err(CalendarError::InvalidStore);
                }
                if b.base_revision.checked_add(1) == Some(state.item_revision)
                    && !state.items.contains(item)
                {
                    return Err(CalendarError::InvalidStore);
                }
            }
        } else if !b.result.is_empty() {
            return Err(CalendarError::InvalidStore);
        }
    }
    Ok(())
}
