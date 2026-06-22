#![allow(clippy::should_implement_trait)]

use serde::{Deserialize, Serialize};

use super::technical::TechnicalAssessmentPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelProviderKind {
    None,
    OpenAi,
    Custom,
    Imported,
}

impl ModelProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::OpenAi => "openai",
            Self::Custom => "custom",
            Self::Imported => "imported",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "openai" => Self::OpenAi,
            "custom" => Self::Custom,
            "imported" => Self::Imported,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSendMode {
    PreviewOnly,
    DetailImage,
}

impl ModelSendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreviewOnly => "preview_only",
            Self::DetailImage => "detail_image",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "detail_image" => Self::DetailImage,
            _ => Self::PreviewOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneProfile {
    General,
    Portrait,
    Action,
    Landscape,
    Custom,
}

impl SceneProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Portrait => "portrait",
            Self::Action => "action",
            Self::Landscape => "landscape",
            Self::Custom => "custom",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "portrait" => Self::Portrait,
            "action" => Self::Action,
            "landscape" => Self::Landscape,
            "custom" => Self::Custom,
            _ => Self::General,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CvPolicy {
    Loose,
    Standard,
    Strict,
}

impl CvPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loose => "loose",
            Self::Standard => "standard",
            Self::Strict => "strict",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "loose" => Self::Loose,
            "strict" => Self::Strict,
            _ => Self::Standard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectRecommendationMode {
    Manual,
}

impl ProjectRecommendationMode {
    pub fn as_str(self) -> &'static str {
        "manual"
    }

    pub fn from_str(_value: &str) -> Self {
        Self::Manual
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationRunType {
    AssetEvaluation,
    BurstRecommendation,
    ProjectRecommendation,
}

impl EvaluationRunType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AssetEvaluation => "asset_evaluation",
            Self::BurstRecommendation => "burst_recommendation",
            Self::ProjectRecommendation => "project_recommendation",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "burst_recommendation" => Self::BurstRecommendation,
            "project_recommendation" => Self::ProjectRecommendation,
            _ => Self::AssetEvaluation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationRunTrigger {
    Upload,
    BurstStable,
    Manual,
    Retry,
}

impl EvaluationRunTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::BurstStable => "burst_stable",
            Self::Manual => "manual",
            Self::Retry => "retry",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "burst_stable" => Self::BurstStable,
            "manual" => Self::Manual,
            "retry" => Self::Retry,
            _ => Self::Upload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationRunStatus {
    Pending,
    Running,
    Ready,
    Failed,
    Skipped,
}

impl EvaluationRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "ready" => Self::Ready,
            "failed" => Self::Failed,
            "skipped" => Self::Skipped,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelProviderSettings {
    pub settings_id: String,
    pub provider_kind: ModelProviderKind,
    pub provider_label: String,
    pub base_url: String,
    pub default_model: String,
    pub default_max_image_side: i64,
    pub default_send_mode: ModelSendMode,
    pub default_batch_size: i64,
    pub configured: bool,
    pub api_key_configured: bool,
    pub key_alias: Option<String>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptPack {
    pub prompt_pack_id: String,
    #[serde(default = "default_prompt_pack_distribution_folder")]
    pub distribution_folder: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub style_tags: Vec<String>,
    pub scene_profile: SceneProfile,
    pub schema: String,
    pub capabilities: Vec<String>,
    pub built_in: bool,
    pub enabled: bool,
    pub prompt_text: String,
    pub prompt_hash: String,
    pub updated_at_ms: i64,
}

pub fn default_prompt_pack_distribution_folder() -> String {
    "user".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptPackContent {
    pub shared_preference: String,
    pub evaluation_instruction: Option<String>,
    pub burst_selection_instruction: Option<String>,
    pub project_selection_instruction: Option<String>,
}

impl PromptPackContent {
    pub fn new(shared_preference: impl Into<String>) -> Self {
        Self {
            shared_preference: shared_preference.into(),
            evaluation_instruction: None,
            burst_selection_instruction: None,
            project_selection_instruction: None,
        }
    }
}

impl Default for PromptPackContent {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectEvaluationSettings {
    pub project_id: String,
    pub auto_evaluate_on_upload: bool,
    pub auto_burst_recommendation_enabled: bool,
    pub project_recommendation_mode: ProjectRecommendationMode,
    pub prompt_pack_id: Option<String>,
    pub model_provider_settings_id: Option<String>,
    pub scene_profile: SceneProfile,
    pub cv_policy: CvPolicy,
    pub cv_policy_overrides: Option<TechnicalAssessmentPolicy>,
    pub allow_risky_model_selects: bool,
    pub max_image_side: Option<i64>,
    pub batch_size: Option<i64>,
    pub updated_at_ms: i64,
}

impl ProjectEvaluationSettings {
    pub fn default_for_project(project_id: impl Into<String>, updated_at_ms: i64) -> Self {
        Self {
            project_id: project_id.into(),
            auto_evaluate_on_upload: false,
            auto_burst_recommendation_enabled: true,
            project_recommendation_mode: ProjectRecommendationMode::Manual,
            prompt_pack_id: None,
            model_provider_settings_id: None,
            scene_profile: SceneProfile::General,
            cv_policy: CvPolicy::Standard,
            cv_policy_overrides: None,
            allow_risky_model_selects: false,
            max_image_side: None,
            batch_size: None,
            updated_at_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationRun {
    pub run_id: String,
    pub project_id: String,
    pub run_type: EvaluationRunType,
    pub trigger: EvaluationRunTrigger,
    pub status: EvaluationRunStatus,
    pub provider_kind: ModelProviderKind,
    pub provider_model: String,
    pub prompt_pack_id: Option<String>,
    pub prompt_pack_version: Option<String>,
    pub prompt_hash: Option<String>,
    pub settings_snapshot_json: String,
    pub error_message: Option<String>,
    pub started_at_ms: Option<i64>,
    pub completed_at_ms: Option<i64>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectAssessment {
    pub assessment_id: String,
    pub project_id: String,
    pub asset_group_id: String,
    pub subject_type: String,
    pub detector_kind: String,
    pub detector_version: String,
    pub status: EvaluationRunStatus,
    pub gate_status: String,
    pub regions_json: String,
    pub signals_json: String,
    pub summary: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}
