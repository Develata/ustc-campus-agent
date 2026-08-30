//! In-memory equal-contract M60 fixture adapter (TD-2). This is explicitly
//! fixture evidence, not accepted M60 implementation. It stores
//! `M60RevisionRef` values and verifies retained evidence by comparing
//! digests, checking revocation, and optionally requiring effective intervals.

use std::collections::{BTreeMap, BTreeSet};

use time::OffsetDateTime;
use ustc_campus_agent_core::source_revision::{SourceRevision, SourceRevisionHealth};

use crate::evidence::M60RevisionRef;
use crate::m60_port::{
    M60EvidencePortError, M60EvidenceUnverifiedReason, M60ProcedureEvidencePort,
    M60RetainedEvidenceOutcome, M60RetainedEvidenceRequest, M60VerificationIdentity,
    M60VerifiedEvidenceSet, compute_evidence_set_digest,
};
use crate::publication::{M60ProcedurePublicationOutcome, M60ProcedurePublicationPort};
use crate::value::{AffairsValueError, SourceId};

/// In-memory M60 fixture adapter. Stores retained accepted/equal-contract
/// revision refs and verifies them against incoming requests. NOT accepted M60
/// implementation — equal-contract fixture evidence only.
#[derive(Debug, Clone)]
pub struct M60FixtureAdapter {
    stored: BTreeMap<(String, String), M60RevisionRef>,
    revoked: BTreeSet<(String, String)>,
    require_effective_interval: bool,
    verifier_id: String,
    evidence_contract_version: u16,
    failure_mode: Option<M60EvidencePortError>,
    revision_health: SourceRevisionHealth,
}

impl M60FixtureAdapter {
    /// Builds one fixture adapter with the given verifier identity and
    /// evidence contract version.
    ///
    /// # Errors
    ///
    /// Returns [`AffairsValueError`] when `verifier_id` is empty or
    /// `evidence_contract_version` is zero.
    pub fn new(
        verifier_id: &str,
        evidence_contract_version: u16,
    ) -> Result<Self, AffairsValueError> {
        // Validate early so expect() in verify_retained is justified.
        let _ = M60VerificationIdentity::new(
            verifier_id.to_owned(),
            OffsetDateTime::from_unix_timestamp(0).expect("epoch is valid"),
            evidence_contract_version,
        )?;
        Ok(Self {
            stored: BTreeMap::new(),
            revoked: BTreeSet::new(),
            require_effective_interval: false,
            verifier_id: verifier_id.to_owned(),
            evidence_contract_version,
            failure_mode: None,
            revision_health: SourceRevisionHealth::Current,
        })
    }

    /// Stores one retained revision ref. Later requests carrying the same
    /// `(source_id, revision_id)` with matching digests will verify.
    pub fn store(&mut self, revision_ref: M60RevisionRef) -> &mut Self {
        let key = (
            revision_ref.source_id().as_str().to_owned(),
            revision_ref.revision_id().to_owned(),
        );
        self.stored.insert(key, revision_ref);
        self
    }

    /// Marks a `(source_id, revision_id)` pair as revoked. Later requests
    /// carrying that ref will return `RevokedOrUnaccepted`.
    pub fn revoke(&mut self, source_id: &SourceId, revision_id: &str) -> &mut Self {
        self.revoked
            .insert((source_id.as_str().to_owned(), revision_id.to_owned()));
        self
    }

    /// When enabled, requests with any ref lacking both `effective_from` and
    /// `effective_to` will return `EffectiveIntervalMissing`.
    pub fn require_effective_interval(&mut self, require: bool) -> &mut Self {
        self.require_effective_interval = require;
        self
    }

    /// Simulates an infrastructure failure. When set, `verify_retained`
    /// returns the given error instead of checking refs.
    pub fn set_failure_mode(&mut self, mode: Option<M60EvidencePortError>) -> &mut Self {
        self.failure_mode = mode;
        self
    }

    /// Selects the transaction-current health returned by the publication
    /// fixture port. Production M60 derives this from source policy and retained
    /// state; callers cannot supply it directly to the publication service.
    pub fn set_revision_health(&mut self, health: SourceRevisionHealth) -> &mut Self {
        self.revision_health = health;
        self
    }

    fn verify_retained_inner(
        &self,
        request: &M60RetainedEvidenceRequest,
    ) -> M60RetainedEvidenceOutcome {
        for req_ref in request.revision_refs() {
            let key = (
                req_ref.source_id().as_str().to_owned(),
                req_ref.revision_id().to_owned(),
            );
            let Some(stored) = self.stored.get(&key) else {
                return M60RetainedEvidenceOutcome::Unverified(
                    M60EvidenceUnverifiedReason::MissingRevision,
                );
            };
            if stored.raw_digest() != req_ref.raw_digest()
                || stored.normalized_digest() != req_ref.normalized_digest()
            {
                return M60RetainedEvidenceOutcome::Unverified(
                    M60EvidenceUnverifiedReason::DigestMismatch,
                );
            }
            if self.revoked.contains(&key) {
                return M60RetainedEvidenceOutcome::Unverified(
                    M60EvidenceUnverifiedReason::RevokedOrUnaccepted,
                );
            }
            if self.require_effective_interval
                && req_ref.effective_from().is_none()
                && req_ref.effective_to().is_none()
            {
                return M60RetainedEvidenceOutcome::Unverified(
                    M60EvidenceUnverifiedReason::EffectiveIntervalMissing,
                );
            }
        }

        let digest = compute_evidence_set_digest(request.revision_refs());
        let revision_count =
            u8::try_from(request.revision_refs().len()).expect("request length is checked 1..=16");
        let identity = M60VerificationIdentity::new(
            self.verifier_id.clone(),
            request.as_of(),
            self.evidence_contract_version,
        )
        .expect("validated in constructor");
        let verified_set = M60VerifiedEvidenceSet::new(digest, revision_count, identity)
            .expect("revision_count is checked >0");
        M60RetainedEvidenceOutcome::Verified(verified_set)
    }
}

impl M60ProcedureEvidencePort for M60FixtureAdapter {
    fn verify_retained(
        &self,
        request: &M60RetainedEvidenceRequest,
    ) -> Result<M60RetainedEvidenceOutcome, M60EvidencePortError> {
        if let Some(mode) = self.failure_mode {
            return Err(mode);
        }
        Ok(self.verify_retained_inner(request))
    }
}

impl M60ProcedurePublicationPort for M60FixtureAdapter {
    fn verify_publication(
        &self,
        _revision: &SourceRevision,
        request: &M60RetainedEvidenceRequest,
    ) -> Result<M60ProcedurePublicationOutcome, M60EvidencePortError> {
        if let Some(mode) = self.failure_mode {
            return Err(mode);
        }
        if self.revision_health != SourceRevisionHealth::Current {
            return Ok(M60ProcedurePublicationOutcome::SourceNotCurrent(
                self.revision_health,
            ));
        }
        Ok(match self.verify_retained_inner(request) {
            M60RetainedEvidenceOutcome::Verified(verified) => {
                M60ProcedurePublicationOutcome::CurrentVerified(verified)
            }
            M60RetainedEvidenceOutcome::Unverified(reason) => {
                M60ProcedurePublicationOutcome::Unverified(reason)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::Sha256;
    use crate::value::ProcedureId;

    fn rev(source: &str, idx: usize) -> M60RevisionRef {
        M60RevisionRef::new(
            SourceId::parse(source).expect("valid source"),
            format!("rev:{source}:{idx}"),
            OffsetDateTime::from_unix_timestamp(0).expect("epoch"),
            None,
            None,
            None,
            Sha256::new(format!(
                "sha256:{}",
                std::iter::repeat_n('0', 64).collect::<String>()
            ))
            .expect("valid digest"),
            Sha256::new(format!(
                "sha256:{}",
                std::iter::repeat_n('1', 64).collect::<String>()
            ))
            .expect("valid digest"),
        )
        .expect("valid revision ref")
    }

    fn request(refs: Vec<M60RevisionRef>) -> M60RetainedEvidenceRequest {
        M60RetainedEvidenceRequest::new(
            ProcedureId::parse("proc:fixture").expect("valid id"),
            OffsetDateTime::from_unix_timestamp(1000).expect("valid time"),
            refs,
        )
        .expect("valid request")
    }

    #[test]
    fn verified_happy_path() {
        let r = rev("s1", 0);
        let mut adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        adapter.store(r.clone());
        let outcome = adapter
            .verify_retained(&request(vec![r]))
            .expect("no infra error");
        assert!(matches!(outcome, M60RetainedEvidenceOutcome::Verified(_)));
    }

    #[test]
    fn missing_revision() {
        let r = rev("s1", 0);
        let adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        let outcome = adapter
            .verify_retained(&request(vec![r]))
            .expect("no infra error");
        assert!(matches!(
            outcome,
            M60RetainedEvidenceOutcome::Unverified(M60EvidenceUnverifiedReason::MissingRevision)
        ));
    }

    #[test]
    fn digest_mismatch() {
        let r_stored = rev("s1", 0);
        let r_request = M60RevisionRef::new(
            SourceId::parse("s1").expect("valid"),
            "rev:s1:0".to_owned(),
            OffsetDateTime::from_unix_timestamp(0).expect("epoch"),
            None,
            None,
            None,
            Sha256::new(format!(
                "sha256:{}",
                std::iter::repeat_n('f', 64).collect::<String>()
            ))
            .expect("valid"),
            Sha256::new(format!(
                "sha256:{}",
                std::iter::repeat_n('1', 64).collect::<String>()
            ))
            .expect("valid"),
        )
        .expect("valid");
        let mut adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        adapter.store(r_stored);
        let outcome = adapter
            .verify_retained(&request(vec![r_request]))
            .expect("no infra error");
        assert!(matches!(
            outcome,
            M60RetainedEvidenceOutcome::Unverified(M60EvidenceUnverifiedReason::DigestMismatch)
        ));
    }

    #[test]
    fn revoked_or_unaccepted() {
        let r = rev("s1", 0);
        let mut adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        adapter.store(r.clone());
        adapter.revoke(r.source_id(), r.revision_id());
        let outcome = adapter
            .verify_retained(&request(vec![r]))
            .expect("no infra error");
        assert!(matches!(
            outcome,
            M60RetainedEvidenceOutcome::Unverified(
                M60EvidenceUnverifiedReason::RevokedOrUnaccepted
            )
        ));
    }

    #[test]
    fn effective_interval_missing() {
        let r = rev("s1", 0);
        let mut adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        adapter.store(r.clone());
        adapter.require_effective_interval(true);
        let outcome = adapter
            .verify_retained(&request(vec![r]))
            .expect("no infra error");
        assert!(matches!(
            outcome,
            M60RetainedEvidenceOutcome::Unverified(
                M60EvidenceUnverifiedReason::EffectiveIntervalMissing
            )
        ));
    }

    #[test]
    fn store_unavailable() {
        let r = rev("s1", 0);
        let mut adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        adapter.store(r.clone());
        adapter.set_failure_mode(Some(M60EvidencePortError::StoreUnavailable));
        let result = adapter.verify_retained(&request(vec![r]));
        assert!(matches!(
            result,
            Err(M60EvidencePortError::StoreUnavailable)
        ));
    }

    #[test]
    fn store_corrupted() {
        let r = rev("s1", 0);
        let mut adapter = M60FixtureAdapter::new("verifier:fixture", 1).expect("valid");
        adapter.store(r.clone());
        adapter.set_failure_mode(Some(M60EvidencePortError::StoreCorrupted));
        let result = adapter.verify_retained(&request(vec![r]));
        assert!(matches!(result, Err(M60EvidencePortError::StoreCorrupted)));
    }
}
