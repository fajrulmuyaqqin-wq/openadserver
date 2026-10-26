use crate::common::utils::sigmoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenseLayer {
    pub weights: Vec<Vec<f32>>, // [output_dim][input_dim]
    pub bias: Vec<f32>,         // [output_dim]
}

impl DenseLayer {
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        // Xavier/Glorot-like initialization
        let limit = (6.0 / (input_dim + output_dim) as f32).sqrt();
        let mut weights = Vec::with_capacity(output_dim);
        for i in 0..output_dim {
            let mut row = Vec::with_capacity(input_dim);
            for j in 0..input_dim {
                let pseudo_rand =
                    (((i * 31 + j * 17) % 1000) as f32 / 1000.0) * 2.0 * limit - limit;
                row.push(pseudo_rand);
            }
            weights.push(row);
        }
        let bias = vec![0.0f32; output_dim];
        Self { weights, bias }
    }

    pub fn forward(&self, input: &[f32], relu: bool) -> Vec<f32> {
        let mut out = Vec::with_capacity(self.bias.len());
        for (i, row) in self.weights.iter().enumerate() {
            let mut sum = self.bias[i];
            for (j, &val) in input.iter().enumerate() {
                if j < row.len() {
                    sum += row[j] * val;
                }
            }
            if relu {
                out.push(sum.max(0.0));
            } else {
                out.push(sum);
            }
        }
        out
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepFM {
    pub sparse_feature_dims: Vec<usize>,
    pub dense_feature_dim: usize,
    pub embedding_dim: usize,

    // 1st order linear weights: [feature_idx][vocab_idx] -> scalar
    pub linear_weights: Vec<Vec<f32>>,
    pub dense_linear: Vec<f32>,
    pub bias: f32,

    // 2nd order embeddings: [feature_idx][vocab_idx] -> Vec<f32> (length = embedding_dim)
    pub embeddings: Vec<Vec<Vec<f32>>>,

    // DNN layers
    pub dnn_layers: Vec<DenseLayer>,
    pub output_layer: DenseLayer,
}

impl DeepFM {
    pub fn new(
        sparse_feature_dims: Vec<usize>,
        dense_feature_dim: usize,
        embedding_dim: usize,
        dnn_hidden_units: Vec<usize>,
    ) -> Self {
        let mut linear_weights = Vec::new();
        let mut embeddings = Vec::new();

        for &dim in &sparse_feature_dims {
            let vocab_size = dim.max(1);
            linear_weights.push(vec![0.0f32; vocab_size]);

            let limit = (6.0 / (vocab_size + embedding_dim) as f32).sqrt();
            let mut emb_table = Vec::with_capacity(vocab_size);
            for v in 0..vocab_size {
                let mut vec = Vec::with_capacity(embedding_dim);
                for d in 0..embedding_dim {
                    let pseudo = (((v * 23 + d * 47) % 1000) as f32 / 1000.0) * 2.0 * limit - limit;
                    vec.push(pseudo);
                }
                emb_table.push(vec);
            }
            embeddings.push(emb_table);
        }

        let dense_linear = vec![0.0f32; dense_feature_dim];

        // Total input dim to DNN = (num_sparse_features * embedding_dim) + dense_feature_dim
        let dnn_input_dim = sparse_feature_dims.len() * embedding_dim + dense_feature_dim;

        let mut dnn_layers = Vec::new();
        let mut prev_dim = dnn_input_dim;
        for &hidden_dim in &dnn_hidden_units {
            dnn_layers.push(DenseLayer::new(prev_dim, hidden_dim));
            prev_dim = hidden_dim;
        }

        let output_layer = DenseLayer::new(prev_dim, 1);

        Self {
            sparse_feature_dims,
            dense_feature_dim,
            embedding_dim,
            linear_weights,
            dense_linear,
            bias: 0.0,
            embeddings,
            dnn_layers,
            output_layer,
        }
    }

    pub fn forward(&self, sparse_features: &[usize], dense_features: &[f32]) -> f64 {
        // 1. First-order Linear component
        let mut linear_out = self.bias as f64;
        for (i, &feat_idx) in sparse_features.iter().enumerate() {
            if i < self.linear_weights.len() {
                let weights = &self.linear_weights[i];
                let safe_idx = feat_idx % weights.len();
                linear_out += weights[safe_idx] as f64;
            }
        }
        for (j, &val) in dense_features.iter().enumerate() {
            if j < self.dense_linear.len() {
                linear_out += (self.dense_linear[j] as f64) * (val as f64);
            }
        }

        // 2. Second-order FM component
        // FM = 0.5 * sum_{f=1}^k [ (sum_i v_{i,f})^2 - sum_i (v_{i,f}^2) ]
        let mut sum_v = vec![0.0f64; self.embedding_dim];
        let mut sum_sq_v = vec![0.0f64; self.embedding_dim];

        // Also gather concatenated embeddings for DNN
        let mut dnn_input =
            Vec::with_capacity(sparse_features.len() * self.embedding_dim + dense_features.len());

        for (i, &feat_idx) in sparse_features.iter().enumerate() {
            if i < self.embeddings.len() {
                let table = &self.embeddings[i];
                let safe_idx = feat_idx % table.len();
                let emb = &table[safe_idx];

                for (f, &val) in emb.iter().enumerate() {
                    let v = val as f64;
                    sum_v[f] += v;
                    sum_sq_v[f] += v * v;
                    dnn_input.push(val);
                }
            }
        }

        for &dense_val in dense_features {
            dnn_input.push(dense_val);
        }

        let mut fm_out = 0.0f64;
        for f in 0..self.embedding_dim {
            fm_out += 0.5 * (sum_v[f] * sum_v[f] - sum_sq_v[f]);
        }

        // 3. DNN component
        let mut current_dnn = dnn_input;
        for layer in &self.dnn_layers {
            current_dnn = layer.forward(&current_dnn, true);
        }
        let dnn_final = self.output_layer.forward(&current_dnn, false);
        let dnn_out = dnn_final.first().copied().unwrap_or(0.0) as f64;

        // 4. Combine and apply sigmoid
        let logits = linear_out + fm_out + dnn_out;
        sigmoid(logits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepfm_forward() {
        let deepfm = DeepFM::new(vec![50, 100, 20], 3, 8, vec![64, 32]);
        let pred = deepfm.forward(&[5, 12, 3], &[0.1, 0.5, 1.0]);
        assert!(pred > 0.0 && pred < 1.0);
    }
}
