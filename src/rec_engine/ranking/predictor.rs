use crate::ml_engine::serving::OnlinePredictor;
use crate::schemas::internal::{AdCandidate, PredictionResult, UserContext};

pub trait Predictor: Send + Sync {
    fn predict(&self, user_context: &UserContext, candidate: &AdCandidate) -> PredictionResult;

    fn predict_batch(
        &self,
        user_context: &UserContext,
        candidates: &[AdCandidate],
    ) -> Vec<PredictionResult> {
        candidates
            .iter()
            .map(|c| self.predict(user_context, c))
            .collect()
    }
}

pub struct StatisticalPredictor {
    pub default_ctr: f64,
    pub default_cvr: f64,
    pub smoothing_impressions: f64,
    pub smoothing_clicks: f64,
}

impl Default for StatisticalPredictor {
    fn default() -> Self {
        Self {
            default_ctr: 0.01,
            default_cvr: 0.001,
            smoothing_impressions: 1000.0,
            smoothing_clicks: 100.0,
        }
    }
}

impl StatisticalPredictor {
    pub fn new(default_ctr: f64, default_cvr: f64) -> Self {
        Self {
            default_ctr,
            default_cvr,
            ..Default::default()
        }
    }
}

impl Predictor for StatisticalPredictor {
    fn predict(&self, _user_context: &UserContext, candidate: &AdCandidate) -> PredictionResult {
        let impressions = candidate
            .metadata
            .get("impressions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as f64;
        let clicks = candidate
            .metadata
            .get("clicks")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as f64;
        let conversions = candidate
            .metadata
            .get("conversions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as f64;

        let smoothed_ctr = (clicks + self.smoothing_clicks * self.default_ctr)
            / (impressions + self.smoothing_clicks);

        let smoothed_cvr = if clicks > 0.0 {
            (conversions + self.smoothing_clicks * self.default_cvr)
                / (clicks + self.smoothing_clicks)
        } else {
            self.default_cvr
        };

        PredictionResult {
            campaign_id: candidate.campaign_id,
            creative_id: candidate.creative_id,
            pctr: smoothed_ctr,
            pcvr: smoothed_cvr,
            model_version: "statistical_v1".to_string(),
            latency_ms: 0.01,
        }
    }
}

pub struct MLPredictor {
    online_predictor: OnlinePredictor,
}

impl MLPredictor {
    pub fn new(online_predictor: OnlinePredictor) -> Self {
        Self { online_predictor }
    }
}

impl Predictor for MLPredictor {
    fn predict(&self, user_context: &UserContext, candidate: &AdCandidate) -> PredictionResult {
        self.online_predictor
            .predict_candidate(user_context, candidate)
    }

    fn predict_batch(
        &self,
        user_context: &UserContext,
        candidates: &[AdCandidate],
    ) -> Vec<PredictionResult> {
        self.online_predictor
            .predict_batch(user_context, candidates)
    }
}

pub struct EnsemblePredictor {
    predictors: Vec<(Box<dyn Predictor>, f64)>,
}

impl EnsemblePredictor {
    pub fn new(predictors: Vec<(Box<dyn Predictor>, f64)>) -> Self {
        let total_weight: f64 = predictors.iter().map(|(_, w)| *w).sum();
        let normalized = if total_weight > 0.0 {
            predictors
                .into_iter()
                .map(|(p, w)| (p, w / total_weight))
                .collect()
        } else {
            predictors
        };
        Self {
            predictors: normalized,
        }
    }
}

impl Predictor for EnsemblePredictor {
    fn predict(&self, user_context: &UserContext, candidate: &AdCandidate) -> PredictionResult {
        let mut weighted_ctr = 0.0;
        let mut weighted_cvr = 0.0;
        let mut max_latency = 0.0;

        for (predictor, weight) in &self.predictors {
            let res = predictor.predict(user_context, candidate);
            weighted_ctr += res.pctr * weight;
            weighted_cvr += res.pcvr * weight;
            if res.latency_ms > max_latency {
                max_latency = res.latency_ms;
            }
        }

        PredictionResult {
            campaign_id: candidate.campaign_id,
            creative_id: candidate.creative_id,
            pctr: weighted_ctr,
            pcvr: weighted_cvr,
            model_version: "ensemble_v1".to_string(),
            latency_ms: max_latency,
        }
    }
}
