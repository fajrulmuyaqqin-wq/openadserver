use openadserver::common::cache::RedisClient;
use openadserver::rec_engine::retrieval::targeting::{
    CachedCampaign, CachedCreative, CachedTargetingRule, TargetingRetrieval,
};
use openadserver::schemas::internal::UserContext;
use serde_json::json;

fn create_mock_targeting_retrieval() -> TargetingRetrieval {
    let redis = RedisClient::in_memory();
    let pool_options = sqlx::postgres::PgPoolOptions::new();
    let pool = pool_options
        .connect_lazy("postgres://localhost/test")
        .unwrap();
    TargetingRetrieval::new(pool, redis)
}

#[tokio::test]
async fn test_targeting_match_rules() {
    let retrieval = create_mock_targeting_retrieval();

    let mut camp = CachedCampaign {
        id: 1,
        advertiser_id: 10,
        name: "Test Campaign".to_string(),
        bid_type: 1,
        bid_amount: 5.0,
        budget_daily: None,
        budget_total: None,
        spent_today: 0.0,
        spent_total: 0.0,
        freq_cap_daily: None,
        freq_cap_hourly: None,
        creatives: vec![CachedCreative {
            id: 101,
            title: Some("Ad Title".to_string()),
            description: None,
            image_url: None,
            video_url: None,
            landing_url: "https://example.com".to_string(),
            creative_type: 1,
            width: None,
            height: None,
        }],
        targeting_rules: vec![
            CachedTargetingRule {
                rule_type: "os".to_string(),
                rule_value: json!({ "values": ["android"] }),
                is_include: true,
            },
            CachedTargetingRule {
                rule_type: "age".to_string(),
                rule_value: json!({ "min": 18, "max": 35 }),
                is_include: true,
            },
            CachedTargetingRule {
                rule_type: "geo".to_string(),
                rule_value: json!({ "countries": ["US", "ID"] }),
                is_include: true,
            },
        ],
    };

    // User 1: Matches all
    let mut u1 = UserContext::default();
    u1.os = "android".to_string();
    u1.age = Some(25);
    u1.country = "ID".to_string();
    assert!(retrieval.match_targeting(&camp, &u1));

    // User 2: Wrong OS
    let mut u2 = UserContext::default();
    u2.os = "ios".to_string();
    u2.age = Some(25);
    u2.country = "ID".to_string();
    assert!(!retrieval.match_targeting(&camp, &u2));

    // User 3: Age too old
    let mut u3 = UserContext::default();
    u3.os = "android".to_string();
    u3.age = Some(40);
    u3.country = "ID".to_string();
    assert!(!retrieval.match_targeting(&camp, &u3));

    // User 4: Wrong country
    let mut u4 = UserContext::default();
    u4.os = "android".to_string();
    u4.age = Some(20);
    u4.country = "FR".to_string();
    assert!(!retrieval.match_targeting(&camp, &u4));

    // Exclude rule:
    camp.targeting_rules.push(CachedTargetingRule {
        rule_type: "gender".to_string(),
        rule_value: json!({ "values": ["female"] }),
        is_include: false, // Exclude females
    });

    u1.gender = Some("male".to_string());
    assert!(retrieval.match_targeting(&camp, &u1));

    u1.gender = Some("female".to_string());
    assert!(!retrieval.match_targeting(&camp, &u1));
}
