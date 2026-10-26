use crate::ad_server::middleware::metrics::render_metrics;
use crate::common::database::check_health;
use crate::schemas::response::HealthResponse;
use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use serde_json::json;
use std::sync::Arc;

pub fn router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ping", get(ping_handler))
        .route("/ready", get(ready_handler))
        .route("/live", get(live_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

async fn health_handler(State(state): State<Arc<crate::AppState>>) -> Json<HealthResponse> {
    let db_healthy = check_health(&state.pool).await;
    let redis_healthy = state.redis.health_check().await;

    let status = if db_healthy && redis_healthy {
        "healthy".to_string()
    } else {
        "degraded".to_string()
    };

    Json(HealthResponse {
        status,
        version: state.settings.app.version.clone(),
        database: db_healthy,
        redis: redis_healthy,
    })
}

async fn ping_handler() -> Json<serde_json::Value> {
    Json(json!({ "pong": true }))
}

async fn ready_handler(State(state): State<Arc<crate::AppState>>) -> Json<serde_json::Value> {
    let db_healthy = check_health(&state.pool).await;
    let redis_healthy = state.redis.health_check().await;

    if !db_healthy || !redis_healthy {
        Json(json!({ "ready": false, "reason": "Dependencies not ready" }))
    } else {
        Json(json!({ "ready": true }))
    }
}

async fn live_handler() -> Json<serde_json::Value> {
    Json(json!({ "alive": true }))
}

async fn metrics_handler() -> Response {
    let body = render_metrics();
    (
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
        .into_response()
}
