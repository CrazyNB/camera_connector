use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    QualityAnalysisStatus, QualityScore, TechnicalAssessment,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyWeights {
    pub sharpness: f64,
    pub exposure: f64,
    pub composition: f64,
    pub highlight_clipping_penalty: f64,
    pub shadow_clipping_penalty: f64,
    pub diversity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyProfile {
    pub profile_id: String,
    pub name: String,
    pub built_in: bool,
    pub strategy_version: String,
    pub burst_window_ms: i64,
    pub min_group_size: usize,
    pub weights: StrategyWeights,
    pub reject_if_sharpness_below: f64,
    pub flag_if_overall_below: f64,
    pub near_duplicate_similarity_above: f64,
    pub max_llm_candidates_per_group: usize,
    pub auto_delete: bool,
    pub auto_hide_low_score: bool,
    pub mark_best: bool,
    pub keep_raw_pairs: bool,
    pub llm_enabled: bool,
    pub updated_at_ms: i64,
}

impl StrategyProfile {
    pub fn general() -> Self {
        Self {
            profile_id: "general".to_string(),
            name: "General".to_string(),
            built_in: true,
            strategy_version: "strategy-v1".to_string(),
            burst_window_ms: 1200,
            min_group_size: 2,
            weights: StrategyWeights {
                sharpness: 0.40,
                exposure: 0.22,
                composition: 0.12,
                highlight_clipping_penalty: -0.14,
                shadow_clipping_penalty: -0.08,
                diversity: 0.04,
            },
            reject_if_sharpness_below: 0.25,
            flag_if_overall_below: 0.40,
            near_duplicate_similarity_above: 0.92,
            max_llm_candidates_per_group: 5,
            auto_delete: false,
            auto_hide_low_score: false,
            mark_best: true,
            keep_raw_pairs: true,
            llm_enabled: false,
            updated_at_ms: 0,
        }
    }

    pub fn built_in_profiles() -> Vec<Self> {
        vec![
            Self::general(),
            Self::conservative(),
            Self::portrait(),
            Self::action(),
            Self::landscape(),
        ]
    }

    pub fn conservative() -> Self {
        Self {
            profile_id: "conservative".to_string(),
            name: "Conservative".to_string(),
            weights: StrategyWeights {
                sharpness: 0.46,
                exposure: 0.24,
                composition: 0.08,
                highlight_clipping_penalty: -0.18,
                shadow_clipping_penalty: -0.10,
                diversity: 0.02,
            },
            reject_if_sharpness_below: 0.32,
            flag_if_overall_below: 0.46,
            auto_hide_low_score: true,
            ..Self::general()
        }
    }

    pub fn portrait() -> Self {
        Self {
            profile_id: "portrait".to_string(),
            name: "Portrait".to_string(),
            weights: StrategyWeights {
                sharpness: 0.38,
                exposure: 0.24,
                composition: 0.12,
                highlight_clipping_penalty: -0.10,
                shadow_clipping_penalty: -0.06,
                diversity: 0.04,
            },
            reject_if_sharpness_below: 0.24,
            flag_if_overall_below: 0.38,
            ..Self::general()
        }
    }

    pub fn action() -> Self {
        Self {
            profile_id: "action".to_string(),
            name: "Action".to_string(),
            burst_window_ms: 700,
            weights: StrategyWeights {
                sharpness: 0.52,
                exposure: 0.18,
                composition: 0.07,
                highlight_clipping_penalty: -0.11,
                shadow_clipping_penalty: -0.08,
                diversity: 0.04,
            },
            reject_if_sharpness_below: 0.35,
            flag_if_overall_below: 0.44,
            ..Self::general()
        }
    }

    pub fn landscape() -> Self {
        Self {
            profile_id: "landscape".to_string(),
            name: "Landscape".to_string(),
            burst_window_ms: 1800,
            weights: StrategyWeights {
                sharpness: 0.34,
                exposure: 0.28,
                composition: 0.12,
                highlight_clipping_penalty: -0.18,
                shadow_clipping_penalty: -0.06,
                diversity: 0.04,
            },
            reject_if_sharpness_below: 0.22,
            flag_if_overall_below: 0.40,
            ..Self::general()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionSource {
    LocalRule,
    LocalCv,
    LocalStub,
    Llm,
    UserOverride,
}

impl SelectionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalRule => "local_rule",
            Self::LocalCv => "local_cv",
            Self::LocalStub => "local_stub",
            Self::Llm => "llm",
            Self::UserOverride => "user_override",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "local_cv" => Self::LocalCv,
            "local_stub" => Self::LocalStub,
            "llm" => Self::Llm,
            "user_override" => Self::UserOverride,
            _ => Self::LocalRule,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionRecommendationStatus {
    Pending,
    Ready,
    Accepted,
    Stale,
    Failed,
    Unsupported,
    NeedsReview,
    UserOverridden,
    Cleared,
    KeptAll,
    LowScoreHidden,
    NoSelection,
}

impl SelectionRecommendationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Accepted => "accepted",
            Self::Stale => "stale",
            Self::Failed => "failed",
            Self::Unsupported => "unsupported",
            Self::NeedsReview => "needs_review",
            Self::UserOverridden => "user_overridden",
            Self::Cleared => "cleared",
            Self::KeptAll => "kept_all",
            Self::LowScoreHidden => "low_score_hidden",
            Self::NoSelection => "no_selection",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "ready" => Self::Ready,
            "accepted" => Self::Accepted,
            "stale" => Self::Stale,
            "failed" => Self::Failed,
            "unsupported" => Self::Unsupported,
            "needs_review" => Self::NeedsReview,
            "user_overridden" => Self::UserOverridden,
            "cleared" => Self::Cleared,
            "kept_all" => Self::KeptAll,
            "low_score_hidden" => Self::LowScoreHidden,
            "no_selection" => Self::NoSelection,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionRecommendationScope {
    BurstGroup,
    Project,
}

impl SelectionRecommendationScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BurstGroup => "burst_group",
            Self::Project => "project",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "project" => Self::Project,
            _ => Self::BurstGroup,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopedSelectionRecommendation {
    pub recommendation_id: String,
    pub run_id: Option<String>,
    pub scope: SelectionRecommendationScope,
    pub project_id: String,
    pub subject_id: String,
    pub selected_asset_group_ids: Vec<String>,
    pub candidate_asset_group_ids: Vec<String>,
    pub rejected_asset_group_ids: Vec<String>,
    pub source: SelectionSource,
    pub status: SelectionRecommendationStatus,
    pub confidence: f64,
    pub reason: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionRecommendation {
    pub recommendation_id: String,
    pub burst_group_id: String,
    pub strategy_profile_id: String,
    pub scorer_version: String,
    pub strategy_version: String,
    pub grouping_version: i64,
    pub best_asset_group_id: Option<String>,
    pub alternate_asset_group_ids: Vec<String>,
    pub low_score_asset_group_ids: Vec<String>,
    pub near_duplicate_asset_group_ids: Vec<String>,
    pub source: SelectionSource,
    pub status: SelectionRecommendationStatus,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub llm_review_id: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

pub fn recommend_burst_group_from_model_evaluations(
    project_id: &str,
    burst_group_id: &str,
    evaluations: &[ModelEvaluation],
    _assessments: &[TechnicalAssessment],
    now_ms: i64,
) -> ScopedSelectionRecommendation {
    let source = model_recommendation_source(evaluations);
    let mut selectable = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.status == ModelEvaluationStatus::Ready
                && evaluation.selectable
                && evaluation.tier != ModelEvaluationTier::Reject
        })
        .collect::<Vec<_>>();
    selectable.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.asset_group_id.cmp(&right.asset_group_id))
    });
    let rejected_asset_group_ids = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.status != ModelEvaluationStatus::Ready
                || !evaluation.selectable
                || evaluation.tier == ModelEvaluationTier::Reject
        })
        .map(|evaluation| evaluation.asset_group_id.clone())
        .collect::<Vec<_>>();
    let Some(best) = selectable.first().copied() else {
        return ScopedSelectionRecommendation {
            recommendation_id: scoped_recommendation_id(
                SelectionRecommendationScope::BurstGroup,
                project_id,
                burst_group_id,
                now_ms,
            ),
            run_id: None,
            scope: SelectionRecommendationScope::BurstGroup,
            project_id: project_id.to_string(),
            subject_id: burst_group_id.to_string(),
            selected_asset_group_ids: Vec::new(),
            candidate_asset_group_ids: Vec::new(),
            rejected_asset_group_ids,
            source,
            status: SelectionRecommendationStatus::NoSelection,
            confidence: 0.0,
            reason: "no selectable burst member".to_string(),
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        };
    };
    let candidate_asset_group_ids = selectable
        .iter()
        .filter(|evaluation| evaluation.asset_group_id != best.asset_group_id)
        .map(|evaluation| evaluation.asset_group_id.clone())
        .collect::<Vec<_>>();
    let runner_up_score = selectable
        .iter()
        .find(|evaluation| evaluation.asset_group_id != best.asset_group_id)
        .map(|evaluation| evaluation.score)
        .unwrap_or(0);
    ScopedSelectionRecommendation {
        recommendation_id: scoped_recommendation_id(
            SelectionRecommendationScope::BurstGroup,
            project_id,
            burst_group_id,
            now_ms,
        ),
        run_id: None,
        scope: SelectionRecommendationScope::BurstGroup,
        project_id: project_id.to_string(),
        subject_id: burst_group_id.to_string(),
        selected_asset_group_ids: vec![best.asset_group_id.clone()],
        candidate_asset_group_ids,
        rejected_asset_group_ids,
        source,
        status: SelectionRecommendationStatus::Ready,
        confidence: (0.55 + (best.score - runner_up_score).max(0) as f64 / 100.0).clamp(0.0, 0.98),
        reason: "highest selectable model evaluation in burst".to_string(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    }
}

pub fn recommend_project_selects(
    project_id: &str,
    evaluations: &[ModelEvaluation],
    burst_recommendations: &[ScopedSelectionRecommendation],
    now_ms: i64,
) -> ScopedSelectionRecommendation {
    let source = model_recommendation_source(evaluations);
    let burst_member_ids = burst_recommendations
        .iter()
        .flat_map(|recommendation| {
            recommendation
                .selected_asset_group_ids
                .iter()
                .chain(recommendation.candidate_asset_group_ids.iter())
                .chain(recommendation.rejected_asset_group_ids.iter())
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    let burst_selected_ids = burst_recommendations
        .iter()
        .filter(|recommendation| recommendation.status == SelectionRecommendationStatus::Ready)
        .flat_map(|recommendation| recommendation.selected_asset_group_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut selected = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.status == ModelEvaluationStatus::Ready
                && evaluation.selectable
                && matches!(
                    evaluation.tier,
                    ModelEvaluationTier::Excellent | ModelEvaluationTier::Good
                )
                && (!burst_member_ids.contains(&evaluation.asset_group_id)
                    || burst_selected_ids.contains(&evaluation.asset_group_id))
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.asset_group_id.cmp(&right.asset_group_id))
    });
    let selected_asset_group_ids = selected
        .iter()
        .map(|evaluation| evaluation.asset_group_id.clone())
        .collect::<Vec<_>>();
    let candidate_asset_group_ids = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.status == ModelEvaluationStatus::Ready
                && evaluation.selectable
                && evaluation.tier == ModelEvaluationTier::Normal
                && (!burst_member_ids.contains(&evaluation.asset_group_id)
                    || burst_selected_ids.contains(&evaluation.asset_group_id))
        })
        .map(|evaluation| evaluation.asset_group_id.clone())
        .collect::<Vec<_>>();
    let rejected_asset_group_ids = evaluations
        .iter()
        .filter(|evaluation| {
            evaluation.status != ModelEvaluationStatus::Ready
                || !evaluation.selectable
                || matches!(
                    evaluation.tier,
                    ModelEvaluationTier::Weak | ModelEvaluationTier::Reject
                )
        })
        .map(|evaluation| evaluation.asset_group_id.clone())
        .collect::<Vec<_>>();
    let status = if selected_asset_group_ids.is_empty() {
        SelectionRecommendationStatus::NoSelection
    } else {
        SelectionRecommendationStatus::Ready
    };
    ScopedSelectionRecommendation {
        recommendation_id: scoped_recommendation_id(
            SelectionRecommendationScope::Project,
            project_id,
            project_id,
            now_ms,
        ),
        run_id: None,
        scope: SelectionRecommendationScope::Project,
        project_id: project_id.to_string(),
        subject_id: project_id.to_string(),
        selected_asset_group_ids,
        candidate_asset_group_ids,
        rejected_asset_group_ids,
        source,
        status,
        confidence: if status == SelectionRecommendationStatus::Ready {
            0.72
        } else {
            0.0
        },
        reason: "project model selects from good or excellent evaluations".to_string(),
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    }
}

fn model_recommendation_source(evaluations: &[ModelEvaluation]) -> SelectionSource {
    if !evaluations.is_empty()
        && evaluations
            .iter()
            .all(|evaluation| evaluation.evaluator_kind == ModelEvaluatorKind::LocalStub)
    {
        SelectionSource::LocalStub
    } else if evaluations
        .iter()
        .any(|evaluation| evaluation.evaluator_kind == ModelEvaluatorKind::LlmVlm)
    {
        SelectionSource::Llm
    } else {
        SelectionSource::LocalRule
    }
}

pub fn recommend_from_scores(
    burst_group_id: &str,
    profile: &StrategyProfile,
    scores: &[QualityScore],
    grouping_version: i64,
    now_ms: i64,
) -> SelectionRecommendation {
    let mut ready_scores = scores
        .iter()
        .filter(|score| score.analysis_status == QualityAnalysisStatus::Ready)
        .collect::<Vec<_>>();
    let unsupported_ids = scores
        .iter()
        .filter(|score| score.analysis_status == QualityAnalysisStatus::Unsupported)
        .map(|score| score.asset_group_id.clone())
        .collect::<Vec<_>>();

    if ready_scores.is_empty() {
        return SelectionRecommendation {
            recommendation_id: recommendation_id(
                burst_group_id,
                &profile.profile_id,
                grouping_version,
            ),
            burst_group_id: burst_group_id.to_string(),
            strategy_profile_id: profile.profile_id.clone(),
            scorer_version: scores
                .first()
                .map(|score| score.scorer_version.clone())
                .unwrap_or_else(|| "local-v1".to_string()),
            strategy_version: profile.strategy_version.clone(),
            grouping_version,
            best_asset_group_id: None,
            alternate_asset_group_ids: Vec::new(),
            low_score_asset_group_ids: unsupported_ids,
            near_duplicate_asset_group_ids: Vec::new(),
            source: SelectionSource::LocalRule,
            status: SelectionRecommendationStatus::NeedsReview,
            confidence: 0.0,
            reasons: vec!["no supported scores".to_string()],
            llm_review_id: None,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        };
    }

    ready_scores.sort_by(|left, right| {
        weighted_score(right, profile)
            .total_cmp(&weighted_score(left, profile))
            .then_with(|| right.overall.total_cmp(&left.overall))
            .then_with(|| left.asset_group_id.cmp(&right.asset_group_id))
    });

    let low_score_asset_group_ids = scores
        .iter()
        .filter(|score| {
            score.analysis_status != QualityAnalysisStatus::Ready
                || score.sharpness.value < profile.reject_if_sharpness_below
                || score.overall < profile.flag_if_overall_below
        })
        .map(|score| score.asset_group_id.clone())
        .collect::<Vec<_>>();

    let best = ready_scores
        .iter()
        .copied()
        .find(|score| score.sharpness.value >= profile.reject_if_sharpness_below)
        .or_else(|| ready_scores.first().copied());
    let best_asset_group_id = best.map(|score| score.asset_group_id.clone());
    let best_id = best_asset_group_id.as_deref();
    let alternate_asset_group_ids = ready_scores
        .iter()
        .filter(|score| Some(score.asset_group_id.as_str()) != best_id)
        .map(|score| score.asset_group_id.clone())
        .collect::<Vec<_>>();
    let confidence = best
        .map(|score| {
            let top = weighted_score(score, profile);
            let runner_up = ready_scores
                .iter()
                .find(|candidate| candidate.asset_group_id != score.asset_group_id)
                .map(|candidate| weighted_score(candidate, profile))
                .unwrap_or(0.0);
            (0.55 + (top - runner_up).max(0.0)).clamp(0.0, 0.98)
        })
        .unwrap_or(0.0);
    let mut reasons = Vec::new();
    if let Some(best) = best {
        reasons.push(format!(
            "best technical score: sharpness {:.2}, exposure {:.2}",
            best.sharpness.value, best.exposure.value
        ));
    }
    if !unsupported_ids.is_empty() {
        reasons.push("some frames need review".to_string());
    }

    SelectionRecommendation {
        recommendation_id: recommendation_id(burst_group_id, &profile.profile_id, grouping_version),
        burst_group_id: burst_group_id.to_string(),
        strategy_profile_id: profile.profile_id.clone(),
        scorer_version: ready_scores
            .first()
            .map(|score| score.scorer_version.clone())
            .unwrap_or_else(|| "local-v1".to_string()),
        strategy_version: profile.strategy_version.clone(),
        grouping_version,
        best_asset_group_id,
        alternate_asset_group_ids,
        low_score_asset_group_ids,
        near_duplicate_asset_group_ids: Vec::new(),
        source: SelectionSource::LocalCv,
        status: SelectionRecommendationStatus::Ready,
        confidence,
        reasons,
        llm_review_id: None,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    }
}

fn weighted_score(score: &QualityScore, profile: &StrategyProfile) -> f64 {
    let composition_weight = profile.weights.composition.min(0.12);
    let weighted = score.sharpness.value * profile.weights.sharpness
        + score.exposure.value * profile.weights.exposure
        + score.composition.value * composition_weight
        + score.highlight_clipping_penalty.value * profile.weights.highlight_clipping_penalty
        + score.shadow_clipping_penalty.value * profile.weights.shadow_clipping_penalty;
    weighted.clamp(0.0, 1.0)
}

fn recommendation_id(
    burst_group_id: &str,
    strategy_profile_id: &str,
    grouping_version: i64,
) -> String {
    let key = format!("{burst_group_id}:{strategy_profile_id}:{grouping_version}");
    let mut hash = 1469598103934665603_u64;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("recommendation-{hash:016x}")
}

fn scoped_recommendation_id(
    scope: SelectionRecommendationScope,
    project_id: &str,
    subject_id: &str,
    now_ms: i64,
) -> String {
    let key = format!("{}:{project_id}:{subject_id}:{now_ms}", scope.as_str());
    let mut hash = 1469598103934665603_u64;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("selection-recommendation-{hash:016x}")
}
