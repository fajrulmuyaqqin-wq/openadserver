use crate::common::cache::{CacheKeys, RedisClient};
use crate::common::errors::AppError;
use crate::common::utils::{current_date, current_hour};
use crate::rec_engine::filter::base::Filter;
use crate::schemas::internal::{AdCandidate, FrequencyInfo, UserContext};
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub struct FrequencyFilter {
    redis: RedisClient,
    pub default_daily_cap: i32,
    pub default_hourly_cap: i32,
}

impl FrequencyFilter {
    pub fn new(
        redis: RedisClient,
        default_daily_cap: Option<i32>,
        default_hourly_cap: Option<i32>,
    ) -> Self {
        Self {
            redis,
            default_daily_cap: default_daily_cap.unwrap_or(3),
            default_hourly_cap: default_hourly_cap.unwrap_or(1),
        }
    }

    async fn get_frequency(&self, user_id: &str, campaign_id: i64) -> FrequencyInfo {
        let today = current_date();
        let hour = current_hour();

        let daily_key = CacheKeys::freq_daily(user_id, campaign_id, &today);
        let hourly_key = CacheKeys::freq_hourly(user_id, campaign_id, &hour);

        let daily_count = self
            .redis
            .get(&daily_key)
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);

        let hourly_count = self
            .redis
            .get(&hourly_key)
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0);

        FrequencyInfo {
            user_id: user_id.to_string(),
            campaign_id,
            daily_count,
            hourly_count,
            daily_cap: Some(self.default_daily_cap),
            hourly_cap: Some(self.default_hourly_cap),
        }
    }
}

#[async_trait]
impl Filter for FrequencyFilter {
    async fn filter(
        &self,
        candidates: Vec<AdCandidate>,
        user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError> {
        let user_id = match &user_context.user_id {
            Some(u) if !u.is_empty() => u,
            _ => return Ok(candidates), // No frequency control without user_id
        };

        if candidates.is_empty() {
            return Ok(candidates);
        }

        let unique_campaigns: HashSet<i64> = candidates.iter().map(|c| c.campaign_id).collect();
        let mut freq_map: HashMap<i64, FrequencyInfo> = HashMap::new();

        for camp_id in unique_campaigns {
            let freq = self.get_frequency(user_id, camp_id).await;
            freq_map.insert(camp_id, freq);
        }

        let initial_count = candidates.len();
        let filtered: Vec<AdCandidate> = candidates
            .into_iter()
            .filter(|c| {
                if let Some(freq) = freq_map.get(&c.campaign_id) {
                    !freq.is_capped()
                } else {
                    true
                }
            })
            .collect();

        let removed = initial_count - filtered.len();
        if removed > 0 {
            debug!(
                user_id = %user_id,
                "FrequencyFilter removed {} candidates", removed
            );
        }

        Ok(filtered)
    }
}
