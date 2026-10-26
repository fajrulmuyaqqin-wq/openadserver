use clap::{Parser, Subcommand};
use openadserver::ad_server::services::{AdService, EventService};
use openadserver::common::cache::RedisClient;
use openadserver::common::config::Settings;
use openadserver::common::database::{create_pool, init_schema, seed_sample_data};
use openadserver::common::logger::init_logger;
use openadserver::create_app;
use openadserver::ml_engine::serving::OnlinePredictor;
use openadserver::rec_engine::filter::budget::BudgetFilter;
use openadserver::rec_engine::filter::frequency::FrequencyFilter;
use openadserver::rec_engine::filter::quality::QualityFilter;
use openadserver::rec_engine::ranking::bidding::RankingStrategy;
use openadserver::rec_engine::ranking::predictor::{MLPredictor, StatisticalPredictor};
use openadserver::rec_engine::ranking::reranker::{DiversityReranker, ExplorationReranker};
use openadserver::rec_engine::retrieval::targeting::TargetingRetrieval;
use openadserver::rec_engine::{RecommendationConfig, RecommendationEngine};
use openadserver::AppState;
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(name = "openadserver")]
#[command(about = "High performance computational advertising system in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the ad serving API server (default)
    Serve,
    /// Initialize the PostgreSQL database schema and seed default data
    InitDb,
    /// Seed mock advertisers, campaigns, creatives, and targeting rules
    MockData,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::load();
    init_logger(&settings.logging.level, &settings.logging.format);

    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Serve);

    match command {
        Commands::InitDb => {
            info!("Running database initialization...");
            let pool = create_pool(&settings.database).await?;
            init_schema(&pool).await?;
            seed_sample_data(&pool).await?;
            info!("Database initialized successfully!");
        }
        Commands::MockData => {
            info!("Generating mock advertising data...");
            let pool = create_pool(&settings.database).await?;
            init_schema(&pool).await?;
            seed_sample_data(&pool).await?;
            seed_extra_mock_data(&pool).await?;
            info!("Mock data generated successfully!");
        }
        Commands::Serve => {
            info!(
                app = %settings.app.name,
                version = %settings.app.version,
                env = %settings.app.env,
                "Starting OpenAdServer..."
            );

            // Connect Database
            let pool = create_pool(&settings.database).await?;

            // Auto-initialize schema in dev/debug mode
            if settings.app.debug || settings.app.env == "dev" {
                let _ = init_schema(&pool).await;
                let _ = seed_sample_data(&pool).await;
            }

            // Connect Redis
            let redis = RedisClient::new(&settings.redis).await?;

            // Build Recommendation Engine
            let retrieval = Box::new(TargetingRetrieval::new(pool.clone(), redis.clone()));
            let filters: Vec<Box<dyn openadserver::rec_engine::filter::Filter>> = vec![
                Box::new(BudgetFilter::new(redis.clone(), None)),
                Box::new(FrequencyFilter::new(
                    redis.clone(),
                    Some(settings.frequency.default_daily_cap),
                    Some(settings.frequency.default_hourly_cap),
                )),
                Box::new(QualityFilter::default()),
            ];

            // Build Predictor (DeepFM if configured, otherwise Statistical)
            let predictor: Box<dyn openadserver::rec_engine::ranking::Predictor> =
                if settings.ad_serving.enable_ml_prediction {
                    let online_predictor = OnlinePredictor::with_deepfm(
                        vec![10000, 10, 1000, 10000, 10000, 10], // sparse vocab sizes
                        3,                                       // dense dim
                        settings.ml.embedding_dim,
                        vec![128, 64],
                    );
                    Box::new(MLPredictor::new(online_predictor))
                } else {
                    Box::new(StatisticalPredictor::new(
                        settings.ad_serving.default_pctr,
                        settings.ad_serving.default_pcvr,
                    ))
                };

            let rerankers: Vec<Box<dyn openadserver::rec_engine::ranking::Reranker>> = vec![
                Box::new(DiversityReranker::default()),
                Box::new(ExplorationReranker::default()),
            ];

            let rec_config = RecommendationConfig {
                max_retrieval: 100,
                enable_budget_filter: true,
                enable_frequency_filter: true,
                enable_quality_filter: true,
                enable_ml_prediction: settings.ad_serving.enable_ml_prediction,
                fallback_ctr: settings.ad_serving.default_pctr,
                fallback_cvr: settings.ad_serving.default_pcvr,
                ranking_strategy: RankingStrategy::ECPM,
                min_ecpm: 0.01,
                enable_diversity_rerank: true,
                enable_exploration: true,
                exploration_epsilon: 0.1,
                diversity_lambda: 0.7,
            };

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
                settings: settings.clone(),
            });

            let app = create_app(state);

            let addr = format!("{}:{}", settings.server.host, settings.server.port);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            info!("OpenAdServer listening on http://{}", addr);

            axum::serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await?;
        }
    }

    Ok(())
}

async fn seed_extra_mock_data(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mock_advs = vec![
        ("Gaming Corp", "Epic Games Ltd", 50000.0),
        ("Retail Store", "Super Shop Inc", 100000.0),
        ("Fintech Bank", "Smart Financial LLC", 80000.0),
        ("Edu Tech", "Learning World", 30000.0),
        ("Food Delivery", "Quick Eats Co", 20000.0),
    ];

    for (name, company, balance) in mock_advs {
        let adv_row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO advertisers (name, company, contact_email, balance, status)
            VALUES ($1, $2, $3, $4, 1)
            RETURNING id
            "#,
        )
        .bind(name)
        .bind(company)
        .bind(format!("contact@{}", name.to_lowercase().replace(' ', "")))
        .bind(bigdecimal::BigDecimal::from(balance as i64))
        .fetch_one(pool)
        .await?;

        let camp_row: (i64,) = sqlx::query_as(
            r#"
            INSERT INTO campaigns (advertiser_id, name, budget_daily, budget_total, bid_type, bid_amount, status)
            VALUES ($1, $2, $3, $4, 2, $5, 1)
            RETURNING id
            "#,
        )
        .bind(adv_row.0)
        .bind(format!("Campaign - {}", name))
        .bind(bigdecimal::BigDecimal::from(1000))
        .bind(bigdecimal::BigDecimal::from(20000))
        .bind(bigdecimal::BigDecimal::from(2))
        .fetch_one(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO creatives (campaign_id, title, description, image_url, landing_url, creative_type, width, height, status)
            VALUES ($1, $2, $3, $4, $5, 1, 300, 250, 1)
            "#,
        )
        .bind(camp_row.0)
        .bind(format!("Discover {}", name))
        .bind("Special offer for a limited time only!")
        .bind("https://via.placeholder.com/300x250")
        .bind("https://example.com/landing")
        .execute(pool)
        .await?;
    }

    Ok(())
}
