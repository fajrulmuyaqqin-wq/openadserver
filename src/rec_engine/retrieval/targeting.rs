use crate::common::cache::{CacheKeys, RedisClient};
use crate::common::errors::AppError;
use crate::models::{Campaign, Creative, TargetingRule};
use crate::rec_engine::retrieval::base::Retrieval;
use crate::schemas::internal::{AdCandidate, UserContext};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCampaign {
    pub id: i64,
    pub advertiser_id: i64,
    pub name: String,
    pub bid_type: i16,
    pub bid_amount: f64,
    pub budget_daily: Option<f64>,
    pub budget_total: Option<f64>,
    pub spent_today: f64,
    pub spent_total: f64,
    pub freq_cap_daily: Option<i16>,
    pub freq_cap_hourly: Option<i16>,
    pub creatives: Vec<CachedCreative>,
    pub targeting_rules: Vec<CachedTargetingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedCreative {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub landing_url: String,
    pub creative_type: i16,
    pub width: Option<i16>,
    pub height: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTargetingRule {
    pub rule_type: String,
    pub rule_value: serde_json::Value,
    pub is_include: bool,
}

pub struct TargetingRetrieval {
    pool: PgPool,
    redis: RedisClient,
    cache_ttl: u64,
}

impl TargetingRetrieval {
    pub fn new(pool: PgPool, redis: RedisClient) -> Self {
        Self {
            pool,
            redis,
            cache_ttl: 300,
        }
    }

    async fn get_active_campaigns(&self) -> Result<Vec<CachedCampaign>, AppError> {
        let cache_key = CacheKeys::active_ads();

        if let Ok(Some(cached)) = self.redis.get_json::<Vec<CachedCampaign>>(cache_key).await {
            return Ok(cached);
        }

        // Query database with graceful fallback if DB is unreachable
        let campaigns: Vec<Campaign> = match sqlx::query_as::<_, Campaign>(
            "SELECT * FROM campaigns WHERE status = 1 LIMIT 1000",
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "Failed to query active campaigns from database: {}. Returning empty.",
                    e
                );
                return Ok(Vec::new());
            }
        };

        if campaigns.is_empty() {
            return Ok(Vec::new());
        }

        let campaign_ids: Vec<i64> = campaigns.iter().map(|c| c.id).collect();

        let creatives: Vec<Creative> = match sqlx::query_as::<_, Creative>(
            "SELECT * FROM creatives WHERE campaign_id = ANY($1) AND status = 1",
        )
        .bind(&campaign_ids)
        .fetch_all(&self.pool)
        .await
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to query creatives: {}", e);
                Vec::new()
            }
        };

        let targeting_rules: Vec<TargetingRule> = match sqlx::query_as::<_, TargetingRule>(
            "SELECT * FROM targeting_rules WHERE campaign_id = ANY($1)",
        )
        .bind(&campaign_ids)
        .fetch_all(&self.pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to query targeting rules: {}", e);
                Vec::new()
            }
        };

        // Group creatives by campaign_id
        let mut creatives_by_campaign: HashMap<i64, Vec<CachedCreative>> = HashMap::new();
        for cr in creatives {
            creatives_by_campaign
                .entry(cr.campaign_id)
                .or_default()
                .push(CachedCreative {
                    id: cr.id,
                    title: cr.title,
                    description: cr.description,
                    image_url: cr.image_url,
                    video_url: cr.video_url,
                    landing_url: cr.landing_url,
                    creative_type: cr.creative_type,
                    width: cr.width,
                    height: cr.height,
                });
        }

        // Group rules by campaign_id
        let mut rules_by_campaign: HashMap<i64, Vec<CachedTargetingRule>> = HashMap::new();
        for tr in targeting_rules {
            rules_by_campaign
                .entry(tr.campaign_id)
                .or_default()
                .push(CachedTargetingRule {
                    rule_type: tr.rule_type,
                    rule_value: tr.rule_value,
                    is_include: tr.is_include,
                });
        }

        use bigdecimal::ToPrimitive;
        let mut active_list = Vec::new();
        for camp in campaigns {
            let crs = creatives_by_campaign.remove(&camp.id).unwrap_or_default();
            if crs.is_empty() {
                continue;
            }

            let rules = rules_by_campaign.remove(&camp.id).unwrap_or_default();
            let bid_amt = camp.bid_amount.to_f64().unwrap_or(1.0);
            let b_daily = camp.budget_daily.and_then(|b| b.to_f64());
            let b_total = camp.budget_total.and_then(|b| b.to_f64());
            let s_today = camp.spent_today.to_f64().unwrap_or(0.0);
            let s_total = camp.spent_total.to_f64().unwrap_or(0.0);

            active_list.push(CachedCampaign {
                id: camp.id,
                advertiser_id: camp.advertiser_id,
                name: camp.name,
                bid_type: camp.bid_type,
                bid_amount: bid_amt,
                budget_daily: b_daily,
                budget_total: b_total,
                spent_today: s_today,
                spent_total: s_total,
                freq_cap_daily: camp.freq_cap_daily,
                freq_cap_hourly: camp.freq_cap_hourly,
                creatives: crs,
                targeting_rules: rules,
            });
        }

        if !active_list.is_empty() {
            let _ = self
                .redis
                .set_json(cache_key, &active_list, Some(self.cache_ttl))
                .await;
        }

        Ok(active_list)
    }

    pub fn match_targeting(&self, campaign: &CachedCampaign, user_context: &UserContext) -> bool {
        if campaign.targeting_rules.is_empty() {
            return true;
        }

        for rule in &campaign.targeting_rules {
            let matched = self.match_rule(&rule.rule_type, &rule.rule_value, user_context);
            if rule.is_include && !matched {
                return false;
            }
            if !rule.is_include && matched {
                return false;
            }
        }

        true
    }

    fn match_rule(
        &self,
        rule_type: &str,
        rule_value: &serde_json::Value,
        user_context: &UserContext,
    ) -> bool {
        match rule_type {
            "age" => {
                if let Some(user_age) = user_context.age {
                    let min_age =
                        rule_value.get("min").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let max_age = rule_value
                        .get("max")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(999) as i32;
                    user_age >= min_age && user_age <= max_age
                } else {
                    true
                }
            }
            "gender" => {
                if let Some(user_gender) = &user_context.gender {
                    if let Some(values) = rule_value.get("values").and_then(|v| v.as_array()) {
                        let ug_lower = user_gender.to_lowercase();
                        values
                            .iter()
                            .filter_map(|v| v.as_str())
                            .any(|v| v.to_lowercase() == ug_lower)
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            "geo" => {
                let mut matches = true;
                if let Some(countries) = rule_value.get("countries").and_then(|v| v.as_array()) {
                    if !user_context.country.is_empty() {
                        let uc_upper = user_context.country.to_uppercase();
                        let country_match = countries
                            .iter()
                            .filter_map(|c| c.as_str())
                            .any(|c| c.to_uppercase() == uc_upper);
                        if !country_match {
                            matches = false;
                        }
                    }
                }

                if matches {
                    if let Some(cities) = rule_value.get("cities").and_then(|v| v.as_array()) {
                        if !user_context.city.is_empty() {
                            let ucity_lower = user_context.city.to_lowercase();
                            let city_match = cities
                                .iter()
                                .filter_map(|c| c.as_str())
                                .any(|c| c.to_lowercase() == ucity_lower);
                            if !city_match {
                                matches = false;
                            }
                        }
                    }
                }

                matches
            }
            "os" => {
                if let Some(os_list) = rule_value.get("values").and_then(|v| v.as_array()) {
                    if !user_context.os.is_empty() {
                        let uos_lower = user_context.os.to_lowercase();
                        os_list
                            .iter()
                            .filter_map(|v| v.as_str())
                            .any(|v| v.to_lowercase() == uos_lower)
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            "interest" => {
                if let Some(interests) = rule_value.get("values").and_then(|v| v.as_array()) {
                    if !user_context.interests.is_empty() {
                        let user_interests_lower: Vec<String> = user_context
                            .interests
                            .iter()
                            .map(|i| i.to_lowercase())
                            .collect();
                        interests
                            .iter()
                            .filter_map(|v| v.as_str())
                            .any(|t| user_interests_lower.contains(&t.to_lowercase()))
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            "app_category" => {
                if let Some(cats) = rule_value.get("values").and_then(|v| v.as_array()) {
                    if !user_context.app_categories.is_empty() {
                        let user_cats_lower: Vec<String> = user_context
                            .app_categories
                            .iter()
                            .map(|c| c.to_lowercase())
                            .collect();
                        cats.iter()
                            .filter_map(|v| v.as_str())
                            .any(|c| user_cats_lower.contains(&c.to_lowercase()))
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            _ => true,
        }
    }
}

#[async_trait]
impl Retrieval for TargetingRetrieval {
    async fn retrieve(
        &self,
        user_context: &UserContext,
        _slot_id: &str,
        limit: usize,
    ) -> Result<Vec<AdCandidate>, AppError> {
        let campaigns = self.get_active_campaigns().await?;
        if campaigns.is_empty() {
            debug!("No active campaigns found in targeting retrieval");
            return Ok(Vec::new());
        }

        let mut candidates = Vec::new();

        for camp in &campaigns {
            if !self.match_targeting(camp, user_context) {
                continue;
            }

            for cr in &camp.creatives {
                let candidate = AdCandidate {
                    campaign_id: camp.id,
                    creative_id: cr.id,
                    advertiser_id: camp.advertiser_id,
                    bid: camp.bid_amount,
                    bid_type: camp.bid_type,
                    targeting_score: 1.0,
                    pctr: 0.0,
                    pcvr: 0.0,
                    ecpm: 0.0,
                    score: 0.0,
                    title: cr.title.clone(),
                    description: cr.description.clone(),
                    image_url: cr.image_url.clone(),
                    video_url: cr.video_url.clone(),
                    landing_url: cr.landing_url.clone(),
                    creative_type: cr.creative_type,
                    width: cr.width,
                    height: cr.height,
                    metadata: HashMap::new(),
                };

                candidates.push(candidate);
                if candidates.len() >= limit {
                    break;
                }
            }

            if candidates.len() >= limit {
                break;
            }
        }

        debug!(
            "Targeting retrieval produced {} candidates",
            candidates.len()
        );
        Ok(candidates)
    }

    async fn refresh(&self) -> Result<(), AppError> {
        let cache_key = CacheKeys::active_ads();
        self.redis.delete(cache_key).await?;
        info!("Targeting retrieval cache cleared");
        Ok(())
    }
}
