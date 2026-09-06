//! Proposed changes and their terminal receipts belong to the original Calendar store.
use super::*;

pub(super) const MAX_PROPOSALS: usize = 128;
// One result plus one added item, worst-case title escaping and integer growth.
pub(super) const PENDING_RESERVE_BYTES: usize = 4096;
const LIFETIME_SECS: u64 = 30 * 60;
const PROPOSAL_PREFIX: &str = "calendar:proposal:";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum CalendarMutation {
    Record {
        title: String,
        scheduled_for: Option<String>,
    },
    Update {
        item_id: String,
        title: String,
        #[serde(deserialize_with = "required_option")]
        scheduled_for: Option<String>,
    },
    Delete {
        item_id: String,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarProposalStatus {
    Pending,
    Applied,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarProposal {
    pub id: String,
    pub request_id: String,
    pub subject: String,
    pub mutation: CalendarMutation,
    #[serde(deserialize_with = "required_option")]
    pub before: Option<CalendarItem>,
    pub base_revision: u64,
    pub created_at_unix_secs: u64,
    pub expires_at_unix_secs: u64,
    pub status: CalendarProposalStatus,
    #[serde(deserialize_with = "required_option")]
    pub result: Option<CalendarItem>,
}
fn required_option<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(deserializer)
}
impl CalendarMutation {
    fn validate(&self) -> Result<(), CalendarError> {
        match self {
            Self::Record {
                title,
                scheduled_for,
            }
            | Self::Update {
                title,
                scheduled_for,
                ..
            } => {
                validate_title(title)?;
                validate_scheduled_for(scheduled_for.as_deref())?;
            }
            Self::Delete { .. } => {}
        }
        if let Some(id) = self.target() {
            item_sequence(id)?;
        }
        Ok(())
    }
    fn target(&self) -> Option<&str> {
        match self {
            Self::Record { .. } => None,
            Self::Update { item_id, .. } | Self::Delete { item_id } => Some(item_id),
        }
    }
}
impl CalendarStore {
    pub fn propose(
        &mut self,
        subject: &str,
        request_id: &str,
        mutation: CalendarMutation,
        now: u64,
    ) -> Result<CalendarProposal, CalendarError> {
        self.resolve_uncertain_durability()?;
        checked_key(subject)?;
        checked_key(request_id)?;
        if let Some(existing) = self
            .state
            .proposals
            .iter()
            .find(|p| p.subject == subject && p.request_id == request_id)
        {
            return if existing.mutation == mutation {
                Ok(existing.clone())
            } else {
                Err(CalendarError::ProposalConflict)
            };
        }
        mutation.validate()?;
        self.state
            .item_revision
            .checked_add(1)
            .ok_or(CalendarError::CounterExhausted)?;
        if self.state.proposals.len() >= MAX_PROPOSALS {
            return Err(CalendarError::ProposalLimitExceeded);
        }
        let expires_at_unix_secs = now
            .checked_add(LIFETIME_SECS)
            .ok_or(CalendarError::ClockUnavailable)?;
        let before = before(&self.state, &mutation)?;
        // Admission proves the item mutation fits before saving a confirmable proposal.
        // No item revision or item is changed by this disposable capacity preview.
        let mut preview = self.state.clone();
        apply(&mut preview, &mutation, expires_at_unix_secs)?;
        storage::check_items(&preview)?;
        let mut next = self.state.clone();
        let id = next.next_proposal_id;
        next.next_proposal_id = id.checked_add(1).ok_or(CalendarError::CounterExhausted)?;
        let proposal = CalendarProposal {
            id: format!("{PROPOSAL_PREFIX}{id}"),
            request_id: request_id.to_owned(),
            subject: subject.to_owned(),
            mutation,
            before,
            base_revision: next.item_revision,
            created_at_unix_secs: now,
            expires_at_unix_secs,
            status: CalendarProposalStatus::Pending,
            result: None,
        };
        next.schema = storage::V2.to_owned();
        next.proposals.push(proposal.clone());
        self.commit(next)?;
        Ok(proposal)
    }
    pub fn proposals(&mut self, subject: &str) -> Result<Vec<CalendarProposal>, CalendarError> {
        self.resolve_uncertain_durability()?;
        checked_key(subject)?;
        Ok(self
            .state
            .proposals
            .iter()
            .filter(|p| p.subject == subject)
            .cloned()
            .collect())
    }
    pub fn confirm_proposal(
        &mut self,
        subject: &str,
        id: &str,
        now: u64,
    ) -> Result<CalendarProposal, CalendarError> {
        self.resolve_uncertain_durability()?;
        let index = self.proposal_index(subject, id)?;
        let proposal = &self.state.proposals[index];
        if proposal.status == CalendarProposalStatus::Applied {
            return Ok(proposal.clone());
        }
        if proposal.status != CalendarProposalStatus::Pending {
            return Err(CalendarError::ProposalConflict);
        }
        if now >= proposal.expires_at_unix_secs {
            return Err(CalendarError::ProposalExpired);
        }
        if now < proposal.created_at_unix_secs {
            return Err(CalendarError::ClockUnavailable);
        }
        if proposal.base_revision != self.state.item_revision
            || before(&self.state, &proposal.mutation)? != proposal.before
        {
            return Err(CalendarError::ProposalConflict);
        }
        let mut next = self.state.clone();
        let result = apply(&mut next, &proposal.mutation, now)?;
        advance_revision(&mut next)?;
        next.proposals[index].status = CalendarProposalStatus::Applied;
        next.proposals[index].result = Some(result);
        let receipt = next.proposals[index].clone();
        self.commit(next)?;
        Ok(receipt)
    }
    pub fn cancel_proposal(
        &mut self,
        subject: &str,
        id: &str,
        now: u64,
    ) -> Result<CalendarProposal, CalendarError> {
        self.resolve_uncertain_durability()?;
        let index = self.proposal_index(subject, id)?;
        let proposal = &self.state.proposals[index];
        if proposal.status == CalendarProposalStatus::Cancelled {
            return Ok(proposal.clone());
        }
        if proposal.status != CalendarProposalStatus::Pending {
            return Err(CalendarError::ProposalConflict);
        }
        if now < proposal.created_at_unix_secs {
            return Err(CalendarError::ClockUnavailable);
        }
        let mut next = self.state.clone();
        next.proposals[index].status = CalendarProposalStatus::Cancelled;
        let receipt = next.proposals[index].clone();
        self.commit(next)?;
        Ok(receipt)
    }
    fn proposal_index(&self, subject: &str, id: &str) -> Result<usize, CalendarError> {
        checked_key(subject)?;
        self.state
            .proposals
            .iter()
            .position(|p| p.subject == subject && p.id == id)
            .ok_or(CalendarError::ProposalNotFound)
    }
    pub(super) fn mutate_legacy(
        &mut self,
        mutation: CalendarMutation,
        now: u64,
    ) -> Result<CalendarItem, CalendarError> {
        self.resolve_uncertain_durability()?;
        let mut next = self.state.clone();
        let item = apply(&mut next, &mutation, now)?;
        advance_revision(&mut next)?;
        self.commit(next)?;
        Ok(item)
    }
}
fn checked_key(value: &str) -> Result<(), CalendarError> {
    if value.is_empty() || value.len() > 256 || !value.bytes().all(|b| (0x21..=0x7e).contains(&b)) {
        Err(CalendarError::InvalidProposal)
    } else {
        Ok(())
    }
}
fn item_sequence(id: &str) -> Result<u64, CalendarError> {
    let value = id
        .strip_prefix(ITEM_ID_PREFIX)
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .ok_or(CalendarError::InvalidProposal)?;
    if id != format!("{ITEM_ID_PREFIX}{value}") {
        return Err(CalendarError::InvalidProposal);
    }
    Ok(value)
}
fn before(
    state: &PersistedCalendar,
    mutation: &CalendarMutation,
) -> Result<Option<CalendarItem>, CalendarError> {
    mutation
        .target()
        .map(|id| {
            state
                .items
                .iter()
                .find(|item| item.id == id)
                .cloned()
                .ok_or(CalendarError::ItemNotFound)
        })
        .transpose()
}
fn advance_revision(state: &mut PersistedCalendar) -> Result<(), CalendarError> {
    state.item_revision = state
        .item_revision
        .checked_add(1)
        .ok_or(CalendarError::CounterExhausted)?;
    state.schema = storage::V2.to_owned();
    Ok(())
}
fn apply(
    state: &mut PersistedCalendar,
    mutation: &CalendarMutation,
    now: u64,
) -> Result<CalendarItem, CalendarError> {
    mutation.validate()?;
    let result = match mutation {
        CalendarMutation::Record {
            title,
            scheduled_for,
        } => {
            if state.items.len() >= MAX_ITEMS {
                return Err(CalendarError::ItemLimitExceeded);
            }
            let sequence = state.next_id;
            state.next_id = sequence
                .checked_add(1)
                .ok_or(CalendarError::CounterExhausted)?;
            let item = CalendarItem {
                id: format!("{ITEM_ID_PREFIX}{sequence}"),
                title: validate_title(title)?.to_owned(),
                scheduled_for: scheduled_for.clone(),
                created_at_unix_secs: now,
            };
            state.items.push(item.clone());
            item
        }
        CalendarMutation::Update {
            item_id,
            title,
            scheduled_for,
        } => {
            let item = state
                .items
                .iter_mut()
                .find(|i| &i.id == item_id)
                .ok_or(CalendarError::ItemNotFound)?;
            item.title = validate_title(title)?.to_owned();
            item.scheduled_for.clone_from(scheduled_for);
            item.clone()
        }
        CalendarMutation::Delete { item_id } => {
            let index = state
                .items
                .iter()
                .position(|i| &i.id == item_id)
                .ok_or(CalendarError::ItemNotFound)?;
            state.items.remove(index)
        }
    };
    state.items.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(result)
}

pub(super) fn validate_state(state: &PersistedCalendar) -> Result<(), CalendarError> {
    let invalid = CalendarError::InvalidStore;
    if state.schema == STORE_SCHEMA_VERSION {
        return if state.item_revision == 0
            && state.next_proposal_id == 1
            && state.proposals.is_empty()
        {
            Ok(())
        } else {
            Err(invalid)
        };
    }
    if state.proposals.len() > MAX_PROPOSALS
        || state.next_proposal_id != state.proposals.len() as u64 + 1
    {
        return Err(invalid);
    }
    let mut keys = BTreeSet::new();
    let mut applied_revisions = BTreeSet::new();
    for (index, p) in state.proposals.iter().enumerate() {
        checked_key(&p.subject).map_err(|_| invalid)?;
        checked_key(&p.request_id).map_err(|_| invalid)?;
        p.mutation.validate().map_err(|_| invalid)?;
        if p.id != format!("{PROPOSAL_PREFIX}{}", index + 1)
            || !keys.insert((&p.subject, &p.request_id))
            || p.created_at_unix_secs.checked_add(LIFETIME_SECS) != Some(p.expires_at_unix_secs)
            || p.base_revision > state.item_revision
        {
            return Err(invalid);
        }
        match (&p.mutation, &p.before) {
            (CalendarMutation::Record { .. }, None) => {}
            (
                CalendarMutation::Update { item_id, .. } | CalendarMutation::Delete { item_id },
                Some(item),
            ) if item_id == &item.id => {
                validate_item(item, state.next_id)?;
            }
            _ => return Err(invalid),
        }
        if p.status == CalendarProposalStatus::Applied {
            if p.base_revision >= state.item_revision || !applied_revisions.insert(p.base_revision)
            {
                return Err(invalid);
            }
            let item = p.result.as_ref().ok_or(invalid)?;
            validate_item(item, state.next_id)?;
            match &p.mutation {
                CalendarMutation::Record {
                    title,
                    scheduled_for,
                } => {
                    if item.title != title.trim()
                        || &item.scheduled_for != scheduled_for
                        || item.created_at_unix_secs < p.created_at_unix_secs
                        || item.created_at_unix_secs >= p.expires_at_unix_secs
                    {
                        return Err(invalid);
                    }
                }
                CalendarMutation::Update {
                    item_id,
                    title,
                    scheduled_for,
                } => {
                    let old = p.before.as_ref().ok_or(invalid)?;
                    if &item.id != item_id
                        || item.title != title.trim()
                        || &item.scheduled_for != scheduled_for
                        || item.created_at_unix_secs != old.created_at_unix_secs
                    {
                        return Err(invalid);
                    }
                }
                CalendarMutation::Delete { .. } => {
                    if p.result != p.before {
                        return Err(invalid);
                    }
                }
            }
            if p.base_revision.checked_add(1) == Some(state.item_revision) {
                let current = state.items.iter().find(|i| i.id == item.id);
                match p.mutation {
                    CalendarMutation::Delete { .. } if current.is_none() => {}
                    CalendarMutation::Record { .. } | CalendarMutation::Update { .. }
                        if current == Some(item) => {}
                    _ => return Err(invalid),
                }
            }
        } else if p.result.is_some() {
            return Err(invalid);
        }
        if p.status == CalendarProposalStatus::Pending
            && p.base_revision == state.item_revision
            && before(state, &p.mutation).map_err(|_| invalid)? != p.before
        {
            return Err(invalid);
        }
    }
    Ok(())
}
fn validate_item(item: &CalendarItem, next_id: u64) -> Result<(), CalendarError> {
    if item_sequence(&item.id).map_err(|_| CalendarError::InvalidStore)? >= next_id
        || validate_title(&item.title).map_err(|_| CalendarError::InvalidStore)? != item.title
    {
        return Err(CalendarError::InvalidStore);
    }
    validate_scheduled_for(item.scheduled_for.as_deref())
        .map_err(|_| CalendarError::InvalidStore)?;
    Ok(())
}

#[cfg(test)]
mod tests;
