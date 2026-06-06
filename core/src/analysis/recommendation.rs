use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    ModelEvaluation, ModelEvaluationStatus, ModelEvaluationTier, ModelEvaluatorKind,
    TechnicalAssessment,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionSource {
    LocalStub,
    Imported,
    Llm,
}

impl SelectionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalStub => "local_stub",
            Self::Imported => "imported",
            Self::Llm => "llm",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "local_stub" => Some(Self::LocalStub),
            "imported" => Some(Self::Imported),
            "llm" => Some(Self::Llm),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionRecommendationStatus {
    Pending,
    Ready,
    Stale,
    Failed,
    NoSelection,
}

impl SelectionRecommendationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Ready => "ready",
            Self::Stale => "stale",
            Self::Failed => "failed",
            Self::NoSelection => "no_selection",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "ready" => Self::Ready,
            "stale" => Self::Stale,
            "failed" => Self::Failed,
            "no_selection" => Self::NoSelection,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
pub struct SelectionRecommendation {
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

pub fn recommend_burst_group_from_model_evaluations(
    project_id: &str,
    burst_group_id: &str,
    evaluations: &[ModelEvaluation],
    _assessments: &[TechnicalAssessment],
    now_ms: i64,
) -> SelectionRecommendation {
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
        return SelectionRecommendation {
            recommendation_id: selection_recommendation_id(
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
    SelectionRecommendation {
        recommendation_id: selection_recommendation_id(
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

pub fn recommend_project_model_selections(
    project_id: &str,
    evaluations: &[ModelEvaluation],
    burst_recommendations: &[SelectionRecommendation],
    now_ms: i64,
) -> SelectionRecommendation {
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
    SelectionRecommendation {
        recommendation_id: selection_recommendation_id(
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
        reason: "project model recommendation from good or excellent evaluations".to_string(),
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
        SelectionSource::Imported
    }
}

pub(crate) fn selection_recommendation_id(
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
