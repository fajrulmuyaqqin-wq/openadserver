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
use openadserver::AppState;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_vast_app(with_ad: bool) -> axum::Router {
    let settings = Settings::default();
    let redis = RedisClient::in_memory();
    let pool_options = sqlx::postgres::PgPoolOptions::new();
    let pool = pool_options
        .connect_lazy("postgres://localhost/test")
        .unwrap();

    if with_ad {
        let campaigns = vec![CachedCampaign {
            id: 10,
            advertiser_id: 1,
            name: "Video Campaign".to_string(),
            bid_type: 1,
            bid_amount: 15.0,
            budget_daily: None,
            budget_total: None,
            spent_today: 0.0,
            spent_total: 0.0,
            freq_cap_daily: None,
            freq_cap_hourly: None,
            creatives: vec![CachedCreative {
                id: 101,
                title: Some("Premium Video Ad".to_string()),
                description: Some("Watch our new trailer".to_string()),
                image_url: Some("https://example.com/thumb.jpg".to_string()),
                video_url: Some("https://example.com/ad_video.mp4".to_string()),
                landing_url: "https://example.com/watch".to_string(),
                creative_type: 3, // Video
                width: Some(1920),
                height: Some(1080),
            }],
            targeting_rules: vec![CachedTargetingRule {
                rule_type: "os".to_string(),
                rule_value: json!({ "values": ["android"] }),
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
async fn test_vast_empty_fallback() {
    let app = setup_vast_app(false).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/ad/vast?slot_id=video_slot_01&os=android")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/xml; charset=utf-8"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let xml = String::from_utf8(body.to_vec()).unwrap();
    assert!(xml.contains("<VAST version=\"3.0\"/>"));
}

#[tokio::test]
async fn test_vast_ad_filled_xml() {
    let app = setup_vast_app(true).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/ad/vast?slot_id=video_slot_01&os=android")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/xml; charset=utf-8"
    );

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let xml = String::from_utf8(body.to_vec()).unwrap();

    assert!(xml.contains("<VAST version=\"3.0\">"));
    assert!(xml.contains("<AdTitle><![CDATA[Premium Video Ad]]></AdTitle>"));
    assert!(xml.contains("<ClickThrough><![CDATA[https://example.com/watch]]></ClickThrough>"));
    assert!(xml.contains("https://example.com/ad_video.mp4"));
    assert!(xml.contains("<Tracking event=\"start\">"));
    assert!(xml.contains("<Tracking event=\"complete\">"));
}
