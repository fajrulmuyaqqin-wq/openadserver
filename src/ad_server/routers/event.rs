use crate::ad_server::middleware::metrics::{
    AD_CLICKS_TOTAL, AD_CONVERSIONS_TOTAL, AD_IMPRESSIONS_TOTAL,
};
use crate::common::errors::AppError;
use crate::common::utils::current_timestamp;
use crate::models::EventType;
use crate::schemas::request::EventRequest;
use crate::schemas::response::EventResponse;
use axum::{
    extract::{Query, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

pub fn router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/track", post(track_event_post))
        .route("/track", get(track_event_get))
        .with_state(state)
}

async fn track_event_post(
    State(state): State<Arc<crate::AppState>>,
    Json(event): Json<EventRequest>,
) -> Result<Json<EventResponse>, AppError> {
    info!(
        request_id = %event.request_id,
        ad_id = %event.ad_id,
        event_type = %event.event_type,
        "Event received via POST"
    );

    let timestamp = event.timestamp.unwrap_or_else(current_timestamp);

    let success = state
        .event_service
        .track_event(
            &event.request_id,
            &event.ad_id,
            &event.event_type,
            event.user_id.as_deref(),
            Some(timestamp),
            event.extra.as_ref(),
        )
        .await?;

    if success {
        update_event_metrics(&event.ad_id, &event.event_type);
    }

    Ok(Json(EventResponse {
        success,
        message: Some(if success {
            "Event recorded".to_string()
        } else {
            "Failed to record event".to_string()
        }),
    }))
}

#[derive(Debug, Deserialize)]
pub struct TrackQuery {
    #[serde(rename = "type", alias = "event_type")]
    pub event_type: String,
    #[serde(alias = "request_id")]
    pub req: String,
    #[serde(alias = "ad_id")]
    pub ad: String,
}

async fn track_event_get(
    State(state): State<Arc<crate::AppState>>,
    Query(query): Query<TrackQuery>,
) -> Result<Json<EventResponse>, AppError> {
    info!(
        request_id = %query.req,
        ad_id = %query.ad,
        event_type = %query.event_type,
        "Pixel event received via GET"
    );

    let timestamp = current_timestamp();

    let success = state
        .event_service
        .track_event(
            &query.req,
            &query.ad,
            &query.event_type,
            None,
            Some(timestamp),
            None,
        )
        .await?;

    if success {
        update_event_metrics(&query.ad, &query.event_type);
    }

    Ok(Json(EventResponse {
        success,
        message: Some(if success {
            "Event recorded".to_string()
        } else {
            "Failed to record event".to_string()
        }),
    }))
}

fn update_event_metrics(ad_id: &str, event_type_str: &str) {
    if let Some(event_type) = EventType::from_str_name(event_type_str) {
        match event_type {
            EventType::Impression => {
                AD_IMPRESSIONS_TOTAL.with_label_values(&[ad_id]).inc();
            }
            EventType::Click => {
                AD_CLICKS_TOTAL.with_label_values(&[ad_id]).inc();
            }
            EventType::Conversion => {
                AD_CONVERSIONS_TOTAL.with_label_values(&[ad_id]).inc();
            }
        }
    }
}
