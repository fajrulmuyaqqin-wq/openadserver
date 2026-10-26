pub mod ad_server;
pub mod common;
pub mod ml_engine;
pub mod models;
pub mod rec_engine;
pub mod schemas;

use ad_server::services::{AdService, EventService};
use axum::{
    body::Body,
    http::{header::HeaderName, HeaderValue, Request, Response},
    middleware::{self, Next},
    Router,
};
use common::cache::RedisClient;
use common::config::Settings;
use common::utils::generate_request_id;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Instant;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: RedisClient,
    pub ad_service: Arc<AdService>,
    pub event_service: Arc<EventService>,
    pub settings: Settings,
}

pub fn create_app(state: Arc<AppState>) -> Router {
    let health_router = ad_server::routers::health::router(state.clone());
    let ad_router = ad_server::routers::ad::router(state.clone());
    let event_router = ad_server::routers::event::router(state.clone());
    let admin_router = ad_server::routers::admin::router(state.clone());
    let vast_router = ad_server::routers::vast::router(state.clone());
    let openrtb_router = ad_server::routers::openrtb::router(state.clone());

    Router::new()
        .merge(health_router)
        .nest("/api/v1/ad", ad_router)
        .nest("/api/v1/ad", vast_router)
        .nest("/api/v1/event", event_router)
        .nest("/api/v1", admin_router)
        .nest("/api/v1/openrtb2", openrtb_router)
        .layer(middleware::from_fn(request_timing_middleware))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn request_timing_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    let start = Instant::now();
    let req_id = generate_request_id();
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    let mut res = next.run(req).await;

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    info!(
        method = %method,
        path = %path,
        status = res.status().as_u16(),
        duration_ms = duration_ms,
        request_id = %req_id,
        "HTTP request completed"
    );

    if let Ok(hdr_val) = HeaderValue::from_str(&req_id) {
        res.headers_mut()
            .insert(HeaderName::from_static("x-request-id"), hdr_val);
    }
    if let Ok(hdr_val) = HeaderValue::from_str(&format!("{:.2}ms", duration_ms)) {
        res.headers_mut()
            .insert(HeaderName::from_static("x-response-time"), hdr_val);
    }

    res
}
