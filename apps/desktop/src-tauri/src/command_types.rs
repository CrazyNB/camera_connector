use std::path::PathBuf;

use camera_connector_core::{
    CameraConnectorService, ProjectSyncApplySummary, TechnicalAssessmentPolicy,
};
use serde::{Deserialize, Serialize};

pub struct DesktopState {
    pub service: CameraConnectorService,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetPageRequest {
    pub project_id: String,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserMarksRequest {
    pub project_id: String,
    pub group_id: String,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveModelProviderSettingsRequest {
    pub settings_id: String,
    pub provider_kind: String,
    pub provider_label: String,
    pub base_url: String,
    pub default_model: String,
    pub default_max_image_side: i64,
    pub default_send_mode: String,
    pub default_batch_size: i64,
    pub configured: bool,
    pub api_key: Option<String>,
    pub key_alias: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopModelProviderSettings {
    pub settings_id: String,
    pub provider_kind: String,
    pub provider_label: String,
    pub base_url: String,
    pub default_model: String,
    pub default_max_image_side: i64,
    pub default_send_mode: String,
    pub default_batch_size: i64,
    pub configured: bool,
    pub api_key_configured: bool,
    pub key_alias: Option<String>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopPromptPack {
    pub prompt_pack_id: String,
    pub distribution_folder: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub style_tags: Vec<String>,
    pub scene_profile: String,
    pub schema: String,
    pub capabilities: Vec<String>,
    pub built_in: bool,
    pub enabled: bool,
    pub shared_preference: Option<String>,
    pub prompt_hash: String,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatePromptPackRequest {
    pub name: String,
    pub style_tags: Vec<String>,
    pub scene_profile: String,
    pub distribution_folder: String,
    pub shared_preference: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForkPromptPackRequest {
    pub source_prompt_pack_id: String,
    pub name: String,
    pub distribution_folder: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SavePromptPackRequest {
    pub prompt_pack_id: String,
    pub name: String,
    pub style_tags: Vec<String>,
    pub scene_profile: String,
    pub shared_preference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DesktopProjectEvaluationSettings {
    pub project_id: String,
    pub auto_evaluate_on_upload: bool,
    pub auto_burst_recommendation_enabled: bool,
    pub project_recommendation_mode: String,
    pub prompt_pack_id: Option<String>,
    pub model_provider_settings_id: Option<String>,
    pub scene_profile: String,
    pub cv_policy: String,
    pub cv_policy_overrides: Option<TechnicalAssessmentPolicy>,
    pub allow_risky_model_selects: bool,
    pub max_image_side: Option<i64>,
    pub batch_size: Option<i64>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnqueueModelEvaluationRequest {
    pub project_id: String,
    pub asset_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnqueueModelEvaluationResponse {
    pub enqueued_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesktopCvAssessmentRequest {
    pub project_id: String,
    pub limit: Option<usize>,
    pub asset_group_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopCvAssessmentResponse {
    pub assessed_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub subject_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopCvAssessmentProgress {
    pub project_id: String,
    pub scope: String,
    pub total_count: usize,
    pub assessed_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub subject_count: usize,
    pub current_group_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubjectAssessmentsRequest {
    pub project_id: String,
    pub asset_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncProjectSnapshotRequest {
    pub project_id: String,
    pub snapshot_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncProjectSnapshotUrlRequest {
    pub project_id: String,
    pub snapshot_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncProjectSnapshotResponse {
    pub matched_assets: usize,
    pub matched_groups: usize,
    pub applied_user_marks: usize,
    pub applied_model_evaluations: usize,
    pub applied_selection_recommendations: usize,
    pub unresolved_records: usize,
    pub ambiguous_records: usize,
}

impl From<ProjectSyncApplySummary> for SyncProjectSnapshotResponse {
    fn from(summary: ProjectSyncApplySummary) -> Self {
        Self {
            matched_assets: summary.matched_assets,
            matched_groups: summary.matched_groups,
            applied_user_marks: summary.applied_user_marks,
            applied_model_evaluations: summary.applied_model_evaluations,
            applied_selection_recommendations: summary.applied_selection_recommendations,
            unresolved_records: summary.unresolved_records,
            ambiguous_records: summary.ambiguous_records,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopError {
    pub code: String,
    pub message: String,
}
