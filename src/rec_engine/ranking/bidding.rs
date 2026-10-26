use crate::models::BidType;
use crate::schemas::internal::AdCandidate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(i32)]
pub enum RankingStrategy {
    #[default]
    ECPM = 1,
    Revenue = 2,
    Engagement = 3,
    Conversion = 4,
    Hybrid = 5,
}

pub struct Bidding {
    pub strategy: RankingStrategy,
    pub min_ecpm: f64,
}

impl Default for Bidding {
    fn default() -> Self {
        Self {
            strategy: RankingStrategy::ECPM,
            min_ecpm: 0.01,
        }
    }
}

impl Bidding {
    pub fn new(strategy: RankingStrategy, min_ecpm: Option<f64>) -> Self {
        Self {
            strategy,
            min_ecpm: min_ecpm.unwrap_or(0.01),
        }
    }

    pub fn calculate_ecpm(&self, candidate: &AdCandidate) -> f64 {
        let bid = candidate.bid;
        let pctr = candidate.pctr.max(0.0001);
        let pcvr = candidate.pcvr.max(0.0001);

        let ecpm = match BidType::from(candidate.bid_type) {
            BidType::CPM => bid,
            BidType::CPC => bid * pctr * 1000.0,
            BidType::CPA => bid * pctr * pcvr * 1000.0,
            BidType::OCPM => bid * pctr * 1000.0,
        };

        ecpm.max(self.min_ecpm)
    }

    pub fn calculate_score(&self, candidate: &AdCandidate) -> f64 {
        let ecpm = candidate.ecpm;
        let pctr = candidate.pctr;
        let pcvr = candidate.pcvr;

        match self.strategy {
            RankingStrategy::ECPM => ecpm,
            RankingStrategy::Revenue => {
                let quality_factor = (pctr / 0.01).min(2.0);
                ecpm * quality_factor
            }
            RankingStrategy::Engagement => ecpm * (1.0 + pctr * 10.0),
            RankingStrategy::Conversion => ecpm * (1.0 + pcvr * 100.0),
            RankingStrategy::Hybrid => {
                let ctr_factor = 1.0 + pctr * 5.0;
                let cvr_factor = 1.0 + pcvr * 20.0;
                ecpm * ctr_factor * cvr_factor
            }
        }
    }

    pub fn rank(&self, candidates: &mut [AdCandidate], apply_ecpm: bool) {
        for candidate in candidates.iter_mut() {
            if apply_ecpm {
                candidate.ecpm = self.calculate_ecpm(candidate);
            }
            candidate.score = self.calculate_score(candidate);
        }

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

pub struct SecondPriceAuction {
    pub increment: f64,
}

impl Default for SecondPriceAuction {
    fn default() -> Self {
        Self { increment: 0.01 }
    }
}

impl SecondPriceAuction {
    pub fn new(increment: f64) -> Self {
        Self { increment }
    }

    pub fn run_auction(&self, candidates: &[AdCandidate]) -> (Option<AdCandidate>, f64) {
        if candidates.is_empty() {
            return (None, 0.0);
        }

        let winner = candidates[0].clone();
        if candidates.len() == 1 {
            return (Some(winner), self.increment);
        }

        let second_price = candidates[1].ecpm;
        let price = second_price + self.increment;
        (Some(winner), price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_ecpm() {
        let bidding = Bidding::default();
        let mut candidate = AdCandidate::default();
        candidate.bid = 2.0;
        candidate.bid_type = 2; // CPC
        candidate.pctr = 0.02;

        let ecpm = bidding.calculate_ecpm(&candidate);
        // 2.0 * 0.02 * 1000 = 40.0
        assert!((ecpm - 40.0).abs() < 1e-4);
    }

    #[test]
    fn test_ranking() {
        let bidding = Bidding::default();
        let mut c1 = AdCandidate::default();
        c1.bid = 1.0;
        c1.bid_type = 1; // CPM -> ecpm = 1.0

        let mut c2 = AdCandidate::default();
        c2.bid = 5.0;
        c2.bid_type = 1; // CPM -> ecpm = 5.0

        let mut list = vec![c1, c2];
        bidding.rank(&mut list, true);

        assert_eq!(list[0].bid, 5.0);
        assert_eq!(list[1].bid, 1.0);
    }
}
