//! Six-outcome `affairs.get` ladder, `CannotVerifyReason`, infrastructure
//! error, and the query carrier (M71-v8n §4 / taskbook "Exact semantics").
//!
//! The six outcomes are mutually exclusive. One current artifact maximum per
//! procedure. Infrastructure failure is typed `GetProcedureError`, never
//! `NotFound`.

use time::OffsetDateTime;

use crate::public_view::{CutoffMetadata, Freshness, PublicProcedureView};
use crate::value::ProcedureId;

/// Why a `CannotVerify` outcome terminated the ladder. Closed four-variant
/// enum (M71-v8n §4.2 / taskbook).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CannotVerifyReason {
    SourceRevisionUnverified,
    EffectiveIntervalMissing,
    LastVerifiedStaleBeyondPolicy,
    PublicEvidenceProjectionOverflow { mandatory_count: u8 },
}

/// The exact six-outcome `affairs.get` ladder. Order is frozen by the M71
/// application service: state/current → known-at cutoff → freshness →
/// unresolved/incomparable conflict → stale-beyond-policy → retained M60
/// verification/effective interval → public projection → Found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetProcedureOutcome {
    /// The procedure has one Current artifact, the evidence is Verified, and
    /// the public projection was built.
    Found {
        view: Box<PublicProcedureView>,
        freshness: Freshness,
        as_of: OffsetDateTime,
    },
    /// The procedure exists but its current artifact's `known_at` is after the
    /// cutoff. Carries cutoff metadata (caller-provided vs system-now).
    NotYetKnown {
        procedure_id: ProcedureId,
        known_at: OffsetDateTime,
        as_of: OffsetDateTime,
        cutoff_metadata: CutoffMetadata,
    },
    /// The procedure was archived; no Current artifact exists.
    Archived {
        procedure_id: ProcedureId,
        archived_at: OffsetDateTime,
    },
    /// No procedure by that ID exists. Carries only the ID; no existence
    /// oracle.
    NotFound { procedure_id: ProcedureId },
    /// An unresolved/incomparable material conflict among retained peer
    /// sources was detected before projection. Carries safe conflict detail.
    Conflict {
        procedure_id: ProcedureId,
        conflict: crate::public_view::ConflictDetail,
    },
    /// The ladder could not produce a verified, presentable, projectable
    /// view. Closed four reasons.
    CannotVerify {
        procedure_id: ProcedureId,
        reason: CannotVerifyReason,
    },
}

/// Infrastructure failure of the `affairs.get` query. Never a public
/// `NotFound` or unverified semantic outcome; the M71 service maps M60 port
/// errors here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GetProcedureError {
    PersistenceUnavailable,
    JournalCorrupted,
    StoreCorrupted,
    M60StoreUnavailable,
    M60StoreCorrupted,
    InternalInconsistent,
}

impl std::fmt::Display for GetProcedureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PersistenceUnavailable => "affairs.get: persistence unavailable",
            Self::JournalCorrupted => "affairs.get: journal corrupted",
            Self::StoreCorrupted => "affairs.get: store corrupted",
            Self::M60StoreUnavailable => "affairs.get: M60 store unavailable",
            Self::M60StoreCorrupted => "affairs.get: M60 store corrupted",
            Self::InternalInconsistent => "affairs.get: internal inconsistent",
        };
        formatter.write_str(s)
    }
}

impl std::error::Error for GetProcedureError {}

/// The query carrier. `as_of = None` is a new authorized read; the M71 service
/// fills it from the clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffairsGetQuery {
    procedure_id: ProcedureId,
    as_of: Option<OffsetDateTime>,
}

impl AffairsGetQuery {
    /// Builds one query.
    #[must_use]
    pub fn new(procedure_id: ProcedureId, as_of: Option<OffsetDateTime>) -> Self {
        Self {
            procedure_id,
            as_of,
        }
    }

    #[must_use]
    pub fn procedure_id(&self) -> &ProcedureId {
        &self.procedure_id
    }

    #[must_use]
    pub const fn as_of(&self) -> Option<OffsetDateTime> {
        self.as_of
    }
}
