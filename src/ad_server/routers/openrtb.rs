use crate::common::errors::AppError;
use crate::common::utils::generate_request_id;
use crate::schemas::openrtb::{OpenRtbBid, OpenRtbBidRequest, OpenRtbBidResponse, OpenRtbSeatBid};
use crate::schemas::request::{AdRequest, DeviceInfo, GeoInfo};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use std::sync::Arc;
use tracing::info;

pub fn router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/bid", post(openrtb_bid_handler))
        .with_state(state)
}

async fn openrtb_bid_handler(
    State(state): State<Arc<crate::AppState>>,
    headers: HeaderMap,
    Json(bid_request): Json<OpenRtbBidRequest>,
) -> Result<Response, AppError> {
    let request_id = generate_request_id();

    let first_imp = match bid_request.imp.first() {
        Some(imp) => imp,
        None => return Ok((StatusCode::NO_CONTENT, ()).into_response()),
    };

    let slot_id = first_imp
        .tagid
        .clone()
        .unwrap_or_else(|| "openrtb_default_slot".to_string());
    let bid_floor = first_imp.bidfloor.unwrap_or(0.0);

    let os = bid_request
        .device
        .as_ref()
        .and_then(|d| d.os.clone())
        .unwrap_or_else(|| "android".to_string());
    let ip = bid_request
        .device
        .as_ref()
        .and_then(|d| d.ip.clone())
        .or_else(|| {
            headers
                .get("x-forwarded-for")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        });
    let country = bid_request
        .device
        .as_ref()
        .and_then(|d| d.geo.as_ref())
        .and_then(|g| g.country.clone());
    let city = bid_request
        .device
        .as_ref()
        .and_then(|d| d.geo.as_ref())
        .and_then(|g| g.city.clone());
    let user_id = bid_request.user.as_ref().and_then(|u| u.id.clone());

    let (w, h) = if let Some(b) = &first_imp.banner {
        (b.w, b.h)
    } else if let Some(v) = &first_imp.video {
        (v.w, v.h)
    } else {
        (None, None)
    };

    let ad_request = AdRequest {
        slot_id,
        user_id,
        device: DeviceInfo {
            os,
            os_version: bid_request.device.as_ref().and_then(|d| d.osv.clone()),
            model: bid_request.device.as_ref().and_then(|d| d.model.clone()),
            brand: bid_request.device.as_ref().and_then(|d| d.make.clone()),
            screen_width: w,
            screen_height: h,
            language: bid_request.device.as_ref().and_then(|d| d.language.clone()),
        },
        geo: Some(GeoInfo {
            ip,
            country,
            region: None,
            city,
            latitude: None,
            longitude: None,
        }),
        context: None,
        user_features: None,
        num_ads: 1,
    };

    info!(
        openrtb_id = %bid_request.id,
        bid_floor = bid_floor,
        "OpenRTB 2.5 bid request received"
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

    // Filter candidate against bid floor
    let winning_candidate = candidates.into_iter().find(|c| c.ecpm >= bid_floor);

    if let Some(candidate) = winning_candidate {
        let win_notice_url = format!(
            "{}/api/v1/event/track?type=win&req={}&ad={}&price=${{AUCTION_PRICE}}",
            base_url, request_id, candidate.campaign_id
        );

        let adm = if let Some(video_url) = &candidate.video_url {
            format!(
                r#"<VAST version="3.0"><Ad id="{}"><InLine><AdSystem>OpenAdServer</AdSystem><AdTitle>{}</AdTitle><Creatives><Creative><Linear><Duration>00:00:15</Duration><MediaFiles><MediaFile type="video/mp4"><![CDATA[{}]]></MediaFile></MediaFiles></Linear></Creative></Creatives></InLine></Ad></VAST>"#,
                candidate.campaign_id,
                candidate.title.as_deref().unwrap_or("Ad"),
                video_url
            )
        } else {
            let img = candidate.image_url.as_deref().unwrap_or("");
            format!(
                r#"<a href="{}" target="_blank"><img src="{}" width="{}" height="{}" alt="{}"/></a>"#,
                candidate.landing_url,
                img,
                candidate.width.unwrap_or(300),
                candidate.height.unwrap_or(250),
                candidate.title.as_deref().unwrap_or("Ad")
            )
        };

        let bid = OpenRtbBid {
            id: format!("bid_{}", request_id),
            impid: first_imp.id.clone(),
            price: (candidate.ecpm * 100.0).round() / 100.0,
            adid: Some(format!(
                "ad_{}_{}",
                candidate.campaign_id, candidate.creative_id
            )),
            nurl: Some(win_notice_url),
            adm: Some(adm),
            adomain: None,
            iurl: candidate.image_url,
            cid: Some(candidate.campaign_id.to_string()),
            crid: Some(candidate.creative_id.to_string()),
            w: candidate.width.map(|w| w as i32),
            h: candidate.height.map(|h| h as i32),
        };

        let response = OpenRtbBidResponse {
            id: bid_request.id,
            seatbid: vec![OpenRtbSeatBid {
                bid: vec![bid],
                seat: Some("openadserver".to_string()),
                group: None,
            }],
            bidid: Some(format!("bid_{}", request_id)),
            cur: Some("USD".to_string()),
            nbr: None,
        };

        Ok(Json(response).into_response())
    } else {
        // OpenRTB 2.5 No-Bid: HTTP 204 No Content
        Ok((StatusCode::NO_CONTENT, ()).into_response())
    }
}
