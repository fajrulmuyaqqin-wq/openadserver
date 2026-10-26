use crate::ad_server::middleware::metrics::{
    AD_REQUESTS_TOTAL, HTTP_REQUESTS_TOTAL, HTTP_REQUEST_DURATION,
};
use crate::common::errors::AppError;
use crate::common::utils::generate_request_id;
use crate::models::CreativeType;
use crate::schemas::request::AdRequest;
use crate::schemas::response::{AdListResponse, AdResponse, CreativeResponse, TrackingUrls};
use axum::{extract::State, http::HeaderMap, response::Json, routing::post, Router};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

pub fn router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/request", post(request_ads_handler))
        .with_state(state)
}

async fn request_ads_handler(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
    Json(mut ad_request): Json<AdRequest>,
) -> Result<Json<AdListResponse>, AppError> {
    let start = Instant::now();
    let request_id = generate_request_id();

    // Determine client IP
    if ad_request.geo.is_none() {
        ad_request.geo = Some(Default::default());
    }
    if let Some(geo) = ad_request.geo.as_mut() {
        if geo.ip.is_none() {
            let client_ip = headers
                .get("x-forwarded-for")
                .or_else(|| headers.get("x-real-ip"))
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string());
            geo.ip = client_ip;
        }
    }

    info!(
        request_id = %request_id,
        slot_id = %ad_request.slot_id,
        user_id = ?ad_request.user_id,
        num_ads = ad_request.num_ads,
        "Ad request received"
    );

    // Call ad service
    let (candidates, _metrics) = state.ad_service.serve_ads(&ad_request, &request_id).await?;

    let host_header = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:8000");
    let scheme = if headers.contains_key("x-forwarded-proto") {
        headers
            .get("x-forwarded-proto")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("http")
    } else {
        "http"
    };
    let base_url = format!("{}://{}", scheme, host_header);

    let mut ads = Vec::new();
    for candidate in candidates.into_iter().take(ad_request.num_ads) {
        let tracking = TrackingUrls {
            impression_url: format!(
                "{}/api/v1/event/track?type=impression&req={}&ad={}",
                base_url, request_id, candidate.campaign_id
            ),
            click_url: format!(
                "{}/api/v1/event/track?type=click&req={}&ad={}",
                base_url, request_id, candidate.campaign_id
            ),
            conversion_url: Some(format!(
                "{}/api/v1/event/track?type=conversion&req={}&ad={}",
                base_url, request_id, candidate.campaign_id
            )),
        };

        let creative_type_str = CreativeType::from(candidate.creative_type)
            .as_str()
            .to_string();

        let creative = CreativeResponse {
            title: candidate.title,
            description: candidate.description,
            image_url: candidate.image_url,
            video_url: candidate.video_url,
            landing_url: candidate.landing_url,
            width: candidate.width,
            height: candidate.height,
            creative_type: creative_type_str,
        };

        let metadata = if state.settings.app.debug {
            let mut meta = HashMap::new();
            meta.insert(
                "ecpm".to_string(),
                serde_json::Value::from((candidate.ecpm * 10000.0).round() / 10000.0),
            );
            meta.insert(
                "pctr".to_string(),
                serde_json::Value::from((candidate.pctr * 1000000.0).round() / 1000000.0),
            );
            Some(meta)
        } else {
            None
        };

        let ad = AdResponse {
            ad_id: format!("ad_{}_{}", candidate.campaign_id, candidate.creative_id),
            campaign_id: candidate.campaign_id,
            creative_id: candidate.creative_id,
            creative,
            tracking,
            metadata,
        };
        ads.push(ad);
    }

    let count = ads.len();

    // Update Prometheus metrics
    AD_REQUESTS_TOTAL
        .with_label_values(&[&ad_request.slot_id, "success"])
        .inc();
    HTTP_REQUESTS_TOTAL
        .with_label_values(&["POST", "/api/v1/ad/request", "200"])
        .inc();
    HTTP_REQUEST_DURATION
        .with_label_values(&["POST", "/api/v1/ad/request"])
        .observe(start.elapsed().as_secs_f64());

    Ok(Json(AdListResponse {
        request_id,
        ads,
        count,
    }))
}
