use crate::common::config::DatabaseConfig;
use crate::common::errors::AppError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::{info, warn};

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    let conn_str = config.connection_string();
    let pool = PgPoolOptions::new()
        .max_connections(config.pool_size)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .connect(&conn_str)
        .await?;

    info!(
        host = %config.host,
        port = %config.port,
        database = %config.name,
        "PostgreSQL database pool connected"
    );

    Ok(pool)
}

pub async fn check_health(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| true)
        .unwrap_or_else(|e| {
            warn!("Database health check failed: {}", e);
            false
        })
}

pub async fn init_schema(pool: &PgPool) -> Result<(), AppError> {
    info!("Initializing database schema...");

    let schema_sql = r#"
    CREATE EXTENSION IF NOT EXISTS pg_trgm;

    CREATE TABLE IF NOT EXISTS advertisers (
        id BIGSERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        company VARCHAR(255),
        contact_email VARCHAR(255),
        contact_phone VARCHAR(50),
        balance DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
        credit_limit DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
        status SMALLINT DEFAULT 1 NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_advertiser_status ON advertisers(status);

    CREATE TABLE IF NOT EXISTS campaigns (
        id BIGSERIAL PRIMARY KEY,
        advertiser_id BIGINT NOT NULL REFERENCES advertisers(id) ON DELETE CASCADE,
        name VARCHAR(255) NOT NULL,
        description TEXT,
        budget_daily DECIMAL(15,2),
        budget_total DECIMAL(15,2),
        spent_today DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
        spent_total DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
        bid_type SMALLINT DEFAULT 1 NOT NULL,
        bid_amount DECIMAL(10,4) DEFAULT 1.0000 NOT NULL,
        start_time TIMESTAMP WITH TIME ZONE,
        end_time TIMESTAMP WITH TIME ZONE,
        freq_cap_daily SMALLINT,
        freq_cap_hourly SMALLINT,
        status SMALLINT DEFAULT 1 NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_campaign_advertiser ON campaigns(advertiser_id);
    CREATE INDEX IF NOT EXISTS idx_campaign_status ON campaigns(status);

    CREATE TABLE IF NOT EXISTS creatives (
        id BIGSERIAL PRIMARY KEY,
        campaign_id BIGINT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
        title VARCHAR(255),
        description TEXT,
        image_url VARCHAR(500),
        video_url VARCHAR(500),
        landing_url VARCHAR(500) NOT NULL,
        creative_type SMALLINT DEFAULT 1 NOT NULL,
        width SMALLINT,
        height SMALLINT,
        status SMALLINT DEFAULT 1 NOT NULL,
        impressions BIGINT DEFAULT 0 NOT NULL,
        clicks BIGINT DEFAULT 0 NOT NULL,
        conversions BIGINT DEFAULT 0 NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_creative_campaign ON creatives(campaign_id);
    CREATE INDEX IF NOT EXISTS idx_creative_status ON creatives(status);

    CREATE TABLE IF NOT EXISTS targeting_rules (
        id BIGSERIAL PRIMARY KEY,
        campaign_id BIGINT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
        rule_type VARCHAR(50) NOT NULL,
        rule_value JSONB NOT NULL,
        is_include BOOLEAN DEFAULT TRUE NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_targeting_campaign ON targeting_rules(campaign_id);
    CREATE INDEX IF NOT EXISTS idx_targeting_type ON targeting_rules(rule_type);

    CREATE TABLE IF NOT EXISTS ad_events (
        id BIGSERIAL PRIMARY KEY,
        request_id VARCHAR(64) NOT NULL,
        campaign_id BIGINT REFERENCES campaigns(id) ON DELETE SET NULL,
        creative_id BIGINT REFERENCES creatives(id) ON DELETE SET NULL,
        event_type SMALLINT NOT NULL,
        event_time TIMESTAMP WITH TIME ZONE NOT NULL,
        user_id VARCHAR(64),
        ip_address VARCHAR(45),
        cost DECIMAL(10,6) DEFAULT 0.000000 NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_event_request ON ad_events(request_id);
    CREATE INDEX IF NOT EXISTS idx_event_campaign ON ad_events(campaign_id);
    CREATE INDEX IF NOT EXISTS idx_event_time ON ad_events(event_time);

    CREATE TABLE IF NOT EXISTS hourly_stats (
        id BIGSERIAL PRIMARY KEY,
        campaign_id BIGINT NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
        creative_id BIGINT REFERENCES creatives(id) ON DELETE SET NULL,
        stat_hour TIMESTAMP WITH TIME ZONE NOT NULL,
        impressions BIGINT DEFAULT 0 NOT NULL,
        clicks BIGINT DEFAULT 0 NOT NULL,
        conversions BIGINT DEFAULT 0 NOT NULL,
        cost DECIMAL(15,4) DEFAULT 0.0000 NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_stats_campaign_hour ON hourly_stats(campaign_id, stat_hour);
    "#;

    sqlx::raw_sql(schema_sql).execute(pool).await?;
    info!("Database schema initialized successfully");
    Ok(())
}

pub async fn seed_sample_data(pool: &PgPool) -> Result<(), AppError> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM advertisers")
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        info!("Sample data already exists, skipping seed");
        return Ok(());
    }

    info!("Seeding initial sample data...");

    // Insert advertiser
    let adv_id: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO advertisers (name, company, balance, status)
        VALUES ($1, $2, $3, 1)
        RETURNING id
        "#,
    )
    .bind("Demo Advertiser")
    .bind("Demo Company Inc.")
    .bind(bigdecimal::BigDecimal::from(10000))
    .fetch_one(pool)
    .await?;

    // Insert campaign
    let camp_id: (i64,) = sqlx::query_as(
        r#"
        INSERT INTO campaigns (advertiser_id, name, budget_daily, budget_total, bid_type, bid_amount, status)
        VALUES ($1, $2, $3, $4, 1, $5, 1)
        RETURNING id
        "#,
    )
    .bind(adv_id.0)
    .bind("Demo Campaign")
    .bind(bigdecimal::BigDecimal::from(100))
    .bind(bigdecimal::BigDecimal::from(1000))
    .bind(bigdecimal::BigDecimal::from(5))
    .fetch_one(pool)
    .await?;

    // Insert creative
    sqlx::query(
        r#"
        INSERT INTO creatives (campaign_id, title, description, image_url, landing_url, creative_type, width, height, status)
        VALUES ($1, $2, $3, $4, $5, 1, 300, 250, 1)
        "#,
    )
    .bind(camp_id.0)
    .bind("Demo Ad")
    .bind("This is a demo advertisement")
    .bind("https://via.placeholder.com/300x250")
    .bind("https://example.com")
    .execute(pool)
    .await?;

    info!("Sample data seeded successfully");
    Ok(())
}
