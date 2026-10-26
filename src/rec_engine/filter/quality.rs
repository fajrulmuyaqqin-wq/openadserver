use crate::common::errors::AppError;
use crate::rec_engine::filter::base::Filter;
use crate::schemas::internal::{AdCandidate, UserContext};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct QualityFilter {
    pub require_image: bool,
    pub require_title: bool,
    pub min_ctr: f64,
    pub min_cvr: f64,
}

impl Default for QualityFilter {
    fn default() -> Self {
        Self {
            require_image: false,
            require_title: false,
            min_ctr: 0.0,
            min_cvr: 0.0,
        }
    }
}

#[async_trait]
impl Filter for QualityFilter {
    async fn filter(
        &self,
        candidates: Vec<AdCandidate>,
        _user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError> {
        let filtered = candidates
            .into_iter()
            .filter(|c| {
                if c.landing_url.is_empty() {
                    return false;
                }
                if self.require_image && c.image_url.as_deref().unwrap_or("").is_empty() {
                    return false;
                }
                if self.require_title && c.title.as_deref().unwrap_or("").is_empty() {
                    return false;
                }
                if self.min_ctr > 0.0 && c.pctr < self.min_ctr {
                    return false;
                }
                if self.min_cvr > 0.0 && c.pcvr < self.min_cvr {
                    return false;
                }
                true
            })
            .collect();
        Ok(filtered)
    }
}

pub struct DiversityFilter {
    pub max_per_advertiser: usize,
}

impl Default for DiversityFilter {
    fn default() -> Self {
        Self {
            max_per_advertiser: 3,
        }
    }
}

#[async_trait]
impl Filter for DiversityFilter {
    async fn filter(
        &self,
        candidates: Vec<AdCandidate>,
        _user_context: &UserContext,
    ) -> Result<Vec<AdCandidate>, AppError> {
        let mut advertiser_counts: HashMap<i64, usize> = HashMap::new();
        let mut result = Vec::new();

        for candidate in candidates {
            let count = advertiser_counts
                .entry(candidate.advertiser_id)
                .or_insert(0);
            if *count < self.max_per_advertiser {
                *count += 1;
                result.push(candidate);
            }
        }

        Ok(result)
    }
}
