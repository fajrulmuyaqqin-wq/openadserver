use crate::schemas::internal::{AdCandidate, UserContext};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

pub trait Reranker: Send + Sync {
    fn rerank(
        &self,
        candidates: Vec<AdCandidate>,
        user_context: &UserContext,
        num_results: usize,
    ) -> Vec<AdCandidate>;
}

pub struct DiversityReranker {
    pub lambda_param: f64,
    pub max_per_advertiser: usize,
}

impl Default for DiversityReranker {
    fn default() -> Self {
        Self {
            lambda_param: 0.7,
            max_per_advertiser: 2,
        }
    }
}

impl DiversityReranker {
    pub fn new(lambda_param: f64, max_per_advertiser: usize) -> Self {
        Self {
            lambda_param,
            max_per_advertiser,
        }
    }
}

impl Reranker for DiversityReranker {
    fn rerank(
        &self,
        candidates: Vec<AdCandidate>,
        _user_context: &UserContext,
        num_results: usize,
    ) -> Vec<AdCandidate> {
        if candidates.is_empty() || num_results == 0 {
            return Vec::new();
        }

        let mut result: Vec<AdCandidate> = Vec::new();
        let mut remaining = candidates;
        let mut advertiser_counts: HashMap<i64, usize> = HashMap::new();

        while result.len() < num_results && !remaining.is_empty() {
            let mut best_idx = None;
            let mut best_score = f64::NEG_INFINITY;

            for (i, candidate) in remaining.iter().enumerate() {
                let adv_count = advertiser_counts
                    .get(&candidate.advertiser_id)
                    .copied()
                    .unwrap_or(0);
                if adv_count >= self.max_per_advertiser {
                    continue;
                }

                let relevance = candidate.score;
                let mut diversity_penalty = 0.0;
                for selected in &result {
                    if selected.advertiser_id == candidate.advertiser_id {
                        diversity_penalty += 0.5;
                    }
                    if selected.creative_type == candidate.creative_type {
                        diversity_penalty += 0.2;
                    }
                }

                let mmr_score =
                    self.lambda_param * relevance - (1.0 - self.lambda_param) * diversity_penalty;

                if mmr_score > best_score {
                    best_score = mmr_score;
                    best_idx = Some(i);
                }
            }

            if let Some(idx) = best_idx {
                let selected = remaining.remove(idx);
                *advertiser_counts.entry(selected.advertiser_id).or_insert(0) += 1;
                result.push(selected);
            } else {
                break;
            }
        }

        result
    }
}

pub struct ExplorationReranker {
    pub epsilon: f64,
    pub new_ad_boost: f64,
    pub min_impressions_for_stable: i64,
}

impl Default for ExplorationReranker {
    fn default() -> Self {
        Self {
            epsilon: 0.1,
            new_ad_boost: 1.5,
            min_impressions_for_stable: 1000,
        }
    }
}

impl ExplorationReranker {
    pub fn new(epsilon: f64, new_ad_boost: f64, min_impressions_for_stable: i64) -> Self {
        Self {
            epsilon,
            new_ad_boost,
            min_impressions_for_stable,
        }
    }
}

impl Reranker for ExplorationReranker {
    fn rerank(
        &self,
        mut candidates: Vec<AdCandidate>,
        _user_context: &UserContext,
        _num_results: usize,
    ) -> Vec<AdCandidate> {
        if candidates.is_empty() {
            return candidates;
        }

        let mut rng = rand::thread_rng();

        // Epsilon-greedy: explore by shuffling
        if rng.gen::<f64>() < self.epsilon {
            candidates.shuffle(&mut rng);
            return candidates;
        }

        // Boost new ads with low impressions
        for candidate in &mut candidates {
            let impressions = candidate
                .metadata
                .get("impressions")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if impressions < self.min_impressions_for_stable {
                candidate.score *= self.new_ad_boost;
            }
        }

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates
    }
}

pub struct CompositeReranker {
    rerankers: Vec<Box<dyn Reranker>>,
}

impl CompositeReranker {
    pub fn new(rerankers: Vec<Box<dyn Reranker>>) -> Self {
        Self { rerankers }
    }
}

impl Reranker for CompositeReranker {
    fn rerank(
        &self,
        mut candidates: Vec<AdCandidate>,
        user_context: &UserContext,
        num_results: usize,
    ) -> Vec<AdCandidate> {
        for r in &self.rerankers {
            candidates = r.rerank(candidates, user_context, num_results);
            if candidates.is_empty() {
                break;
            }
        }
        candidates
    }
}
