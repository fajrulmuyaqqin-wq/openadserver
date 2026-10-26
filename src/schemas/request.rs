use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub os: String,
    pub os_version: Option<String>,
    pub model: Option<String>,
    pub brand: Option<String>,
    pub screen_width: Option<i32>,
    pub screen_height: Option<i32>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeoInfo {
    pub ip: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextInfo {
    pub app_id: Option<String>,
    pub app_name: Option<String>,
    pub app_version: Option<String>,
    pub app_bundle: Option<String>,
    pub carrier: Option<String>,
    pub network: Option<String>,
    pub connection_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserFeatures {
    pub age: Option<i32>,
    pub gender: Option<String>,
    pub interests: Option<Vec<String>>,
    pub app_categories: Option<Vec<String>>,
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdRequest {
    pub slot_id: String,
    pub user_id: Option<String>,
    pub device: DeviceInfo,
    pub geo: Option<GeoInfo>,
    pub context: Option<ContextInfo>,
    pub user_features: Option<UserFeatures>,
    #[serde(default = "default_num_ads")]
    pub num_ads: usize,
}

fn default_num_ads() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRequest {
    pub request_id: String,
    pub ad_id: String,
    pub event_type: String,
    pub timestamp: Option<i64>,
    pub user_id: Option<String>,
    pub extra: Option<HashMap<String, serde_json::Value>>,
}
