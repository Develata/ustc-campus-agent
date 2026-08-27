use std::error::Error;
use std::fmt;

use crate::{
    BoardFeedPolicy, ChangePublicationRepository, ChangePublicationRepositoryError,
    PublishedChangeEvent, render_atom,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeFeedReceipt {
    policy: BoardFeedPolicy,
    items: Vec<PublishedChangeEvent>,
    atom: String,
}

impl ChangeFeedReceipt {
    #[must_use]
    pub const fn policy(&self) -> &BoardFeedPolicy {
        &self.policy
    }

    #[must_use]
    pub fn items(&self) -> &[PublishedChangeEvent] {
        &self.items
    }

    #[must_use]
    pub fn atom(&self) -> &str {
        &self.atom
    }
}

pub struct ChangeFeedQueryService<'a, R: ChangePublicationRepository> {
    repository: &'a R,
}

impl<'a, R: ChangePublicationRepository> ChangeFeedQueryService<'a, R> {
    #[must_use]
    pub const fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        policy: &BoardFeedPolicy,
    ) -> Result<ChangeFeedReceipt, ChangeFeedQueryError> {
        let items = self
            .repository
            .feed_items(policy.board_id())
            .map_err(ChangeFeedQueryError::Repository)?;
        let atom = render_atom(policy, &items).map_err(|_| ChangeFeedQueryError::Projection)?;
        Ok(ChangeFeedReceipt {
            policy: policy.clone(),
            items,
            atom,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeFeedQueryError {
    Repository(ChangePublicationRepositoryError),
    Projection,
}

impl fmt::Display for ChangeFeedQueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Repository(error) => write!(formatter, "change feed repository failed: {error}"),
            Self::Projection => formatter.write_str("change feed projection failed"),
        }
    }
}

impl Error for ChangeFeedQueryError {}
