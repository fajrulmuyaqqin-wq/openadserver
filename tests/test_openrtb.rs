use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use openadserver::ad_server::services::{AdService, EventService};
use openadserver::common::cache::{CacheKeys, RedisClient};
use openadserver::common::config::Settings;
use openadserver::create_app;
use openadserver::rec_engine::filter::quality::QualityFilter;
use openadserver::rec_engine::ranking::predictor::StatisticalPredictor;
use openadserver::rec_engine::ranking::reranker::DiversityReranker;
use openadserver::rec_engine::retrieval::targeting::{
    CachedCampaign, CachedCreative, CachedTargetingRule, TargetingRetrieval,
};
use openadserver::rec_engine::{RecommendationConfig, RecommendationEngine};
use openadserver::schemas::openrtb::{
    OpenRtbBanner, OpenRtbBidRequest, OpenRtbBidResponse, OpenRtbDevice, OpenRtbGeo, OpenRtbImp,
    OpenRtbUser,
};
use openadserver::AppState;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_openrtb_app(with_ad: bool) -> axum::Router {
    let settings = Settings::default();
    let redis = RedisClient::in_memory();
    let pool_options = sqlx::postgres::PgPoolOptions::new();
    let pool = pool_options
        .connect_lazy("postgres://localhost/test")
        .unwrap();

    if with_ad {
        let campaigns = vec![CachedCampaign {
            id: 20,
            advertiser_id: 2,
            name: "RTB Campaign".to_string(),
            bid_type: 1, // CPM
            bid_amount: 10.0,
            budget_daily: None,
            budget_total: None,
            spent_today: 0.0,
            spent_total: 0.0,
            freq_cap_daily: None,
            freq_cap_hourly: None,
            creatives: vec![CachedCreative {
                id: 201,
                title: Some("RTB Banner Ad".to_string()),
                description: Some("Click for deals".to_string()),
                image_url: Some("https://example.com/banner.jpg".to_string()),
                video_url: None,
                landing_url: "https://example.com/rtb-landing".to_string(),
                creative_type: 1,
                width: Some(300),
                height: Some(250),
            }],
            targeting_rules: vec![CachedTargetingRule {
                rule_type: "geo".to_string(),
                rule_value: json!({ "countries": ["ID", "US"] }),
                is_include: true,
            }],
        }];
        let _ = redis
            .set_json(CacheKeys::active_ads(), &campaigns, None)
            .await;
    }

    let retrieval = Box::new(TargetingRetrieval::new(pool.clone(), redis.clone()));
    let filters: Vec<Box<dyn openadserver::rec_engine::filter::Filter>> =
        vec![Box::new(QualityFilter::default())];
    let predictor = Box::new(StatisticalPredictor::new(0.05, 0.01));
    let rerankers: Vec<Box<dyn openadserver::rec_engine::ranking::Reranker>> =
        vec![Box::new(DiversityReranker::default())];

    let mut rec_config = RecommendationConfig::default();
    rec_config.enable_budget_filter = false;
    rec_config.enable_frequency_filter = false;

    let rec_engine = Arc::new(RecommendationEngine::new(
        rec_config, retrieval, filters, predictor, rerankers,
    ));

    let ad_service = Arc::new(AdService::new(rec_engine));
    let event_service = Arc::new(EventService::new(pool.clone(), redis.clone()));

    let state = Arc::new(AppState {
        pool,
        redis,
        ad_service,
        event_service,
        settings,
    });

    create_app(state)
}

#[tokio::test]
async fn test_openrtb_bid_match() {
    let app = setup_openrtb_app(true).await;

    let bid_req = OpenRtbBidRequest {
        id: "req_rtb_12345".to_string(),
        imp: vec![OpenRtbImp {
            id: "imp_1".to_string(),
            banner: Some(OpenRtbBanner {
                w: Some(300),
                h: Some(250),
                pos: None,
                mimes: Some(vec!["image/jpeg".to_string()]),
            }),
            video: None,
            bidfloor: Some(5.0),
            bidfloorcur: Some("USD".to_string()),
            secure: Some(1),
            tagid: Some("slot_sidebar_300x250".to_string()),
        }],
        site: None,
        app: None,
        device: Some(OpenRtbDevice {
            ua: Some("Mozilla/5.0".to_string()),
            ip: Some("127.0.0.1".to_string()),
            geo: Some(OpenRtbGeo {
                country: Some("ID".to_string()),
                lat: None,
                lon: None,
                region: None,
                city: Some("Jakarta".to_string()),
            }),
            make: Some("Google".to_string()),
            model: Some("Pixel 8".to_string()),
            os: Some("android".to_string()),
            osv: Some("14".to_string()),
            language: Some("en".to_string()),
            ifa: None,
        }),
        user: Some(OpenRtbUser {
            id: Some("user_rtb_99".to_string()),
            buyeruid: None,
            yob: None,
            gender: None,
            keywords: None,
        }),
        test: None,
        tmax: Some(100),
        bcat: None,
        badv: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/openrtb2/bid")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&bid_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: OpenRtbBidResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(res.id, "req_rtb_12345");
    assert_eq!(res.seatbid.len(), 1);
    let seat = &res.seatbid[0];
    assert_eq!(seat.seat.as_deref(), Some("openadserver"));
    assert_eq!(seat.bid.len(), 1);

    let bid = &seat.bid[0];
    assert_eq!(bid.impid, "imp_1");
    assert!(bid.price >= 5.0);
    assert!(bid.nurl.is_some());
    assert!(bid.adm.is_some());
}

#[tokio::test]
async fn test_openrtb_no_bid_floor_too_high() {
    let app = setup_openrtb_app(true).await;

    // Floor price 50.0 is higher than campaign eCPM (10.0)
    let bid_req = OpenRtbBidRequest {
        id: "req_rtb_high_floor".to_string(),
        imp: vec![OpenRtbImp {
            id: "imp_1".to_string(),
            banner: Some(OpenRtbBanner {
                w: Some(300),
                h: Some(250),
                pos: None,
                mimes: None,
            }),
            video: None,
            bidfloor: Some(50.0),
            bidfloorcur: Some("USD".to_string()),
            secure: None,
            tagid: None,
        }],
        site: None,
        app: None,
        device: Some(OpenRtbDevice {
            ua: None,
            ip: None,
            geo: Some(OpenRtbGeo {
                country: Some("ID".to_string()),
                lat: None,
                lon: None,
                region: None,
                city: None,
            }),
            make: None,
            model: None,
            os: Some("android".to_string()),
            osv: None,
            language: None,
            ifa: None,
        }),
        user: None,
        test: None,
        tmax: None,
        bcat: None,
        badv: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/openrtb2/bid")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&bid_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Standard OpenRTB 2.5: HTTP 204 No Content for no-bid
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_openrtb_no_bid_empty() {
    let app = setup_openrtb_app(false).await;

    let bid_req = OpenRtbBidRequest {
        id: "req_rtb_empty".to_string(),
        imp: vec![OpenRtbImp {
            id: "imp_1".to_string(),
            banner: None,
            video: None,
            bidfloor: None,
            bidfloorcur: None,
            secure: None,
            tagid: None,
        }],
        site: None,
        app: None,
        device: None,
        user: None,
        test: None,
        tmax: None,
        bcat: None,
        badv: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/openrtb2/bid")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&bid_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
