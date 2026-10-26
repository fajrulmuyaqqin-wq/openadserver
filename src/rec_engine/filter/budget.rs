use crate::common::cache::{CacheKeys, RedisClient};
use crate::common::errors::AppError;
use crate::common::utils::current_date;
use crate::rec_engine::filter::base::Filter;
use crate::schemas::internal::{AdCandidate, BudgetInfo, UserContext};
use async_trait::async_trait;
use std::collections::HashSet;
use tracing::debug;

pub struct BudgetFilter {
    redis: RedisClient,
    pub buffer_ratio: f64,
}

impl BudgetFilter {
    pub fn new(redis: RedisClient, buffer_ratio: Option<f64>) -> Self {
        Self {
            redis,
            buffer_ratio: buffer_ratio.unwrap_or(0.95),
        }
    }

    async fn get_budget(&self, campaign_id: i64) -> BudgetInfo {
        let today = current_date();
        let key = CacheKeys::budget(campaign_id, &today);

        if let Ok(map) = self.redis.hgetall(&key).await {
            if !map.is_empty() {
                let daily = map.get("budget_daily").and_then(|v| v.parse::<f64>().ok());
                let total = map.get("budget_total").and_then(|v| v.parse::<f64>().ok());
                let spent_d = map
                    .get("spent_today")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let spent_t = map
                    .get("spent_total")
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);

                return BudgetInfo {
                    campaign_id,
                    budget_daily: daily,
                    budget_total: total,
                    spent_today: spent_d,
                    spent_total: spent_t,
                };
            }
        }

        // Default: assume budget available
        BudgetInfo {
            campaign_id,
            budget_daily: None,
            budget_total: None,
            spent_today: 0.0,
            spent_total: 0.0,
        }
    }
}

#[async_trait]
impl Filter for BudgetFilter {
    async fn filter(
        &self,
        candidates: Vec<AdCandidate>,
        _user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError> {
        if candidates.is_empty() {
            return Ok(candidates);
        }

        let unique_campaigns: HashSet<i64> = candidates.iter().map(|c| c.campaign_id).collect();
        let mut budget_map = std::collections::HashMap::new();

        for camp_id in unique_campaigns {
            let info = self.get_budget(camp_id).await;
            budget_map.insert(camp_id, info);
        }

        let initial_count = candidates.len();
        let filtered: Vec<AdCandidate> = candidates
            .into_iter()
            .filter(|c| {
                if let Some(info) = budget_map.get(&c.campaign_id) {
                    if let Some(daily) = info.budget_daily {
                        if info.spent_today >= daily * self.buffer_ratio {
                            return false;
                        }
                    }
                    if let Some(total) = info.budget_total {
                        if info.spent_total >= total * self.buffer_ratio {
                            return false;
                        }
                    }
                    true
                } else {
                    true
                }
            })
            .collect();

        let removed = initial_count - filtered.len();
        if removed > 0 {
            debug!("BudgetFilter filtered out {} candidates", removed);
        }

        Ok(filtered)
    }
}
