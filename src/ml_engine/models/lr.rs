use crate::common::utils::sigmoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticRegression {
    pub sparse_feature_dims: Vec<usize>,
    pub dense_feature_dim: usize,
    pub sparse_weights: Vec<Vec<f32>>, // [feature_idx][vocab_idx] -> scalar weight
    pub dense_weights: Vec<f32>,       // [dense_idx] -> scalar weight
    pub bias: f32,
}

impl LogisticRegression {
    pub fn new(sparse_feature_dims: Vec<usize>, dense_feature_dim: usize) -> Self {
        let sparse_weights = sparse_feature_dims
            .iter()
            .map(|&dim| vec![0.0f32; dim.max(1)])
            .collect();
        let dense_weights = vec![0.0f32; dense_feature_dim];

        Self {
            sparse_feature_dims,
            dense_feature_dim,
            sparse_weights,
            dense_weights,
            bias: 0.0,
        }
    }

    pub fn forward(&self, sparse_features: &[usize], dense_features: &[f32]) -> f64 {
        let mut logits = self.bias as f64;

        for (i, &feat_idx) in sparse_features.iter().enumerate() {
            if i < self.sparse_weights.len() {
                let weights = &self.sparse_weights[i];
                let safe_idx = feat_idx % weights.len();
                logits += weights[safe_idx] as f64;
            }
        }

        for (j, &val) in dense_features.iter().enumerate() {
            if j < self.dense_weights.len() {
                logits += (self.dense_weights[j] as f64) * (val as f64);
            }
        }

        sigmoid(logits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lr_forward() {
        let mut lr = LogisticRegression::new(vec![10, 20], 2);
        lr.bias = 0.0;
        lr.sparse_weights[0][1] = 0.5;
        lr.dense_weights[0] = 0.2;

        let pred = lr.forward(&[1, 5], &[1.0, 0.0]);
        assert!(pred > 0.5);
    }
}
