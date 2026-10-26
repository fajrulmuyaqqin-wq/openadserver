use crate::common::errors::AppError;
use crate::rec_engine::filter::base::{CompositeFilter, Filter};
use crate::rec_engine::ranking::bidding::{Bidding, RankingStrategy};
use crate::rec_engine::ranking::predictor::Predictor;
use crate::rec_engine::ranking::reranker::{CompositeReranker, Reranker};
use crate::rec_engine::retrieval::base::Retrieval;
use crate::schemas::internal::{AdCandidate, UserContext};
use std::time::Instant;
use tracing::{debug, info};

#[derive(Debug, Clone, Default)]
pub struct RecommendationMetrics {
    pub retrieval_count: usize,
    pub post_filter_count: usize,
    pub post_ranking_count: usize,
    pub final_count: usize,

    pub retrieval_ms: f64,
    pub filter_ms: f64,
    pub prediction_ms: f64,
    pub ranking_ms: f64,
    pub rerank_ms: f64,
    pub total_ms: f64,
}

#[derive(Debug, Clone)]
pub struct RecommendationConfig {
    pub max_retrieval: usize,
    pub enable_budget_filter: bool,
    pub enable_frequency_filter: bool,
    pub enable_quality_filter: bool,
    pub enable_ml_prediction: bool,
    pub fallback_ctr: f64,
    pub fallback_cvr: f64,
    pub ranking_strategy: RankingStrategy,
    pub min_ecpm: f64,
    pub enable_diversity_rerank: bool,
    pub enable_exploration: bool,
    pub exploration_epsilon: f64,
    pub diversity_lambda: f64,
}

impl Default for RecommendationConfig {
    fn default() -> Self {
        Self {
            max_retrieval: 100,
            enable_budget_filter: true,
            enable_frequency_filter: true,
            enable_quality_filter: true,
            enable_ml_prediction: false,
            fallback_ctr: 0.01,
            fallback_cvr: 0.001,
            ranking_strategy: RankingStrategy::ECPM,
            min_ecpm: 0.01,
            enable_diversity_rerank: true,
            enable_exploration: true,
            exploration_epsilon: 0.1,
            diversity_lambda: 0.7,
        }
    }
}

pub struct RecommendationEngine {
    pub config: RecommendationConfig,
    pub retrieval: Box<dyn Retrieval>,
    pub filter: Option<CompositeFilter>,
    pub predictor: Box<dyn Predictor>,
    pub bidding: Bidding,
    pub reranker: Option<CompositeReranker>,
}

impl RecommendationEngine {
    pub fn new(
        config: RecommendationConfig,
        retrieval: Box<dyn Retrieval>,
        filters: Vec<Box<dyn Filter>>,
        predictor: Box<dyn Predictor>,
        rerankers: Vec<Box<dyn Reranker>>,
    ) -> Self {
        let bidding = Bidding::new(config.ranking_strategy, Some(config.min_ecpm));
        let filter = if !filters.is_empty() {
            Some(CompositeFilter::new(filters))
        } else {
            None
        };
        let reranker = if !rerankers.is_empty() {
            Some(CompositeReranker::new(rerankers))
        } else {
            None
        };

        Self {
            config,
            retrieval,
            filter,
            predictor,
            bidding,
            reranker,
        }
    }

    pub async fn recommend(
        &self,
        user_context: &UserContext,
        slot_id: &str,
        num_ads: usize,
    ) -> Result<(Vec<AdCandidate>, RecommendationMetrics), AppError> {
        let mut metrics = RecommendationMetrics::default();
        let total_timer = Instant::now();

        // 1. Retrieval
        let t_retrieval = Instant::now();
        let mut candidates = self
            .retrieval
            .retrieve(user_context, slot_id, self.config.max_retrieval)
            .await?;
        metrics.retrieval_ms = t_retrieval.elapsed().as_secs_f64() * 1000.0;
        metrics.retrieval_count = candidates.len();

        debug!(
            slot_id = %slot_id,
            count = candidates.len(),
            "Retrieved candidates"
        );

        if candidates.is_empty() {
            metrics.total_ms = total_timer.elapsed().as_secs_f64() * 1000.0;
            return Ok((Vec::new(), metrics));
        }

        // 2. Filtering
        let t_filter = Instant::now();
        if let Some(f) = &self.filter {
            candidates = f.filter(candidates, user_context).await?;
        }
        metrics.filter_ms = t_filter.elapsed().as_secs_f64() * 1000.0;
        metrics.post_filter_count = candidates.len();

        if candidates.is_empty() {
            metrics.total_ms = total_timer.elapsed().as_secs_f64() * 1000.0;
            return Ok((Vec::new(), metrics));
        }

        // 3. Prediction
        let t_pred = Instant::now();
        let predictions = self.predictor.predict_batch(user_context, &candidates);
        for (candidate, pred) in candidates.iter_mut().zip(predictions) {
            candidate.pctr = pred.pctr;
            candidate.pcvr = pred.pcvr;
        }
        metrics.prediction_ms = t_pred.elapsed().as_secs_f64() * 1000.0;

        // 4. Ranking
        let t_rank = Instant::now();
        self.bidding.rank(&mut candidates, true);
        metrics.ranking_ms = t_rank.elapsed().as_secs_f64() * 1000.0;
        metrics.post_ranking_count = candidates.len();

        // 5. Re-ranking
        let t_rerank = Instant::now();
        if let Some(r) = &self.reranker {
            candidates = r.rerank(candidates, user_context, num_ads * 2);
        }
        metrics.rerank_ms = t_rerank.elapsed().as_secs_f64() * 1000.0;

        // 6. Slice to final num_ads
        candidates.truncate(num_ads);
        metrics.final_count = candidates.len();
        metrics.total_ms = total_timer.elapsed().as_secs_f64() * 1000.0;

        info!(
            retrieval = metrics.retrieval_count,
            final_count = metrics.final_count,
            total_ms = metrics.total_ms,
            "Recommendation completed"
        );

        Ok((candidates, metrics))
    }

    pub async fn refresh_cache(&self) -> Result<(), AppError> {
        self.retrieval.refresh().await?;
        Ok(())
    }
}
