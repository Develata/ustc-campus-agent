//! Calendar application commands bind the admitted subject and server time.
//! Item/proposal transitions and persistence remain in the Calendar owner.
use crate::AffairsComposition;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use ustc_campus_agent_simple_calendar::{
    CalendarError, CalendarItem, CalendarMutation, CalendarProposal,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProposeCalendarIntent {
    pub schema: String,
    pub request_id: String,
    pub mutation: CalendarMutation,
}
#[derive(Serialize)]
pub(crate) struct CalendarProposalList {
    schema: &'static str,
    now_unix_secs: u64,
    timezone: &'static str,
    proposals: Vec<CalendarProposal>,
    items: Vec<CalendarItem>,
}
fn now() -> Result<u64, CalendarError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| CalendarError::ClockUnavailable)
}
impl AffairsComposition {
    fn calendar_subject(&self) -> String {
        format!(
            "{}/{}",
            self.current_tenant_id.as_str(),
            self.current_user_id.as_str()
        )
    }
    pub(crate) fn calendar_proposals(&mut self) -> Result<CalendarProposalList, CalendarError> {
        let subject = self.calendar_subject();
        Ok(CalendarProposalList {
            schema: "calendar-proposals/v1",
            now_unix_secs: now()?,
            timezone: "UTC+08:00",
            proposals: self.calendar.proposals(&subject)?,
            items: self.calendar.items()?.to_vec(),
        })
    }
    pub(crate) fn propose_calendar(
        &mut self,
        intent: ProposeCalendarIntent,
    ) -> Result<CalendarProposal, CalendarError> {
        if intent.schema != "calendar-proposal/v1" {
            return Err(CalendarError::InvalidProposal);
        }
        let subject = self.calendar_subject();
        self.calendar
            .propose(&subject, &intent.request_id, intent.mutation, now()?)
    }
    pub(crate) fn propose_calendar_from_agent(
        &mut self,
        mutation: CalendarMutation,
    ) -> Result<CalendarProposal, CalendarError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CalendarError::ClockUnavailable)?
            .as_nanos();
        let sequence = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| CalendarError::CounterExhausted)?;
        self.propose_calendar(ProposeCalendarIntent {
            schema: "calendar-proposal/v1".to_owned(),
            request_id: format!("agent:{nonce}:{sequence}"),
            mutation,
        })
    }
    pub(crate) fn confirm_calendar_proposal(
        &mut self,
        id: &str,
    ) -> Result<CalendarProposal, CalendarError> {
        let subject = self.calendar_subject();
        self.calendar.confirm_proposal(&subject, id, now()?)
    }
    pub(crate) fn cancel_calendar_proposal(
        &mut self,
        id: &str,
    ) -> Result<CalendarProposal, CalendarError> {
        let subject = self.calendar_subject();
        self.calendar.cancel_proposal(&subject, id, now()?)
    }
}
