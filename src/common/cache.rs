use crate::common::config::RedisConfig;
use crate::common::errors::AppError;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct CacheKeys;

impl CacheKeys {
    pub fn campaign(campaign_id: i64) -> String {
        format!("ad:campaign:{}", campaign_id)
    }

    pub fn campaign_creative(campaign_id: i64, creative_id: i64) -> String {
        format!("ad:creative:{}:{}", campaign_id, creative_id)
    }

    pub fn user_feature(user_id: &str) -> String {
        format!("user:feature:{}", user_id)
    }

    pub fn freq_daily(user_id: &str, campaign_id: i64, date: &str) -> String {
        format!("freq:daily:{}:{}:{}", user_id, campaign_id, date)
    }

    pub fn freq_hourly(user_id: &str, campaign_id: i64, hour: &str) -> String {
        format!("freq:hourly:{}:{}:{}", user_id, campaign_id, hour)
    }

    pub fn stat_hourly(campaign_id: i64, hour: &str) -> String {
        format!("stat:{}:{}", campaign_id, hour)
    }

    pub fn active_ads() -> &'static str {
        "ads:active"
    }

    pub fn budget(campaign_id: i64, date: &str) -> String {
        format!("budget:{}:{}", campaign_id, date)
    }

    pub fn model(model_name: &str, version: &str) -> String {
        format!("model:{}:{}", model_name, version)
    }

    pub fn rate_limit(client_id: &str, endpoint: &str) -> String {
        format!("ratelimit:{}:{}", client_id, endpoint)
    }
}

type InMemoryStore = Arc<RwLock<HashMap<String, (String, Option<i64>)>>>;

#[derive(Clone)]
pub struct RedisClient {
    manager: Option<ConnectionManager>,
    // In-memory fallback if Redis is unavailable or optional
    in_memory: InMemoryStore,
}

impl RedisClient {
    pub fn in_memory() -> Self {
        Self {
            manager: None,
            in_memory: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn new(config: &RedisConfig) -> Result<Self, AppError> {
        let conn_str = config.connection_string();
        let client = redis::Client::open(conn_str.as_str())?;
        match ConnectionManager::new(client).await {
            Ok(mgr) => {
                info!(
                    host = %config.host,
                    port = %config.port,
                    db = %config.db,
                    "Redis connection established"
                );
                Ok(Self {
                    manager: Some(mgr),
                    in_memory: Arc::new(RwLock::new(HashMap::new())),
                })
            }
            Err(e) => {
                warn!(
                    "Failed to connect to Redis: {}. Using in-memory fallback.",
                    e
                );
                Ok(Self {
                    manager: None,
                    in_memory: Arc::new(RwLock::new(HashMap::new())),
                })
            }
        }
    }

    pub async fn health_check(&self) -> bool {
        if let Some(mut mgr) = self.manager.clone() {
            let res: Result<String, _> = redis::cmd("PING").query_async(&mut mgr).await;
            res.is_ok()
        } else {
            true
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let res: Option<String> = mgr.get(key).await?;
            Ok(res)
        } else {
            let map = self.in_memory.read().await;
            Ok(map.get(key).map(|(v, _)| v.clone()))
        }
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            if let Some(ttl) = ttl_secs {
                let _: () = mgr.set_ex(key, value, ttl).await?;
            } else {
                let _: () = mgr.set(key, value).await?;
            }
            Ok(())
        } else {
            let mut map = self.in_memory.write().await;
            let expiry = ttl_secs.map(|s| chrono::Utc::now().timestamp() + s as i64);
            map.insert(key.to_string(), (value.to_string(), expiry));
            Ok(())
        }
    }

    pub async fn delete(&self, key: &str) -> Result<bool, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let count: i64 = mgr.del(key).await?;
            Ok(count > 0)
        } else {
            let mut map = self.in_memory.write().await;
            Ok(map.remove(key).is_some())
        }
    }

    pub async fn expire(&self, key: &str, seconds: i64) -> Result<bool, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let res: bool = mgr.expire(key, seconds).await?;
            Ok(res)
        } else {
            Ok(true)
        }
    }

    pub async fn incr(&self, key: &str, amount: i64) -> Result<i64, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let res: i64 = mgr.incr(key, amount).await?;
            Ok(res)
        } else {
            let mut map = self.in_memory.write().await;
            let current = map
                .get(key)
                .and_then(|(v, _)| v.parse::<i64>().ok())
                .unwrap_or(0);
            let next = current + amount;
            map.insert(key.to_string(), (next.to_string(), None));
            Ok(next)
        }
    }

    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<String>, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let res: Option<String> = mgr.hget(key, field).await?;
            Ok(res)
        } else {
            let subkey = format!("{}:{}", key, field);
            self.get(&subkey).await
        }
    }

    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<(), AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let _: () = mgr.hset(key, field, value).await?;
            Ok(())
        } else {
            let subkey = format!("{}:{}", key, field);
            self.set(&subkey, value, None).await
        }
    }

    pub async fn hincrby(&self, key: &str, field: &str, amount: i64) -> Result<i64, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let res: i64 = mgr.hincr(key, field, amount).await?;
            Ok(res)
        } else {
            let subkey = format!("{}:{}", key, field);
            self.incr(&subkey, amount).await
        }
    }

    pub async fn hgetall(&self, key: &str) -> Result<HashMap<String, String>, AppError> {
        if let Some(mut mgr) = self.manager.clone() {
            let res: HashMap<String, String> = mgr.hgetall(key).await?;
            Ok(res)
        } else {
            Ok(HashMap::new())
        }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
        match self.get(key).await? {
            Some(raw) => {
                let parsed = serde_json::from_str(&raw)?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), AppError> {
        let serialized = serde_json::to_string(value)?;
        self.set(key, &serialized, ttl_secs).await
    }
}
