use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use camera_connector_core::{
    AssetFormatRole, AssetGroupPage, AssetGroupQuery, AssetGroupSort, AssetUserMarks,
    CameraConnectorDashboard, CameraConnectorRuntime, CameraConnectorService, CvPolicy,
    EvaluationRun, ImporterError, ModelProviderKind, ModelProviderSettings, ModelSendMode,
    ObjectFormat, PreviewSample, Project, ProjectEvaluationSettings, ProjectRecommendationMode,
    PromptProfile, PromptProfileVersion, PushProtocol, ReceiverConfigRequest,
    ReceiverSettingsConfig, ReceiverSettingsUpdate, SceneProfile, ScopedSelectionRecommendation,
    StoredObjectLocation, StrategyProfile, SubjectAssessment,
};
use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jlong, jstring, JNI_TRUE};
use jni::{Env, EnvUnowned};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum MobileCoreError {
    #[error("{0}")]
    Core(#[from] ImporterError),
    #[error("invalid protocol: {0}")]
    InvalidProtocol(String),
    #[error("invalid storage location kind: {0}")]
    InvalidLocationKind(String),
    #[error("invalid {field}: {value}")]
    InvalidConfigValue { field: &'static str, value: String },
    #[error("invalid asset format: {0}")]
    InvalidAssetFormat(String),
    #[error("invalid asset role: {0}")]
    InvalidAssetRole(String),
    #[error("mobile core pointer is null")]
    NullCore,
    #[error("input pointer is null: {0}")]
    NullInput(&'static str),
    #[error("input is not valid UTF-8: {0}")]
    InvalidUtf8(&'static str),
    #[error("response contains an interior nul byte")]
    InteriorNul,
    #[error("{0}")]
    Jni(#[from] jni::errors::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

pub type MobileCoreResult<T> = std::result::Result<T, MobileCoreError>;

#[derive(Debug, Clone)]
pub struct MobileCore {
    service: CameraConnectorService,
    runtime: CameraConnectorRuntime,
    async_runtime: Arc<tokio::runtime::Runtime>,
    action_clock_ms: Arc<AtomicI64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileReceiverSettingsPatch {
    pub protocol: Option<String>,
    pub bind_host: Option<String>,
    pub ftp_port: Option<u16>,
    pub sftp_port: Option<u16>,
    pub output_dir: Option<String>,
    pub state_dir: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub defer_publish: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileAssetGroupQuery {
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub original_path: Option<String>,
    pub remote_addr: Option<String>,
    pub format: Option<String>,
    pub role: Option<String>,
    pub sort: Option<String>,
    pub recommendation_state: Option<String>,
    pub score_min: Option<f64>,
    pub score_max: Option<f64>,
    pub analysis_status: Option<String>,
    pub review_queue: Option<String>,
    pub strategy_profile_id: Option<String>,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileUserMarksPatch {
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileAccountView {
    pub username: String,
    pub device_name: String,
    pub password_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileRemoveAccountView {
    pub username: String,
    pub removed: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct MobileModelProviderSettingsPatch {
    provider_kind: Option<String>,
    provider_label: Option<String>,
    base_url: Option<String>,
    default_model: Option<String>,
    default_max_image_side: Option<i64>,
    default_send_mode: Option<String>,
    default_batch_size: Option<i64>,
    configured: Option<bool>,
    api_key: Option<String>,
    key_alias: Option<String>,
    updated_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct MobileProjectEvaluationSettingsPatch {
    model_evaluation_enabled: Option<bool>,
    auto_evaluate_on_upload: Option<bool>,
    auto_burst_recommendation_enabled: Option<bool>,
    project_recommendation_mode: Option<String>,
    prompt_profile_id: Option<String>,
    scene_profile: Option<String>,
    cv_policy: Option<String>,
    allow_risky_model_selects: Option<bool>,
    max_image_side: Option<i64>,
    batch_size: Option<i64>,
    updated_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
struct MobileSubjectAssessmentPayload {
    assessment_id: String,
    project_id: String,
    asset_group_id: String,
    subject_type: String,
    detector_kind: String,
    detector_version: String,
    status: String,
    gate_status: String,
    regions: Value,
    signals: Value,
    summary: String,
    created_at_ms: Option<i64>,
    updated_at_ms: Option<i64>,
}

impl MobileCore {
    pub fn new(config_path: Option<String>) -> Self {
        let service = CameraConnectorService::new(config_path.map(PathBuf::from));
        Self {
            runtime: CameraConnectorRuntime::new(service.clone()),
            service,
            async_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("mobile async runtime should initialize"),
            ),
            action_clock_ms: Arc::new(AtomicI64::new(0)),
        }
    }

    fn next_action_time_ms(&self) -> i64 {
        let now = current_time_ms();
        let mut previous = self.action_clock_ms.load(Ordering::Relaxed);
        loop {
            let next = now.max(previous.saturating_add(1));
            match self.action_clock_ms.compare_exchange_weak(
                previous,
                next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return next,
                Err(observed) => previous = observed,
            }
        }
    }

    pub fn config_path(&self) -> String {
        self.service.config_path().to_string_lossy().into_owned()
    }

    pub fn default_state_dir(&self) -> String {
        self.service.state_dir().to_string_lossy().into_owned()
    }

    pub fn create_project_json(&self, name: String) -> MobileCoreResult<String> {
        let project = self.service.create_project(name)?;
        project_json(project)
    }

    pub fn rename_project_json(
        &self,
        project_id: String,
        name: String,
    ) -> MobileCoreResult<String> {
        let project = self.service.rename_project(&project_id, name)?;
        project_json(project)
    }

    pub fn list_projects_json(&self) -> MobileCoreResult<String> {
        let projects = self.service.list_projects()?;
        project_list_json(projects)
    }

    pub fn set_active_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        self.service.set_active_project(&project_id)?;
        let project = self
            .service
            .active_project()?
            .ok_or_else(|| ImporterError::internal("active project was not found after update"))?;
        project_json(project)
    }

    pub fn archive_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let project = self.service.archive_project(&project_id)?;
        project_json(project)
    }

    pub fn restore_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let project = self.service.restore_project(&project_id)?;
        project_json(project)
    }

    pub fn active_project_json(&self) -> MobileCoreResult<String> {
        let project = self.service.active_project()?;
        project_option_json(project)
    }

    pub fn project_dashboard_json(
        &self,
        project_id: String,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let dashboard: CameraConnectorDashboard = self.service.project_dashboard(
            &project_id,
            AssetGroupQuery::default(),
            offset as usize,
            limit as usize,
            false,
        )?;
        Ok(serde_json::to_string(&dashboard)?)
    }

    pub fn project_asset_group_page_json(
        &self,
        project_id: String,
        query_json: String,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let query = asset_group_query_from_json(&query_json)?;
        let page: AssetGroupPage = self.service.project_asset_group_page_with_query(
            &project_id,
            query,
            offset as usize,
            limit as usize,
        )?;
        Ok(serde_json::to_string(&page)?)
    }

    pub fn project_selects_asset_group_page_json(
        &self,
        project_id: String,
        strategy_profile_id: Option<String>,
        offset: u32,
        limit: u32,
    ) -> MobileCoreResult<String> {
        let page: AssetGroupPage = self.service.project_selects_asset_group_page(
            &project_id,
            strategy_profile_id.as_deref(),
            offset as usize,
            limit as usize,
        )?;
        Ok(serde_json::to_string(&page)?)
    }

    pub fn project_group_assets_json(
        &self,
        project_id: String,
        group_id: String,
    ) -> MobileCoreResult<String> {
        let assets = self.service.project_group_assets(&project_id, &group_id)?;
        Ok(serde_json::to_string(&assets)?)
    }

    pub fn move_project_group_json(
        &self,
        source_project_id: String,
        group_id: String,
        target_project_id: String,
    ) -> MobileCoreResult<String> {
        let group = self.service.move_project_asset_group(
            &source_project_id,
            &group_id,
            &target_project_id,
        )?;
        Ok(serde_json::to_string(&group)?)
    }

    pub fn set_asset_group_user_marks_json(
        &self,
        project_id: String,
        group_id: String,
        patch_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileUserMarksPatch = if patch_json.trim().is_empty() {
            MobileUserMarksPatch::default()
        } else {
            serde_json::from_str(&patch_json)?
        };
        let marks = self.service.set_asset_group_user_marks(
            &project_id,
            &group_id,
            patch.favorite,
            patch.marked,
        )?;
        user_marks_json(marks)
    }

    pub fn claim_next_publish_item_json(&self) -> MobileCoreResult<String> {
        let item = self.service.claim_next_publish_item()?;
        Ok(serde_json::to_string(&item)?)
    }

    pub fn mark_publish_completed_json(&self, queue_id: String) -> MobileCoreResult<String> {
        self.service.mark_publish_completed(&queue_id)?;
        Ok(serde_json::to_string(&json!({
            "queue_id": queue_id,
            "completed": true,
        }))?)
    }

    pub fn complete_publish_json(
        &self,
        queue_id: String,
        final_filename: String,
        location_kind: String,
        location: String,
    ) -> MobileCoreResult<String> {
        let final_location = parse_storage_location(location_kind, location)?;
        let record = self
            .service
            .complete_publish(&queue_id, &final_filename, final_location)?;
        Ok(serde_json::to_string(&record)?)
    }

    pub fn mark_publish_failed_json(
        &self,
        queue_id: String,
        error: String,
    ) -> MobileCoreResult<String> {
        self.service.mark_publish_failed(&queue_id, &error)?;
        Ok(serde_json::to_string(&json!({
            "queue_id": queue_id,
            "failed": true,
        }))?)
    }

    pub fn release_failed_publish_retries_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let released_count = self.service.release_failed_publish_retries(&project_id)?;
        Ok(serde_json::to_string(&json!({
            "project_id": project_id,
            "released_count": released_count,
        }))?)
    }

    pub fn drain_analysis_jobs_json(&self, limit: u32) -> MobileCoreResult<String> {
        let summary = self.service.drain_analysis_jobs(limit as usize)?;
        Ok(serde_json::to_string(&summary)?)
    }

    pub fn drain_analysis_jobs_with_provider_configured_json(
        &self,
        limit: u32,
        provider_configured: bool,
    ) -> MobileCoreResult<String> {
        let summary = self
            .service
            .drain_analysis_jobs_with_provider_configured(limit as usize, provider_configured)?;
        Ok(serde_json::to_string(&summary)?)
    }

    pub fn score_asset_group_preview_json(
        &self,
        asset_group_id: String,
        sample_json: String,
        scorer_version: String,
    ) -> MobileCoreResult<String> {
        let sample = serde_json::from_str::<PreviewSample>(&sample_json)?;
        let score =
            self.service
                .score_asset_group_preview(&asset_group_id, sample, &scorer_version)?;
        Ok(serde_json::to_string(&score)?)
    }

    pub fn score_asset_group_preview_with_provider_configured_json(
        &self,
        asset_group_id: String,
        sample_json: String,
        scorer_version: String,
        provider_configured: bool,
    ) -> MobileCoreResult<String> {
        let sample = serde_json::from_str::<PreviewSample>(&sample_json)?;
        let score = self
            .service
            .score_asset_group_preview_with_provider_configured(
                &asset_group_id,
                sample,
                &scorer_version,
                provider_configured,
            )?;
        Ok(serde_json::to_string(&score)?)
    }

    pub fn recommend_burst_group_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .recommend_burst_group(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn accept_recommended_best_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .accept_recommended_best(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn mark_burst_needs_review_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .mark_burst_needs_review(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn restore_automatic_recommendation_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .restore_automatic_recommendation(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn clear_recommendation_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .clear_recommendation(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn keep_all_candidates_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .keep_all_candidates(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn hide_low_score_candidates_json(
        &self,
        burst_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .hide_low_score_candidates(&burst_group_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn override_recommended_best_json(
        &self,
        burst_group_id: String,
        best_asset_group_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let recommendation = self.service.override_recommended_best(
            &burst_group_id,
            &best_asset_group_id,
            strategy_profile_id.as_deref(),
        )?;
        Ok(serde_json::to_string(&recommendation)?)
    }

    pub fn split_burst_member_json(
        &self,
        burst_group_id: String,
        member_group_id: String,
    ) -> MobileCoreResult<String> {
        let burst = self
            .service
            .split_burst_member(&burst_group_id, &member_group_id)?;
        Ok(serde_json::to_string(&burst)?)
    }

    pub fn merge_burst_member_json(
        &self,
        target_burst_group_id: String,
        member_group_id: String,
    ) -> MobileCoreResult<String> {
        let burst = self
            .service
            .merge_burst_member(&target_burst_group_id, &member_group_id)?;
        Ok(serde_json::to_string(&burst)?)
    }

    pub fn model_provider_settings_json(&self) -> MobileCoreResult<String> {
        let settings = self
            .service
            .model_provider_settings()?
            .unwrap_or_else(default_unconfigured_provider_settings);
        Ok(serde_json::to_string(&model_provider_settings_json_value(
            &settings,
        ))?)
    }

    pub fn save_model_provider_settings_json(
        &self,
        settings_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileModelProviderSettingsPatch = serde_json::from_str(&settings_json)?;
        let settings = ModelProviderSettings {
            settings_id: "global".to_string(),
            provider_kind: patch
                .provider_kind
                .as_deref()
                .map(parse_model_provider_kind)
                .transpose()?
                .unwrap_or(ModelProviderKind::None),
            provider_label: patch.provider_label.unwrap_or_default(),
            base_url: patch.base_url.unwrap_or_default(),
            default_model: patch.default_model.unwrap_or_default(),
            default_max_image_side: patch.default_max_image_side.unwrap_or(1024),
            default_send_mode: patch
                .default_send_mode
                .as_deref()
                .map(parse_model_send_mode)
                .transpose()?
                .unwrap_or(ModelSendMode::PreviewOnly),
            default_batch_size: patch.default_batch_size.unwrap_or(1),
            configured: patch.configured.unwrap_or(false),
            api_key_configured: patch
                .api_key
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_some()
                || patch
                    .key_alias
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .is_some(),
            key_alias: patch.key_alias,
            updated_at_ms: patch.updated_at_ms.unwrap_or_else(current_time_ms),
        };
        let saved = self
            .service
            .save_model_provider_settings_with_api_key(settings, patch.api_key)?;
        Ok(serde_json::to_string(&model_provider_settings_json_value(
            &saved,
        ))?)
    }

    pub fn project_evaluation_settings_json(&self, project_id: String) -> MobileCoreResult<String> {
        let settings = self
            .service
            .project_evaluation_settings(&project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        Ok(serde_json::to_string(
            &project_evaluation_settings_json_value(&settings),
        )?)
    }

    pub fn save_project_evaluation_settings_json(
        &self,
        project_id: String,
        settings_json: String,
    ) -> MobileCoreResult<String> {
        let patch: MobileProjectEvaluationSettingsPatch = serde_json::from_str(&settings_json)?;
        let existing = self
            .service
            .project_evaluation_settings(&project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(
                    project_id.clone(),
                    current_time_ms(),
                )
            });
        let settings = ProjectEvaluationSettings {
            project_id,
            model_evaluation_enabled: patch
                .model_evaluation_enabled
                .unwrap_or(existing.model_evaluation_enabled),
            auto_evaluate_on_upload: patch
                .auto_evaluate_on_upload
                .unwrap_or(existing.auto_evaluate_on_upload),
            auto_burst_recommendation_enabled: patch
                .auto_burst_recommendation_enabled
                .unwrap_or(existing.auto_burst_recommendation_enabled),
            project_recommendation_mode: patch
                .project_recommendation_mode
                .as_deref()
                .map(parse_project_recommendation_mode)
                .transpose()?
                .unwrap_or(ProjectRecommendationMode::Manual),
            prompt_profile_id: patch.prompt_profile_id.or(existing.prompt_profile_id),
            scene_profile: patch
                .scene_profile
                .as_deref()
                .map(parse_scene_profile)
                .transpose()?
                .unwrap_or(existing.scene_profile),
            cv_policy: patch
                .cv_policy
                .as_deref()
                .map(parse_cv_policy)
                .transpose()?
                .unwrap_or(existing.cv_policy),
            allow_risky_model_selects: patch
                .allow_risky_model_selects
                .unwrap_or(existing.allow_risky_model_selects),
            max_image_side: patch.max_image_side.or(existing.max_image_side),
            batch_size: patch.batch_size.or(existing.batch_size),
            updated_at_ms: patch.updated_at_ms.unwrap_or_else(current_time_ms),
        };
        let saved = self.service.save_project_evaluation_settings(settings)?;
        Ok(serde_json::to_string(
            &project_evaluation_settings_json_value(&saved),
        )?)
    }

    pub fn prompt_profiles_for_project_json(&self, project_id: String) -> MobileCoreResult<String> {
        let profiles = self.service.prompt_profiles_for_project(&project_id)?;
        let values = profiles
            .iter()
            .map(prompt_profile_json_value)
            .collect::<Vec<_>>();
        Ok(serde_json::to_string(&values)?)
    }

    pub fn global_prompt_profiles_json(&self) -> MobileCoreResult<String> {
        let profiles = self.service.global_prompt_profiles()?;
        let mut values = Vec::with_capacity(profiles.len());
        for profile in &profiles {
            values.push(prompt_profile_json_value_with_text(
                profile,
                self.service
                    .active_prompt_text_for_profile(&profile.prompt_profile_id)?,
            ));
        }
        Ok(serde_json::to_string(&values)?)
    }

    pub fn fork_global_prompt_profile_json(
        &self,
        source_profile_id: String,
        name: String,
    ) -> MobileCoreResult<String> {
        let profile = self.service.fork_global_prompt_profile(
            &source_profile_id,
            name,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(
            &prompt_profile_json_value_with_text(
                &profile,
                self.service
                    .active_prompt_text_for_profile(&profile.prompt_profile_id)?,
            ),
        )?)
    }

    pub fn save_global_prompt_version_json(
        &self,
        prompt_profile_id: String,
        prompt_text: String,
    ) -> MobileCoreResult<String> {
        let profile = self.service.save_global_prompt_profile_version(
            &prompt_profile_id,
            prompt_text,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(
            &prompt_profile_json_value_with_text(
                &profile,
                self.service
                    .active_prompt_text_for_profile(&profile.prompt_profile_id)?,
            ),
        )?)
    }

    pub fn fork_prompt_profile_json(
        &self,
        project_id: String,
        source_profile_id: String,
        name: String,
    ) -> MobileCoreResult<String> {
        let profile = self.service.fork_prompt_profile_for_project(
            &project_id,
            &source_profile_id,
            name,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(&prompt_profile_json_value(&profile))?)
    }

    pub fn save_prompt_version_json(
        &self,
        project_id: String,
        prompt_profile_id: String,
        prompt_text: String,
    ) -> MobileCoreResult<String> {
        let version = self.service.save_prompt_profile_version(
            &project_id,
            &prompt_profile_id,
            prompt_text,
            self.next_action_time_ms(),
        )?;
        Ok(serde_json::to_string(&prompt_profile_version_json_value(
            &version,
        ))?)
    }

    pub fn generate_project_recommendation_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let recommendation = self
            .service
            .generate_project_recommendation(&project_id, self.next_action_time_ms())?;
        scoped_recommendation_json(recommendation)
    }

    pub fn latest_project_recommendation_run_status_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let run = self
            .service
            .latest_project_recommendation_run_status(&project_id)?;
        Ok(serde_json::to_string(
            &run.map(|run| evaluation_run_json_value(&run)),
        )?)
    }

    pub fn should_schedule_subject_assessment_json(
        &self,
        project_id: String,
    ) -> MobileCoreResult<String> {
        let should_schedule = self
            .service
            .should_schedule_subject_assessment(&project_id)?;
        Ok(serde_json::to_string(&should_schedule)?)
    }

    pub fn save_subject_assessment_json(
        &self,
        assessment_json: String,
    ) -> MobileCoreResult<String> {
        let payload: MobileSubjectAssessmentPayload = serde_json::from_str(&assessment_json)?;
        let now_ms = self.next_action_time_ms();
        let assessment = SubjectAssessment {
            assessment_id: payload.assessment_id,
            project_id: payload.project_id,
            asset_group_id: payload.asset_group_id,
            subject_type: payload.subject_type,
            detector_kind: payload.detector_kind,
            detector_version: payload.detector_version,
            status: parse_evaluation_run_status(&payload.status)?,
            gate_status: payload.gate_status,
            regions_json: serde_json::to_string(&payload.regions)?,
            signals_json: serde_json::to_string(&payload.signals)?,
            summary: payload.summary,
            created_at_ms: payload.created_at_ms.unwrap_or(now_ms),
            updated_at_ms: payload.updated_at_ms.unwrap_or(now_ms),
        };
        let saved = self.service.save_subject_assessment(assessment)?;
        Ok(serde_json::to_string(&subject_assessment_json_value(
            &saved,
        ))?)
    }

    pub fn subject_assessments_for_asset_groups_json(
        &self,
        project_id: String,
        group_ids_json: String,
    ) -> MobileCoreResult<String> {
        let group_ids: Vec<String> = serde_json::from_str(&group_ids_json)?;
        let assessments = self
            .service
            .subject_assessments_for_asset_groups(&project_id, &group_ids)?;
        let values = assessments
            .iter()
            .map(subject_assessment_json_value)
            .collect::<Vec<_>>();
        Ok(serde_json::to_string(&values)?)
    }

    pub fn strategy_profiles_json(&self) -> MobileCoreResult<String> {
        let profiles = self.service.strategy_profiles()?;
        Ok(serde_json::to_string(&profiles)?)
    }

    pub fn save_strategy_profile_json(&self, profile_json: String) -> MobileCoreResult<String> {
        let profile = serde_json::from_str::<StrategyProfile>(&profile_json)?;
        let saved = self.service.save_custom_strategy_profile(profile)?;
        Ok(serde_json::to_string(&saved)?)
    }

    pub fn review_queue_summary_json(
        &self,
        project_id: String,
        strategy_profile_id: Option<String>,
    ) -> MobileCoreResult<String> {
        let summary = self
            .service
            .project_review_queue_summary(&project_id, strategy_profile_id.as_deref())?;
        Ok(serde_json::to_string(&summary)?)
    }

    pub fn save_receiver_settings_json(
        &self,
        patch: MobileReceiverSettingsPatch,
    ) -> MobileCoreResult<String> {
        let (settings, _) = self.service.set_receiver_settings(patch.try_into()?)?;
        Ok(serde_json::to_string(&settings)?)
    }

    pub fn save_device_account_json(
        &self,
        username: String,
        password: Option<String>,
        device_name: String,
    ) -> MobileCoreResult<String> {
        let (account, _) = self
            .service
            .set_account(username, password.as_deref(), device_name)?;
        let password_configured = account.password_configured();
        let view = MobileAccountView {
            username: account.username,
            device_name: account.device_name,
            password_configured,
        };
        Ok(serde_json::to_string(&view)?)
    }

    pub fn remove_device_account_json(&self, username: String) -> MobileCoreResult<String> {
        let (removed, _) = self.service.remove_account(&username)?;
        Ok(serde_json::to_string(&MobileRemoveAccountView {
            username,
            removed,
        })?)
    }

    pub fn start_receiver_json(&self) -> MobileCoreResult<String> {
        let status =
            self.async_runtime
                .block_on(self.runtime.start_receiver(ReceiverConfigRequest {
                    protocol: None,
                    bind_host: None,
                    port: None,
                    output_dir: None,
                    state_dir: None,
                    username: None,
                    password: None,
                    advertised_host: None,
                    source_name: None,
                    defer_publish: None,
                }))?;
        Ok(serde_json::to_string(&status)?)
    }

    pub fn stop_receiver_json(&self) -> MobileCoreResult<String> {
        let status = self.async_runtime.block_on(self.runtime.stop_receiver())?;
        Ok(serde_json::to_string(&status)?)
    }
}

impl TryFrom<MobileReceiverSettingsPatch> for ReceiverSettingsUpdate {
    type Error = MobileCoreError;

    fn try_from(patch: MobileReceiverSettingsPatch) -> MobileCoreResult<Self> {
        Ok(Self {
            protocol: patch.protocol.map(parse_protocol).transpose()?,
            bind_host: patch.bind_host,
            ftp_port: patch.ftp_port,
            sftp_port: patch.sftp_port,
            output_dir: patch.output_dir.map(PathBuf::from),
            state_dir: patch.state_dir.map(PathBuf::from),
            advertised_host: patch.advertised_host,
            source_name: patch.source_name,
            defer_publish: patch.defer_publish,
        })
    }
}

fn parse_protocol(protocol: String) -> MobileCoreResult<PushProtocol> {
    match protocol.trim().to_ascii_lowercase().as_str() {
        "ftp" => Ok(PushProtocol::Ftp),
        "sftp" => Ok(PushProtocol::Sftp),
        _ => Err(MobileCoreError::InvalidProtocol(protocol)),
    }
}

fn parse_model_provider_kind(value: &str) -> MobileCoreResult<ModelProviderKind> {
    match value {
        "none" => Ok(ModelProviderKind::None),
        "openai" => Ok(ModelProviderKind::OpenAi),
        "custom" => Ok(ModelProviderKind::Custom),
        "imported" => Ok(ModelProviderKind::Imported),
        _ => invalid_config_value("provider_kind", value),
    }
}

fn parse_model_send_mode(value: &str) -> MobileCoreResult<ModelSendMode> {
    match value {
        "preview_only" => Ok(ModelSendMode::PreviewOnly),
        "review_image" => Ok(ModelSendMode::ReviewImage),
        _ => invalid_config_value("default_send_mode", value),
    }
}

fn parse_scene_profile(value: &str) -> MobileCoreResult<SceneProfile> {
    match value {
        "general" => Ok(SceneProfile::General),
        "portrait" => Ok(SceneProfile::Portrait),
        "action" => Ok(SceneProfile::Action),
        "landscape" => Ok(SceneProfile::Landscape),
        "custom" => Ok(SceneProfile::Custom),
        _ => invalid_config_value("scene_profile", value),
    }
}

fn parse_cv_policy(value: &str) -> MobileCoreResult<CvPolicy> {
    match value {
        "loose" => Ok(CvPolicy::Loose),
        "standard" => Ok(CvPolicy::Standard),
        "strict" => Ok(CvPolicy::Strict),
        _ => invalid_config_value("cv_policy", value),
    }
}

fn parse_project_recommendation_mode(value: &str) -> MobileCoreResult<ProjectRecommendationMode> {
    match value {
        "manual" => Ok(ProjectRecommendationMode::Manual),
        _ => invalid_config_value("project_recommendation_mode", value),
    }
}

fn parse_evaluation_run_status(
    value: &str,
) -> MobileCoreResult<camera_connector_core::EvaluationRunStatus> {
    match value {
        "pending" => Ok(camera_connector_core::EvaluationRunStatus::Pending),
        "running" => Ok(camera_connector_core::EvaluationRunStatus::Running),
        "ready" => Ok(camera_connector_core::EvaluationRunStatus::Ready),
        "failed" => Ok(camera_connector_core::EvaluationRunStatus::Failed),
        "skipped" => Ok(camera_connector_core::EvaluationRunStatus::Skipped),
        _ => invalid_config_value("status", value),
    }
}

fn invalid_config_value<T>(field: &'static str, value: &str) -> MobileCoreResult<T> {
    Err(MobileCoreError::InvalidConfigValue {
        field,
        value: value.to_string(),
    })
}

fn parse_storage_location(
    kind: String,
    location: String,
) -> MobileCoreResult<StoredObjectLocation> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "local_path" => Ok(StoredObjectLocation::local_path(location)),
        "document_uri" => Ok(StoredObjectLocation::document_uri(location)),
        "media_uri" => Ok(StoredObjectLocation::media_uri(location)),
        "photo_asset" => Ok(StoredObjectLocation::photo_asset(location)),
        _ => Err(MobileCoreError::InvalidLocationKind(kind)),
    }
}

#[allow(dead_code)]
fn _assert_settings_config_is_serializable(settings: &ReceiverSettingsConfig) -> String {
    serde_json::to_string(settings).expect("receiver settings should serialize")
}

/// # Safety
///
/// `config_path` must be either null or a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create(
    config_path: *const c_char,
) -> *mut MobileCore {
    let config_path = optional_c_string(config_path).ok().flatten();
    Box::into_raw(Box::new(MobileCore::new(config_path)))
}

/// # Safety
///
/// `core` must be a pointer returned by `camera_connector_mobile_core_create`.
/// Passing the same pointer more than once is invalid.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_destroy(core: *mut MobileCore) {
    if !core.is_null() {
        drop(Box::from_raw(core));
    }
}

/// # Safety
///
/// `value` must be a pointer returned by one of this crate's string-returning
/// FFI functions. Passing the same pointer more than once is invalid.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_config_path(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| Ok(json!(core_ref(core)?.config_path())))
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_default_state_dir(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| Ok(json!(core_ref(core)?.default_state_dir())))
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `name` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_create_project_json(
    core: *const MobileCore,
    name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let name = required_c_string(name, "name")?;
        let project = core_ref(core)?.create_project_json(name)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_list_projects_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let projects = core_ref(core)?.list_projects_json()?;
        parse_json_value(&projects)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_set_active_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let project = core_ref(core)?.set_active_project_json(project_id)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_rename_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
    name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let name = required_c_string(name, "name")?;
        let project = core_ref(core)?.rename_project_json(project_id, name)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_archive_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let project = core_ref(core)?.archive_project_json(project_id)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_restore_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let project = core_ref(core)?.restore_project_json(project_id)?;
        parse_json_value(&project)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_active_project_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let project = core_ref(core)?.active_project_json()?;
        parse_json_value(&project)
    })
}

/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_dashboard_json(
    core: *const MobileCore,
    project_id: *const c_char,
    offset: u32,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let dashboard = core_ref(core)?.project_dashboard_json(project_id, offset, limit)?;
        parse_json_value(&dashboard)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `query_json` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_asset_group_page_json(
    core: *const MobileCore,
    project_id: *const c_char,
    query_json: *const c_char,
    offset: u32,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let query_json = required_c_string(query_json, "query_json")?;
        let page =
            core_ref(core)?.project_asset_group_page_json(project_id, query_json, offset, limit)?;
        parse_json_value(&page)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_selects_asset_group_page_json(
    core: *const MobileCore,
    project_id: *const c_char,
    strategy_profile_id: *const c_char,
    offset: u32,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let page = core_ref(core)?.project_selects_asset_group_page_json(
            project_id,
            strategy_profile_id,
            offset,
            limit,
        )?;
        parse_json_value(&page)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `group_id` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_group_assets_json(
    core: *const MobileCore,
    project_id: *const c_char,
    group_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let group_id = required_c_string(group_id, "group_id")?;
        let assets = core_ref(core)?.project_group_assets_json(project_id, group_id)?;
        parse_json_value(&assets)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_move_project_group_json(
    core: *const MobileCore,
    source_project_id: *const c_char,
    group_id: *const c_char,
    target_project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let source_project_id = required_c_string(source_project_id, "source_project_id")?;
        let group_id = required_c_string(group_id, "group_id")?;
        let target_project_id = required_c_string(target_project_id, "target_project_id")?;
        let group = core_ref(core)?.move_project_group_json(
            source_project_id,
            group_id,
            target_project_id,
        )?;
        parse_json_value(&group)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_set_asset_group_user_marks_json(
    core: *const MobileCore,
    project_id: *const c_char,
    group_id: *const c_char,
    patch_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let group_id = required_c_string(group_id, "group_id")?;
        let patch_json = required_c_string(patch_json, "patch_json")?;
        let marks =
            core_ref(core)?.set_asset_group_user_marks_json(project_id, group_id, patch_json)?;
        parse_json_value(&marks)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_claim_next_publish_item_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let item = core_ref(core)?.claim_next_publish_item_json()?;
        parse_json_value(&item)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `queue_id` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_mark_publish_completed_json(
    core: *const MobileCore,
    queue_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let queue_id = required_c_string(queue_id, "queue_id")?;
        let result = core_ref(core)?.mark_publish_completed_json(queue_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// All string pointers must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_complete_publish_json(
    core: *const MobileCore,
    queue_id: *const c_char,
    final_filename: *const c_char,
    location_kind: *const c_char,
    location: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let queue_id = required_c_string(queue_id, "queue_id")?;
        let final_filename = required_c_string(final_filename, "final_filename")?;
        let location_kind = required_c_string(location_kind, "location_kind")?;
        let location = required_c_string(location, "location")?;
        let result = core_ref(core)?.complete_publish_json(
            queue_id,
            final_filename,
            location_kind,
            location,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `queue_id` and `error` must be valid, null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_mark_publish_failed_json(
    core: *const MobileCore,
    queue_id: *const c_char,
    error: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let queue_id = required_c_string(queue_id, "queue_id")?;
        let error = required_c_string(error, "error")?;
        let result = core_ref(core)?.mark_publish_failed_json(queue_id, error)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_release_failed_publish_retries_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let result = core_ref(core)?.release_failed_publish_retries_json(project_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_drain_analysis_jobs_json(
    core: *const MobileCore,
    limit: u32,
) -> *mut c_char {
    ffi_response(|| {
        let result = core_ref(core)?.drain_analysis_jobs_json(limit)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_drain_analysis_jobs_with_provider_configured_json(
    core: *const MobileCore,
    limit: u32,
    provider_configured: bool,
) -> *mut c_char {
    ffi_response(|| {
        let result = core_ref(core)?
            .drain_analysis_jobs_with_provider_configured_json(limit, provider_configured)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_score_asset_group_preview_json(
    core: *const MobileCore,
    asset_group_id: *const c_char,
    sample_json: *const c_char,
    scorer_version: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let asset_group_id = required_c_string(asset_group_id, "asset_group_id")?;
        let sample_json = required_c_string(sample_json, "sample_json")?;
        let scorer_version = required_c_string(scorer_version, "scorer_version")?;
        let result = core_ref(core)?.score_asset_group_preview_json(
            asset_group_id,
            sample_json,
            scorer_version,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_recommend_burst_group_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result =
            core_ref(core)?.recommend_burst_group_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_accept_recommended_best_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result =
            core_ref(core)?.accept_recommended_best_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_mark_burst_needs_review_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result =
            core_ref(core)?.mark_burst_needs_review_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_restore_automatic_recommendation_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result = core_ref(core)?
            .restore_automatic_recommendation_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_clear_recommendation_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result =
            core_ref(core)?.clear_recommendation_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_keep_all_candidates_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result =
            core_ref(core)?.keep_all_candidates_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_hide_low_score_candidates_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result =
            core_ref(core)?.hide_low_score_candidates_json(burst_group_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` and `best_asset_group_id` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_override_recommended_best_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    best_asset_group_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let best_asset_group_id = required_c_string(best_asset_group_id, "best_asset_group_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result = core_ref(core)?.override_recommended_best_json(
            burst_group_id,
            best_asset_group_id,
            strategy_profile_id,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `burst_group_id` and `member_group_id` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_split_burst_member_json(
    core: *const MobileCore,
    burst_group_id: *const c_char,
    member_group_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let burst_group_id = required_c_string(burst_group_id, "burst_group_id")?;
        let member_group_id = required_c_string(member_group_id, "member_group_id")?;
        let result = core_ref(core)?.split_burst_member_json(burst_group_id, member_group_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `target_burst_group_id` and `member_group_id` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_merge_burst_member_json(
    core: *const MobileCore,
    target_burst_group_id: *const c_char,
    member_group_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let target_burst_group_id =
            required_c_string(target_burst_group_id, "target_burst_group_id")?;
        let member_group_id = required_c_string(member_group_id, "member_group_id")?;
        let result =
            core_ref(core)?.merge_burst_member_json(target_burst_group_id, member_group_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_model_provider_settings_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let settings = core_ref(core)?.model_provider_settings_json()?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `settings_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_model_provider_settings_json(
    core: *const MobileCore,
    settings_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let settings_json = required_c_string(settings_json, "settings_json")?;
        let settings = core_ref(core)?.save_model_provider_settings_json(settings_json)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_project_evaluation_settings_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let settings = core_ref(core)?.project_evaluation_settings_json(project_id)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `settings_json` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_project_evaluation_settings_json(
    core: *const MobileCore,
    project_id: *const c_char,
    settings_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let settings_json = required_c_string(settings_json, "settings_json")?;
        let settings =
            core_ref(core)?.save_project_evaluation_settings_json(project_id, settings_json)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_prompt_profiles_for_project_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let profiles = core_ref(core)?.prompt_profiles_for_project_json(project_id)?;
        parse_json_value(&profiles)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_score_asset_group_preview_with_provider_configured_json(
    core: *const MobileCore,
    asset_group_id: *const c_char,
    sample_json: *const c_char,
    scorer_version: *const c_char,
    provider_configured: bool,
) -> *mut c_char {
    ffi_response(|| {
        let asset_group_id = required_c_string(asset_group_id, "asset_group_id")?;
        let sample_json = required_c_string(sample_json, "sample_json")?;
        let scorer_version = required_c_string(scorer_version, "scorer_version")?;
        let result = core_ref(core)?.score_asset_group_preview_with_provider_configured_json(
            asset_group_id,
            sample_json,
            scorer_version,
            provider_configured,
        )?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_fork_prompt_profile_json(
    core: *const MobileCore,
    project_id: *const c_char,
    source_profile_id: *const c_char,
    name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let source_profile_id = required_c_string(source_profile_id, "source_profile_id")?;
        let name = required_c_string(name, "name")?;
        let profile =
            core_ref(core)?.fork_prompt_profile_json(project_id, source_profile_id, name)?;
        parse_json_value(&profile)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// String pointers must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_prompt_version_json(
    core: *const MobileCore,
    project_id: *const c_char,
    prompt_profile_id: *const c_char,
    prompt_text: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let prompt_profile_id = required_c_string(prompt_profile_id, "prompt_profile_id")?;
        let prompt_text = required_c_string(prompt_text, "prompt_text")?;
        let version =
            core_ref(core)?.save_prompt_version_json(project_id, prompt_profile_id, prompt_text)?;
        parse_json_value(&version)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_generate_project_recommendation_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let recommendation = core_ref(core)?.generate_project_recommendation_json(project_id)?;
        parse_json_value(&recommendation)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_latest_project_recommendation_run_status_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let run = core_ref(core)?.latest_project_recommendation_run_status_json(project_id)?;
        parse_json_value(&run)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_should_schedule_subject_assessment_json(
    core: *const MobileCore,
    project_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let result = core_ref(core)?.should_schedule_subject_assessment_json(project_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `assessment_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_subject_assessment_json(
    core: *const MobileCore,
    assessment_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let assessment_json = required_c_string(assessment_json, "assessment_json")?;
        let assessment = core_ref(core)?.save_subject_assessment_json(assessment_json)?;
        parse_json_value(&assessment)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` and `group_ids_json` must be valid, null-terminated UTF-8 C strings.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_subject_assessments_for_asset_groups_json(
    core: *const MobileCore,
    project_id: *const c_char,
    group_ids_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let group_ids_json = required_c_string(group_ids_json, "group_ids_json")?;
        let assessments = core_ref(core)?
            .subject_assessments_for_asset_groups_json(project_id, group_ids_json)?;
        parse_json_value(&assessments)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_strategy_profiles_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let result = core_ref(core)?.strategy_profiles_json()?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `profile_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_strategy_profile_json(
    core: *const MobileCore,
    profile_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let profile_json = required_c_string(profile_json, "profile_json")?;
        let result = core_ref(core)?.save_strategy_profile_json(profile_json)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `project_id` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_review_queue_summary_json(
    core: *const MobileCore,
    project_id: *const c_char,
    strategy_profile_id: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let project_id = required_c_string(project_id, "project_id")?;
        let strategy_profile_id = optional_c_string(strategy_profile_id)?;
        let result = core_ref(core)?.review_queue_summary_json(project_id, strategy_profile_id)?;
        parse_json_value(&result)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `patch_json` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_receiver_settings_json(
    core: *const MobileCore,
    patch_json: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let patch_json = required_c_string(patch_json, "patch_json")?;
        let patch = serde_json::from_str::<MobileReceiverSettingsPatch>(&patch_json)?;
        let settings = core_ref(core)?.save_receiver_settings_json(patch)?;
        parse_json_value(&settings)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `username` and `device_name` must be valid, null-terminated UTF-8 C strings.
/// `password` must be either null or a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_save_device_account_json(
    core: *const MobileCore,
    username: *const c_char,
    password: *const c_char,
    device_name: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let username = required_c_string(username, "username")?;
        let password = optional_c_string(password)?;
        let device_name = required_c_string(device_name, "device_name")?;
        let account = core_ref(core)?.save_device_account_json(username, password, device_name)?;
        parse_json_value(&account)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
/// `username` must be a valid, null-terminated UTF-8 C string.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_remove_device_account_json(
    core: *const MobileCore,
    username: *const c_char,
) -> *mut c_char {
    ffi_response(|| {
        let username = required_c_string(username, "username")?;
        let removed = core_ref(core)?.remove_device_account_json(username)?;
        parse_json_value(&removed)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_start_receiver_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let status = core_ref(core)?.start_receiver_json()?;
        parse_json_value(&status)
    })
}

/// # Safety
///
/// `core` must be a valid pointer returned by `camera_connector_mobile_core_create`.
#[no_mangle]
pub unsafe extern "C" fn camera_connector_mobile_core_stop_receiver_json(
    core: *const MobileCore,
) -> *mut c_char {
    ffi_response(|| {
        let status = core_ref(core)?.stop_receiver_json()?;
        parse_json_value(&status)
    })
}

fn parse_json_value(json: &str) -> MobileCoreResult<Value> {
    Ok(serde_json::from_str(json)?)
}

fn project_json(project: Project) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&project.into_view())?)
}

fn project_option_json(project: Option<Project>) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&project.map(Project::into_view))?)
}

fn project_list_json(projects: Vec<Project>) -> MobileCoreResult<String> {
    let views = projects
        .into_iter()
        .map(Project::into_view)
        .collect::<Vec<_>>();
    Ok(serde_json::to_string(&views)?)
}

fn user_marks_json(marks: AssetUserMarks) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&marks)?)
}

fn default_unconfigured_provider_settings() -> ModelProviderSettings {
    ModelProviderSettings {
        settings_id: "global".to_string(),
        provider_kind: ModelProviderKind::None,
        provider_label: String::new(),
        base_url: String::new(),
        default_model: String::new(),
        default_max_image_side: 1024,
        default_send_mode: ModelSendMode::PreviewOnly,
        default_batch_size: 1,
        configured: false,
        api_key_configured: false,
        key_alias: None,
        updated_at_ms: current_time_ms(),
    }
}

fn model_provider_settings_json_value(settings: &ModelProviderSettings) -> Value {
    json!({
        "settings_id": settings.settings_id,
        "provider_kind": settings.provider_kind.as_str(),
        "provider_label": settings.provider_label,
        "base_url": settings.base_url,
        "default_model": settings.default_model,
        "default_max_image_side": settings.default_max_image_side,
        "default_send_mode": settings.default_send_mode.as_str(),
        "default_batch_size": settings.default_batch_size,
        "configured": settings.configured,
        "api_key_configured": settings.api_key_configured,
        "key_alias": settings.key_alias,
        "updated_at_ms": settings.updated_at_ms,
    })
}

fn project_evaluation_settings_json_value(settings: &ProjectEvaluationSettings) -> Value {
    json!({
        "project_id": settings.project_id,
        "model_evaluation_enabled": settings.model_evaluation_enabled,
        "auto_evaluate_on_upload": settings.auto_evaluate_on_upload,
        "auto_burst_recommendation_enabled": settings.auto_burst_recommendation_enabled,
        "project_recommendation_mode": settings.project_recommendation_mode.as_str(),
        "prompt_profile_id": settings.prompt_profile_id,
        "scene_profile": settings.scene_profile.as_str(),
        "cv_policy": settings.cv_policy.as_str(),
        "allow_risky_model_selects": settings.allow_risky_model_selects,
        "max_image_side": settings.max_image_side,
        "batch_size": settings.batch_size,
        "updated_at_ms": settings.updated_at_ms,
    })
}

fn prompt_profile_json_value(profile: &PromptProfile) -> Value {
    json!({
        "prompt_profile_id": profile.prompt_profile_id,
        "scope": profile.scope.as_str(),
        "project_id": profile.project_id,
        "name": profile.name,
        "style_tags": profile.style_tags,
        "scene_profile": profile.scene_profile.as_str(),
        "active_version_id": profile.active_version_id,
        "built_in": profile.built_in,
        "enabled": profile.enabled,
    })
}

fn prompt_profile_json_value_with_text(
    profile: &PromptProfile,
    prompt_text: Option<String>,
) -> Value {
    let mut value = prompt_profile_json_value(profile);
    value["active_prompt_text"] = prompt_text.map(Value::String).unwrap_or(Value::Null);
    value
}

fn prompt_profile_version_json_value(version: &PromptProfileVersion) -> Value {
    json!({
        "prompt_version_id": version.prompt_version_id,
        "prompt_profile_id": version.prompt_profile_id,
        "output_schema_version": version.output_schema_version,
        "prompt_hash": version.prompt_hash,
        "created_at_ms": version.created_at_ms,
    })
}

fn evaluation_run_json_value(run: &EvaluationRun) -> Value {
    json!({
        "run_id": run.run_id,
        "project_id": run.project_id,
        "run_type": run.run_type.as_str(),
        "trigger": run.trigger.as_str(),
        "status": run.status.as_str(),
        "provider_kind": run.provider_kind.as_str(),
        "provider_model": run.provider_model,
        "prompt_profile_id": run.prompt_profile_id,
        "prompt_version_id": run.prompt_version_id,
        "prompt_hash": run.prompt_hash,
        "error_message": run.error_message,
        "started_at_ms": run.started_at_ms,
        "completed_at_ms": run.completed_at_ms,
        "created_at_ms": run.created_at_ms,
    })
}

fn subject_assessment_json_value(assessment: &SubjectAssessment) -> Value {
    json!({
        "assessment_id": assessment.assessment_id,
        "project_id": assessment.project_id,
        "asset_group_id": assessment.asset_group_id,
        "subject_type": assessment.subject_type,
        "detector_kind": assessment.detector_kind,
        "detector_version": assessment.detector_version,
        "status": assessment.status.as_str(),
        "gate_status": assessment.gate_status,
        "regions": serde_json::from_str::<Value>(&assessment.regions_json).unwrap_or(Value::Null),
        "signals": serde_json::from_str::<Value>(&assessment.signals_json).unwrap_or(Value::Null),
        "summary": assessment.summary,
        "created_at_ms": assessment.created_at_ms,
        "updated_at_ms": assessment.updated_at_ms,
    })
}

fn scoped_recommendation_json(
    recommendation: ScopedSelectionRecommendation,
) -> MobileCoreResult<String> {
    Ok(serde_json::to_string(&recommendation)?)
}

fn current_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

fn asset_group_query_from_json(query_json: &str) -> MobileCoreResult<AssetGroupQuery> {
    let query: MobileAssetGroupQuery = if query_json.trim().is_empty() {
        MobileAssetGroupQuery::default()
    } else {
        serde_json::from_str(query_json)?
    };
    Ok(AssetGroupQuery {
        username: query.username.and_then(non_blank),
        source_name: query.source_name.and_then(non_blank),
        original_path: query.original_path.and_then(non_blank),
        remote_addr: query.remote_addr.and_then(non_blank),
        format: query
            .format
            .and_then(non_blank)
            .map(|value| {
                ObjectFormat::from_str(&value)
                    .map_err(|_| MobileCoreError::InvalidAssetFormat(value))
            })
            .transpose()?,
        role: query
            .role
            .and_then(non_blank)
            .map(|value| {
                AssetFormatRole::from_str(&value)
                    .map_err(|_| MobileCoreError::InvalidAssetRole(value))
            })
            .transpose()?,
        sort: query
            .sort
            .and_then(non_blank)
            .and_then(|value| AssetGroupSort::from_wire(&value))
            .unwrap_or_default(),
        recommendation_state: query.recommendation_state.and_then(non_blank),
        score_min: query.score_min,
        score_max: query.score_max,
        analysis_status: query.analysis_status.and_then(non_blank),
        review_queue: query.review_queue.and_then(non_blank),
        strategy_profile_id: query.strategy_profile_id.and_then(non_blank),
        favorite: query.favorite,
        marked: query.marked,
    })
}

fn non_blank(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn core_ref<'a>(core: *const MobileCore) -> MobileCoreResult<&'a MobileCore> {
    if core.is_null() {
        Err(MobileCoreError::NullCore)
    } else {
        Ok(unsafe { &*core })
    }
}

fn optional_c_string(value: *const c_char) -> MobileCoreResult<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(c_string(value, "optional")?))
    }
}

fn required_c_string(value: *const c_char, name: &'static str) -> MobileCoreResult<String> {
    if value.is_null() {
        Err(MobileCoreError::NullInput(name))
    } else {
        c_string(value, name)
    }
}

fn c_string(value: *const c_char, name: &'static str) -> MobileCoreResult<String> {
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .map(ToOwned::to_owned)
        .map_err(|_| MobileCoreError::InvalidUtf8(name))
}

fn ffi_response(action: impl FnOnce() -> MobileCoreResult<Value>) -> *mut c_char {
    let response = match action() {
        Ok(value) => json!({
            "ok": true,
            "value": value,
            "error": Value::Null,
        }),
        Err(error) => json!({
            "ok": false,
            "value": Value::Null,
            "error": error.to_string(),
        }),
    };
    string_to_ffi(
        serde_json::to_string(&response)
            .unwrap_or_else(|error| format!(r#"{{"ok":false,"value":null,"error":"{}"}}"#, error)),
    )
}

fn string_to_ffi(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => CString::new(
            r#"{"ok":false,"value":null,"error":"response contains an interior nul byte"}"#,
        )
        .expect("static error response should not contain nul")
        .into_raw(),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_create(
    mut env: EnvUnowned,
    _class: JClass,
    config_path: JString,
) -> jlong {
    env.with_env(|env| -> Result<jlong, jni::errors::Error> {
        let config_path = optional_java_string(env, config_path).unwrap_or(None);
        Ok(Box::into_raw(Box::new(MobileCore::new(config_path))) as jlong)
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// # Safety
///
/// `handle` must be a pointer value returned by
/// `Java_com_cameraconnector_app_core_NativeMobileCore_create`. Passing the
/// same handle more than once is invalid.
#[no_mangle]
pub unsafe extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_destroy(
    _env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) {
    if handle != 0 {
        drop(Box::from_raw(handle as *mut MobileCore));
    }
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_createProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    name: JString,
) -> jstring {
    env.with_env(|env| {
        let name = required_java_string(env, name, "name");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.create_project_json(name?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_listProjectsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let projects = mobile_core_from_handle(handle)?.list_projects_json()?;
            parse_json_value(&projects)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_setActiveProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.set_active_project_json(project_id?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_renameProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    name: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let name = required_java_string(env, name, "name");
        java_response(env, || {
            let project =
                mobile_core_from_handle(handle)?.rename_project_json(project_id?, name?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_archiveProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.archive_project_json(project_id?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_restoreProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.restore_project_json(project_id?)?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_activeProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let project = mobile_core_from_handle(handle)?.active_project_json()?;
            parse_json_value(&project)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectDashboardJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    offset: jint,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let dashboard = mobile_core_from_handle(handle)?.project_dashboard_json(
                project_id?,
                offset.max(0) as u32,
                limit.max(0) as u32,
            )?;
            parse_json_value(&dashboard)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectAssetGroupPageJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    query_json: JString,
    offset: jint,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let query_json = required_java_string(env, query_json, "query_json");
        java_response(env, || {
            let page = mobile_core_from_handle(handle)?.project_asset_group_page_json(
                project_id?,
                query_json?,
                offset.max(0) as u32,
                limit.max(0) as u32,
            )?;
            parse_json_value(&page)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectSelectsAssetGroupPageJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    strategy_profile_id: JString,
    offset: jint,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let page = mobile_core_from_handle(handle)?.project_selects_asset_group_page_json(
                project_id?,
                strategy_profile_id?,
                offset.max(0) as u32,
                limit.max(0) as u32,
            )?;
            parse_json_value(&page)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectGroupAssetsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_id = required_java_string(env, group_id, "group_id");
        java_response(env, || {
            let assets = mobile_core_from_handle(handle)?
                .project_group_assets_json(project_id?, group_id?)?;
            parse_json_value(&assets)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_moveProjectGroupJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    source_project_id: JString,
    group_id: JString,
    target_project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let source_project_id = required_java_string(env, source_project_id, "source_project_id");
        let group_id = required_java_string(env, group_id, "group_id");
        let target_project_id = required_java_string(env, target_project_id, "target_project_id");
        java_response(env, || {
            let group = mobile_core_from_handle(handle)?.move_project_group_json(
                source_project_id?,
                group_id?,
                target_project_id?,
            )?;
            parse_json_value(&group)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_setAssetGroupUserMarksJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_id: JString,
    patch_json: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_id = required_java_string(env, group_id, "group_id");
        let patch_json = required_java_string(env, patch_json, "patch_json");
        java_response(env, || {
            let marks = mobile_core_from_handle(handle)?.set_asset_group_user_marks_json(
                project_id?,
                group_id?,
                patch_json?,
            )?;
            parse_json_value(&marks)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_claimNextPublishItemJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let item = mobile_core_from_handle(handle)?.claim_next_publish_item_json()?;
            parse_json_value(&item)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_markPublishCompletedJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    queue_id: JString,
) -> jstring {
    env.with_env(|env| {
        let queue_id = required_java_string(env, queue_id, "queue_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.mark_publish_completed_json(queue_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_completePublishJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    queue_id: JString,
    final_filename: JString,
    location_kind: JString,
    location: JString,
) -> jstring {
    env.with_env(|env| {
        let queue_id = required_java_string(env, queue_id, "queue_id");
        let final_filename = required_java_string(env, final_filename, "final_filename");
        let location_kind = required_java_string(env, location_kind, "location_kind");
        let location = required_java_string(env, location, "location");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.complete_publish_json(
                queue_id?,
                final_filename?,
                location_kind?,
                location?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_markPublishFailedJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    queue_id: JString,
    error: JString,
) -> jstring {
    env.with_env(|env| {
        let queue_id = required_java_string(env, queue_id, "queue_id");
        let error = required_java_string(env, error, "error");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.mark_publish_failed_json(queue_id?, error?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_releaseFailedPublishRetriesJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .release_failed_publish_retries_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_drainAnalysisJobsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    limit: jint,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.drain_analysis_jobs_json(limit.max(0) as u32)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_drainAnalysisJobsWithProviderConfiguredJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    limit: jint,
    provider_configured: jboolean,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .drain_analysis_jobs_with_provider_configured_json(
                    limit.max(0) as u32,
                    provider_configured == JNI_TRUE,
                )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_scoreAssetGroupPreviewJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    asset_group_id: JString,
    sample_json: JString,
    scorer_version: JString,
) -> jstring {
    env.with_env(|env| {
        let asset_group_id = required_java_string(env, asset_group_id, "asset_group_id");
        let sample_json = required_java_string(env, sample_json, "sample_json");
        let scorer_version = required_java_string(env, scorer_version, "scorer_version");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.score_asset_group_preview_json(
                asset_group_id?,
                sample_json?,
                scorer_version?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_scoreAssetGroupPreviewWithProviderConfiguredJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    asset_group_id: JString,
    sample_json: JString,
    scorer_version: JString,
    provider_configured: jboolean,
) -> jstring {
    env.with_env(|env| {
        let asset_group_id = required_java_string(env, asset_group_id, "asset_group_id");
        let sample_json = required_java_string(env, sample_json, "sample_json");
        let scorer_version = required_java_string(env, scorer_version, "scorer_version");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .score_asset_group_preview_with_provider_configured_json(
                    asset_group_id?,
                    sample_json?,
                    scorer_version?,
                    provider_configured == JNI_TRUE,
                )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_recommendBurstGroupJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .recommend_burst_group_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_acceptRecommendedBestJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .accept_recommended_best_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_markBurstNeedsReviewJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .mark_burst_needs_review_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_restoreAutomaticRecommendationJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .restore_automatic_recommendation_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_clearRecommendationJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .clear_recommendation_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_keepAllCandidatesJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .keep_all_candidates_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_hideLowScoreCandidatesJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .hide_low_score_candidates_json(burst_group_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_overrideRecommendedBestJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    best_asset_group_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let best_asset_group_id =
            required_java_string(env, best_asset_group_id, "best_asset_group_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.override_recommended_best_json(
                burst_group_id?,
                best_asset_group_id?,
                strategy_profile_id?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_splitBurstMemberJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    burst_group_id: JString,
    member_group_id: JString,
) -> jstring {
    env.with_env(|env| {
        let burst_group_id = required_java_string(env, burst_group_id, "burst_group_id");
        let member_group_id = required_java_string(env, member_group_id, "member_group_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .split_burst_member_json(burst_group_id?, member_group_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_mergeBurstMemberJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    target_burst_group_id: JString,
    member_group_id: JString,
) -> jstring {
    env.with_env(|env| {
        let target_burst_group_id =
            required_java_string(env, target_burst_group_id, "target_burst_group_id");
        let member_group_id = required_java_string(env, member_group_id, "member_group_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .merge_burst_member_json(target_burst_group_id?, member_group_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_modelProviderSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.model_provider_settings_json()?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveModelProviderSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    settings_json: JString,
) -> jstring {
    env.with_env(|env| {
        let settings_json = required_java_string(env, settings_json, "settings_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .save_model_provider_settings_json(settings_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_projectEvaluationSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.project_evaluation_settings_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveProjectEvaluationSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    settings_json: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let settings_json = required_java_string(env, settings_json, "settings_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .save_project_evaluation_settings_json(project_id?, settings_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_promptProfilesForProjectJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.prompt_profiles_for_project_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_globalPromptProfilesJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.global_prompt_profiles_json()?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_forkGlobalPromptProfileJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    source_profile_id: JString,
    name: JString,
) -> jstring {
    env.with_env(|env| {
        let source_profile_id = required_java_string(env, source_profile_id, "source_profile_id");
        let name = required_java_string(env, name, "name");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .fork_global_prompt_profile_json(source_profile_id?, name?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveGlobalPromptVersionJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    prompt_profile_id: JString,
    prompt_text: JString,
) -> jstring {
    env.with_env(|env| {
        let prompt_profile_id = required_java_string(env, prompt_profile_id, "prompt_profile_id");
        let prompt_text = required_java_string(env, prompt_text, "prompt_text");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .save_global_prompt_version_json(prompt_profile_id?, prompt_text?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_forkPromptProfileJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    source_profile_id: JString,
    name: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let source_profile_id = required_java_string(env, source_profile_id, "source_profile_id");
        let name = required_java_string(env, name, "name");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.fork_prompt_profile_json(
                project_id?,
                source_profile_id?,
                name?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_savePromptVersionJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    prompt_profile_id: JString,
    prompt_text: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let prompt_profile_id = required_java_string(env, prompt_profile_id, "prompt_profile_id");
        let prompt_text = required_java_string(env, prompt_text, "prompt_text");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.save_prompt_version_json(
                project_id?,
                prompt_profile_id?,
                prompt_text?,
            )?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_generateProjectRecommendationJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .generate_project_recommendation_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_latestProjectRecommendationRunStatusJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .latest_project_recommendation_run_status_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_shouldScheduleSubjectAssessmentJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .should_schedule_subject_assessment_json(project_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveSubjectAssessmentJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    assessment_json: JString,
) -> jstring {
    env.with_env(|env| {
        let assessment_json = required_java_string(env, assessment_json, "assessment_json");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.save_subject_assessment_json(assessment_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_subjectAssessmentsForAssetGroupsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    group_ids_json: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let group_ids_json = required_java_string(env, group_ids_json, "group_ids_json");
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .subject_assessments_for_asset_groups_json(project_id?, group_ids_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_strategyProfilesJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?.strategy_profiles_json()?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveStrategyProfileJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    profile_json: JString,
) -> jstring {
    env.with_env(|env| {
        let profile_json = required_java_string(env, profile_json, "profile_json");
        java_response(env, || {
            let result =
                mobile_core_from_handle(handle)?.save_strategy_profile_json(profile_json?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_reviewQueueSummaryJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    project_id: JString,
    strategy_profile_id: JString,
) -> jstring {
    env.with_env(|env| {
        let project_id = required_java_string(env, project_id, "project_id");
        let strategy_profile_id = optional_java_string(env, strategy_profile_id);
        java_response(env, || {
            let result = mobile_core_from_handle(handle)?
                .review_queue_summary_json(project_id?, strategy_profile_id?)?;
            parse_json_value(&result)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveReceiverSettingsJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    patch_json: JString,
) -> jstring {
    env.with_env(|env| {
        let patch_json = required_java_string(env, patch_json, "patch_json");
        java_response(env, || {
            let patch = serde_json::from_str::<MobileReceiverSettingsPatch>(&patch_json?)?;
            let settings = mobile_core_from_handle(handle)?.save_receiver_settings_json(patch)?;
            parse_json_value(&settings)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_saveDeviceAccountJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    username: JString,
    password: JString,
    device_name: JString,
) -> jstring {
    env.with_env(|env| {
        let username = required_java_string(env, username, "username");
        let password = optional_java_string(env, password);
        let device_name = required_java_string(env, device_name, "device_name");
        java_response(env, || {
            let account = mobile_core_from_handle(handle)?.save_device_account_json(
                username?,
                password?,
                device_name?,
            )?;
            parse_json_value(&account)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_removeDeviceAccountJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
    username: JString,
) -> jstring {
    env.with_env(|env| {
        let username = required_java_string(env, username, "username");
        java_response(env, || {
            let removed = mobile_core_from_handle(handle)?.remove_device_account_json(username?)?;
            parse_json_value(&removed)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_startReceiverJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let status = mobile_core_from_handle(handle)?.start_receiver_json()?;
            parse_json_value(&status)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[no_mangle]
pub extern "system" fn Java_com_cameraconnector_app_core_NativeMobileCore_stopReceiverJson(
    mut env: EnvUnowned,
    _class: JClass,
    handle: jlong,
) -> jstring {
    env.with_env(|env| {
        java_response(env, || {
            let status = mobile_core_from_handle(handle)?.stop_receiver_json()?;
            parse_json_value(&status)
        })
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

fn mobile_core_from_handle<'a>(handle: jlong) -> MobileCoreResult<&'a MobileCore> {
    if handle == 0 {
        Err(MobileCoreError::NullCore)
    } else {
        Ok(unsafe { &*(handle as *const MobileCore) })
    }
}

fn optional_java_string(env: &mut Env, value: JString) -> MobileCoreResult<Option<String>> {
    if value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value.try_to_string(env)?))
    }
}

fn required_java_string(
    env: &mut Env,
    value: JString,
    name: &'static str,
) -> MobileCoreResult<String> {
    optional_java_string(env, value)?.ok_or(MobileCoreError::NullInput(name))
}

fn java_response(
    env: &mut Env,
    action: impl FnOnce() -> MobileCoreResult<Value>,
) -> Result<jstring, jni::errors::Error> {
    let response = match action() {
        Ok(value) => json!({
            "ok": true,
            "value": value,
            "error": Value::Null,
        }),
        Err(error) => json!({
            "ok": false,
            "value": Value::Null,
            "error": error.to_string(),
        }),
    };
    let raw = serde_json::to_string(&response)
        .unwrap_or_else(|error| format!(r#"{{"ok":false,"value":null,"error":"{}"}}"#, error));
    JString::from_str(env, raw).map(|value| value.into_raw())
}
