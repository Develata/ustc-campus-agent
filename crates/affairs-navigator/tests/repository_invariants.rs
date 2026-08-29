#![allow(clippy::unwrap_used)]

//! Checked fixture repository seeding and non-mutation evidence.

mod common;

use affairs_navigator::*;
use common::*;

fn artifact_for(procedure: &str, artifact_id: &str) -> artifact::ProcedureArtifact {
    build_artifact_full_with_id(
        artifact_id,
        procedure,
        50,
        150,
        evidence::EvidenceConflictState::NoKnownConflict,
        evidence::AuthorityComparison::Equivalent,
        None,
        100,
        200,
        vec![assessment(
            evidence::AffairsAuthority::OfficialBulletin,
            "src1",
            evidence::AuthoritySubject::ProcedureTitle,
            100,
            100,
            None,
            None,
        )],
        Vec::new(),
    )
}

#[test]
fn rejects_procedure_mismatch_without_mutation() {
    let mut repo = repository::InMemoryAffairsRepository::new();
    let artifact = artifact_for("proc:a", "artifact:a:v1");
    let state = artifact::ProcedurePublicationState::current(
        value::ProcedureId::parse("proc:b").unwrap(),
        artifact.artifact_id().clone(),
    );

    assert_eq!(
        repo.seed(artifact, state).unwrap_err(),
        repository::RepositorySeedError::ProcedureIdMismatch
    );
    let a = value::ProcedureId::parse("proc:a").unwrap();
    let b = value::ProcedureId::parse("proc:b").unwrap();
    assert!(repo.find_publication_state(&a).unwrap().is_none());
    assert!(repo.find_publication_state(&b).unwrap().is_none());
    assert!(repo.find_current_artifact(&a).unwrap().is_none());
    assert!(repo.find_current_artifact(&b).unwrap().is_none());
}

#[test]
fn rejects_current_artifact_mismatch_without_mutation() {
    let mut repo = repository::InMemoryAffairsRepository::new();
    let artifact = artifact_for("proc:a", "artifact:a:v1");
    let state = artifact::ProcedurePublicationState::current(
        value::ProcedureId::parse("proc:a").unwrap(),
        value::ArtifactId::parse("artifact:other:v1").unwrap(),
    );

    assert_eq!(
        repo.seed(artifact, state).unwrap_err(),
        repository::RepositorySeedError::CurrentArtifactIdMismatch
    );
    let id = value::ProcedureId::parse("proc:a").unwrap();
    assert!(repo.find_publication_state(&id).unwrap().is_none());
    assert!(repo.find_current_artifact(&id).unwrap().is_none());
}

#[test]
fn rejects_duplicate_identities_without_overwriting_the_first_pair() {
    let mut repo = repository::InMemoryAffairsRepository::new();
    let first = artifact_for("proc:a", "artifact:a:v1");
    let first_state = artifact::ProcedurePublicationState::current(
        first.procedure_id().clone(),
        first.artifact_id().clone(),
    );
    repo.seed(first.clone(), first_state.clone()).unwrap();

    assert_eq!(
        repo.seed(first, first_state).unwrap_err(),
        repository::RepositorySeedError::DuplicateArtifact
    );

    let second = artifact_for("proc:a", "artifact:b:v1");
    let second_state = artifact::ProcedurePublicationState::current(
        second.procedure_id().clone(),
        second.artifact_id().clone(),
    );
    assert_eq!(
        repo.seed(second, second_state).unwrap_err(),
        repository::RepositorySeedError::DuplicatePublicationState
    );

    let id = value::ProcedureId::parse("proc:a").unwrap();
    let retained = repo.find_current_artifact(&id).unwrap().unwrap();
    assert_eq!(retained.artifact_id().as_str(), "artifact:a:v1");
    assert_eq!(
        repo.find_publication_state(&id)
            .unwrap()
            .unwrap()
            .current_artifact_id()
            .unwrap()
            .as_str(),
        "artifact:a:v1"
    );
}
