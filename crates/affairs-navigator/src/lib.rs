//! # M71 Affairs Navigator — bounded query and publication foundation
//!
//! Implementation of the frozen M71 public algebra and ratified TD-2 seam
//! (M71-v8n). This crate provides:
//!
//! - Checked M71 value/public DTO types with NO Serde (§11.1).
//! - The deterministic six-outcome lookup ladder (`AffairsGetService`).
//! - Conflict-before-projection and bounded deterministic public evidence
//!   projection.
//! - The exact retained M60 verification port (`M60ProcedureEvidencePort`)
//!   and an in-memory equal-contract M60 fixture adapter.
//! - An in-memory current-artifact/publication-state repository seeded through
//!   checked fixture constructors.
//! - The sealed M71 evidence-lineage receipt (`M71EvidenceLineage`) and
//!   exhaustive conversion-ready accessors.
//!
//! ## No-bypass contract (TD-2)
//!
//! This crate depends on `time`, `sha2`, and M60-owned value carriers from
//! `ustc-campus-agent-core`. There is no dependency on M10, M80, client, or
//! storage. The
//! M71 application service is the sole caller of `M60ProcedureEvidencePort`;
//! M10 cannot bypass M71 to reach M60. This is structurally enforced by the
//! Cargo dependency graph: no `use` statement can name an
//! M10/client/storage type.
//!
//! ## Product nonclaims
//!
//! This is retained partial evidence, not a complete `PROC-011` product path.
//! The in-memory M60 fixture adapter is equal-contract test evidence, not
//! production M60 authority. M00 administrator auth, durable persistence,
//! M10/Web composition and restart recovery remain outside this slice. See
//! `README.md` for the complete nonclaim list.

#![cfg_attr(test, allow(clippy::unwrap_used))]
#![forbid(unsafe_code)]

// ---------------------------------------------------------------------------
// Public modules
// ---------------------------------------------------------------------------

pub mod application_port;
pub mod artifact;
pub mod clock;
pub mod evidence;
pub mod lineage;
pub mod m60_port;
pub mod outcome;
pub mod public_view;
pub mod publication;
pub mod repository;
pub mod service;
pub mod value;

// ---------------------------------------------------------------------------
// Fixture modules (pub for integration test access; documented as fixtures)
// ---------------------------------------------------------------------------

pub mod m60_fixture;

// ---------------------------------------------------------------------------
// Internal modules
// ---------------------------------------------------------------------------

pub(crate) mod projection;

// ---------------------------------------------------------------------------
// Re-exports of the primary public API surface
// ---------------------------------------------------------------------------

pub use application_port::M71AffairsGetPort;
pub use artifact::{
    BoardPolicy, Contact, Deadline, DeadlineKind, EntryPoint, Prerequisite, ProcedureArtifact,
    ProcedurePublicationState, ProcedureStep,
};
pub use clock::{AffairsClock, FixedClock};
pub use evidence::{
    AffairsAuthority, AffairsAuthorityAssessment, AffairsEvidenceAssessment, AuthorityComparison,
    AuthorityDerivation, AuthoritySubject, ConflictKind, EvidenceConflictState, M60RevisionRef,
    ProcedureEvidenceContext, Sha256, UncertaintyState, ValidityHorizon, conflict_description,
    derive_valid_interval,
};
pub use lineage::{EvidenceNotRequiredReason, M71EvidenceLineage};
pub use m60_port::{
    M60EvidencePortError, M60EvidenceUnverifiedReason, M60ProcedureEvidencePort,
    M60RetainedEvidenceOutcome, M60RetainedEvidenceRequest, M60VerificationIdentity,
    M60VerifiedEvidenceSet,
};
pub use outcome::{AffairsGetQuery, CannotVerifyReason, GetProcedureError, GetProcedureOutcome};
pub use public_view::{
    ConflictDetail, ConflictState, CutoffMetadata, CutoffSource, Freshness, LookupPath,
    ProjectionMetadata, PublicEvidenceAssessmentView, PublicEvidenceView, PublicPrerequisiteView,
    PublicProcedureView,
};
pub use publication::*;
pub use repository::{AffairsRepository, InMemoryAffairsRepository, RepositorySeedError};
pub use service::{AffairsGetService, M71AffairsGetReceipt};
pub use value::{
    ActorRef, AffairsValueError, AffairsValueErrorKind, ArtifactId, AudienceTag, BoardId,
    BoardPolicyVersion, ContactChannel, ContactName, ContactRef, DeadlineLabel, EffectiveInterval,
    EntryPointLabel, Instruction, MaterializationReceiptId, PrerequisiteCondition, ProcedureId,
    ProcedurePublicationReceiptId, ProcedureReviewId, SourceId, Title, Url,
};

// ---------------------------------------------------------------------------
// Compile-time proof: no `extern crate` declaration for an
// M10/M80/client/storage crate
// can resolve because none are listed in `Cargo.toml`. This is the structural
// no-bypass proof for TD-2.
// ---------------------------------------------------------------------------

#[doc(hidden)]
const _NO_BYPASS_COMPILE_TIME_PROOF: () = ();

/// Compile-time proof that no M10/client/storage crate is in the dependency
/// graph. This doctest attempts to `use` a crate that exists in the workspace
/// but is NOT a dependency of `affairs-navigator`. It MUST fail to compile —
/// that failure IS the TD-2 no-bypass proof.
#[cfg(doctest)]
#[doc = "```compile_fail
// The following `use` fails because the client crate is not a dependency.
use ustc_campus_agent_client_core;
```"]
const _NO_BYPASS_DOCTEST_ANCHOR: () = ();
