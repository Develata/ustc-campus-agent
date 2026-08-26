//! In-memory affairs repository for fixture-seeded query testing. Stores
//! `ProcedureArtifact`s and `ProcedurePublicationState`s keyed by ID. This is
//! NOT a production repository — it is fixture evidence for the M71 spike.

use std::collections::BTreeMap;

use crate::artifact::{ProcedureArtifact, ProcedurePublicationState};
use crate::value::{ArtifactId, ProcedureId};

/// Why a fixture artifact/publication-state pair was rejected before mutation.
/// Payloads are intentionally omitted so rejected caller data cannot leak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositorySeedError {
    ProcedureIdMismatch,
    CurrentArtifactIdMismatch,
    IncompatiblePublicationState,
    DuplicateArtifact,
    DuplicatePublicationState,
}

/// In-memory repository. Seeded through checked fixture constructors; the
/// service reads through the `AffairsRepository` trait.
#[derive(Debug, Default)]
pub struct InMemoryAffairsRepository {
    artifacts: BTreeMap<ArtifactId, ProcedureArtifact>,
    publication_states: BTreeMap<ProcedureId, ProcedurePublicationState>,
}

impl InMemoryAffairsRepository {
    /// Builds one empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seeds one coherent artifact/publication-state pair atomically. Every
    /// invariant and duplicate check runs before either map is mutated.
    pub fn seed(
        &mut self,
        artifact: ProcedureArtifact,
        state: ProcedurePublicationState,
    ) -> Result<&mut Self, RepositorySeedError> {
        if artifact.procedure_id() != state.procedure_id() {
            return Err(RepositorySeedError::ProcedureIdMismatch);
        }
        match (state.current_artifact_id(), state.archived_at()) {
            (Some(current), None) if current == artifact.artifact_id() => {}
            (Some(_), None) => {
                return Err(RepositorySeedError::CurrentArtifactIdMismatch);
            }
            (None, Some(_)) => {}
            _ => {
                return Err(RepositorySeedError::IncompatiblePublicationState);
            }
        }
        if self.artifacts.contains_key(artifact.artifact_id()) {
            return Err(RepositorySeedError::DuplicateArtifact);
        }
        if self.publication_states.contains_key(state.procedure_id()) {
            return Err(RepositorySeedError::DuplicatePublicationState);
        }

        self.artifacts
            .insert(artifact.artifact_id().clone(), artifact);
        self.publication_states
            .insert(state.procedure_id().clone(), state);
        Ok(self)
    }
}

/// Read-only repository trait consumed by the M71 application service.
pub trait AffairsRepository: Send + Sync {
    /// Returns the current artifact for `procedure_id`, if one exists.
    fn find_current_artifact(&self, procedure_id: &ProcedureId) -> Option<ProcedureArtifact>;

    /// Returns the publication state for `procedure_id`, if it exists.
    fn find_publication_state(
        &self,
        procedure_id: &ProcedureId,
    ) -> Option<ProcedurePublicationState>;
}

impl AffairsRepository for InMemoryAffairsRepository {
    fn find_current_artifact(&self, procedure_id: &ProcedureId) -> Option<ProcedureArtifact> {
        let state = self.publication_states.get(procedure_id)?;
        let artifact_id = state.current_artifact_id()?;
        self.artifacts.get(artifact_id).cloned()
    }

    fn find_publication_state(
        &self,
        procedure_id: &ProcedureId,
    ) -> Option<ProcedurePublicationState> {
        self.publication_states.get(procedure_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_repo_returns_none() {
        let repo = InMemoryAffairsRepository::new();
        let pid = ProcedureId::parse("proc:none").expect("valid id");
        assert!(repo.find_current_artifact(&pid).is_none());
        assert!(repo.find_publication_state(&pid).is_none());
    }
}
