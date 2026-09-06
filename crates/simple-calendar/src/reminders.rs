//! Durable station-inbox scheduling; no external notification provider is implied.
use super::*;

pub const MAX_REMINDERS: usize = 512;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReminderStatus {
    Scheduled,
    Delivered,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarReminder {
    pub id: String,
    pub item_id: String,
    pub title: String,
    pub scheduled_for: String,
    pub due_unix_secs: u64,
    pub status: ReminderStatus,
    pub delivered_at_unix_secs: Option<u64>,
    pub read_at_unix_secs: Option<u64>,
}
impl CalendarStore {
    /// A tick commits all due inbox deliveries atomically and returns persisted receipts.
    pub fn dispatch_reminders(&mut self, now: u64) -> Result<Vec<CalendarReminder>, CalendarError> {
        self.resolve_uncertain_durability()?;
        let mut next = self.state.clone();
        let mut changed = false;
        for reminder in &mut next.reminders {
            if reminder.status == ReminderStatus::Scheduled && reminder.due_unix_secs <= now {
                reminder.status = ReminderStatus::Delivered;
                reminder.delivered_at_unix_secs = Some(now);
                changed = true;
            }
        }
        if changed {
            self.commit(next)?;
        }
        Ok(self.state.reminders.clone())
    }
    pub fn reminders(&mut self) -> Result<Vec<CalendarReminder>, CalendarError> {
        self.resolve_uncertain_durability()?;
        Ok(self.state.reminders.clone())
    }
    pub fn read_reminder(&mut self, id: &str, now: u64) -> Result<CalendarReminder, CalendarError> {
        self.resolve_uncertain_durability()?;
        let index = self
            .state
            .reminders
            .iter()
            .position(|r| r.id == id)
            .ok_or(CalendarError::ItemNotFound)?;
        let reminder = &self.state.reminders[index];
        if reminder.status != ReminderStatus::Delivered {
            return Err(CalendarError::ProposalConflict);
        }
        if reminder.read_at_unix_secs.is_some() {
            return Ok(reminder.clone());
        }
        if reminder.delivered_at_unix_secs.is_none_or(|t| t > now) {
            return Err(CalendarError::ClockUnavailable);
        }
        let mut next = self.state.clone();
        next.reminders[index].read_at_unix_secs = Some(now);
        let receipt = next.reminders[index].clone();
        self.commit(next)?;
        Ok(receipt)
    }
}
pub(super) fn schedule(
    state: &mut PersistedCalendar,
    item: &CalendarItem,
) -> Result<(), CalendarError> {
    cancel(state, &item.id);
    let Some(date) = &item.scheduled_for else {
        return Ok(());
    };
    let parsed =
        OffsetDateTime::parse(date, &Rfc3339).map_err(|_| CalendarError::InvalidScheduledFor)?;
    let due =
        u64::try_from(parsed.unix_timestamp()).map_err(|_| CalendarError::InvalidScheduledFor)?;
    if state.reminders.len() >= MAX_REMINDERS {
        return Err(CalendarError::ItemLimitExceeded);
    }
    state.schema = storage::V3.to_owned();
    state.reminders.push(CalendarReminder {
        id: format!("calendar:reminder:{}", state.reminders.len() + 1),
        item_id: item.id.clone(),
        title: item.title.clone(),
        scheduled_for: date.clone(),
        due_unix_secs: due,
        status: ReminderStatus::Scheduled,
        delivered_at_unix_secs: None,
        read_at_unix_secs: None,
    });
    Ok(())
}
pub(super) fn cancel(state: &mut PersistedCalendar, id: &str) {
    for r in &mut state.reminders {
        if r.item_id == id && r.status == ReminderStatus::Scheduled {
            r.status = ReminderStatus::Cancelled;
        }
    }
}
pub(super) fn validate(state: &PersistedCalendar) -> Result<(), CalendarError> {
    if state.reminders.len() > MAX_REMINDERS {
        return Err(CalendarError::InvalidStore);
    }
    let mut scheduled = BTreeSet::new();
    for (i, r) in state.reminders.iter().enumerate() {
        validate_title(&r.title).map_err(|_| CalendarError::InvalidStore)?;
        let date = OffsetDateTime::parse(&r.scheduled_for, &Rfc3339)
            .map_err(|_| CalendarError::InvalidStore)?;
        if r.id != format!("calendar:reminder:{}", i + 1)
            || u64::try_from(date.unix_timestamp()).ok() != Some(r.due_unix_secs)
            || proposals::item_sequence(&r.item_id).is_err()
        {
            return Err(CalendarError::InvalidStore);
        }
        if r.status == ReminderStatus::Delivered {
            if r.delivered_at_unix_secs.is_none_or(|t| t < r.due_unix_secs)
                || r.read_at_unix_secs
                    .zip(r.delivered_at_unix_secs)
                    .is_some_and(|(a, b)| a < b)
            {
                return Err(CalendarError::InvalidStore);
            }
        } else if r.delivered_at_unix_secs.is_some() || r.read_at_unix_secs.is_some() {
            return Err(CalendarError::InvalidStore);
        }
        if r.status == ReminderStatus::Scheduled
            && (!scheduled.insert(&r.item_id)
                || !state.items.iter().any(|item| {
                    item.id == r.item_id
                        && item.title == r.title
                        && item.scheduled_for.as_ref() == Some(&r.scheduled_for)
                }))
        {
            return Err(CalendarError::InvalidStore);
        }
    }
    Ok(())
}
