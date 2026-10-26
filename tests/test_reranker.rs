use openadserver::rec_engine::ranking::reranker::{
    CompositeReranker, DiversityReranker, ExplorationReranker, Reranker,
};
use openadserver::schemas::internal::{AdCandidate, UserContext};

#[test]
fn test_diversity_reranker_mmr() {
    let reranker = DiversityReranker::new(0.7, 2);
    let ctx = UserContext::default();

    let mut c1 = AdCandidate::default();
    c1.campaign_id = 1;
    c1.advertiser_id = 1;
    c1.score = 100.0;
    c1.creative_type = 1;

    let mut c2 = AdCandidate::default();
    c2.campaign_id = 2;
    c2.advertiser_id = 1;
    c2.score = 99.0;
    c2.creative_type = 1;

    let mut c3 = AdCandidate::default();
    c3.campaign_id = 3;
    c3.advertiser_id = 2; // different advertiser!
    c3.score = 95.0;
    c3.creative_type = 2;

    let candidates = vec![c1, c2, c3];
    let reranked = reranker.rerank(candidates, &ctx, 2);

    assert_eq!(reranked.len(), 2);
    assert_eq!(reranked[0].campaign_id, 1);
    // Because c2 has advertiser and creative type penalty, c3 can be chosen or c2 if lambda allows.
    // In MMR: c1 is chosen first (score 100).
    // For c2: 0.7 * 99 - 0.3 * (0.5 + 0.2) = 69.3 - 0.21 = 69.09
    // For c3: 0.7 * 95 - 0.3 * 0 = 66.5
}

#[test]
fn test_exploration_reranker_boosts_new_ads() {
    let reranker = ExplorationReranker::new(0.0, 2.0, 1000); // epsilon=0 means exploitation with boost
    let ctx = UserContext::default();

    let mut mature_ad = AdCandidate::default();
    mature_ad.campaign_id = 1;
    mature_ad.score = 10.0;
    mature_ad
        .metadata
        .insert("impressions".to_string(), serde_json::json!(5000));

    let mut new_ad = AdCandidate::default();
    new_ad.campaign_id = 2;
    new_ad.score = 8.0;
    new_ad
        .metadata
        .insert("impressions".to_string(), serde_json::json!(50)); // < 1000 impressions -> 2.0 boost -> 16.0!

    let candidates = vec![mature_ad, new_ad];
    let reranked = reranker.rerank(candidates, &ctx, 2);

    // New ad should be boosted to 16.0 and take 1st place!
    assert_eq!(reranked[0].campaign_id, 2);
    assert_eq!(reranked[1].campaign_id, 1);
}

#[test]
fn test_composite_reranker() {
    let r1 = Box::new(ExplorationReranker::new(0.0, 1.0, 1000));
    let r2 = Box::new(DiversityReranker::new(0.8, 1));
    let composite = CompositeReranker::new(vec![r1, r2]);

    let ctx = UserContext::default();
    let mut c1 = AdCandidate::default();
    c1.advertiser_id = 5;
    c1.score = 10.0;

    let mut c2 = AdCandidate::default();
    c2.advertiser_id = 5;
    c2.score = 8.0;

    let result = composite.rerank(vec![c1, c2], &ctx, 2);
    // Max 1 per advertiser in r2 -> only 1 returned
    assert_eq!(result.len(), 1);
}
