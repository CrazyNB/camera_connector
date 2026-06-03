use serde::{Deserialize, Serialize};

use super::{
    QualityAnalysisStatus, QualityScore, SelectionRecommendation, SelectionRecommendationStatus,
    StrategyProfile,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewQueueCount {
    pub queue: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewQueueSummary {
    pub project_id: String,
    pub strategy_profile_id: String,
    pub total_units: usize,
    pub pending_count: usize,
    pub unconfirmed_best_count: usize,
    pub needs_review_count: usize,
    pub low_score_candidate_count: usize,
    pub near_duplicate_count: usize,
    pub unsupported_count: usize,
    pub user_overridden_count: usize,
    pub queues: Vec<ReviewQueueCount>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewUnitFlags {
    pub pending: bool,
    pub unconfirmed_best: bool,
    pub needs_review: bool,
    pub low_score_candidate: bool,
    pub near_duplicate: bool,
    pub unsupported: bool,
    pub user_overridden: bool,
}

impl ReviewUnitFlags {
    pub fn matches_queue(&self, queue: &str) -> bool {
        match normalized_review_queue_key(queue).as_str() {
            "all" => true,
            "pending" => self.pending,
            "unconfirmed_best" => self.unconfirmed_best,
            "needs_review" => self.needs_review,
            "low_score_candidates" => self.low_score_candidate,
            "near_duplicates" => self.near_duplicate,
            "unsupported" => self.unsupported,
            "user_overridden" => self.user_overridden,
            _ => false,
        }
    }
}

pub fn normalized_review_queue_key(queue: &str) -> String {
    let value = queue.trim().to_ascii_lowercase();
    match value.as_str() {
        "" | "all" | "review" | "review_queue" => "all".to_string(),
        "ready" | "best" | "unconfirmed" | "unconfirmed_best" => "unconfirmed_best".to_string(),
        "needs_review" | "review_required" | "manual_review" => "needs_review".to_string(),
        "low_score" | "low_score_candidate" | "low_score_candidates" => {
            "low_score_candidates".to_string()
        }
        "near_duplicate" | "near_duplicates" => "near_duplicates".to_string(),
        "unsupported" => "unsupported".to_string(),
        "user_overridden" | "overridden" => "user_overridden".to_string(),
        "pending" | "queued" => "pending".to_string(),
        "model_select" | "model_selects" | "algorithm_select" | "algorithm_selects" => {
            "model_selects".to_string()
        }
        "favorite" | "favorites" => "favorites".to_string(),
        "flag" | "flagged" | "marked" => "flagged".to_string(),
        "quality_risk" | "technical_risk" | "risk" => "quality_risk".to_string(),
        "pending_analysis" | "analysis_pending" => "pending_analysis".to_string(),
        _ => value,
    }
}

pub fn review_unit_flags(
    recommendation: Option<&SelectionRecommendation>,
    scores: &[QualityScore],
    profile: &StrategyProfile,
    user_override_state: Option<&str>,
    recommendation_pending: bool,
) -> ReviewUnitFlags {
    let status = recommendation.map(|value| value.status);
    let quality_pending = scores.is_empty()
        || scores.iter().any(|score| {
            matches!(
                score.analysis_status,
                QualityAnalysisStatus::Pending
                    | QualityAnalysisStatus::Analyzing
                    | QualityAnalysisStatus::Stale
            )
        });
    let user_overridden = user_override_state
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || status == Some(SelectionRecommendationStatus::UserOverridden);
    let low_score_suppressed = matches!(
        status,
        Some(
            SelectionRecommendationStatus::LowScoreHidden
                | SelectionRecommendationStatus::KeptAll
                | SelectionRecommendationStatus::Cleared
        )
    );
    let unsupported = scores
        .iter()
        .any(|score| score.analysis_status == QualityAnalysisStatus::Unsupported);
    let low_score = !low_score_suppressed
        && (recommendation
            .map(|value| !value.low_score_asset_group_ids.is_empty())
            .unwrap_or(false)
            || scores.iter().any(|score| {
                score.analysis_status == QualityAnalysisStatus::Ready
                    && (score.overall < profile.flag_if_overall_below
                        || score.sharpness.value < profile.reject_if_sharpness_below)
            }));
    let near_duplicate = recommendation
        .map(|value| !value.near_duplicate_asset_group_ids.is_empty())
        .unwrap_or(false);
    let pending = status == Some(SelectionRecommendationStatus::Pending)
        || recommendation_pending
        || (recommendation.is_none() && quality_pending);
    let unconfirmed_best = status == Some(SelectionRecommendationStatus::Ready)
        && recommendation
            .and_then(|value| value.best_asset_group_id.as_ref())
            .is_some()
        && !user_overridden;
    let needs_review = unsupported
        || matches!(
            status,
            Some(
                SelectionRecommendationStatus::NeedsReview
                    | SelectionRecommendationStatus::Unsupported
                    | SelectionRecommendationStatus::Failed
                    | SelectionRecommendationStatus::Stale
                    | SelectionRecommendationStatus::Cleared
            )
        );

    ReviewUnitFlags {
        pending,
        unconfirmed_best,
        needs_review,
        low_score_candidate: low_score,
        near_duplicate,
        unsupported,
        user_overridden,
    }
}

impl ReviewQueueSummary {
    pub fn empty(project_id: impl Into<String>, strategy_profile_id: impl Into<String>) -> Self {
        let mut summary = Self {
            project_id: project_id.into(),
            strategy_profile_id: strategy_profile_id.into(),
            total_units: 0,
            pending_count: 0,
            unconfirmed_best_count: 0,
            needs_review_count: 0,
            low_score_candidate_count: 0,
            near_duplicate_count: 0,
            unsupported_count: 0,
            user_overridden_count: 0,
            queues: Vec::new(),
        };
        summary.refresh_queues();
        summary
    }

    pub fn add_unit(
        &mut self,
        recommendation: Option<&SelectionRecommendation>,
        scores: &[QualityScore],
        profile: &StrategyProfile,
        user_override_state: Option<&str>,
        recommendation_pending: bool,
    ) {
        self.total_units = self.total_units.saturating_add(1);
        let flags = review_unit_flags(
            recommendation,
            scores,
            profile,
            user_override_state,
            recommendation_pending,
        );

        if flags.pending {
            self.pending_count = self.pending_count.saturating_add(1);
        }
        if flags.unconfirmed_best {
            self.unconfirmed_best_count = self.unconfirmed_best_count.saturating_add(1);
        }
        if flags.needs_review {
            self.needs_review_count = self.needs_review_count.saturating_add(1);
        }
        if flags.low_score_candidate {
            self.low_score_candidate_count = self.low_score_candidate_count.saturating_add(1);
        }
        if flags.near_duplicate {
            self.near_duplicate_count = self.near_duplicate_count.saturating_add(1);
        }
        if flags.unsupported {
            self.unsupported_count = self.unsupported_count.saturating_add(1);
        }
        if flags.user_overridden {
            self.user_overridden_count = self.user_overridden_count.saturating_add(1);
        }
        self.refresh_queues();
    }

    pub fn queue_count(&self, queue: &str) -> Option<usize> {
        self.queues
            .iter()
            .find(|entry| entry.queue == queue)
            .map(|entry| entry.count)
    }

    fn refresh_queues(&mut self) {
        self.queues = vec![
            ReviewQueueCount {
                queue: "pending".to_string(),
                count: self.pending_count,
            },
            ReviewQueueCount {
                queue: "unconfirmed_best".to_string(),
                count: self.unconfirmed_best_count,
            },
            ReviewQueueCount {
                queue: "needs_review".to_string(),
                count: self.needs_review_count,
            },
            ReviewQueueCount {
                queue: "low_score_candidates".to_string(),
                count: self.low_score_candidate_count,
            },
            ReviewQueueCount {
                queue: "near_duplicates".to_string(),
                count: self.near_duplicate_count,
            },
            ReviewQueueCount {
                queue: "unsupported".to_string(),
                count: self.unsupported_count,
            },
            ReviewQueueCount {
                queue: "user_overridden".to_string(),
                count: self.user_overridden_count,
            },
        ];
    }
}
