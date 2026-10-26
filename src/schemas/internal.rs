use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdCandidate {
    pub campaign_id: i64,
    pub creative_id: i64,
    pub advertiser_id: i64,
    pub bid: f64,
    pub bid_type: i16,

    pub targeting_score: f64,
    pub pctr: f64,
    pub pcvr: f64,
    pub ecpm: f64,
    pub score: f64,

    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub landing_url: String,
    pub creative_type: i16,
    pub width: Option<i16>,
    pub height: Option<i16>,

    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for AdCandidate {
    fn default() -> Self {
        Self {
            campaign_id: 0,
            creative_id: 0,
            advertiser_id: 0,
            bid: 1.0,
            bid_type: 1,
            targeting_score: 1.0,
            pctr: 0.0,
            pcvr: 0.0,
            ecpm: 0.0,
            score: 0.0,
            title: None,
            description: None,
            image_url: None,
            video_url: None,
            landing_url: String::new(),
            creative_type: 1,
            width: None,
            height: None,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Option<String>,
    pub user_hash: u64,

    pub os: String,
    pub os_version: String,
    pub device_model: String,
    pub device_brand: String,

    pub ip: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub app_id: String,
    pub app_name: String,
    pub network: String,
    pub carrier: String,

    pub age: Option<i32>,
    pub gender: Option<String>,
    pub interests: Vec<String>,
    pub app_categories: Vec<String>,
    pub custom_features: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub campaign_id: i64,
    pub creative_id: i64,
    pub pctr: f64,
    pub pcvr: f64,
    pub model_version: String,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyInfo {
    pub user_id: String,
    pub campaign_id: i64,
    pub daily_count: i32,
    pub hourly_count: i32,
    pub daily_cap: Option<i32>,
    pub hourly_cap: Option<i32>,
}

impl FrequencyInfo {
    pub fn is_capped(&self) -> bool {
        if let Some(cap) = self.daily_cap {
            if self.daily_count >= cap {
                return true;
            }
        }
        if let Some(cap) = self.hourly_cap {
            if self.hourly_count >= cap {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetInfo {
    pub campaign_id: i64,
    pub budget_daily: Option<f64>,
    pub budget_total: Option<f64>,
    pub spent_today: f64,
    pub spent_total: f64,
}

impl BudgetInfo {
    pub fn remaining_daily(&self) -> Option<f64> {
        self.budget_daily.map(|b| (b - self.spent_today).max(0.0))
    }

    pub fn remaining_total(&self) -> Option<f64> {
        self.budget_total.map(|b| (b - self.spent_total).max(0.0))
    }

    pub fn has_budget(&self) -> bool {
        if let Some(daily) = self.budget_daily {
            if self.spent_today >= daily {
                return false;
            }
        }
        if let Some(total) = self.budget_total {
            if self.spent_total >= total {
                return false;
            }
        }
        true
    }
}
