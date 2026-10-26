use openadserver::models::BidType;
use openadserver::rec_engine::ranking::bidding::{Bidding, RankingStrategy, SecondPriceAuction};
use openadserver::schemas::internal::AdCandidate;

#[test]
fn test_calculate_ecpm_cpm() {
    let bidding = Bidding::new(RankingStrategy::ECPM, Some(0.01));
    let mut candidate = AdCandidate::default();
    candidate.bid = 5.0;
    candidate.bid_type = BidType::CPM as i16;

    let ecpm = bidding.calculate_ecpm(&candidate);
    assert_eq!(ecpm, 5.0);
}

#[test]
fn test_calculate_ecpm_cpc() {
    let bidding = Bidding::new(RankingStrategy::ECPM, Some(0.01));
    let mut candidate = AdCandidate::default();
    candidate.bid = 2.5;
    candidate.bid_type = BidType::CPC as i16;
    candidate.pctr = 0.02;

    let ecpm = bidding.calculate_ecpm(&candidate);
    // bid * pctr * 1000 = 2.5 * 0.02 * 1000 = 50.0
    assert!((ecpm - 50.0).abs() < 1e-4);
}

#[test]
fn test_calculate_ecpm_cpa() {
    let bidding = Bidding::new(RankingStrategy::ECPM, Some(0.01));
    let mut candidate = AdCandidate::default();
    candidate.bid = 50.0;
    candidate.bid_type = BidType::CPA as i16;
    candidate.pctr = 0.02;
    candidate.pcvr = 0.1;

    let ecpm = bidding.calculate_ecpm(&candidate);
    // bid * pctr * pcvr * 1000 = 50.0 * 0.02 * 0.1 * 1000 = 100.0
    assert!((ecpm - 100.0).abs() < 1e-4);
}

#[test]
fn test_ranking_sort_descending() {
    let bidding = Bidding::new(RankingStrategy::ECPM, Some(0.01));
    let mut c1 = AdCandidate::default();
    c1.campaign_id = 1;
    c1.bid = 1.0;
    c1.bid_type = BidType::CPM as i16;

    let mut c2 = AdCandidate::default();
    c2.campaign_id = 2;
    c2.bid = 10.0;
    c2.bid_type = BidType::CPM as i16;

    let mut c3 = AdCandidate::default();
    c3.campaign_id = 3;
    c3.bid = 5.0;
    c3.bid_type = BidType::CPM as i16;

    let mut candidates = vec![c1, c2, c3];
    bidding.rank(&mut candidates, true);

    assert_eq!(candidates[0].campaign_id, 2);
    assert_eq!(candidates[1].campaign_id, 3);
    assert_eq!(candidates[2].campaign_id, 1);
}

#[test]
fn test_ranking_strategies() {
    let mut candidate = AdCandidate::default();
    candidate.bid = 10.0;
    candidate.bid_type = BidType::CPM as i16;
    candidate.pctr = 0.05;
    candidate.pcvr = 0.02;
    candidate.ecpm = 10.0;

    let bidding_ecpm = Bidding::new(RankingStrategy::ECPM, None);
    assert_eq!(bidding_ecpm.calculate_score(&candidate), 10.0);

    let bidding_rev = Bidding::new(RankingStrategy::Revenue, None);
    // quality_factor = min(0.05 / 0.01, 2.0) = 2.0 -> score = 20.0
    assert_eq!(bidding_rev.calculate_score(&candidate), 20.0);

    let bidding_eng = Bidding::new(RankingStrategy::Engagement, None);
    // 10.0 * (1 + 0.05 * 10) = 15.0
    assert_eq!(bidding_eng.calculate_score(&candidate), 15.0);
}

#[test]
fn test_second_price_auction() {
    let auction = SecondPriceAuction::new(0.01);

    let mut c1 = AdCandidate::default();
    c1.campaign_id = 1;
    c1.ecpm = 10.0;

    let mut c2 = AdCandidate::default();
    c2.campaign_id = 2;
    c2.ecpm = 6.0;

    let (winner, price) = auction.run_auction(&[c1.clone(), c2]);
    assert_eq!(winner.unwrap().campaign_id, 1);
    assert!((price - 6.01).abs() < 1e-4);
}
