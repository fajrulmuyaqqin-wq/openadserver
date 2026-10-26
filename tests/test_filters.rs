use openadserver::rec_engine::filter::base::{CompositeFilter, Filter, PassThroughFilter};
use openadserver::rec_engine::filter::quality::{DiversityFilter, QualityFilter};
use openadserver::schemas::internal::{AdCandidate, UserContext};

#[tokio::test]
async fn test_quality_filter() {
    let filter = QualityFilter {
        require_image: true,
        require_title: true,
        min_ctr: 0.01,
        min_cvr: 0.0,
    };

    let ctx = UserContext::default();

    let mut valid = AdCandidate::default();
    valid.landing_url = "https://example.com".to_string();
    valid.image_url = Some("https://example.com/img.png".to_string());
    valid.title = Some("Valid Title".to_string());
    valid.pctr = 0.02;

    let mut missing_img = valid.clone();
    missing_img.image_url = None;

    let mut low_ctr = valid.clone();
    low_ctr.pctr = 0.005;

    let candidates = vec![valid.clone(), missing_img, low_ctr];
    let result = filter.filter(candidates, &ctx).await.unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].title.as_deref(), Some("Valid Title"));
}

#[tokio::test]
async fn test_diversity_filter() {
    let filter = DiversityFilter {
        max_per_advertiser: 2,
    };
    let ctx = UserContext::default();

    let mut c1 = AdCandidate::default();
    c1.advertiser_id = 1;
    let mut c2 = AdCandidate::default();
    c2.advertiser_id = 1;
    let mut c3 = AdCandidate::default();
    c3.advertiser_id = 1; // 3rd ad from advertiser 1
    let mut c4 = AdCandidate::default();
    c4.advertiser_id = 2; // advertiser 2

    let candidates = vec![c1, c2, c3, c4];
    let result = filter.filter(candidates, &ctx).await.unwrap();

    // Should allow only 2 from advertiser 1, plus 1 from advertiser 2 = 3 ads
    assert_eq!(result.len(), 3);
    assert_eq!(result.iter().filter(|c| c.advertiser_id == 1).count(), 2);
    assert_eq!(result.iter().filter(|c| c.advertiser_id == 2).count(), 1);
}

#[tokio::test]
async fn test_composite_filter() {
    let f1 = Box::new(PassThroughFilter);
    let f2 = Box::new(DiversityFilter {
        max_per_advertiser: 1,
    });
    let composite = CompositeFilter::new(vec![f1, f2]);

    let ctx = UserContext::default();
    let mut c1 = AdCandidate::default();
    c1.advertiser_id = 10;
    let mut c2 = AdCandidate::default();
    c2.advertiser_id = 10;

    let result = composite.filter(vec![c1, c2], &ctx).await.unwrap();
    assert_eq!(result.len(), 1);
}
