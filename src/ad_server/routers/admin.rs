use crate::common::cache::{CacheKeys, RedisClient};
use crate::common::errors::AppError;
use crate::models::{Advertiser, Campaign, Creative, TargetingRule};
use crate::schemas::admin::{
    CampaignDetailResponse, CreateAdvertiserRequest, CreateCampaignRequest, CreateCreativeRequest,
    ListQuery, UpdateAdvertiserRequest, UpdateCampaignRequest, UpdateCreativeRequest,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use bigdecimal::{BigDecimal, FromPrimitive};
use serde_json::json;
use std::sync::Arc;
use tracing::info;

pub fn router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        // Advertisers
        .route("/advertiser", post(create_advertiser))
        .route("/advertiser", get(list_advertisers))
        .route("/advertiser/{id}", get(get_advertiser))
        .route("/advertiser/{id}", put(update_advertiser))
        // Campaigns
        .route("/campaign", post(create_campaign))
        .route("/campaign", get(list_campaigns))
        .route("/campaign/{id}", get(get_campaign))
        .route("/campaign/{id}", put(update_campaign))
        .route("/campaign/{id}", delete(delete_campaign))
        // Creatives
        .route("/creative", post(create_creative))
        .route("/creative", get(list_creatives))
        .route("/creative/{id}", get(get_creative))
        .route("/creative/{id}", put(update_creative))
        .route("/creative/{id}", delete(delete_creative))
        .with_state(state)
}

// ============================================================================
// Advertisers Handlers
// ============================================================================

async fn create_advertiser(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<CreateAdvertiserRequest>,
) -> Result<(StatusCode, Json<Advertiser>), AppError> {
    let balance = payload
        .balance
        .and_then(BigDecimal::from_f64)
        .unwrap_or_else(|| BigDecimal::from(0));
    let credit_limit = payload
        .credit_limit
        .and_then(BigDecimal::from_f64)
        .unwrap_or_else(|| BigDecimal::from(0));

    let advertiser = sqlx::query_as::<_, Advertiser>(
        r#"
        INSERT INTO advertisers (name, company, contact_email, contact_phone, balance, credit_limit, status)
        VALUES ($1, $2, $3, $4, $5, $6, 1)
        RETURNING *
        "#,
    )
    .bind(payload.name)
    .bind(payload.company)
    .bind(payload.contact_email)
    .bind(payload.contact_phone)
    .bind(balance)
    .bind(credit_limit)
    .fetch_one(&state.pool)
    .await?;

    info!(advertiser_id = advertiser.id, name = %advertiser.name, "Advertiser created");
    Ok((StatusCode::CREATED, Json(advertiser)))
}

async fn list_advertisers(
    State(state): State<Arc<crate::AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Advertiser>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100) as i64;
    let offset = (query.page.unwrap_or(1).saturating_sub(1) * (limit as u32)) as i64;

    let advertisers = if let Some(status) = query.status {
        sqlx::query_as::<_, Advertiser>(
            "SELECT * FROM advertisers WHERE status = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, Advertiser>(
            "SELECT * FROM advertisers ORDER BY id DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(advertisers))
}

async fn get_advertiser(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Advertiser>, AppError> {
    let advertiser = sqlx::query_as::<_, Advertiser>("SELECT * FROM advertisers WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Advertiser with ID {} not found", id)))?;

    Ok(Json(advertiser))
}

async fn update_advertiser(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateAdvertiserRequest>,
) -> Result<Json<Advertiser>, AppError> {
    let current = get_advertiser(State(state.clone()), Path(id)).await?.0;

    let name = payload.name.unwrap_or(current.name);
    let company = payload.company.or(current.company);
    let contact_email = payload.contact_email.or(current.contact_email);
    let contact_phone = payload.contact_phone.or(current.contact_phone);
    let balance = payload
        .balance
        .and_then(BigDecimal::from_f64)
        .unwrap_or(current.balance);
    let credit_limit = payload
        .credit_limit
        .and_then(BigDecimal::from_f64)
        .unwrap_or(current.credit_limit);
    let status = payload.status.unwrap_or(current.status);

    let updated = sqlx::query_as::<_, Advertiser>(
        r#"
        UPDATE advertisers
        SET name = $1, company = $2, contact_email = $3, contact_phone = $4,
            balance = $5, credit_limit = $6, status = $7, updated_at = NOW()
        WHERE id = $8
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(company)
    .bind(contact_email)
    .bind(contact_phone)
    .bind(balance)
    .bind(credit_limit)
    .bind(status)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated))
}

// ============================================================================
// Campaigns Handlers
// ============================================================================

async fn create_campaign(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<(StatusCode, Json<CampaignDetailResponse>), AppError> {
    let bid_amount =
        BigDecimal::from_f64(payload.bid_amount).unwrap_or_else(|| BigDecimal::from(1));
    let budget_daily = payload.budget_daily.and_then(BigDecimal::from_f64);
    let budget_total = payload.budget_total.and_then(BigDecimal::from_f64);

    let mut tx = state.pool.begin().await?;

    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        INSERT INTO campaigns (
            advertiser_id, name, description, budget_daily, budget_total,
            bid_type, bid_amount, freq_cap_daily, freq_cap_hourly, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1)
        RETURNING *
        "#,
    )
    .bind(payload.advertiser_id)
    .bind(payload.name)
    .bind(payload.description)
    .bind(budget_daily)
    .bind(budget_total)
    .bind(payload.bid_type)
    .bind(bid_amount)
    .bind(payload.freq_cap_daily)
    .bind(payload.freq_cap_hourly)
    .fetch_one(&mut *tx)
    .await?;

    let mut targeting_rules = Vec::new();
    if let Some(rules) = payload.targeting_rules {
        for rule in rules {
            let tr = sqlx::query_as::<_, TargetingRule>(
                r#"
                INSERT INTO targeting_rules (campaign_id, rule_type, rule_value, is_include)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#,
            )
            .bind(campaign.id)
            .bind(rule.rule_type)
            .bind(rule.rule_value)
            .bind(rule.is_include)
            .fetch_one(&mut *tx)
            .await?;
            targeting_rules.push(tr);
        }
    }

    tx.commit().await?;

    // Invalidate Redis active campaign cache
    invalidate_active_cache(&state.redis).await;

    info!(campaign_id = campaign.id, name = %campaign.name, "Campaign created");

    Ok((
        StatusCode::CREATED,
        Json(CampaignDetailResponse {
            campaign,
            creatives: Vec::new(),
            targeting_rules,
        }),
    ))
}

async fn list_campaigns(
    State(state): State<Arc<crate::AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Campaign>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100) as i64;
    let offset = (query.page.unwrap_or(1).saturating_sub(1) * (limit as u32)) as i64;

    let campaigns = match (query.advertiser_id, query.status) {
        (Some(adv_id), Some(st)) => {
            sqlx::query_as::<_, Campaign>(
                "SELECT * FROM campaigns WHERE advertiser_id = $1 AND status = $2 ORDER BY id DESC LIMIT $3 OFFSET $4",
            )
            .bind(adv_id)
            .bind(st)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        (Some(adv_id), None) => {
            sqlx::query_as::<_, Campaign>(
                "SELECT * FROM campaigns WHERE advertiser_id = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
            )
            .bind(adv_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        (None, Some(st)) => {
            sqlx::query_as::<_, Campaign>(
                "SELECT * FROM campaigns WHERE status = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
            )
            .bind(st)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, Campaign>(
                "SELECT * FROM campaigns ORDER BY id DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
    };

    Ok(Json(campaigns))
}

async fn get_campaign(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<CampaignDetailResponse>, AppError> {
    let campaign = sqlx::query_as::<_, Campaign>("SELECT * FROM campaigns WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Campaign with ID {} not found", id)))?;

    let creatives = sqlx::query_as::<_, Creative>(
        "SELECT * FROM creatives WHERE campaign_id = $1 ORDER BY id ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    let targeting_rules = sqlx::query_as::<_, TargetingRule>(
        "SELECT * FROM targeting_rules WHERE campaign_id = $1 ORDER BY id ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(CampaignDetailResponse {
        campaign,
        creatives,
        targeting_rules,
    }))
}

async fn update_campaign(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCampaignRequest>,
) -> Result<Json<CampaignDetailResponse>, AppError> {
    let current_detail = get_campaign(State(state.clone()), Path(id)).await?.0;
    let current = current_detail.campaign;

    let name = payload.name.unwrap_or(current.name);
    let description = payload.description.or(current.description);
    let budget_daily = payload
        .budget_daily
        .and_then(BigDecimal::from_f64)
        .or(current.budget_daily);
    let budget_total = payload
        .budget_total
        .and_then(BigDecimal::from_f64)
        .or(current.budget_total);
    let bid_type = payload.bid_type.unwrap_or(current.bid_type);
    let bid_amount = payload
        .bid_amount
        .and_then(BigDecimal::from_f64)
        .unwrap_or(current.bid_amount);
    let freq_cap_daily = payload.freq_cap_daily.or(current.freq_cap_daily);
    let freq_cap_hourly = payload.freq_cap_hourly.or(current.freq_cap_hourly);
    let status = payload.status.unwrap_or(current.status);

    let mut tx = state.pool.begin().await?;

    let campaign = sqlx::query_as::<_, Campaign>(
        r#"
        UPDATE campaigns
        SET name = $1, description = $2, budget_daily = $3, budget_total = $4,
            bid_type = $5, bid_amount = $6, freq_cap_daily = $7, freq_cap_hourly = $8,
            status = $9, updated_at = NOW()
        WHERE id = $10
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(budget_daily)
    .bind(budget_total)
    .bind(bid_type)
    .bind(bid_amount)
    .bind(freq_cap_daily)
    .bind(freq_cap_hourly)
    .bind(status)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    let mut targeting_rules = current_detail.targeting_rules;
    if let Some(rules) = payload.targeting_rules {
        // Replace existing rules
        sqlx::query("DELETE FROM targeting_rules WHERE campaign_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        targeting_rules.clear();
        for rule in rules {
            let tr = sqlx::query_as::<_, TargetingRule>(
                r#"
                INSERT INTO targeting_rules (campaign_id, rule_type, rule_value, is_include)
                VALUES ($1, $2, $3, $4)
                RETURNING *
                "#,
            )
            .bind(campaign.id)
            .bind(rule.rule_type)
            .bind(rule.rule_value)
            .bind(rule.is_include)
            .fetch_one(&mut *tx)
            .await?;
            targeting_rules.push(tr);
        }
    }

    tx.commit().await?;

    invalidate_active_cache(&state.redis).await;

    Ok(Json(CampaignDetailResponse {
        campaign,
        creatives: current_detail.creatives,
        targeting_rules,
    }))
}

async fn delete_campaign(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE campaigns SET status = 0, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    invalidate_active_cache(&state.redis).await;

    info!(campaign_id = id, "Campaign soft deleted");
    Ok(Json(
        json!({ "success": true, "message": "Campaign deleted" }),
    ))
}

// ============================================================================
// Creatives Handlers
// ============================================================================

async fn create_creative(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<CreateCreativeRequest>,
) -> Result<(StatusCode, Json<Creative>), AppError> {
    let creative = sqlx::query_as::<_, Creative>(
        r#"
        INSERT INTO creatives (
            campaign_id, title, description, image_url, video_url, landing_url,
            creative_type, width, height, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 1)
        RETURNING *
        "#,
    )
    .bind(payload.campaign_id)
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.image_url)
    .bind(payload.video_url)
    .bind(payload.landing_url)
    .bind(payload.creative_type)
    .bind(payload.width)
    .bind(payload.height)
    .fetch_one(&state.pool)
    .await?;

    invalidate_active_cache(&state.redis).await;

    info!(
        creative_id = creative.id,
        campaign_id = creative.campaign_id,
        "Creative created"
    );
    Ok((StatusCode::CREATED, Json(creative)))
}

async fn list_creatives(
    State(state): State<Arc<crate::AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Creative>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100) as i64;
    let offset = (query.page.unwrap_or(1).saturating_sub(1) * (limit as u32)) as i64;

    let creatives = match (query.campaign_id, query.status) {
        (Some(cid), Some(st)) => {
            sqlx::query_as::<_, Creative>(
                "SELECT * FROM creatives WHERE campaign_id = $1 AND status = $2 ORDER BY id DESC LIMIT $3 OFFSET $4",
            )
            .bind(cid)
            .bind(st)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        (Some(cid), None) => {
            sqlx::query_as::<_, Creative>(
                "SELECT * FROM creatives WHERE campaign_id = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
            )
            .bind(cid)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        (None, Some(st)) => {
            sqlx::query_as::<_, Creative>(
                "SELECT * FROM creatives WHERE status = $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
            )
            .bind(st)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, Creative>(
                "SELECT * FROM creatives ORDER BY id DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.pool)
            .await?
        }
    };

    Ok(Json(creatives))
}

async fn get_creative(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Creative>, AppError> {
    let creative = sqlx::query_as::<_, Creative>("SELECT * FROM creatives WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Creative with ID {} not found", id)))?;

    Ok(Json(creative))
}

async fn update_creative(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCreativeRequest>,
) -> Result<Json<Creative>, AppError> {
    let current = get_creative(State(state.clone()), Path(id)).await?.0;

    let title = payload.title.or(current.title);
    let description = payload.description.or(current.description);
    let image_url = payload.image_url.or(current.image_url);
    let video_url = payload.video_url.or(current.video_url);
    let landing_url = payload.landing_url.unwrap_or(current.landing_url);
    let creative_type = payload.creative_type.unwrap_or(current.creative_type);
    let width = payload.width.or(current.width);
    let height = payload.height.or(current.height);
    let status = payload.status.unwrap_or(current.status);

    let updated = sqlx::query_as::<_, Creative>(
        r#"
        UPDATE creatives
        SET title = $1, description = $2, image_url = $3, video_url = $4,
            landing_url = $5, creative_type = $6, width = $7, height = $8,
            status = $9, updated_at = NOW()
        WHERE id = $10
        RETURNING *
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(image_url)
    .bind(video_url)
    .bind(landing_url)
    .bind(creative_type)
    .bind(width)
    .bind(height)
    .bind(status)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    invalidate_active_cache(&state.redis).await;

    Ok(Json(updated))
}

async fn delete_creative(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("UPDATE creatives SET status = 0, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    invalidate_active_cache(&state.redis).await;

    info!(creative_id = id, "Creative soft deleted");
    Ok(Json(
        json!({ "success": true, "message": "Creative deleted" }),
    ))
}

async fn invalidate_active_cache(redis: &RedisClient) {
    let _ = redis.delete(CacheKeys::active_ads()).await;
}
