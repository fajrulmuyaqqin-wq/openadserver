use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, TextEncoder,
};
use std::sync::LazyLock;

pub static HTTP_REQUESTS_TOTAL: LazyLock<CounterVec> = LazyLock::new(|| {
    register_counter_vec!(
        "liteads_http_requests_total",
        "Total HTTP requests",
        &["method", "endpoint", "status"]
    )
    .expect("metric can be created")
});

pub static HTTP_REQUEST_DURATION: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        "liteads_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5]
    )
    .expect("metric can be created")
});

pub static AD_REQUESTS_TOTAL: LazyLock<CounterVec> = LazyLock::new(|| {
    register_counter_vec!(
        "liteads_ad_requests_total",
        "Total ad requests",
        &["slot_id", "status"]
    )
    .expect("metric can be created")
});

pub static AD_IMPRESSIONS_TOTAL: LazyLock<CounterVec> = LazyLock::new(|| {
    register_counter_vec!(
        "liteads_ad_impressions_total",
        "Total ad impressions",
        &["campaign_id"]
    )
    .expect("metric can be created")
});

pub static AD_CLICKS_TOTAL: LazyLock<CounterVec> = LazyLock::new(|| {
    register_counter_vec!(
        "liteads_ad_clicks_total",
        "Total ad clicks",
        &["campaign_id"]
    )
    .expect("metric can be created")
});

pub static AD_CONVERSIONS_TOTAL: LazyLock<CounterVec> = LazyLock::new(|| {
    register_counter_vec!(
        "liteads_ad_conversions_total",
        "Total ad conversions",
        &["campaign_id"]
    )
    .expect("metric can be created")
});

pub fn render_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap_or_default()
}
