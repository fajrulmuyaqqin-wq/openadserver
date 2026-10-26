use chrono::Utc;
use md5::{Digest, Md5};
use rand::Rng;
use uuid::Uuid;

pub fn generate_request_id() -> String {
    format!("req_{}", Uuid::new_v4().simple())
}

pub fn generate_id() -> i64 {
    let timestamp = Utc::now().timestamp_millis();
    let random_bits = rand::thread_rng().gen_range(0..=0xFFFF) as i64;
    (timestamp << 16) | random_bits
}

pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

pub fn current_timestamp_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn current_date() -> String {
    Utc::now().format("%Y%m%d").to_string()
}

pub fn current_hour() -> String {
    Utc::now().format("%Y%m%d%H").to_string()
}

pub fn hash_string(s: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn hash_user_id(user_id: &str) -> u64 {
    let hex_hash = hash_string(user_id);
    let slice = if hex_hash.len() >= 8 {
        &hex_hash[..8]
    } else {
        &hex_hash
    };
    u64::from_str_radix(slice, 16).unwrap_or(0)
}

pub fn safe_divide(numerator: f64, denominator: f64, default_val: f64) -> f64 {
    if denominator == 0.0 || denominator.is_nan() {
        default_val
    } else {
        numerator / denominator
    }
}

pub fn clamp(val: f64, min_val: f64, max_val: f64) -> f64 {
    val.max(min_val).min(max_val)
}

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_user_id() {
        let h1 = hash_user_id("user_123");
        let h2 = hash_user_id("user_123");
        assert_eq!(h1, h2);
        assert!(h1 > 0);
    }

    #[test]
    fn test_sigmoid() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-6);
        assert!(sigmoid(100.0) > 0.999);
        assert!(sigmoid(-100.0) < 0.001);
    }

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10.0, 0.0, 5.0), 5.0);
        assert_eq!(safe_divide(10.0, 2.0, 0.0), 5.0);
    }
}
