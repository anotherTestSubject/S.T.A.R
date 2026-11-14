use crate::novelty::CandidateScore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParetoRole {
    BestPhysicalMatch,
    TopNovelDiscovery,
    HighestConfidence,
    ParetoOptimal,
    Dominated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedCandidate {
    pub score: CandidateScore,
    pub is_pareto_optimal: bool,
    pub role: ParetoRole,
    pub pareto_rank: usize,
}

fn dominates(a: &CandidateScore, b: &CandidateScore) -> bool {
    let not_worse = a.similarity >= b.similarity
        && a.combined_novelty >= b.combined_novelty
        && a.confidence >= b.confidence;

    let strictly_better = a.similarity > b.similarity
        || a.combined_novelty > b.combined_novelty
        || a.confidence > b.confidence;

    not_worse && strictly_better
}

pub fn rank_with_pareto(candidates: &[CandidateScore]) -> Vec<RankedCandidate> {
    let n = candidates.len();
    if n == 0 {
        return Vec::new();
    }

    let mut is_dominated = vec![false; n];

    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            if dominates(&candidates[j], &candidates[i]) {
                is_dominated[i] = true;
                break;
            }
        }
    }

    let mut best_sim_idx: Option<usize> = None;
    let mut best_nov_idx: Option<usize> = None;
    let mut best_conf_idx: Option<usize> = None;

    for i in 0..n {
        if !is_dominated[i] {
            if best_sim_idx.map_or(true, |idx| candidates[i].similarity > candidates[idx].similarity) {
                best_sim_idx = Some(i);
            }
            if !candidates[i].is_known_analog {
                if best_nov_idx
                    .map_or(true, |idx| candidates[i].combined_novelty > candidates[idx].combined_novelty)
                {
                    best_nov_idx = Some(i);
                }
            }
            if best_conf_idx.map_or(true, |idx| candidates[i].confidence > candidates[idx].confidence) {
                best_conf_idx = Some(i);
            }
        }
    }

    let mut ranked: Vec<RankedCandidate> = candidates
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let is_opt = !is_dominated[i];
            let role = if Some(i) == best_sim_idx {
                ParetoRole::BestPhysicalMatch
            } else if Some(i) == best_nov_idx {
                ParetoRole::TopNovelDiscovery
            } else if Some(i) == best_conf_idx {
                ParetoRole::HighestConfidence
            } else if is_opt {
                ParetoRole::ParetoOptimal
            } else {
                ParetoRole::Dominated
            };

            RankedCandidate {
                score: c.clone(),
                is_pareto_optimal: is_opt,
                role,
                pareto_rank: if is_opt { 1 } else { 2 },
            }
        })
        .collect();

    ranked.sort_by(|a, b| {
        a.pareto_rank.cmp(&b.pareto_rank).then_with(|| {
            b.score
                .similarity
                .partial_cmp(&a.score.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    ranked
}
