use crate::common::cache::RedisClient;
use crate::common::errors::AppError;
use crate::rec_engine::retrieval::base::Retrieval;
use crate::schemas::internal::{AdCandidate, UserContext};
use async_trait::async_trait;
use tracing::debug;

pub struct CacheRetrieval {
    redis: RedisClient,
    cache_prefix: String,
}

impl CacheRetrieval {
    pub fn new(redis: RedisClient, cache_prefix: Option<String>) -> Self {
        Self {
            redis,
            cache_prefix: cache_prefix.unwrap_or_else(|| "retrieval".to_string()),
        }
    }

    async fn get_cached(&self, key: &str, limit: usize) -> Vec<AdCandidate> {
        match self.redis.get_json::<Vec<AdCandidate>>(key).await {
            Ok(Some(mut list)) => {
                list.truncate(limit);
                list
            }
            _ => Vec::new(),
        }
    }
}

#[async_trait]
impl Retrieval for CacheRetrieval {
    async fn retrieve(
        &self,
        user_context: &UserContext,
        slot_id: &str,
        limit: usize,
    ) -> Result<Vec<AdCandidate>, AppError> {
        let mut candidates = Vec::new();

        // 1. Try user-specific cache
        if let Some(user_id) = &user_context.user_id {
            let user_key = format!("{}:user:{}", self.cache_prefix, user_id);
            let user_candidates = self.get_cached(&user_key, limit).await;
            candidates.extend(user_candidates);
        }

        // 2. Try slot-specific cache
        if candidates.len() < limit {
            let slot_key = format!("{}:slot:{}", self.cache_prefix, slot_id);
            let slot_candidates = self.get_cached(&slot_key, limit - candidates.len()).await;
            candidates.extend(slot_candidates);
        }

        // 3. Try global hot ads
        if candidates.len() < limit {
            let hot_key = format!("{}:hot", self.cache_prefix);
            let hot_candidates = self.get_cached(&hot_key, limit - candidates.len()).await;
            candidates.extend(hot_candidates);
        }

        debug!("Cache retrieval returned {} candidates", candidates.len());
        Ok(candidates)
    }
}
