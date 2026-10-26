use crate::common::errors::AppError;
use crate::schemas::internal::{AdCandidate, UserContext};
use async_trait::async_trait;

#[async_trait]
pub trait Filter: Send + Sync {
    async fn filter(
        &self,
        candidates: Vec<AdCandidate>,
        user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError>;
}

pub struct PassThroughFilter;

#[async_trait]
impl Filter for PassThroughFilter {
    async fn filter(
        &self,
        candidates: Vec<AdCandidate>,
        _user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError> {
        Ok(candidates)
    }
}

pub struct CompositeFilter {
    filters: Vec<Box<dyn Filter>>,
}

impl CompositeFilter {
    pub fn new(filters: Vec<Box<dyn Filter>>) -> Self {
        Self { filters }
    }
}

#[async_trait]
impl Filter for CompositeFilter {
    async fn filter(
        &self,
        mut candidates: Vec<AdCandidate>,
        user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError> {
        for f in &self.filters {
            candidates = f.filter(candidates, user_context).await?;
            if candidates.is_empty() {
                break;
            }
        }
        Ok(candidates)
    }
}
