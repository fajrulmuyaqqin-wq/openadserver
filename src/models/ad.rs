use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Advertiser {
    pub id: i64,
    pub name: String,
    pub company: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub balance: BigDecimal,
    pub credit_limit: BigDecimal,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Campaign {
    pub id: i64,
    pub advertiser_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub budget_daily: Option<BigDecimal>,
    pub budget_total: Option<BigDecimal>,
    pub spent_today: BigDecimal,
    pub spent_total: BigDecimal,
    pub bid_type: i16,
    pub bid_amount: BigDecimal,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub freq_cap_daily: Option<i16>,
    pub freq_cap_hourly: Option<i16>,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Creative {
    pub id: i64,
    pub campaign_id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub landing_url: String,
    pub creative_type: i16,
    pub width: Option<i16>,
    pub height: Option<i16>,
    pub status: i16,
    pub impressions: i64,
    pub clicks: i64,
    pub conversions: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TargetingRule {
    pub id: i64,
    pub campaign_id: i64,
    pub rule_type: String,
    pub rule_value: serde_json::Value,
    pub is_include: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdEvent {
    pub id: i64,
    pub request_id: String,
    pub campaign_id: Option<i64>,
    pub creative_id: Option<i64>,
    pub event_type: i16,
    pub event_time: DateTime<Utc>,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub cost: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HourlyStat {
    pub id: i64,
    pub campaign_id: i64,
    pub creative_id: Option<i64>,
    pub stat_hour: DateTime<Utc>,
    pub impressions: i64,
    pub clicks: i64,
    pub conversions: i64,
    pub cost: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
