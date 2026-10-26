use crate::models::{Campaign, Creative, TargetingRule};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdvertiserRequest {
    pub name: String,
    pub company: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub balance: Option<f64>,
    pub credit_limit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdvertiserRequest {
    pub name: Option<String>,
    pub company: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub balance: Option<f64>,
    pub credit_limit: Option<f64>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetingRuleDto {
    pub rule_type: String,
    pub rule_value: serde_json::Value,
    #[serde(default = "default_true")]
    pub is_include: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignRequest {
    pub advertiser_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub budget_daily: Option<f64>,
    pub budget_total: Option<f64>,
    #[serde(default = "default_bid_type")]
    pub bid_type: i16,
    pub bid_amount: f64,
    pub freq_cap_daily: Option<i16>,
    pub freq_cap_hourly: Option<i16>,
    pub targeting_rules: Option<Vec<TargetingRuleDto>>,
}

fn default_bid_type() -> i16 {
    1 // Default CPM
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCampaignRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub budget_daily: Option<f64>,
    pub budget_total: Option<f64>,
    pub bid_type: Option<i16>,
    pub bid_amount: Option<f64>,
    pub freq_cap_daily: Option<i16>,
    pub freq_cap_hourly: Option<i16>,
    pub status: Option<i16>,
    pub targeting_rules: Option<Vec<TargetingRuleDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCreativeRequest {
    pub campaign_id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub landing_url: String,
    #[serde(default = "default_creative_type")]
    pub creative_type: i16,
    pub width: Option<i16>,
    pub height: Option<i16>,
}

fn default_creative_type() -> i16 {
    1 // Default Banner
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCreativeRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub landing_url: Option<String>,
    pub creative_type: Option<i16>,
    pub width: Option<i16>,
    pub height: Option<i16>,
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignDetailResponse {
    pub campaign: Campaign,
    pub creatives: Vec<Creative>,
    pub targeting_rules: Vec<TargetingRule>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<i16>,
    pub advertiser_id: Option<i64>,
    pub campaign_id: Option<i64>,
}
