use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    pub landing_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i16>,
    pub creative_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingUrls {
    pub impression_url: String,
    pub click_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversion_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdResponse {
    pub ad_id: String,
    pub campaign_id: i64,
    pub creative_id: i64,
    pub creative: CreativeResponse,
    pub tracking: TrackingUrls,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdListResponse {
    pub request_id: String,
    pub ads: Vec<AdResponse>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: bool,
    pub redis: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}
