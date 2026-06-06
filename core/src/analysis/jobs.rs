use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisJobType {
    DetectBurstForAssetGroup,
    AssessAssetGroupTechnicalQuality,
    AssessPortraitSubject,
    EvaluateAssetGroupWithModel,
    RecommendBurstGroup,
    GenerateProjectRecommendation,
}

impl AnalysisJobType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DetectBurstForAssetGroup => "detect_burst_for_asset_group",
            Self::AssessAssetGroupTechnicalQuality => "assess_asset_group_technical_quality",
            Self::AssessPortraitSubject => "assess_portrait_subject",
            Self::EvaluateAssetGroupWithModel => "evaluate_asset_group_with_model",
            Self::RecommendBurstGroup => "recommend_burst_group",
            Self::GenerateProjectRecommendation => "generate_project_recommendation",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "assess_asset_group_technical_quality" => Self::AssessAssetGroupTechnicalQuality,
            "assess_portrait_subject" => Self::AssessPortraitSubject,
            "evaluate_asset_group_with_model" => Self::EvaluateAssetGroupWithModel,
            "recommend_burst_group" => Self::RecommendBurstGroup,
            "generate_project_recommendation" => Self::GenerateProjectRecommendation,
            _ => Self::DetectBurstForAssetGroup,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisEntityType {
    AssetGroup,
    BurstGroup,
    Project,
}

impl AnalysisEntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AssetGroup => "asset_group",
            Self::BurstGroup => "burst_group",
            Self::Project => "project",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "burst_group" => Self::BurstGroup,
            "project" => Self::Project,
            _ => Self::AssetGroup,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisJobStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl AnalysisJobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub job_id: String,
    pub project_id: String,
    pub job_type: AnalysisJobType,
    pub entity_type: AnalysisEntityType,
    pub entity_id: String,
    pub dedupe_key: String,
    pub status: AnalysisJobStatus,
    pub priority: i64,
    pub attempts: i64,
    pub next_attempt_at_ms: Option<i64>,
    pub last_error: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAnalysisJob {
    pub project_id: String,
    pub job_type: AnalysisJobType,
    pub entity_type: AnalysisEntityType,
    pub entity_id: String,
    pub dedupe_key: String,
    pub priority: i64,
    pub next_attempt_at_ms: Option<i64>,
}

impl NewAnalysisJob {
    pub fn new(
        project_id: &str,
        job_type: AnalysisJobType,
        entity_type: AnalysisEntityType,
        entity_id: &str,
        dedupe_key: &str,
    ) -> Self {
        Self {
            project_id: project_id.to_string(),
            job_type,
            entity_type,
            entity_id: entity_id.to_string(),
            dedupe_key: dedupe_key.to_string(),
            priority: 0,
            next_attempt_at_ms: None,
        }
    }
}
