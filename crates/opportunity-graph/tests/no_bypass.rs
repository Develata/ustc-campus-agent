#![allow(clippy::unwrap_used)]

//! Public boundary proof for the standalone M72 domain foundation. The crate
//! depends only on M00 identity values, M60 revision values and the retained
//! Course Planning pack. It has no M10/M20/M30/M40/M80, transport, UI or storage
//! implementation dependency.

use ustc_campus_agent_opportunity_graph::*;

#[test]
fn repository_port_is_public_without_storage_dependency() {
    let repository = InMemoryOpportunityProfileRepository::new(2, 2).unwrap();
    let _: &dyn OpportunityProfileRepository = &repository;
}

struct SourcePort;

impl M60OpportunityPort for SourcePort {
    fn revision_health(
        &self,
        _revision: &ustc_campus_agent_core::source_revision::SourceRevision,
    ) -> Result<
        ustc_campus_agent_core::source_revision::SourceRevisionHealth,
        OpportunitySourcePortError,
    > {
        Ok(ustc_campus_agent_core::source_revision::SourceRevisionHealth::Current)
    }
}

#[test]
fn m60_port_is_public_without_retrieval_or_parser_dependency() {
    let source = SourcePort;
    let _: &dyn M60OpportunityPort = &source;
}
