use crate::common::cache::{CacheKeys, RedisClient};
use crate::common::errors::AppError;
use crate::common::utils::{current_date, current_hour};
use crate::models::EventType;
use chrono::{DateTime, TimeZone, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, warn};

pub struct EventService {
    pool: PgPool,
    redis: RedisClient,
}

impl EventService {
    pub fn new(pool: PgPool, redis: RedisClient) -> Self {
        Self { pool, redis }
    }

    pub async fn track_event(
        &self,
        request_id: &str,
        ad_id: &str,
        event_type_str: &str,
        user_id: Option<&str>,
        timestamp: Option<i64>,
        _extra: Option<&HashMap<String, serde_json::Value>>,
    ) -> Result<bool, AppError> {
        let (campaign_id, creative_id) = self.parse_ad_id(ad_id);

        let event_type = match EventType::from_str_name(event_type_str) {
            Some(t) => t,
            None => {
                warn!("Unknown event type: {}", event_type_str);
                return Ok(false);
            }
        };

        let event_time: DateTime<Utc> = match timestamp {
            Some(ts) => Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now),
            None => Utc::now(),
        };

        // 1. Insert into PostgreSQL
        let cost = bigdecimal::BigDecimal::from(0);
        let res = sqlx::query(
            r#"
            INSERT INTO ad_events (request_id, campaign_id, creative_id, event_type, event_time, user_id, cost)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(request_id)
        .bind(campaign_id)
        .bind(creative_id)
        .bind(event_type as i16)
        .bind(event_time)
        .bind(user_id)
        .bind(cost)
        .execute(&self.pool)
        .await;

        if let Err(e) = res {
            error!("Failed to insert ad_event into database: {}", e);
            // We still continue to update real-time redis stats
        }

        // 2. Update real-time stats in Redis
        if let Some(camp_id) = campaign_id {
            self.update_stats(camp_id, event_type).await;

            // 3. Update frequency counter for impressions
            if event_type == EventType::Impression {
                if let Some(uid) = user_id {
                    self.update_frequency(uid, camp_id).await;
                }
            }
        }

        info!(
            request_id = %request_id,
            ad_id = %ad_id,
            event_type = %event_type_str,
            "Event tracked successfully"
        );

        Ok(true)
    }

    fn parse_ad_id(&self, ad_id: &str) -> (Option<i64>, Option<i64>) {
        let parts: Vec<&str> = ad_id.split('_').collect();
        if parts.len() >= 3 {
            let camp = parts[1].parse::<i64>().ok();
            let cr = parts[2].parse::<i64>().ok();
            (camp, cr)
        } else if parts.len() >= 2 {
            let camp = parts[1].parse::<i64>().ok();
            (camp, None)
        } else {
            let camp = ad_id.parse::<i64>().ok();
            (camp, None)
        }
    }

    async fn update_stats(&self, campaign_id: i64, event_type: EventType) {
        let hour = current_hour();
        let key = CacheKeys::stat_hourly(campaign_id, &hour);

        let field = match event_type {
            EventType::Impression => "impressions",
            EventType::Click => "clicks",
            EventType::Conversion => "conversions",
        };

        let _ = self.redis.hincrby(&key, field, 1).await;
        let _ = self.redis.expire(&key, 48 * 3600).await;
    }

    async fn update_frequency(&self, user_id: &str, campaign_id: i64) {
        let today = current_date();
        let hour = current_hour();

        let daily_key = CacheKeys::freq_daily(user_id, campaign_id, &today);
        let _ = self.redis.incr(&daily_key, 1).await;
        let _ = self.redis.expire(&daily_key, 24 * 3600).await;

        let hourly_key = CacheKeys::freq_hourly(user_id, campaign_id, &hour);
        let _ = self.redis.incr(&hourly_key, 1).await;
        let _ = self.redis.expire(&hourly_key, 3600).await;
    }
}
