use openadserver::common::config::Settings;
use openadserver::common::database::create_pool;
use openadserver::schemas::admin::{
    CreateAdvertiserRequest, CreateCampaignRequest, CreateCreativeRequest, TargetingRuleDto,
};
use serde_json::json;

#[test]
fn test_admin_dto_serialization() {
    let camp_req = CreateCampaignRequest {
        advertiser_id: 1,
        name: "Test Holiday Campaign".to_string(),
        description: Some("Year end promo".to_string()),
        budget_daily: Some(500.0),
        budget_total: Some(10000.0),
        bid_type: 2, // CPC
        bid_amount: 1.50,
        freq_cap_daily: Some(5),
        freq_cap_hourly: Some(2),
        targeting_rules: Some(vec![
            TargetingRuleDto {
                rule_type: "geo".to_string(),
                rule_value: json!({ "countries": ["ID", "SG"] }),
                is_include: true,
            },
            TargetingRuleDto {
                rule_type: "age".to_string(),
                rule_value: json!({ "min": 21, "max": 45 }),
                is_include: true,
            },
        ]),
    };

    let serialized = serde_json::to_string(&camp_req).unwrap();
    let deserialized: CreateCampaignRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.name, "Test Holiday Campaign");
    assert_eq!(deserialized.bid_amount, 1.50);
    assert_eq!(deserialized.targeting_rules.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn test_admin_crud_live_db_if_available() {
    let settings = Settings::load();
    let pool = match create_pool(&settings.database).await {
        Ok(p) => p,
        Err(_) => {
            println!("Postgres not reachable, skipping live DB test");
            return;
        }
    };

    // 1. Create Advertiser
    let adv_name = format!("Adv_Test_{}", chrono::Utc::now().timestamp_millis());
    let adv_payload = CreateAdvertiserRequest {
        name: adv_name.clone(),
        company: Some("Tech Inc".to_string()),
        contact_email: Some("contact@tech.com".to_string()),
        contact_phone: Some("+628123456789".to_string()),
        balance: Some(1000.0),
        credit_limit: Some(5000.0),
    };

    let adv_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO advertisers (name, company, contact_email, contact_phone, balance, credit_limit, status)
        VALUES ($1, $2, $3, $4, 1000.0, 5000.0, 1)
        RETURNING id
        "#
    )
    .bind(&adv_payload.name)
    .bind(&adv_payload.company)
    .bind(&adv_payload.contact_email)
    .bind(&adv_payload.contact_phone)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(adv_id > 0);

    // 2. Create Campaign
    let camp_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO campaigns (advertiser_id, name, budget_daily, bid_type, bid_amount, status)
        VALUES ($1, 'Test Live Camp', 200.0, 1, 3.50, 1)
        RETURNING id
        "#,
    )
    .bind(adv_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(camp_id > 0);

    // 3. Create Creative
    let cr_payload = CreateCreativeRequest {
        campaign_id: camp_id,
        title: Some("Banner 300x250".to_string()),
        description: Some("Discount 20%".to_string()),
        image_url: Some("https://example.com/300x250.png".to_string()),
        video_url: None,
        landing_url: "https://example.com/promo".to_string(),
        creative_type: 1,
        width: Some(300),
        height: Some(250),
    };

    let cr_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO creatives (campaign_id, title, description, image_url, landing_url, creative_type, width, height, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1)
        RETURNING id
        "#
    )
    .bind(cr_payload.campaign_id)
    .bind(&cr_payload.title)
    .bind(&cr_payload.description)
    .bind(&cr_payload.image_url)
    .bind(&cr_payload.landing_url)
    .bind(cr_payload.creative_type)
    .bind(cr_payload.width)
    .bind(cr_payload.height)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(cr_id > 0);

    // Clean up created test entities
    let _ = sqlx::query("DELETE FROM creatives WHERE id = $1")
        .bind(cr_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM campaigns WHERE id = $1")
        .bind(camp_id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM advertisers WHERE id = $1")
        .bind(adv_id)
        .execute(&pool)
        .await;
}
