use openadserver::ml_engine::models::deepfm::DeepFM;
use openadserver::ml_engine::models::lr::LogisticRegression;
use openadserver::ml_engine::serving::OnlinePredictor;
use openadserver::schemas::internal::{AdCandidate, UserContext};

#[test]
fn test_lr_forward_bounds() {
    let lr = LogisticRegression::new(vec![100, 200, 50], 3);
    let pred = lr.forward(&[10, 25, 5], &[0.5, 1.2, 0.0]);

    // Sigmoid probability must be bounded between 0 and 1
    assert!(pred > 0.0 && pred < 1.0);
}

#[test]
fn test_deepfm_forward_bounds() {
    let deepfm = DeepFM::new(
        vec![1000, 50, 100], // sparse vocab sizes
        2,                   // dense features
        8,                   // embedding dim
        vec![64, 32],        // hidden layers
    );

    let pred = deepfm.forward(&[123, 12, 45], &[0.5, 0.9]);
    assert!(pred > 0.0 && pred < 1.0);
}

#[test]
fn test_online_predictor_batch() {
    let predictor =
        OnlinePredictor::with_deepfm(vec![10000, 10, 1000, 10000, 10000, 10], 3, 8, vec![64, 32]);

    let mut ctx = UserContext::default();
    ctx.user_id = Some("test_user_999".to_string());
    ctx.os = "android".to_string();
    ctx.country = "US".to_string();
    ctx.age = Some(28);

    let mut c1 = AdCandidate::default();
    c1.campaign_id = 101;
    c1.creative_id = 501;
    c1.bid = 3.5;

    let mut c2 = AdCandidate::default();
    c2.campaign_id = 102;
    c2.creative_id = 502;
    c2.bid = 1.8;

    let results = predictor.predict_batch(&ctx, &[c1, c2]);
    assert_eq!(results.len(), 2);
    assert!(results[0].pctr > 0.0 && results[0].pctr < 1.0);
    assert!(results[1].pctr > 0.0 && results[1].pctr < 1.0);
    assert!(results[0].latency_ms >= 0.0);
}
