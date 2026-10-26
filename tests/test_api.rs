use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use openadserver::ad_server::services::{AdService, EventService};
use openadserver::common::cache::RedisClient;
use openadserver::common::config::Settings;
use openadserver::create_app;
use openadserver::rec_engine::filter::quality::QualityFilter;
use openadserver::rec_engine::ranking::predictor::StatisticalPredictor;
use openadserver::rec_engine::ranking::reranker::DiversityReranker;
use openadserver::rec_engine::retrieval::targeting::TargetingRetrieval;
use openadserver::rec_engine::{RecommendationConfig, RecommendationEngine};
use openadserver::schemas::request::{AdRequest, DeviceInfo, EventRequest, GeoInfo};
use openadserver::AppState;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_test_app() -> axum::Router {
    let settings = Settings::default();
    let redis = RedisClient::in_memory();
    let pool_options = sqlx::postgres::PgPoolOptions::new();
    let pool = pool_options
        .connect_lazy("postgres://localhost/test")
        .unwrap();

    let retrieval = Box::new(TargetingRetrieval::new(pool.clone(), redis.clone()));
    let filters: Vec<Box<dyn openadserver::rec_engine::filter::Filter>> =
        vec![Box::new(QualityFilter::default())];
    let predictor = Box::new(StatisticalPredictor::new(0.02, 0.005));
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
async fn test_health_endpoints() {
    let app = setup_test_app().await;

    // Test /ping
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res["pong"], true);

    // Test /live
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/live").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Test /metrics
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_ad_request_empty_pipeline() {
    let app = setup_test_app().await;

    let ad_req = AdRequest {
        slot_id: "slot_banner_01".to_string(),
        user_id: Some("user_test_123".to_string()),
        device: DeviceInfo {
            os: "android".to_string(),
            os_version: Some("14".to_string()),
            model: Some("Pixel 8".to_string()),
            brand: Some("Google".to_string()),
            screen_width: Some(1080),
            screen_height: Some(2400),
            language: Some("en".to_string()),
        },
        geo: Some(GeoInfo {
            ip: Some("127.0.0.1".to_string()),
            country: Some("ID".to_string()),
            region: None,
            city: Some("Jakarta".to_string()),
            latitude: None,
            longitude: None,
        }),
        context: None,
        user_features: None,
        num_ads: 2,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/ad/request")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&ad_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(res["request_id"].is_string());
    assert_eq!(res["count"], 0);
    assert!(res["ads"].is_array());
}

#[tokio::test]
async fn test_event_track() {
    let app = setup_test_app().await;

    let event_req = EventRequest {
        request_id: "req_test_01".to_string(),
        ad_id: "1_101".to_string(),
        event_type: "impression".to_string(),
        timestamp: None,
        user_id: Some("user_test_123".to_string()),
        extra: None,
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/event/track")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&event_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res["success"], true);

    // Test GET /api/v1/event/track (pixel tracking)
    let response_pixel = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/event/track?event_type=click&request_id=req_test_01&ad_id=1_101&user_id=u123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_pixel.status(), StatusCode::OK);
    let body_pixel = response_pixel
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let res_pixel: serde_json::Value = serde_json::from_slice(&body_pixel).unwrap();
    assert_eq!(res_pixel["success"], true);
}
