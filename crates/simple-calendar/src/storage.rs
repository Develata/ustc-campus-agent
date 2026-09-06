//! Closed storage versions share one item/proposal atomic commit.
use super::*;
use proposals::{MAX_PROPOSALS, PENDING_RESERVE_BYTES};

pub(super) const V2: &str = "ustc-simple-calendar-store/v2";
pub(super) const MAX_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Deserialize)]
#[serde(tag = "schema", deny_unknown_fields)]
enum Wire {
    #[serde(rename = "ustc-simple-calendar-store/v1")]
    V1 {
        next_id: u64,
        items: Vec<CalendarItem>,
    },
    #[serde(rename = "ustc-simple-calendar-store/v2")]
    V2 {
        next_id: u64,
        items: Vec<CalendarItem>,
        item_revision: u64,
        next_proposal_id: u64,
        proposals: Vec<CalendarProposal>,
    },
}
#[derive(Serialize)]
struct ItemView<'a> {
    schema: &'static str,
    next_id: u64,
    items: &'a [CalendarItem],
}
#[derive(Serialize)]
struct V2View<'a> {
    schema: &'static str,
    next_id: u64,
    items: &'a [CalendarItem],
    item_revision: u64,
    next_proposal_id: u64,
    proposals: &'a [CalendarProposal],
}
pub(super) fn decode(bytes: &[u8]) -> Result<PersistedCalendar, CalendarError> {
    match serde_json::from_slice::<Wire>(bytes).map_err(|_| CalendarError::InvalidStore)? {
        Wire::V1 { next_id, items } => {
            if bytes.len() as u64 > MAX_STORE_BYTES {
                return Err(CalendarError::InvalidStore);
            }
            Ok(PersistedCalendar {
                next_id,
                items,
                ..PersistedCalendar::default()
            })
        }
        Wire::V2 {
            next_id,
            items,
            item_revision,
            next_proposal_id,
            proposals,
        } => Ok(PersistedCalendar {
            schema: V2.to_owned(),
            next_id,
            items,
            item_revision,
            next_proposal_id,
            proposals,
        }),
    }
}
pub(super) fn item_bytes(state: &PersistedCalendar) -> Result<Vec<u8>, CalendarError> {
    serde_json::to_vec(&ItemView {
        schema: STORE_SCHEMA_VERSION,
        next_id: state.next_id,
        items: &state.items,
    })
    .map_err(|_| CalendarError::PersistenceUnavailable)
}
pub(super) fn check_items(state: &PersistedCalendar) -> Result<(), CalendarError> {
    if state.items.len() > MAX_ITEMS || item_bytes(state)?.len() as u64 > MAX_STORE_BYTES {
        return Err(CalendarError::ItemLimitExceeded);
    }
    Ok(())
}
pub(super) fn encode_bounded(state: &PersistedCalendar) -> Result<Vec<u8>, CalendarError> {
    check_items(state)?;
    if state.schema == STORE_SCHEMA_VERSION {
        return item_bytes(state);
    }
    if state.schema != V2 || state.proposals.len() > MAX_PROPOSALS {
        return Err(CalendarError::ProposalLimitExceeded);
    }
    let bytes = serde_json::to_vec(&V2View {
        schema: V2,
        next_id: state.next_id,
        items: &state.items,
        item_revision: state.item_revision,
        next_proposal_id: state.next_proposal_id,
        proposals: &state.proposals,
    })
    .map_err(|_| CalendarError::PersistenceUnavailable)?;
    let pending = state
        .proposals
        .iter()
        .filter(|p| p.status == CalendarProposalStatus::Pending)
        .count();
    if bytes
        .len()
        .checked_add(pending * PENDING_RESERVE_BYTES)
        .is_none_or(|total| total as u64 > MAX_FILE_BYTES)
    {
        return Err(CalendarError::ProposalLimitExceeded);
    }
    Ok(bytes)
}
