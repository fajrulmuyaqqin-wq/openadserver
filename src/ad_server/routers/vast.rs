use crate::common::errors::AppError;
use crate::common::utils::generate_request_id;
use crate::schemas::request::{AdRequest, DeviceInfo, GeoInfo};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

pub fn router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/vast", get(vast_handler))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct VastQuery {
    pub slot_id: String,
    pub user_id: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub width: Option<i16>,
    pub height: Option<i16>,
}

async fn vast_handler(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
    Query(query): Query<VastQuery>,
) -> Result<Response<axum::body::Body>, AppError> {
    let request_id = generate_request_id();

    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string());

    let ad_request = AdRequest {
        slot_id: query.slot_id.clone(),
        user_id: query.user_id.clone(),
        device: DeviceInfo {
            os: query.os.unwrap_or_else(|| "android".to_string()),
            os_version: None,
            model: None,
            brand: None,
            screen_width: query.width.map(|w| w as i32),
            screen_height: query.height.map(|h| h as i32),
            language: None,
        },
        geo: Some(GeoInfo {
            ip: client_ip,
            country: query.country,
            region: None,
            city: None,
            latitude: None,
            longitude: None,
        }),
        context: None,
        user_features: None,
        num_ads: 1,
    };

    info!(
        request_id = %request_id,
        slot_id = %query.slot_id,
        "VAST request received"
    );

    let (candidates, _) = state.ad_service.serve_ads(&ad_request, &request_id).await?;

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

    let xml = if let Some(candidate) = candidates.first() {
        let title = candidate.title.as_deref().unwrap_or("Advertisement");
        let desc = candidate.description.as_deref().unwrap_or("");
        let video_url = candidate
            .video_url
            .clone()
            .unwrap_or_else(|| "https://cdn.example.com/sample_ad.mp4".to_string());
        let width = candidate.width.unwrap_or(1920);
        let height = candidate.height.unwrap_or(1080);

        let impression_url = format!(
            "{}/api/v1/event/track?type=impression&req={}&ad={}",
            base_url, request_id, candidate.campaign_id
        );
        let click_url = format!(
            "{}/api/v1/event/track?type=click&req={}&ad={}",
            base_url, request_id, candidate.campaign_id
        );
        let error_url = format!(
            "{}/api/v1/event/track?type=error&req={}&ad={}",
            base_url, request_id, candidate.campaign_id
        );

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="3.0">
  <Ad id="ad_{campaign_id}_{creative_id}">
    <InLine>
      <AdSystem version="1.0">OpenAdServer</AdSystem>
      <AdTitle><![CDATA[{title}]]></AdTitle>
      <Description><![CDATA[{desc}]]></Description>
      <Error><![CDATA[{error_url}]]></Error>
      <Impression><![CDATA[{impression_url}]]></Impression>
      <Creatives>
        <Creative id="cr_{creative_id}" sequence="1">
          <Linear>
            <Duration>00:00:15</Duration>
            <TrackingEvents>
              <Tracking event="start"><![CDATA[{base_url}/api/v1/event/track?type=video_start&req={request_id}&ad={campaign_id}]]></Tracking>
              <Tracking event="firstQuartile"><![CDATA[{base_url}/api/v1/event/track?type=video_first_quartile&req={request_id}&ad={campaign_id}]]></Tracking>
              <Tracking event="midpoint"><![CDATA[{base_url}/api/v1/event/track?type=video_midpoint&req={request_id}&ad={campaign_id}]]></Tracking>
              <Tracking event="thirdQuartile"><![CDATA[{base_url}/api/v1/event/track?type=video_third_quartile&req={request_id}&ad={campaign_id}]]></Tracking>
              <Tracking event="complete"><![CDATA[{base_url}/api/v1/event/track?type=video_complete&req={request_id}&ad={campaign_id}]]></Tracking>
            </TrackingEvents>
            <VideoClicks>
              <ClickThrough><![CDATA[{landing_url}]]></ClickThrough>
              <ClickTracking><![CDATA[{click_url}]]></ClickTracking>
            </VideoClicks>
            <MediaFiles>
              <MediaFile id="mf_{creative_id}" delivery="progressive" type="video/mp4" width="{width}" height="{height}" bitrate="1500">
                <![CDATA[{video_url}]]>
              </MediaFile>
            </MediaFiles>
          </Linear>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>"#,
            campaign_id = candidate.campaign_id,
            creative_id = candidate.creative_id,
            title = title,
            desc = desc,
            error_url = error_url,
            impression_url = impression_url,
            click_url = click_url,
            landing_url = candidate.landing_url,
            video_url = video_url,
            width = width,
            height = height,
            base_url = base_url,
            request_id = request_id,
        )
    } else {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="3.0"/>"#
            .to_string()
    };

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
        .into_response())
}
