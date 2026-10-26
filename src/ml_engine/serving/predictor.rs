use crate::common::utils::hash_string;
use crate::ml_engine::models::deepfm::DeepFM;
use crate::ml_engine::models::lr::LogisticRegression;
use crate::schemas::internal::{AdCandidate, PredictionResult, UserContext};
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub enum ModelType {
    DeepFM(DeepFM),
    LR(LogisticRegression),
    Default,
}

#[derive(Clone)]
pub struct OnlinePredictor {
    model: Arc<ModelType>,
    pub fallback_ctr: f64,
    pub fallback_cvr: f64,
    pub model_version: String,
}

impl Default for OnlinePredictor {
    fn default() -> Self {
        Self::new(ModelType::Default, 0.01, 0.001, "statistical_v1")
    }
}

impl OnlinePredictor {
    pub fn new(
        model_type: ModelType,
        fallback_ctr: f64,
        fallback_cvr: f64,
        model_version: &str,
    ) -> Self {
        Self {
            model: Arc::new(model_type),
            fallback_ctr,
            fallback_cvr,
            model_version: model_version.to_string(),
        }
    }

    pub fn with_deepfm(
        sparse_dims: Vec<usize>,
        dense_dim: usize,
        emb_dim: usize,
        hidden: Vec<usize>,
    ) -> Self {
        let deepfm = DeepFM::new(sparse_dims, dense_dim, emb_dim, hidden);
        Self::new(ModelType::DeepFM(deepfm), 0.01, 0.001, "deepfm_v1")
    }

    pub fn with_lr(sparse_dims: Vec<usize>, dense_dim: usize) -> Self {
        let lr = LogisticRegression::new(sparse_dims, dense_dim);
        Self::new(ModelType::LR(lr), 0.01, 0.001, "lr_v1")
    }

    pub fn predict_candidate(
        &self,
        user_context: &UserContext,
        candidate: &AdCandidate,
    ) -> PredictionResult {
        let start = Instant::now();

        let (pctr, pcvr) = match self.model.as_ref() {
            ModelType::DeepFM(deepfm) => {
                let (sparse_feats, dense_feats) = self.extract_features(user_context, candidate);
                let pctr = deepfm.forward(&sparse_feats, &dense_feats);
                let pcvr = (pctr * 0.1).max(0.0001);
                (pctr, pcvr)
            }
            ModelType::LR(lr) => {
                let (sparse_feats, dense_feats) = self.extract_features(user_context, candidate);
                let pctr = lr.forward(&sparse_feats, &dense_feats);
                let pcvr = (pctr * 0.1).max(0.0001);
                (pctr, pcvr)
            }
            ModelType::Default => {
                // Statistical fallback using historical CTR if available
                let impressions = candidate
                    .metadata
                    .get("impressions")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let clicks = candidate
                    .metadata
                    .get("clicks")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let conversions = candidate
                    .metadata
                    .get("conversions")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let pctr = if impressions > 0 {
                    ((clicks as f64) + 100.0 * self.fallback_ctr) / ((impressions as f64) + 100.0)
                } else {
                    self.fallback_ctr
                };

                let pcvr = if clicks > 0 {
                    ((conversions as f64) + 50.0 * self.fallback_cvr) / ((clicks as f64) + 50.0)
                } else {
                    self.fallback_cvr
                };

                (pctr, pcvr)
            }
        };

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        PredictionResult {
            campaign_id: candidate.campaign_id,
            creative_id: candidate.creative_id,
            pctr,
            pcvr,
            model_version: self.model_version.clone(),
            latency_ms: elapsed,
        }
    }

    pub fn predict_batch(
        &self,
        user_context: &UserContext,
        candidates: &[AdCandidate],
    ) -> Vec<PredictionResult> {
        candidates
            .iter()
            .map(|c| self.predict_candidate(user_context, c))
            .collect()
    }

    fn extract_features(
        &self,
        user_context: &UserContext,
        candidate: &AdCandidate,
    ) -> (Vec<usize>, Vec<f32>) {
        // Hash features into indices
        let user_idx = user_context
            .user_id
            .as_deref()
            .map(|u| (hash_to_u64(u) % 10000) as usize)
            .unwrap_or(0);
        let os_idx = match user_context.os.to_lowercase().as_str() {
            "android" => 1,
            "ios" => 2,
            _ => 0,
        };
        let country_idx = (hash_to_u64(&user_context.country) % 1000) as usize;
        let campaign_idx = (candidate.campaign_id as usize) % 10000;
        let creative_idx = (candidate.creative_id as usize) % 10000;
        let creative_type_idx = candidate.creative_type as usize;

        let sparse = vec![
            user_idx,
            os_idx,
            country_idx,
            campaign_idx,
            creative_idx,
            creative_type_idx,
        ];

        // Dense features
        let age_val = user_context.age.unwrap_or(25) as f32 / 100.0;
        let bid_val = candidate.bid as f32;
        let target_score = candidate.targeting_score as f32;

        let dense = vec![age_val, bid_val, target_score];

        (sparse, dense)
    }
}

fn hash_to_u64(s: &str) -> u64 {
    let hex = hash_string(s);
    let slice = if hex.len() >= 8 { &hex[..8] } else { &hex };
    u64::from_str_radix(slice, 16).unwrap_or(0)
}
