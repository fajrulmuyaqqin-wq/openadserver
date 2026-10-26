use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_app_version")]
    pub version: String,
    #[serde(default)]
    pub debug: bool,
    #[serde(default = "default_env")]
    pub env: String,
}

fn default_app_name() -> String {
    "OpenAdServer".to_string()
}
fn default_app_version() -> String {
    "0.1.0".to_string()
}
fn default_env() -> String {
    "dev".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: default_app_version(),
            debug: false,
            env: default_env(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default)]
    pub reload: bool,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_port() -> u16 {
    8000
}
fn default_workers() -> usize {
    4
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: default_workers(),
            reload: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_host")]
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    #[serde(default = "default_db_name")]
    pub name: String,
    #[serde(default = "default_db_user")]
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_max_overflow")]
    pub max_overflow: u32,
}

fn default_db_host() -> String {
    "localhost".to_string()
}
fn default_db_port() -> u16 {
    5432
}
fn default_db_name() -> String {
    "liteads".to_string()
}
fn default_db_user() -> String {
    "liteads".to_string()
}
fn default_pool_size() -> u32 {
    10
}
fn default_max_overflow() -> u32 {
    20
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: default_db_host(),
            port: default_db_port(),
            name: default_db_name(),
            user: default_db_user(),
            password: "".to_string(),
            pool_size: default_pool_size(),
            max_overflow: default_max_overflow(),
        }
    }
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        if self.password.is_empty() {
            format!(
                "postgres://{}@{}:{}/{}",
                self.user, self.host, self.port, self.name
            )
        } else {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.name
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_host")]
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    #[serde(default)]
    pub db: i64,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

fn default_redis_host() -> String {
    "localhost".to_string()
}
fn default_redis_port() -> u16 {
    6379
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: default_redis_host(),
            port: default_redis_port(),
            db: 0,
            password: "".to_string(),
            pool_size: default_pool_size(),
        }
    }
}

impl RedisConfig {
    pub fn connection_string(&self) -> String {
        if self.password.is_empty() {
            format!("redis://{}:{}/{}", self.host, self.port, self.db)
        } else {
            format!(
                "redis://:{}@{}:{}/{}",
                self.password, self.host, self.port, self.db
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdServingConfig {
    #[serde(default = "default_num_ads")]
    pub default_num_ads: usize,
    #[serde(default = "default_max_ads")]
    pub max_num_ads: usize,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_true")]
    pub enable_ml_prediction: bool,
    #[serde(default)]
    pub model_path: String,
    #[serde(default = "default_pctr")]
    pub default_pctr: f64,
    #[serde(default = "default_pcvr")]
    pub default_pcvr: f64,
}

fn default_num_ads() -> usize {
    1
}
fn default_max_ads() -> usize {
    10
}
fn default_timeout() -> u64 {
    50
}
fn default_true() -> bool {
    true
}
fn default_pctr() -> f64 {
    0.01
}
fn default_pcvr() -> f64 {
    0.001
}

impl Default for AdServingConfig {
    fn default() -> Self {
        Self {
            default_num_ads: default_num_ads(),
            max_num_ads: default_max_ads(),
            timeout_ms: default_timeout(),
            enable_ml_prediction: true,
            model_path: "".to_string(),
            default_pctr: default_pctr(),
            default_pcvr: default_pcvr(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyConfig {
    #[serde(default = "default_daily_cap")]
    pub default_daily_cap: i32,
    #[serde(default = "default_hourly_cap")]
    pub default_hourly_cap: i32,
    #[serde(default = "default_ttl_hours")]
    pub ttl_hours: u32,
}

fn default_daily_cap() -> i32 {
    3
}
fn default_hourly_cap() -> i32 {
    1
}
fn default_ttl_hours() -> u32 {
    24
}

impl Default for FrequencyConfig {
    fn default() -> Self {
        Self {
            default_daily_cap: default_daily_cap(),
            default_hourly_cap: default_hourly_cap(),
            ttl_hours: default_ttl_hours(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    #[serde(default = "default_model_dir")]
    pub model_dir: String,
    #[serde(default = "default_ctr_model")]
    pub ctr_model: String,
    #[serde(default = "default_cvr_model")]
    pub cvr_model: String,
    #[serde(default = "default_embedding_dim")]
    pub embedding_dim: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_model_dir() -> String {
    "./models".to_string()
}
fn default_ctr_model() -> String {
    "deepfm_v1".to_string()
}
fn default_cvr_model() -> String {
    "deepfm_cvr_v1".to_string()
}
fn default_embedding_dim() -> usize {
    8
}
fn default_batch_size() -> usize {
    128
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            model_dir: default_model_dir(),
            ctr_model: default_ctr_model(),
            cvr_model: default_cvr_model(),
            embedding_dim: default_embedding_dim(),
            batch_size: default_batch_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_level() -> String {
    "INFO".to_string()
}
fn default_log_format() -> String {
    "console".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_prometheus_port")]
    pub prometheus_port: u16,
}

fn default_prometheus_port() -> u16 {
    9090
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus_port: default_prometheus_port(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub ad_serving: AdServingConfig,
    #[serde(default)]
    pub frequency: FrequencyConfig,
    #[serde(default)]
    pub ml: MLConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

impl Settings {
    pub fn load() -> Self {
        let mut settings = Self::default();

        // 1. Try to load base.yaml
        let base_path = Path::new("configs/base.yaml");
        if base_path.exists() {
            if let Ok(content) = std::fs::read_to_string(base_path) {
                if let Ok(loaded) = serde_yaml::from_str::<Settings>(&content) {
                    settings = loaded;
                }
            }
        }

        // Determine environment
        let env_mode = env::var("LITEADS_ENV")
            .or_else(|_| env::var("OPENADSERVER_ENV"))
            .unwrap_or_else(|_| settings.app.env.clone());
        settings.app.env = env_mode.clone();

        // 2. Try to load environment-specific config (dev.yaml, prod.yaml)
        let env_config_path = format!("configs/{}.yaml", env_mode);
        let path = Path::new(&env_config_path);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(override_settings) = serde_yaml::from_str::<serde_json::Value>(&content) {
                    // Merge via JSON
                    if let Ok(mut current_val) = serde_json::to_value(&settings) {
                        merge_json(&mut current_val, &override_settings);
                        if let Ok(merged) = serde_json::from_value::<Settings>(current_val) {
                            settings = merged;
                        }
                    }
                }
            }
        }

        // 3. Environment variable overrides
        if let Ok(val) = env::var("LITEADS_DATABASE__HOST")
            .or_else(|_| env::var("OPENADSERVER_DATABASE__HOST"))
            .or_else(|_| env::var("DB_HOST"))
        {
            settings.database.host = val;
        }
        if let Ok(val) = env::var("LITEADS_DATABASE__PORT")
            .or_else(|_| env::var("OPENADSERVER_DATABASE__PORT"))
            .or_else(|_| env::var("DB_PORT"))
        {
            if let Ok(port) = val.parse::<u16>() {
                settings.database.port = port;
            }
        }
        if let Ok(val) = env::var("LITEADS_DATABASE__NAME")
            .or_else(|_| env::var("OPENADSERVER_DATABASE__NAME"))
            .or_else(|_| env::var("DB_NAME"))
        {
            settings.database.name = val;
        }
        if let Ok(val) = env::var("LITEADS_DATABASE__USER")
            .or_else(|_| env::var("OPENADSERVER_DATABASE__USER"))
            .or_else(|_| env::var("DB_USER"))
        {
            settings.database.user = val;
        }
        if let Ok(val) = env::var("LITEADS_DATABASE__PASSWORD")
            .or_else(|_| env::var("OPENADSERVER_DATABASE__PASSWORD"))
            .or_else(|_| env::var("DB_PASSWORD"))
        {
            settings.database.password = val;
        }

        if let Ok(val) = env::var("LITEADS_REDIS__HOST")
            .or_else(|_| env::var("OPENADSERVER_REDIS__HOST"))
            .or_else(|_| env::var("REDIS_HOST"))
        {
            settings.redis.host = val;
        }
        if let Ok(val) = env::var("LITEADS_REDIS__PORT")
            .or_else(|_| env::var("OPENADSERVER_REDIS__PORT"))
            .or_else(|_| env::var("REDIS_PORT"))
        {
            if let Ok(port) = val.parse::<u16>() {
                settings.redis.port = port;
            }
        }
        if let Ok(val) = env::var("LITEADS_REDIS__PASSWORD")
            .or_else(|_| env::var("OPENADSERVER_REDIS__PASSWORD"))
            .or_else(|_| env::var("REDIS_PASSWORD"))
        {
            settings.redis.password = val;
        }

        if let Ok(val) = env::var("PORT").or_else(|_| env::var("SERVER_PORT")) {
            if let Ok(port) = val.parse::<u16>() {
                settings.server.port = port;
            }
        }

        settings
    }
}

fn merge_json(a: &mut serde_json::Value, b: &serde_json::Value) {
    match (a, b) {
        (serde_json::Value::Object(a_map), serde_json::Value::Object(b_map)) => {
            for (k, v) in b_map {
                merge_json(a_map.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a_val, b_val) => {
            *a_val = b_val.clone();
        }
    }
}
