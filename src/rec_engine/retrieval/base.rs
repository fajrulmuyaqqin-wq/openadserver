use crate::common::errors::AppError;
use crate::schemas::internal::{AdCandidate, UserContext};
use async_trait::async_trait;

#[async_trait]
pub trait Retrieval: Send + Sync {
    async fn retrieve(
        &self,
        user_context: &UserContext,
        slot_id: &str,
        limit: usize,
    ) -> Result<Vec<AdCandidate>, AppError>;

    async fn refresh(&self) -> Result<(), AppError> {
        Ok(())
    }
}
