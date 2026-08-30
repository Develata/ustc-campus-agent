//! M71-owned application seam consumed by M10 `application-ingress`.
//!
//! [`M71AffairsGetPort`] is the only M71 trait the M10 server imports
//! (taskbook §3.2 allowlist). It is a one-way seam: a caller supplies an
//! [`AffairsGetQuery`] and receives the sealed [`M71AffairsGetReceipt`]. The
//! port exposes no repository handle, no M60 port, no raw evidence reference,
//! and no public receipt constructor, so callers cannot inject a prebuilt
//! receipt or bypass the M71 outcome/lineage pairing.
//!
//! The blanket implementation delegates to the actual M71 application service
//! ([`AffairsGetService::execute`]), which is the sole constructor of the
//! sealed receipt. The crate admits only M60-owned source-revision values from
//! platform core and no M10/M80/client/storage dependency, so the query port
//! carries no client or storage type.

use crate::{AffairsGetQuery, AffairsGetService, GetProcedureError, M71AffairsGetReceipt};

/// M71 application port consumed by M10. Returns the sealed M71 receipt for one
/// `affairs.get` query; the blanket implementation delegates to the real M71
/// application service.
///
/// The trait is usable as `&dyn M71AffairsGetPort` so the M10 server can hold
/// one erased port object. Only the M71 service can produce a valid
/// implementation, because [`M71AffairsGetReceipt`] has no public constructor:
///
/// ```compile_fail
/// // External code (including an M10 caller of the port) cannot build a
/// // receipt to inject. The constructor is private to the M71 service module.
/// let _ = affairs_navigator::M71AffairsGetReceipt::new;
/// ```
pub trait M71AffairsGetPort: Send + Sync {
    /// Executes the frozen six-outcome `affairs.get` ladder and returns the
    /// sealed M71 receipt.
    ///
    /// # Errors
    ///
    /// Returns [`GetProcedureError`] on infrastructure failure (persistence,
    /// M60 store, internal inconsistency). Infrastructure failure is never
    /// typed as `NotFound` or an unverified semantic outcome.
    fn affairs_get(
        &self,
        query: &AffairsGetQuery,
    ) -> Result<M71AffairsGetReceipt, GetProcedureError>;
}

impl<'a> M71AffairsGetPort for AffairsGetService<'a> {
    fn affairs_get(
        &self,
        query: &AffairsGetQuery,
    ) -> Result<M71AffairsGetReceipt, GetProcedureError> {
        self.execute(query)
    }
}
