use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    append_transfer_record, assess_preview_sample, evaluate_asset_group_with_model_provider,
    evaluate_asset_group_with_stub, group_received_assets, read_connected_devices,
    read_receiver_runtime_status, read_transfer_log, recommend_burst_group_from_model_evaluations,
    recommend_project_model_selections, recommend_selection_with_model_provider,
    scan_received_asset_groups, AnalysisEntityType, AnalysisJob, AnalysisJobType, AssetFormatRole,
    AssetUserMarks, BurstGroup, BurstGroupingProfile, CameraConnectorConfig, ConnectedDevice,
    EvaluationRun, EvaluationRunStatus, EvaluationRunTrigger, EvaluationRunType, ImportSource,
    ModelProviderKind, ModelProviderSettings, ModelProviderSettingsConfig, ModelSendMode,
    NewAnalysisJob, ObjectFormat, PreviewSample, ProjectEvaluationSettings,
    ProjectRecommendationMode, ProjectStatus, PromptProfile, PromptProfileContent,
    PromptProfileVersion, PromptScope, PublishQueueItem, PublishQueueSummary, PushProtocol,
    PushReceiverConfig, ReceivedAsset, ReceivedAssetGroup, ReceiverAccountConfig,
    ReceiverRuntimeStatus, ReceiverSettingsConfig, Result, SceneProfile,
    SelectionCandidateVisualInput, SelectionRecommendation, SelectionRecommendationScope,
    SelectionRecommendationStatus, SqliteStore, StoredAsset, StoredObjectLocation,
    SubjectAssessment, TechnicalAssessment, TransferRecord, TransferStatus,
};

const MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION: &str = "model-evaluation-v1";

#[derive(Debug, Clone)]
pub struct CameraConnectorService {
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ReceiverConfigRequest {
    pub protocol: Option<PushProtocol>,
    pub bind_host: Option<String>,
    pub port: Option<u16>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub defer_publish: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ReceiverSettingsUpdate {
    pub protocol: Option<PushProtocol>,
    pub bind_host: Option<String>,
    pub ftp_port: Option<u16>,
    pub sftp_port: Option<u16>,
    pub output_dir: Option<PathBuf>,
    pub state_dir: Option<PathBuf>,
    pub advertised_host: Option<String>,
    pub source_name: Option<String>,
    pub defer_publish: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct TransferQuery {
    pub status: Option<TransferStatus>,
    pub transfer_id: Option<String>,
    pub original_path: Option<String>,
    pub final_filename: Option<String>,
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub remote_addr: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AssetGroupQuery {
    pub username: Option<String>,
    pub source_name: Option<String>,
    pub original_path: Option<String>,
    pub remote_addr: Option<String>,
    pub format: Option<ObjectFormat>,
    pub role: Option<AssetFormatRole>,
    pub sort: AssetGroupSort,
    pub collection: Option<String>,
    pub favorite: Option<bool>,
    pub marked: Option<bool>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetGroupSort {
    #[default]
    LatestReceived,
    Filename,
    ModelScore,
}

impl AssetGroupSort {
    pub fn from_wire(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "latest_received" | "latest" | "received" => Some(Self::LatestReceived),
            "filename" | "name" => Some(Self::Filename),
            "model_score" | "model_selects" => Some(Self::ModelScore),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuplicateInfo {
    pub index: usize,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferRecordView {
    pub record: TransferRecord,
    pub display_source: Option<String>,
    pub virtual_display_path: String,
    pub final_location_kind: Option<String>,
    pub final_location_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferSummary {
    pub total_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFacetCount {
    pub value: String,
    pub group_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetGroupSummary {
    pub group_count: usize,
    pub asset_count: usize,
    pub groups_with_jpeg: usize,
    pub groups_with_raw: usize,
    pub groups_with_video: usize,
    pub source_counts: Vec<AssetFacetCount>,
    pub remote_addr_counts: Vec<AssetFacetCount>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetGroupPage {
    pub groups: Vec<ReceivedAssetGroup>,
    pub summary: AssetGroupSummary,
    pub offset: usize,
    pub limit: usize,
    pub total_groups: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectedDeviceView {
    pub device: ConnectedDevice,
    pub display_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountView {
    pub username: String,
    pub device_name: String,
    pub password_configured: bool,
    pub online: bool,
    pub active_connections: u32,
    pub last_remote_addr: Option<String>,
    pub last_remote_port: Option<u16>,
    pub last_seen_at_ms: Option<i64>,
    pub last_disconnected_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishQueueFailureView {
    pub queue_id: String,
    pub transfer_id: String,
    pub final_filename: String,
    pub original_path: Option<String>,
    pub display_source: Option<String>,
    pub username: Option<String>,
    pub attempt_count: u32,
    pub last_error: Option<String>,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisDrainSummary {
    pub claimed_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetGroupModelEvaluationInput {
    pub asset_group_id: String,
    pub preview_sample: PreviewSample,
    pub preview_image_data_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemPathsView {
    pub config_path: PathBuf,
    pub state_dir: PathBuf,
    pub output_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraConnectorDashboard {
    pub receiver_status: Option<ReceiverRuntimeStatus>,
    pub receiver_settings: ReceiverSettingsConfig,
    pub paths: SystemPathsView,
    pub accounts: Vec<AccountView>,
    pub devices: Vec<ConnectedDeviceView>,
    pub transfers: TransferSummary,
    pub publish_queue: PublishQueueSummary,
    pub recent_failures: Vec<TransferRecordView>,
    pub recent_publish_failures: Vec<PublishQueueFailureView>,
    pub assets: AssetGroupPage,
}

#[derive(Debug, Clone)]
struct RuntimeModelProvider {
    settings: ModelProviderSettings,
    api_key: Option<String>,
}

impl CameraConnectorService {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }

    pub fn config_path(&self) -> PathBuf {
        CameraConnectorConfig::resolved_path(self.config_path.as_deref())
    }

    pub fn state_dir(&self) -> PathBuf {
        CameraConnectorConfig::default_state_dir(self.config_path.as_deref())
    }

    pub fn load_config(&self) -> Result<CameraConnectorConfig> {
        CameraConnectorConfig::load(self.config_path.as_deref())
    }

    pub fn save_config(&self, config: &CameraConnectorConfig) -> Result<PathBuf> {
        config.save(self.config_path.as_deref())
    }

    pub fn receiver_config(&self, request: ReceiverConfigRequest) -> Result<PushReceiverConfig> {
        let mut app_config = self.load_config()?;
        let receiver_settings = app_config.receiver.clone();
        let protocol = request.protocol.unwrap_or(receiver_settings.protocol);
        let bind_host = request
            .bind_host
            .unwrap_or_else(|| receiver_settings.bind_host.clone());
        let port = request.port.unwrap_or(match protocol {
            PushProtocol::Ftp => receiver_settings.ftp_port,
            PushProtocol::Sftp => receiver_settings.sftp_port,
        });
        let output_dir = request
            .output_dir
            .or(receiver_settings.output_dir.clone())
            .unwrap_or_else(CameraConnectorConfig::default_output_dir);
        let state_dir = request
            .state_dir
            .or(receiver_settings.state_dir.clone())
            .unwrap_or_else(|| self.state_dir());
        let mut config = PushReceiverConfig::new(protocol, bind_host, port, output_dir)
            .with_state_dir(state_dir);
        config.advertised_host = request
            .advertised_host
            .or(receiver_settings.advertised_host);
        config.source_name = request.source_name.or(receiver_settings.source_name);
        config.defer_publish = request
            .defer_publish
            .unwrap_or(receiver_settings.defer_publish);
        let account_state_dir = receiver_settings
            .state_dir
            .clone()
            .unwrap_or_else(|| self.state_dir());
        let active_project_id = app_config.active_project_id.clone();
        app_config.accounts = receiver_account_configs_from_state_dir(account_state_dir)?;
        config.accounts = app_config.effective_accounts(
            request.username.as_deref(),
            request.password.as_deref(),
            config.source_name.as_deref(),
        )?;
        config.active_project_id = active_project_id;
        Ok(config)
    }

    pub fn storage_state_dir(&self) -> Result<PathBuf> {
        Ok(self
            .load_config()?
            .receiver
            .state_dir
            .unwrap_or_else(|| self.state_dir()))
    }

    pub fn storage_store(&self) -> Result<SqliteStore> {
        SqliteStore::open_state_dir(self.storage_state_dir()?)
    }

    fn receiver_account_configs(&self) -> Result<BTreeMap<String, ReceiverAccountConfig>> {
        receiver_account_configs_from_state_dir(self.storage_state_dir()?)
    }

    pub fn create_project(&self, name: impl AsRef<str>) -> Result<crate::Project> {
        self.storage_store()?.create_project(name)
    }

    pub fn rename_project(
        &self,
        project_id: &str,
        name: impl AsRef<str>,
    ) -> Result<crate::Project> {
        self.storage_store()?.rename_project(project_id, name)
    }

    pub fn archive_project(&self, project_id: &str) -> Result<crate::Project> {
        let archived = self.storage_store()?.archive_project(project_id)?;
        let mut config = self.load_config()?;
        if config.active_project_id.as_deref() == Some(project_id) {
            config.active_project_id = None;
            self.save_config(&config)?;
        }
        Ok(archived)
    }

    pub fn restore_project(&self, project_id: &str) -> Result<crate::Project> {
        self.storage_store()?.restore_project(project_id)
    }

    pub fn set_active_project(&self, project_id: &str) -> Result<()> {
        let project = self
            .storage_store()?
            .list_projects()?
            .into_iter()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| crate::ImporterError::internal("project not found"))?;
        if project.status != ProjectStatus::Active {
            return Err(crate::ImporterError::internal("project archived"));
        }
        let mut config = self.load_config()?;
        config.active_project_id = Some(project.project_id);
        self.save_config(&config)?;
        Ok(())
    }

    pub fn active_project(&self) -> Result<Option<crate::Project>> {
        let Some(project_id) = self.load_config()?.active_project_id else {
            return Ok(None);
        };
        Ok(self
            .storage_store()?
            .list_projects()?
            .into_iter()
            .find(|project| {
                project.project_id == project_id && project.status == ProjectStatus::Active
            }))
    }

    pub fn list_projects(&self) -> Result<Vec<crate::Project>> {
        self.storage_store()?.list_projects()
    }

    pub fn model_provider_settings(&self) -> Result<Option<crate::ModelProviderSettings>> {
        Ok(self
            .runtime_model_provider()?
            .map(|provider| provider.settings))
    }

    pub fn model_provider_settings_list(&self) -> Result<Vec<ModelProviderSettings>> {
        Ok(self
            .runtime_model_providers()?
            .into_iter()
            .map(|provider| provider.settings)
            .collect())
    }

    pub fn save_model_provider_settings(
        &self,
        settings: crate::ModelProviderSettings,
    ) -> Result<crate::ModelProviderSettings> {
        self.save_model_provider_settings_with_api_key(settings, None)
    }

    pub fn save_model_provider_settings_with_api_key(
        &self,
        settings: ModelProviderSettings,
        api_key: Option<String>,
    ) -> Result<ModelProviderSettings> {
        let mut config = self.load_config()?;
        let settings_id = normalized_model_provider_settings_id(&settings.settings_id);
        let existing_api_key = model_provider_config_by_id(&config, &settings_id)
            .and_then(|existing| existing.api_key.clone());
        let saved_config =
            model_provider_settings_to_config(settings, api_key.or(existing_api_key));
        upsert_model_provider_config(&mut config, saved_config.clone());
        self.save_config(&config)?;
        Ok(model_provider_settings_from_config(saved_config))
    }

    pub fn delete_model_provider_settings(&self, settings_id: &str) -> Result<bool> {
        let mut config = self.load_config()?;
        let settings_id = normalized_model_provider_settings_id(settings_id);
        let original_len = config.model_providers.len();
        config.model_providers.retain(|provider| {
            normalized_model_provider_settings_id(&provider.settings_id) != settings_id
        });
        let removed = config.model_providers.len() != original_len;
        if removed {
            self.save_config(&config)?;
        }
        Ok(removed)
    }

    pub fn project_evaluation_settings(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEvaluationSettings>> {
        self.storage_store()?
            .project_evaluation_settings(project_id)
    }

    pub fn save_project_evaluation_settings(
        &self,
        mut settings: ProjectEvaluationSettings,
    ) -> Result<ProjectEvaluationSettings> {
        settings.project_recommendation_mode = ProjectRecommendationMode::Manual;
        self.storage_store()?
            .save_project_evaluation_settings(settings)
    }

    pub fn should_schedule_subject_assessment(&self, project_id: &str) -> Result<bool> {
        let settings = self
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        Ok(should_schedule_subject_assessment_for_settings(&settings))
    }

    pub fn save_subject_assessment(
        &self,
        assessment: SubjectAssessment,
    ) -> Result<SubjectAssessment> {
        self.storage_store()?.save_subject_assessment(assessment)
    }

    pub fn subject_assessments_for_asset_groups(
        &self,
        project_id: &str,
        group_ids: &[String],
    ) -> Result<Vec<SubjectAssessment>> {
        self.storage_store()?
            .subject_assessments_for_asset_groups(project_id, group_ids)
    }

    pub fn prompt_profiles_for_project(&self, project_id: &str) -> Result<Vec<PromptProfile>> {
        self.storage_store()?
            .prompt_profiles_for_project(project_id)
    }

    pub fn global_prompt_profiles(&self) -> Result<Vec<PromptProfile>> {
        Ok(self
            .storage_store()?
            .prompt_profiles_for_project("")?
            .into_iter()
            .filter(|profile| profile.scope == PromptScope::Global)
            .collect())
    }

    pub fn active_prompt_text_for_profile(
        &self,
        prompt_profile_id: &str,
    ) -> Result<Option<String>> {
        let store = self.storage_store()?;
        let Some(profile) = store.prompt_profile(prompt_profile_id)? else {
            return Ok(None);
        };
        let Some(version_id) = profile.active_version_id.as_deref() else {
            return Ok(None);
        };
        Ok(store
            .prompt_profile_version(version_id)?
            .map(|version| version.prompt_text))
    }

    pub fn create_global_prompt_profile(
        &self,
        name: impl AsRef<str>,
        style_tags: Vec<String>,
        scene_profile: SceneProfile,
        shared_preference: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptProfile> {
        let store = self.storage_store()?;
        let name = name.as_ref().trim();
        if name.is_empty() {
            return Err(crate::ImporterError::internal(
                "prompt profile name is required",
            ));
        }
        let prompt_text = prompt_profile_content_json_from_input(shared_preference.as_ref())?;
        let profile_id = format!("global-prompt-{}-{}", stable_id_fragment(name), now_ms);
        if store.prompt_profile(&profile_id)?.is_some() {
            return Err(crate::ImporterError::internal(
                "prompt profile already exists",
            ));
        }
        let version_id = format!("{profile_id}-v1");
        let profile = store.save_prompt_profile(PromptProfile {
            prompt_profile_id: profile_id.clone(),
            scope: PromptScope::Global,
            project_id: None,
            name: name.to_string(),
            style_tags: style_tags
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect(),
            scene_profile,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })?;
        let prompt_hash =
            stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, prompt_text.as_str());
        store.save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: version_id,
            prompt_profile_id: profile.prompt_profile_id.clone(),
            prompt_text,
            output_schema_version: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            prompt_hash,
            created_at_ms: now_ms,
        })?;
        store
            .prompt_profile(&profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))
    }

    pub fn fork_global_prompt_profile(
        &self,
        source_profile_id: &str,
        name: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptProfile> {
        let store = self.storage_store()?;
        let source = store
            .prompt_profile(source_profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))?;
        if source.scope != PromptScope::Global || !source.enabled {
            return Err(crate::ImporterError::internal(
                "prompt profile is not available globally",
            ));
        }
        let source_version_id = source.active_version_id.as_deref().ok_or_else(|| {
            crate::ImporterError::internal("prompt profile has no active version")
        })?;
        let source_version = store
            .prompt_profile_version(source_version_id)?
            .ok_or_else(|| crate::ImporterError::internal("active prompt version not found"))?;
        let profile_id = format!(
            "global-prompt-{}-{}",
            stable_id_fragment(source_profile_id),
            now_ms
        );
        if store.prompt_profile(&profile_id)?.is_some() {
            return Err(crate::ImporterError::internal(
                "prompt profile fork already exists",
            ));
        }
        let version_id = format!("{profile_id}-v1");
        let profile = store.save_prompt_profile(PromptProfile {
            prompt_profile_id: profile_id.clone(),
            scope: PromptScope::Global,
            project_id: None,
            name: name.as_ref().trim().to_string(),
            style_tags: source.style_tags,
            scene_profile: source.scene_profile,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })?;
        let source_prompt_text = source_version.prompt_text;
        store.save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: version_id,
            prompt_profile_id: profile.prompt_profile_id.clone(),
            prompt_text: source_prompt_text.clone(),
            output_schema_version: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            prompt_hash: stable_prompt_hash(
                MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION,
                &source_prompt_text,
            ),
            created_at_ms: now_ms,
        })?;
        store
            .prompt_profile(&profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))
    }

    pub fn save_global_prompt_profile_version(
        &self,
        prompt_profile_id: &str,
        prompt_text: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptProfile> {
        let store = self.storage_store()?;
        let profile = store
            .prompt_profile(prompt_profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))?;
        if profile.scope != PromptScope::Global || profile.built_in || !profile.enabled {
            return Err(crate::ImporterError::internal(
                "prompt profile is not editable globally",
            ));
        }
        let prompt_text = prompt_profile_content_json_from_input(prompt_text.as_ref())?;
        let prompt_hash =
            stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, prompt_text.as_str());
        let version_id = format!(
            "{}-v{}-{}",
            stable_id_fragment(prompt_profile_id),
            now_ms,
            &prompt_hash["fnv1a64-".len()..]
        );
        if store.prompt_profile_version(&version_id)?.is_some() {
            return Err(crate::ImporterError::internal(
                "prompt profile version already exists",
            ));
        }
        store.save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: version_id,
            prompt_profile_id: prompt_profile_id.to_string(),
            prompt_text,
            output_schema_version: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            prompt_hash,
            created_at_ms: now_ms,
        })?;
        store
            .prompt_profile(prompt_profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))
    }

    pub fn fork_prompt_profile_for_project(
        &self,
        project_id: &str,
        source_profile_id: &str,
        name: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptProfile> {
        let store = self.storage_store()?;
        ensure_service_project_is_active(&store, project_id)?;
        let source = store
            .prompt_profile(source_profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))?;
        if !prompt_profile_available_to_project(&source, project_id) {
            return Err(crate::ImporterError::internal(
                "prompt profile is not available to this project",
            ));
        }
        let source_version_id = source.active_version_id.as_deref().ok_or_else(|| {
            crate::ImporterError::internal("prompt profile has no active version")
        })?;
        let source_version = store
            .prompt_profile_version(source_version_id)?
            .ok_or_else(|| crate::ImporterError::internal("active prompt version not found"))?;
        let profile_id = format!(
            "project-prompt-{}-{}-{}",
            stable_id_fragment(project_id),
            stable_id_fragment(source_profile_id),
            now_ms
        );
        if store.prompt_profile(&profile_id)?.is_some() {
            return Err(crate::ImporterError::internal(
                "prompt profile fork already exists",
            ));
        }
        let version_id = format!("{profile_id}-v1");
        let profile = store.save_prompt_profile(PromptProfile {
            prompt_profile_id: profile_id.clone(),
            scope: PromptScope::Project,
            project_id: Some(project_id.to_string()),
            name: name.as_ref().trim().to_string(),
            style_tags: source.style_tags,
            scene_profile: source.scene_profile,
            active_version_id: None,
            built_in: false,
            enabled: true,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })?;
        let source_prompt_text = source_version.prompt_text;
        store.save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: version_id,
            prompt_profile_id: profile.prompt_profile_id.clone(),
            prompt_text: source_prompt_text.clone(),
            output_schema_version: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            prompt_hash: stable_prompt_hash(
                MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION,
                &source_prompt_text,
            ),
            created_at_ms: now_ms,
        })?;
        store
            .prompt_profile(&profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))
    }

    pub fn save_prompt_profile_version(
        &self,
        project_id: &str,
        prompt_profile_id: &str,
        prompt_text: impl AsRef<str>,
        now_ms: i64,
    ) -> Result<PromptProfileVersion> {
        let store = self.storage_store()?;
        ensure_service_project_is_active(&store, project_id)?;
        let profile = store
            .prompt_profile(prompt_profile_id)?
            .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))?;
        if profile.scope == PromptScope::Global && profile.built_in {
            return Err(crate::ImporterError::internal(
                "built-in prompt profiles must be forked before editing",
            ));
        }
        if profile.scope != PromptScope::Project
            || profile.project_id.as_deref() != Some(project_id)
            || !profile.enabled
        {
            return Err(crate::ImporterError::internal(
                "prompt profile is not editable for this project",
            ));
        }

        let prompt_text = prompt_profile_content_json_from_input(prompt_text.as_ref())?;
        let prompt_hash =
            stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, prompt_text.as_str());
        let version_id = format!(
            "{}-v{}-{}",
            stable_id_fragment(prompt_profile_id),
            now_ms,
            &prompt_hash["fnv1a64-".len()..]
        );
        if store.prompt_profile_version(&version_id)?.is_some() {
            return Err(crate::ImporterError::internal(
                "prompt profile version already exists",
            ));
        }
        store.save_prompt_profile_version(PromptProfileVersion {
            prompt_version_id: version_id,
            prompt_profile_id: prompt_profile_id.to_string(),
            prompt_text,
            output_schema_version: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
            prompt_hash,
            created_at_ms: now_ms,
        })
    }

    pub fn record_project_transfer(&self, project_id: &str, record: TransferRecord) -> Result<()> {
        self.storage_store()?.record_transfer(project_id, record)
    }

    pub fn claim_next_publish_item(&self) -> Result<Option<PublishQueueItem>> {
        self.storage_store()?.claim_next_publish_item()
    }

    pub fn mark_publish_completed(&self, queue_id: &str) -> Result<()> {
        self.storage_store()?.mark_publish_completed(queue_id)
    }

    pub fn complete_publish(
        &self,
        queue_id: &str,
        final_filename: &str,
        final_location: StoredObjectLocation,
    ) -> Result<TransferRecord> {
        let state_dir = self.storage_state_dir()?;
        let record =
            self.storage_store()?
                .complete_publish(queue_id, final_filename, final_location)?;
        append_transfer_record(&state_dir, &record)?;
        Ok(record)
    }

    pub fn mark_publish_failed(&self, queue_id: &str, error: &str) -> Result<()> {
        self.storage_store()?.mark_publish_failed(queue_id, error)
    }

    pub fn release_failed_publish_retries(&self, project_id: &str) -> Result<usize> {
        self.storage_store()?
            .release_failed_publish_retries(project_id)
    }

    pub fn drain_analysis_jobs(&self, limit: usize) -> Result<AnalysisDrainSummary> {
        let provider_configured = self.provider_configured_for_model_work()?;
        self.drain_analysis_jobs_with_provider_configured(limit, provider_configured)
    }

    pub fn drain_analysis_jobs_with_provider_configured(
        &self,
        limit: usize,
        provider_configured: bool,
    ) -> Result<AnalysisDrainSummary> {
        let store = self.storage_store()?;
        let providers = self.runtime_model_providers()?;
        let now = current_time_ms();
        let jobs = store.claim_analysis_jobs(now, limit)?;
        let claimed_count = jobs.len();
        let mut completed_count = 0;
        let mut failed_count = 0;

        for job in jobs {
            match run_analysis_job(&store, &job, provider_configured, &providers) {
                Ok(()) => {
                    store.complete_analysis_job(&job.job_id)?;
                    completed_count += 1;
                }
                Err(error) => {
                    let retry_at = current_time_ms().saturating_add(30_000);
                    store.fail_analysis_job(&job.job_id, &error.to_string(), retry_at)?;
                    failed_count += 1;
                }
            }
        }

        Ok(AnalysisDrainSummary {
            claimed_count,
            completed_count,
            failed_count,
        })
    }

    pub fn enqueue_model_evaluation_for_asset_groups(
        &self,
        project_id: &str,
        asset_group_ids: &[String],
    ) -> Result<usize> {
        let store = self.storage_store()?;
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        if !settings.model_evaluation_enabled {
            return Err(crate::ImporterError::internal(
                "project model evaluation is disabled",
            ));
        }
        let provider = self
            .runtime_model_provider_for_project(&store, project_id)?
            .ok_or_else(|| crate::ImporterError::internal("model provider is not configured"))?;
        if !model_provider_ready_for_work(&provider.settings)
            || !provider_has_required_secret(&provider)
        {
            return Err(crate::ImporterError::internal(
                "model provider is not configured",
            ));
        }
        let evaluator_version = evaluator_version_for_runtime_provider(Some(&provider));
        let mut enqueued_count = 0;
        let mut seen = BTreeSet::new();
        for asset_group_id in asset_group_ids {
            if !seen.insert(asset_group_id.clone()) {
                continue;
            }
            let owner_project_id = store
                .project_id_for_asset_group(asset_group_id)?
                .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
            if owner_project_id != project_id {
                return Err(crate::ImporterError::internal(
                    "asset group does not belong to project",
                ));
            }
            let mut job = NewAnalysisJob::new(
                project_id,
                AnalysisJobType::EvaluateAssetGroupWithModel,
                AnalysisEntityType::AssetGroup,
                asset_group_id,
                &format!("manual-model-eval:{project_id}:{asset_group_id}:{evaluator_version}"),
            );
            job.priority = 40;
            store.enqueue_analysis_job(job)?;
            enqueued_count += 1;
        }
        Ok(enqueued_count)
    }

    pub fn evaluate_asset_groups_with_model_inputs(
        &self,
        project_id: &str,
        inputs: &[AssetGroupModelEvaluationInput],
    ) -> Result<usize> {
        let store = self.storage_store()?;
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        if !settings.model_evaluation_enabled {
            return Err(crate::ImporterError::internal(
                "project model evaluation is disabled",
            ));
        }
        let provider = self
            .runtime_model_provider_for_project(&store, project_id)?
            .ok_or_else(|| crate::ImporterError::internal("model provider is not configured"))?;
        if !model_provider_ready_for_work(&provider.settings)
            || !provider_has_required_secret(&provider)
        {
            return Err(crate::ImporterError::internal(
                "model provider is not configured",
            ));
        }

        let mut saved_count = 0;
        let mut seen = BTreeSet::new();
        for input in inputs {
            if !seen.insert(input.asset_group_id.clone()) {
                continue;
            }
            let owner_project_id = store
                .project_id_for_asset_group(&input.asset_group_id)?
                .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
            if owner_project_id != project_id {
                return Err(crate::ImporterError::internal(
                    "asset group does not belong to project",
                ));
            }

            let now = current_time_ms();
            let assessment = assess_preview_sample(
                &input.asset_group_id,
                input.preview_sample.clone(),
                "technical-v1",
                now,
            );
            let saved_assessment = store.save_technical_assessment(assessment)?;
            let preview_image_data_url = input
                .preview_image_data_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let evaluation = model_evaluation_for_upload(
                &store,
                project_id,
                &input.asset_group_id,
                &saved_assessment,
                preview_image_data_url,
                Some(&input.preview_sample),
                Some(provider.clone()),
                EvaluationRunTrigger::Manual,
                now,
            )?;
            store.save_model_evaluation(evaluation)?;
            saved_count += 1;
        }
        Ok(saved_count)
    }

    pub fn assess_asset_group_preview(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        assessor_version: &str,
    ) -> Result<TechnicalAssessment> {
        let provider_configured = self.provider_configured_for_model_work()?;
        self.assess_asset_group_preview_with_provider_configured(
            asset_group_id,
            sample,
            assessor_version,
            provider_configured,
        )
    }

    pub fn assess_asset_group_preview_with_provider_configured(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        assessor_version: &str,
        provider_configured: bool,
    ) -> Result<TechnicalAssessment> {
        self.assess_asset_group_preview_with_image_data_url_and_provider_configured(
            asset_group_id,
            sample,
            None,
            assessor_version,
            provider_configured,
        )
    }

    pub fn assess_asset_group_preview_with_image_data_url_and_provider_configured(
        &self,
        asset_group_id: &str,
        sample: PreviewSample,
        preview_image_data_url: Option<&str>,
        assessor_version: &str,
        provider_configured: bool,
    ) -> Result<TechnicalAssessment> {
        let now = current_time_ms();
        let sample_for_model = sample.clone();
        let assessment = assess_preview_sample(asset_group_id, sample, assessor_version, now);
        let store = self.storage_store()?;
        let project_id = store
            .project_id_for_asset_group(asset_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("asset group not found"))?;
        let saved_assessment = store.save_technical_assessment(assessment)?;
        let providers = self.runtime_model_providers()?;
        if self.should_run_upload_model_evaluation(
            &store,
            &project_id,
            provider_configured,
            &providers,
        )? {
            let provider =
                runtime_model_provider_for_project_from_list(&store, &project_id, &providers)?;
            let evaluation = model_evaluation_for_upload(
                &store,
                &project_id,
                asset_group_id,
                &saved_assessment,
                preview_image_data_url,
                Some(&sample_for_model),
                provider,
                EvaluationRunTrigger::Upload,
                now,
            )?;
            store.save_model_evaluation(evaluation)?;
        }
        if let Some(burst) = store.burst_group_for_asset_group(&saved_assessment.asset_group_id)? {
            let profile = default_burst_grouping_profile(&store)?;
            let refined_bursts = store.refine_burst_group_by_visual_similarity(
                &burst.burst_group_id,
                &profile,
                assessor_version,
            )?;
            let bursts = if refined_bursts.is_empty() {
                Vec::new()
            } else {
                refined_bursts
            };
            let settings = store
                .project_evaluation_settings(&project_id)?
                .unwrap_or_else(|| {
                    ProjectEvaluationSettings::default_for_project(&project_id, now)
                });
            if settings.auto_burst_recommendation_enabled {
                for burst in bursts {
                    let dedupe_key = recommend_job_dedupe_key(&burst.burst_group_id);
                    let mut job = NewAnalysisJob::new(
                        &burst.project_id,
                        AnalysisJobType::RecommendBurstGroup,
                        AnalysisEntityType::BurstGroup,
                        &burst.burst_group_id,
                        &dedupe_key,
                    );
                    job.priority = 25;
                    store.enqueue_analysis_job(job)?;
                }
            }
        }
        Ok(saved_assessment)
    }

    pub fn recommend_burst_group_from_model(
        &self,
        burst_group_id: &str,
    ) -> Result<SelectionRecommendation> {
        self.recommend_burst_group_from_model_with_candidate_visuals(burst_group_id, &[])
    }

    pub fn recommend_burst_group_from_model_with_candidate_visuals(
        &self,
        burst_group_id: &str,
        candidate_visuals: &[SelectionCandidateVisualInput],
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let now_ms = current_time_ms();
        let burst = store
            .burst_group(burst_group_id)?
            .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
        let provider = self.runtime_model_provider_for_project(&store, &burst.project_id)?;
        let evaluations = store.model_evaluations_for_asset_groups(
            &burst.member_group_ids,
            evaluator_version_for_runtime_provider(provider.as_ref()),
        )?;
        let assessments = store
            .technical_assessments_for_asset_groups(&burst.member_group_ids, "technical-v1")?;
        let run = burst_recommendation_run(
            &store,
            &burst.project_id,
            burst_group_id,
            EvaluationRunTrigger::Manual,
            provider.as_ref().map(|provider| provider.settings.clone()),
            now_ms,
        )?;
        let settings = store
            .project_evaluation_settings(&burst.project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(&burst.project_id, now_ms)
            });
        let prompt_snapshot = prompt_snapshot_for_settings(&store, &settings)?;
        let prompt_content = prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_content.clone())
            .unwrap_or_default();
        let mut recommendation = burst_selection_recommendation_from_provider_or_evaluations(
            &burst.project_id,
            burst_group_id,
            &evaluations,
            &assessments,
            provider.as_ref(),
            candidate_visuals,
            &prompt_content,
            now_ms,
        )?;
        recommendation.run_id = Some(run.run_id.clone());
        store.save_evaluation_run(run)?;
        store.save_selection_recommendation(recommendation)
    }

    pub fn generate_project_recommendation(
        &self,
        project_id: &str,
        now_ms: i64,
    ) -> Result<SelectionRecommendation> {
        self.generate_project_recommendation_with_candidate_visuals(project_id, &[], now_ms)
    }

    pub fn generate_project_recommendation_with_candidate_visuals(
        &self,
        project_id: &str,
        candidate_visuals: &[SelectionCandidateVisualInput],
        now_ms: i64,
    ) -> Result<SelectionRecommendation> {
        let store = self.storage_store()?;
        let project = store
            .list_projects()?
            .into_iter()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| crate::ImporterError::internal("project not found"))?;
        if project.status == ProjectStatus::Archived {
            return Err(crate::ImporterError::internal("project is archived"));
        }
        let provider = self
            .runtime_model_provider_for_project(&store, project_id)?
            .ok_or_else(|| {
                crate::ImporterError::internal("model provider settings not configured")
            })?;
        if !model_provider_ready_for_work(&provider.settings)
            || !provider_has_required_secret(&provider)
        {
            return Err(crate::ImporterError::internal(
                "model provider is not configured",
            ));
        }
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, now_ms));
        let prompt_snapshot = prompt_snapshot_for_settings(&store, &settings)?;
        let prompt_content = prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_content.clone())
            .unwrap_or_default();
        let run_id = evaluation_run_id(
            project_id,
            EvaluationRunType::ProjectRecommendation,
            project_id,
            now_ms,
        );
        let run = EvaluationRun {
            run_id: run_id.clone(),
            project_id: project_id.to_string(),
            run_type: EvaluationRunType::ProjectRecommendation,
            trigger: EvaluationRunTrigger::Manual,
            status: EvaluationRunStatus::Ready,
            provider_kind: provider.settings.provider_kind,
            provider_model: provider.settings.default_model.clone(),
            prompt_profile_id: prompt_snapshot
                .as_ref()
                .map(|snapshot| snapshot.prompt_profile_id.clone()),
            prompt_version_id: prompt_snapshot
                .as_ref()
                .map(|snapshot| snapshot.prompt_version_id.clone()),
            prompt_hash: prompt_snapshot
                .as_ref()
                .map(|snapshot| snapshot.prompt_hash.clone()),
            settings_snapshot_json: serde_json::to_string(&settings)
                .map_err(|error| crate::ImporterError::internal(error.to_string()))?,
            error_message: None,
            started_at_ms: Some(now_ms),
            completed_at_ms: Some(now_ms),
            created_at_ms: now_ms,
        };
        store.save_evaluation_run(run)?;
        let group_ids = project_recommendation_candidate_group_ids(&store, project_id)?;
        let evaluations = store.model_evaluations_for_asset_groups(
            &group_ids,
            evaluator_version_for_runtime_provider(Some(&provider)),
        )?;
        let burst_recommendations =
            project_burst_recommendations_for_candidates(&store, project_id, &group_ids)?;
        let mut recommendation = project_selection_recommendation_from_provider_or_evaluations(
            project_id,
            &evaluations,
            &burst_recommendations,
            Some(&provider),
            candidate_visuals,
            &prompt_content,
            now_ms,
        )?;
        recommendation.run_id = Some(run_id);
        store.save_selection_recommendation(recommendation)
    }

    pub fn latest_project_recommendation_run_status(
        &self,
        project_id: &str,
    ) -> Result<Option<EvaluationRun>> {
        self.storage_store()?
            .latest_evaluation_run(project_id, EvaluationRunType::ProjectRecommendation)
    }

    pub fn split_burst_member(
        &self,
        burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.storage_store()?
            .split_burst_member(burst_group_id, member_group_id)
    }

    pub fn merge_burst_member(
        &self,
        target_burst_group_id: &str,
        member_group_id: &str,
    ) -> Result<Option<BurstGroup>> {
        self.storage_store()?
            .merge_burst_member(target_burst_group_id, member_group_id)
    }

    fn provider_configured_for_model_work(&self) -> Result<bool> {
        Ok(self.runtime_model_providers()?.iter().any(|provider| {
            model_provider_ready_for_work(&provider.settings)
                && provider_has_required_secret(provider)
        }))
    }

    fn runtime_model_provider(&self) -> Result<Option<RuntimeModelProvider>> {
        Ok(self.runtime_model_providers()?.into_iter().next())
    }

    fn runtime_model_providers(&self) -> Result<Vec<RuntimeModelProvider>> {
        Ok(runtime_model_providers_from_config(self.load_config()?))
    }

    fn runtime_model_provider_for_project(
        &self,
        store: &SqliteStore,
        project_id: &str,
    ) -> Result<Option<RuntimeModelProvider>> {
        let providers = self.runtime_model_providers()?;
        runtime_model_provider_for_project_from_list(store, project_id, &providers)
    }

    fn should_run_upload_model_evaluation(
        &self,
        store: &SqliteStore,
        project_id: &str,
        provider_configured: bool,
        providers: &[RuntimeModelProvider],
    ) -> Result<bool> {
        if !provider_configured {
            return Ok(false);
        }
        let settings = store
            .project_evaluation_settings(project_id)?
            .unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            });
        Ok(settings.model_evaluation_enabled
            && settings.auto_evaluate_on_upload
            && provider_configured_for_project_from_list(store, project_id, providers)?)
    }

    pub fn set_account(
        &self,
        username: impl Into<String>,
        password: Option<&str>,
        device_name: impl Into<String>,
    ) -> Result<(ReceiverAccountConfig, PathBuf)> {
        let account = ReceiverAccountConfig::new(username, password, device_name)?;
        let stored = self.storage_store()?.upsert_receiver_account(account)?;
        Ok((stored.into_account_config()?, self.config_path()))
    }

    pub fn remove_account(&self, username: &str) -> Result<(bool, PathBuf)> {
        let removed = self.storage_store()?.remove_receiver_account(username)?;
        Ok((removed, self.config_path()))
    }

    pub fn set_receiver_settings(
        &self,
        update: ReceiverSettingsUpdate,
    ) -> Result<(ReceiverSettingsConfig, PathBuf)> {
        let mut config = self.load_config()?;
        if let Some(protocol) = update.protocol {
            config.receiver.protocol = protocol;
        }
        if let Some(bind_host) = update.bind_host {
            config.receiver.bind_host = bind_host;
        }
        if let Some(ftp_port) = update.ftp_port {
            config.receiver.ftp_port = ftp_port;
        }
        if let Some(sftp_port) = update.sftp_port {
            config.receiver.sftp_port = sftp_port;
        }
        if let Some(output_dir) = update.output_dir {
            config.receiver.output_dir = Some(output_dir);
        }
        if let Some(state_dir) = update.state_dir {
            config.receiver.state_dir = Some(state_dir);
        }
        if let Some(advertised_host) = update.advertised_host {
            config.receiver.advertised_host = Some(advertised_host);
        }
        if let Some(source_name) = update.source_name {
            config.receiver.source_name = Some(source_name);
        }
        if let Some(defer_publish) = update.defer_publish {
            config.receiver.defer_publish = defer_publish;
        }
        let settings = config.receiver.clone();
        let path = self.save_config(&config)?;
        Ok((settings, path))
    }

    pub fn accounts(&self) -> Result<Vec<AccountView>> {
        let state_dir = self.storage_state_dir()?;
        let accounts = self
            .receiver_account_configs()?
            .into_values()
            .map(account_view)
            .collect();
        let devices = self.connected_devices(&state_dir, None, false)?;
        Ok(accounts_with_devices(accounts, &devices))
    }

    pub fn diagnostic_received_asset_groups(
        &self,
        output_dir: impl AsRef<Path>,
        source: ImportSource,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        scan_received_asset_groups(output_dir, source)
    }

    pub fn diagnostic_transfer_asset_groups(
        &self,
        state_dir: impl AsRef<Path>,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        self.diagnostic_transfer_asset_groups_with_query(state_dir, AssetGroupQuery::default())
    }

    pub fn diagnostic_transfer_asset_groups_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<Vec<ReceivedAssetGroup>> {
        let state_dir = state_dir.as_ref();
        let accounts = receiver_account_configs_from_state_dir(state_dir)?;
        let records = read_transfer_log(state_dir)?
            .into_iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .collect::<Vec<_>>();
        let duplicates = duplicate_info_by_transfer_id(&records, &accounts);
        let assets = records
            .into_iter()
            .map(|record| asset_from_transfer_record(record, &accounts, &duplicates))
            .filter(|asset| asset.format.is_supported_media())
            .collect::<Vec<_>>();
        Ok(group_received_assets(assets)
            .into_iter()
            .filter(|group| asset_group_matches(group, &query))
            .collect())
    }

    pub fn diagnostic_transfer_asset_summary_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
    ) -> Result<AssetGroupSummary> {
        self.diagnostic_transfer_asset_groups_with_query(state_dir, query)
            .map(|groups| summarize_asset_groups(&groups))
    }

    pub fn diagnostic_transfer_asset_group_page_with_query(
        &self,
        state_dir: impl AsRef<Path>,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        let groups = self.diagnostic_transfer_asset_groups_with_query(state_dir, query)?;
        let total_groups = groups.len();
        let summary = summarize_asset_groups(&groups);
        let page_groups = groups
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        Ok(AssetGroupPage {
            groups: page_groups,
            summary,
            offset,
            limit,
            total_groups,
            has_more: offset.saturating_add(limit) < total_groups,
        })
    }

    pub fn project_asset_group_page_with_query(
        &self,
        project_id: &str,
        query: AssetGroupQuery,
        offset: usize,
        limit: usize,
    ) -> Result<AssetGroupPage> {
        self.storage_store()?
            .asset_group_page(project_id, query, offset, limit)
    }

    pub fn project_group_assets(
        &self,
        project_id: &str,
        group_id: &str,
    ) -> Result<Vec<StoredAsset>> {
        self.storage_store()?.assets_for_group(project_id, group_id)
    }

    pub fn set_asset_group_user_marks(
        &self,
        project_id: &str,
        group_id: &str,
        favorite: Option<bool>,
        marked: Option<bool>,
    ) -> Result<AssetUserMarks> {
        self.storage_store()?
            .set_asset_group_user_marks(project_id, group_id, favorite, marked)
    }

    pub fn move_project_asset_group(
        &self,
        source_project_id: &str,
        group_id: &str,
        target_project_id: &str,
    ) -> Result<Option<crate::StoredAssetGroup>> {
        self.storage_store()?
            .move_asset_group(source_project_id, group_id, target_project_id)
    }

    pub fn receiver_status(
        &self,
        output_dir: impl AsRef<Path>,
    ) -> Result<Option<ReceiverRuntimeStatus>> {
        read_receiver_runtime_status(output_dir)
    }

    pub fn diagnostic_transfers(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
    ) -> Result<Vec<TransferRecordView>> {
        let output_dir = output_dir.as_ref();
        let accounts = receiver_account_configs_from_state_dir(output_dir)?;
        let views = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .map(|record| {
                let display_source = record_display_source(&record, &accounts);
                let virtual_display_path = record.virtual_display_path(display_source.as_deref());
                TransferRecordView {
                    final_location_kind: record.final_location_kind().map(ToOwned::to_owned),
                    final_location_label: record.final_location_label(),
                    record,
                    display_source,
                    virtual_display_path,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn project_transfers(
        &self,
        project_id: &str,
        query: TransferQuery,
    ) -> Result<Vec<TransferRecordView>> {
        let accounts = self.receiver_account_configs()?;
        let views = self
            .storage_store()?
            .transfer_records(project_id)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .map(|record| {
                let display_source = record_display_source(&record, &accounts);
                let virtual_display_path = record.virtual_display_path(display_source.as_deref());
                TransferRecordView {
                    final_location_kind: record.final_location_kind().map(ToOwned::to_owned),
                    final_location_label: record.final_location_label(),
                    record,
                    display_source,
                    virtual_display_path,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn diagnostic_transfer_summary_with_query(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
    ) -> Result<TransferSummary> {
        let output_dir = output_dir.as_ref();
        let accounts = receiver_account_configs_from_state_dir(output_dir)?;
        let records = read_transfer_log(output_dir)?
            .into_iter()
            .filter(|record| transfer_matches(record, &query, &accounts))
            .collect::<Vec<_>>();
        Ok(summarize_transfers(&records))
    }

    pub fn project_transfer_summary_with_query(
        &self,
        project_id: &str,
        query: TransferQuery,
    ) -> Result<TransferSummary> {
        let records = self
            .project_transfers(project_id, query)?
            .into_iter()
            .map(|view| view.record)
            .collect::<Vec<_>>();
        Ok(summarize_transfers(&records))
    }

    pub fn diagnostic_recent_failed_transfers(
        &self,
        output_dir: impl AsRef<Path>,
        query: TransferQuery,
        limit: usize,
    ) -> Result<Vec<TransferRecordView>> {
        let mut views = self.diagnostic_transfers(
            output_dir,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                ..query
            },
        )?;
        views.sort_by(|left, right| {
            let left_at = left
                .record
                .completed_at_ms
                .unwrap_or(left.record.started_at_ms);
            let right_at = right
                .record
                .completed_at_ms
                .unwrap_or(right.record.started_at_ms);
            right_at
                .cmp(&left_at)
                .then_with(|| right.record.started_at_ms.cmp(&left.record.started_at_ms))
                .then_with(|| right.record.transfer_id.cmp(&left.record.transfer_id))
        });
        views.truncate(limit);
        Ok(views)
    }

    pub fn project_recent_failed_transfers(
        &self,
        project_id: &str,
        query: TransferQuery,
        limit: usize,
    ) -> Result<Vec<TransferRecordView>> {
        let mut views = self.project_transfers(
            project_id,
            TransferQuery {
                status: Some(TransferStatus::Failed),
                ..query
            },
        )?;
        views.truncate(limit);
        Ok(views)
    }

    pub fn project_recent_publish_failures(
        &self,
        project_id: &str,
        limit: usize,
    ) -> Result<Vec<PublishQueueFailureView>> {
        let accounts = self.receiver_account_configs()?;
        let failures = self
            .storage_store()?
            .failed_publish_items(project_id, limit)?
            .into_iter()
            .map(|item| {
                let display_source = publish_item_display_source(&item, &accounts);
                PublishQueueFailureView {
                    queue_id: item.queue_id,
                    transfer_id: item.transfer_id,
                    final_filename: item.final_filename,
                    original_path: item.original_path,
                    display_source,
                    username: item.username,
                    attempt_count: item.attempt_count,
                    last_error: item.last_error,
                    updated_at_ms: item.updated_at_ms,
                }
            })
            .collect::<Vec<_>>();
        Ok(failures)
    }

    pub fn connected_devices(
        &self,
        output_dir: impl AsRef<Path>,
        username: Option<&str>,
        online: bool,
    ) -> Result<Vec<ConnectedDeviceView>> {
        let accounts = self.receiver_account_configs()?;
        let views = read_connected_devices(output_dir)?
            .into_iter()
            .filter(|device| device_matches(device, username, online))
            .map(|device| {
                let display_source = device_display_source(&device, &accounts)
                    .unwrap_or_else(|| remote_addr_display_label(&device.remote_addr));
                ConnectedDeviceView {
                    device,
                    display_source,
                }
            })
            .collect::<Vec<_>>();
        Ok(views)
    }

    pub fn project_dashboard(
        &self,
        project_id: &str,
        asset_query: AssetGroupQuery,
        offset: usize,
        limit: usize,
        online_devices_only: bool,
    ) -> Result<CameraConnectorDashboard> {
        let state_dir = self.storage_state_dir()?;
        let config = self.load_config()?;
        let receiver_settings = config.receiver.clone();
        let receiver_status = self.receiver_status(&state_dir)?;
        let devices = self.connected_devices(
            &state_dir,
            asset_query.username.as_deref(),
            online_devices_only,
        )?;
        let accounts = accounts_with_devices(self.accounts()?, &devices);
        let store = self.storage_store()?;
        let transfer_query = transfer_query_from_asset_query(&asset_query);
        let transfers =
            self.project_transfer_summary_with_query(project_id, transfer_query.clone())?;
        let output_dir = receiver_status
            .as_ref()
            .and_then(|status| status.output_dir.clone())
            .or_else(|| receiver_settings.output_dir.clone())
            .or_else(|| Some(CameraConnectorConfig::default_output_dir()));
        Ok(CameraConnectorDashboard {
            receiver_settings,
            paths: SystemPathsView {
                config_path: self.config_path(),
                state_dir: state_dir.clone(),
                output_dir,
            },
            receiver_status,
            accounts,
            devices,
            transfers,
            publish_queue: store.publish_queue_summary(project_id)?,
            recent_failures: self.project_recent_failed_transfers(project_id, transfer_query, 5)?,
            recent_publish_failures: self.project_recent_publish_failures(project_id, 5)?,
            assets: store.asset_group_page(project_id, asset_query, offset, limit)?,
        })
    }
}

fn receiver_account_configs_from_state_dir(
    state_dir: impl AsRef<Path>,
) -> Result<BTreeMap<String, ReceiverAccountConfig>> {
    SqliteStore::open_state_dir(state_dir)?
        .receiver_accounts()?
        .into_iter()
        .map(|account| {
            let account = account.into_account_config()?;
            Ok((account.username.clone(), account))
        })
        .collect()
}

fn transfer_query_from_asset_query(query: &AssetGroupQuery) -> TransferQuery {
    TransferQuery {
        status: None,
        transfer_id: None,
        original_path: query.original_path.clone(),
        final_filename: None,
        username: query.username.clone(),
        source_name: query.source_name.clone(),
        remote_addr: query.remote_addr.clone(),
    }
}

fn summarize_transfers(records: &[TransferRecord]) -> TransferSummary {
    TransferSummary {
        total_count: records.len(),
        completed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Completed)
            .count(),
        failed_count: records
            .iter()
            .filter(|record| record.status == TransferStatus::Failed)
            .count(),
    }
}

fn account_view(account: ReceiverAccountConfig) -> AccountView {
    let password_configured = account.password_configured();
    AccountView {
        username: account.username,
        device_name: account.device_name,
        password_configured,
        online: false,
        active_connections: 0,
        last_remote_addr: None,
        last_remote_port: None,
        last_seen_at_ms: None,
        last_disconnected_at_ms: None,
    }
}

fn accounts_with_devices(
    accounts: Vec<AccountView>,
    devices: &[ConnectedDeviceView],
) -> Vec<AccountView> {
    accounts
        .into_iter()
        .map(|mut account| {
            account.online = false;
            account.active_connections = 0;
            account.last_remote_addr = None;
            account.last_remote_port = None;
            account.last_seen_at_ms = None;
            account.last_disconnected_at_ms = None;
            let matching_devices = devices
                .iter()
                .filter(|view| view.device.username.as_deref() == Some(account.username.as_str()));
            let mut active_connections = 0u32;
            let mut latest: Option<&ConnectedDevice> = None;
            for device in matching_devices.map(|view| &view.device) {
                active_connections = active_connections.saturating_add(device.active_connections);
                latest = Some(match latest {
                    Some(current)
                        if current.online && !device.online
                            || current.online == device.online
                                && current.last_seen_at_ms >= device.last_seen_at_ms =>
                    {
                        current
                    }
                    _ => device,
                });
            }
            if let Some(device) = latest {
                account.online = device.online;
                account.active_connections = active_connections;
                account.last_remote_addr = Some(device.remote_addr.clone());
                account.last_remote_port = device.last_remote_port;
                account.last_seen_at_ms = Some(device.last_seen_at_ms);
                account.last_disconnected_at_ms = device.last_disconnected_at_ms;
            }
            account
        })
        .collect()
}

fn run_analysis_job(
    store: &SqliteStore,
    job: &AnalysisJob,
    provider_configured: bool,
    providers: &[RuntimeModelProvider],
) -> Result<()> {
    match (job.job_type, job.entity_type) {
        (AnalysisJobType::DetectBurstForAssetGroup, AnalysisEntityType::AssetGroup) => {
            let profile = default_burst_grouping_profile(store)?;
            let _ =
                store.detect_bursts_for_asset_group(&job.project_id, &job.entity_id, &profile)?;
            Ok(())
        }
        (AnalysisJobType::AssessAssetGroupTechnicalQuality, AnalysisEntityType::AssetGroup) => {
            Ok(())
        }
        (AnalysisJobType::AssessPortraitSubject, AnalysisEntityType::AssetGroup) => {
            // Core owns the scheduling and storage contract. Android/imported clients provide
            // detector output through save_subject_assessment; no detector is bundled here.
            Ok(())
        }
        (AnalysisJobType::EvaluateAssetGroupWithModel, AnalysisEntityType::AssetGroup) => {
            let settings = store
                .project_evaluation_settings(&job.project_id)?
                .unwrap_or_else(|| {
                    ProjectEvaluationSettings::default_for_project(
                        &job.project_id,
                        current_time_ms(),
                    )
                });
            let project_provider_configured = provider_configured
                && provider_configured_for_project_from_list(store, &job.project_id, providers)?;
            if !project_provider_configured || !settings.model_evaluation_enabled {
                return Ok(());
            }
            let assessments = store.technical_assessments_for_asset_groups(
                std::slice::from_ref(&job.entity_id),
                "technical-v1",
            )?;
            let assessment = assessments
                .first()
                .ok_or_else(|| crate::ImporterError::internal("technical assessment not found"))?;
            let provider =
                runtime_model_provider_for_project_from_list(store, &job.project_id, providers)?;
            let evaluation = model_evaluation_for_upload(
                store,
                &job.project_id,
                &job.entity_id,
                assessment,
                None,
                None,
                provider.clone(),
                EvaluationRunTrigger::Manual,
                current_time_ms(),
            )?;
            store.save_model_evaluation(evaluation)?;
            Ok(())
        }
        (AnalysisJobType::RecommendBurstGroup, AnalysisEntityType::BurstGroup) => {
            let settings = store
                .project_evaluation_settings(&job.project_id)?
                .unwrap_or_else(|| {
                    ProjectEvaluationSettings::default_for_project(
                        &job.project_id,
                        current_time_ms(),
                    )
                });
            if !settings.auto_burst_recommendation_enabled {
                return Ok(());
            }
            let burst = store
                .burst_group(&job.entity_id)?
                .ok_or_else(|| crate::ImporterError::internal("burst group not found"))?;
            let provider =
                runtime_model_provider_for_project_from_list(store, &burst.project_id, providers)?;
            let evaluations = store.model_evaluations_for_asset_groups(
                &burst.member_group_ids,
                evaluator_version_for_runtime_provider(provider.as_ref()),
            )?;
            if !evaluations.is_empty() {
                let now_ms = current_time_ms();
                let run = burst_recommendation_run(
                    store,
                    &burst.project_id,
                    &burst.burst_group_id,
                    EvaluationRunTrigger::BurstStable,
                    provider.as_ref().map(|provider| provider.settings.clone()),
                    now_ms,
                )?;
                let assessments = store.technical_assessments_for_asset_groups(
                    &burst.member_group_ids,
                    "technical-v1",
                )?;
                let settings = store
                    .project_evaluation_settings(&burst.project_id)?
                    .unwrap_or_else(|| {
                        ProjectEvaluationSettings::default_for_project(&burst.project_id, now_ms)
                    });
                let prompt_snapshot = prompt_snapshot_for_settings(store, &settings)?;
                let prompt_content = prompt_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.prompt_content.clone())
                    .unwrap_or_default();
                let mut scoped_recommendation =
                    burst_selection_recommendation_from_provider_or_evaluations(
                        &burst.project_id,
                        &burst.burst_group_id,
                        &evaluations,
                        &assessments,
                        provider.as_ref(),
                        &[],
                        &prompt_content,
                        now_ms,
                    )?;
                scoped_recommendation.run_id = Some(run.run_id.clone());
                store.save_evaluation_run(run)?;
                store.save_selection_recommendation(scoped_recommendation)?;
            }
            Ok(())
        }
        (AnalysisJobType::GenerateProjectRecommendation, AnalysisEntityType::Project) => {
            // Manual-only: stale/background project recommendation jobs are completed as ignored work so
            // upload drains cannot create project recommendations or retry them forever.
            Ok(())
        }
        _ => Ok(()),
    }
}

fn project_recommendation_candidate_group_ids(
    store: &SqliteStore,
    project_id: &str,
) -> Result<Vec<String>> {
    let group_ids = store
        .stored_asset_groups(project_id)?
        .into_iter()
        .map(|group| group.group_id)
        .collect::<Vec<_>>();
    let mut candidate_ids = Vec::new();
    let mut burst_group_ids = BTreeSet::new();
    for group_id in &group_ids {
        if let Some(burst) = store.burst_group_for_asset_group(group_id)? {
            burst_group_ids.insert(burst.burst_group_id);
        } else {
            candidate_ids.push(group_id.clone());
        }
    }
    for burst_group_id in burst_group_ids {
        let Some(recommendation) = store.latest_selection_recommendation(
            project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst_group_id,
        )?
        else {
            continue;
        };
        if recommendation.status == SelectionRecommendationStatus::Ready {
            candidate_ids.extend(recommendation.selected_asset_group_ids);
        }
    }
    candidate_ids.sort();
    candidate_ids.dedup();
    Ok(candidate_ids)
}

fn project_burst_recommendations_for_candidates(
    store: &SqliteStore,
    project_id: &str,
    group_ids: &[String],
) -> Result<Vec<SelectionRecommendation>> {
    let mut burst_group_ids = BTreeSet::new();
    for group_id in group_ids {
        if let Some(burst) = store.burst_group_for_asset_group(group_id)? {
            burst_group_ids.insert(burst.burst_group_id);
        }
    }
    let mut burst_recommendations = Vec::new();
    for burst_group_id in burst_group_ids {
        if let Some(recommendation) = store.latest_selection_recommendation(
            project_id,
            SelectionRecommendationScope::BurstGroup,
            &burst_group_id,
        )? {
            burst_recommendations.push(recommendation);
        }
    }
    Ok(burst_recommendations)
}

fn burst_selection_recommendation_from_provider_or_evaluations(
    project_id: &str,
    burst_group_id: &str,
    evaluations: &[crate::ModelEvaluation],
    assessments: &[TechnicalAssessment],
    provider: Option<&RuntimeModelProvider>,
    candidate_visuals: &[SelectionCandidateVisualInput],
    prompt_content: &PromptProfileContent,
    now_ms: i64,
) -> Result<SelectionRecommendation> {
    if let Some(provider) = provider.filter(|provider| {
        matches!(
            provider.settings.provider_kind,
            ModelProviderKind::OpenAi | ModelProviderKind::Custom
        ) && provider_has_required_secret(provider)
    }) {
        return recommend_selection_with_model_provider(
            project_id,
            SelectionRecommendationScope::BurstGroup,
            burst_group_id,
            evaluations,
            assessments,
            candidate_visuals,
            now_ms,
            &provider.settings,
            provider.api_key.as_deref().unwrap_or_default(),
            prompt_content,
        );
    }
    Ok(recommend_burst_group_from_model_evaluations(
        project_id,
        burst_group_id,
        evaluations,
        assessments,
        now_ms,
    ))
}

fn project_selection_recommendation_from_provider_or_evaluations(
    project_id: &str,
    evaluations: &[crate::ModelEvaluation],
    burst_recommendations: &[SelectionRecommendation],
    provider: Option<&RuntimeModelProvider>,
    candidate_visuals: &[SelectionCandidateVisualInput],
    prompt_content: &PromptProfileContent,
    now_ms: i64,
) -> Result<SelectionRecommendation> {
    if let Some(provider) = provider.filter(|provider| {
        matches!(
            provider.settings.provider_kind,
            ModelProviderKind::OpenAi | ModelProviderKind::Custom
        ) && provider_has_required_secret(provider)
    }) {
        return recommend_selection_with_model_provider(
            project_id,
            SelectionRecommendationScope::Project,
            project_id,
            evaluations,
            &[],
            candidate_visuals,
            now_ms,
            &provider.settings,
            provider.api_key.as_deref().unwrap_or_default(),
            prompt_content,
        );
    }
    Ok(recommend_project_model_selections(
        project_id,
        evaluations,
        burst_recommendations,
        now_ms,
    ))
}

fn model_evaluation_for_upload(
    store: &SqliteStore,
    project_id: &str,
    asset_group_id: &str,
    assessment: &crate::TechnicalAssessment,
    preview_image_data_url: Option<&str>,
    preview_sample: Option<&PreviewSample>,
    provider: Option<RuntimeModelProvider>,
    trigger: EvaluationRunTrigger,
    now_ms: i64,
) -> Result<crate::ModelEvaluation> {
    let settings = store
        .project_evaluation_settings(project_id)?
        .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, now_ms));
    let prompt_snapshot = prompt_snapshot_for_settings(store, &settings)?;
    let prompt_content = prompt_snapshot
        .as_ref()
        .map(|snapshot| snapshot.prompt_content.clone())
        .unwrap_or_default();
    let mut evaluation = match provider.as_ref() {
        Some(provider)
            if matches!(
                provider.settings.provider_kind,
                ModelProviderKind::OpenAi | ModelProviderKind::Custom
            ) && provider
                .api_key
                .as_deref()
                .map(str::trim)
                .is_some_and(|key| !key.is_empty()) =>
        {
            evaluate_asset_group_with_model_provider(
                project_id,
                asset_group_id,
                assessment,
                preview_image_data_url,
                preview_sample,
                now_ms,
                &provider.settings,
                provider.api_key.as_deref().unwrap_or_default(),
                &prompt_content,
            )?
        }
        Some(provider) if provider.settings.provider_kind == ModelProviderKind::Imported => {
            evaluate_asset_group_with_stub(project_id, asset_group_id, assessment, now_ms)
        }
        _ => model_evaluation_skipped(
            project_id,
            asset_group_id,
            provider
                .as_ref()
                .map(|provider| provider.settings.default_model.as_str())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("model-unconfigured"),
            "model provider API key is not configured",
            now_ms,
        ),
    };
    let run_id = evaluation_run_id(
        project_id,
        EvaluationRunType::AssetEvaluation,
        asset_group_id,
        now_ms,
    );
    let run = EvaluationRun {
        run_id: run_id.clone(),
        project_id: project_id.to_string(),
        run_type: EvaluationRunType::AssetEvaluation,
        trigger,
        status: EvaluationRunStatus::Ready,
        provider_kind: provider
            .as_ref()
            .map(|provider| provider.settings.provider_kind)
            .unwrap_or(ModelProviderKind::None),
        provider_model: provider
            .map(|provider| provider.settings.default_model)
            .unwrap_or_else(|| "model-stub-v1".to_string()),
        prompt_profile_id: prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_profile_id.clone()),
        prompt_version_id: prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_version_id.clone()),
        prompt_hash: prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_hash.clone()),
        settings_snapshot_json: serde_json::to_string(&settings)
            .map_err(|error| crate::ImporterError::internal(error.to_string()))?,
        error_message: None,
        started_at_ms: Some(now_ms),
        completed_at_ms: Some(now_ms),
        created_at_ms: now_ms,
    };
    store.save_evaluation_run(run)?;
    evaluation.run_id = run_id;
    if let Some(snapshot) = prompt_snapshot {
        evaluation.prompt_profile_id = Some(snapshot.prompt_profile_id);
        evaluation.prompt_version_id = Some(snapshot.prompt_version_id);
        evaluation.prompt_hash = Some(snapshot.prompt_hash);
    }
    Ok(evaluation)
}

fn burst_recommendation_run(
    store: &SqliteStore,
    project_id: &str,
    burst_group_id: &str,
    trigger: EvaluationRunTrigger,
    provider: Option<ModelProviderSettings>,
    now_ms: i64,
) -> Result<EvaluationRun> {
    let settings = store
        .project_evaluation_settings(project_id)?
        .unwrap_or_else(|| ProjectEvaluationSettings::default_for_project(project_id, now_ms));
    let prompt_snapshot = prompt_snapshot_for_settings(store, &settings)?;
    Ok(EvaluationRun {
        run_id: evaluation_run_id(
            project_id,
            EvaluationRunType::BurstRecommendation,
            burst_group_id,
            now_ms,
        ),
        project_id: project_id.to_string(),
        run_type: EvaluationRunType::BurstRecommendation,
        trigger,
        status: EvaluationRunStatus::Ready,
        provider_kind: provider
            .as_ref()
            .map(|settings| settings.provider_kind)
            .unwrap_or(ModelProviderKind::None),
        provider_model: provider
            .map(|settings| settings.default_model)
            .unwrap_or_else(|| "model-stub-v1".to_string()),
        prompt_profile_id: prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_profile_id.clone()),
        prompt_version_id: prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_version_id.clone()),
        prompt_hash: prompt_snapshot
            .as_ref()
            .map(|snapshot| snapshot.prompt_hash.clone()),
        settings_snapshot_json: serde_json::to_string(&settings)
            .map_err(|error| crate::ImporterError::internal(error.to_string()))?,
        error_message: None,
        started_at_ms: Some(now_ms),
        completed_at_ms: Some(now_ms),
        created_at_ms: now_ms,
    })
}

#[derive(Debug, Clone)]
struct PromptSnapshot {
    prompt_profile_id: String,
    prompt_version_id: String,
    prompt_hash: String,
    prompt_content: PromptProfileContent,
}

fn prompt_snapshot_for_settings(
    store: &SqliteStore,
    settings: &ProjectEvaluationSettings,
) -> Result<Option<PromptSnapshot>> {
    let Some(prompt_profile_id) = settings.prompt_profile_id.as_deref() else {
        return Ok(None);
    };
    let profile = store
        .prompt_profiles_for_project(&settings.project_id)?
        .into_iter()
        .find(|profile| profile.prompt_profile_id == prompt_profile_id)
        .ok_or_else(|| crate::ImporterError::internal("prompt profile not found"))?;
    let Some(version_id) = profile.active_version_id else {
        return Ok(None);
    };
    let version = store
        .prompt_profile_version(&version_id)?
        .ok_or_else(|| crate::ImporterError::internal("prompt version not found"))?;
    Ok(Some(PromptSnapshot {
        prompt_profile_id: profile.prompt_profile_id,
        prompt_version_id: version.prompt_version_id,
        prompt_hash: version.prompt_hash,
        prompt_content: prompt_profile_content_from_json(&version.prompt_text)?,
    }))
}

fn model_provider_ready_for_work(settings: &ModelProviderSettings) -> bool {
    if !settings.configured || matches!(settings.provider_kind, ModelProviderKind::None) {
        return false;
    }
    match settings.provider_kind {
        ModelProviderKind::Imported => true,
        ModelProviderKind::OpenAi | ModelProviderKind::Custom => {
            !settings.base_url.trim().is_empty() && !settings.default_model.trim().is_empty()
        }
        ModelProviderKind::None => false,
    }
}

fn provider_has_required_secret(provider: &RuntimeModelProvider) -> bool {
    match provider.settings.provider_kind {
        ModelProviderKind::Imported => true,
        ModelProviderKind::OpenAi | ModelProviderKind::Custom => provider
            .api_key
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty()),
        ModelProviderKind::None => false,
    }
}

fn evaluator_version_for_runtime_provider(provider: Option<&RuntimeModelProvider>) -> &str {
    let Some(provider) = provider else {
        return "model-stub-v1";
    };
    if provider.settings.provider_kind == ModelProviderKind::Imported {
        return "model-stub-v1";
    }
    let model = provider.settings.default_model.trim();
    if model.is_empty() {
        "model-stub-v1"
    } else {
        model
    }
}

fn runtime_model_provider_for_project_from_list(
    store: &SqliteStore,
    project_id: &str,
    providers: &[RuntimeModelProvider],
) -> Result<Option<RuntimeModelProvider>> {
    let Some(settings_id) = store
        .project_evaluation_settings(project_id)?
        .and_then(|settings| settings.model_provider_settings_id)
        .map(|value| normalized_model_provider_settings_id(&value))
    else {
        return Ok(None);
    };
    Ok(providers
        .iter()
        .find(|provider| provider.settings.settings_id == settings_id)
        .cloned())
}

fn provider_configured_for_project_from_list(
    store: &SqliteStore,
    project_id: &str,
    providers: &[RuntimeModelProvider],
) -> Result<bool> {
    Ok(
        runtime_model_provider_for_project_from_list(store, project_id, providers)?
            .as_ref()
            .is_some_and(|provider| {
                model_provider_ready_for_work(&provider.settings)
                    && provider_has_required_secret(provider)
            }),
    )
}

fn model_evaluation_skipped(
    project_id: &str,
    asset_group_id: &str,
    evaluator_version: &str,
    summary: &str,
    now_ms: i64,
) -> crate::ModelEvaluation {
    crate::ModelEvaluation {
        evaluation_id: format!(
            "model-evaluation-skipped-{}",
            stable_id_fragment(asset_group_id)
        ),
        run_id: evaluation_run_id(
            project_id,
            EvaluationRunType::AssetEvaluation,
            asset_group_id,
            now_ms,
        ),
        project_id: project_id.to_string(),
        asset_group_id: asset_group_id.to_string(),
        evaluator_kind: crate::ModelEvaluatorKind::LlmVlm,
        evaluator_version: evaluator_version.to_string(),
        status: crate::ModelEvaluationStatus::Skipped,
        score: 0,
        tier: crate::ModelEvaluationTier::Reject,
        selectable: false,
        summary: summary.to_string(),
        strengths: Vec::new(),
        weaknesses: vec![summary.to_string()],
        technical_warnings: Vec::new(),
        prompt_profile_id: None,
        prompt_version_id: None,
        prompt_hash: None,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    }
}

fn default_burst_grouping_profile(_store: &SqliteStore) -> Result<BurstGroupingProfile> {
    Ok(BurstGroupingProfile::default())
}

fn recommend_job_dedupe_key(burst_group_id: &str) -> String {
    format!("recommend:{burst_group_id}")
}

fn evaluation_run_id(
    project_id: &str,
    run_type: EvaluationRunType,
    subject_id: &str,
    now_ms: i64,
) -> String {
    let key = format!("{}:{project_id}:{subject_id}:{now_ms}", run_type.as_str());
    let mut hash = 1469598103934665603_u64;
    for byte in key.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    format!("evaluation-run-{hash:016x}")
}

fn model_provider_settings_from_config(
    config: ModelProviderSettingsConfig,
) -> ModelProviderSettings {
    let api_key_configured = config
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
        || config
            .key_alias
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some();
    ModelProviderSettings {
        settings_id: normalized_model_provider_settings_id(&config.settings_id),
        provider_kind: ModelProviderKind::from_str(config.provider_kind.trim()),
        provider_label: config.provider_label,
        base_url: config.base_url,
        default_model: config.default_model,
        default_max_image_side: if config.default_max_image_side > 0 {
            config.default_max_image_side
        } else {
            1024
        },
        default_send_mode: ModelSendMode::from_str(config.default_send_mode.trim()),
        default_batch_size: config.default_batch_size.max(1),
        configured: config.configured,
        api_key_configured,
        key_alias: config.key_alias,
        updated_at_ms: config.updated_at_ms,
    }
}

fn model_provider_settings_to_config(
    settings: ModelProviderSettings,
    api_key: Option<String>,
) -> ModelProviderSettingsConfig {
    let api_key = api_key.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    });
    ModelProviderSettingsConfig {
        settings_id: normalized_model_provider_settings_id(&settings.settings_id),
        provider_kind: settings.provider_kind.as_str().to_string(),
        provider_label: settings.provider_label,
        base_url: settings.base_url,
        default_model: settings.default_model,
        default_max_image_side: settings.default_max_image_side.max(1),
        default_send_mode: settings.default_send_mode.as_str().to_string(),
        default_batch_size: settings.default_batch_size.max(1),
        configured: settings.configured,
        api_key,
        key_alias: settings.key_alias,
        updated_at_ms: if settings.updated_at_ms == 0 {
            current_time_ms()
        } else {
            settings.updated_at_ms
        },
    }
}

fn runtime_model_providers_from_config(config: CameraConnectorConfig) -> Vec<RuntimeModelProvider> {
    config
        .model_providers
        .into_iter()
        .filter(|provider| !model_provider_config_is_empty(provider))
        .map(|provider| RuntimeModelProvider {
            api_key: provider.api_key.clone(),
            settings: model_provider_settings_from_config(provider),
        })
        .collect()
}

fn model_provider_config_by_id<'a>(
    config: &'a CameraConnectorConfig,
    settings_id: &str,
) -> Option<&'a ModelProviderSettingsConfig> {
    let settings_id = normalized_model_provider_settings_id(settings_id);
    config.model_providers.iter().find(|provider| {
        normalized_model_provider_settings_id(&provider.settings_id) == settings_id
    })
}

fn upsert_model_provider_config(
    config: &mut CameraConnectorConfig,
    provider: ModelProviderSettingsConfig,
) {
    let settings_id = normalized_model_provider_settings_id(&provider.settings_id);
    if let Some(existing) = config.model_providers.iter_mut().find(|existing| {
        normalized_model_provider_settings_id(&existing.settings_id) == settings_id
    }) {
        *existing = provider;
    } else {
        config.model_providers.push(provider);
    }
}

fn model_provider_config_is_empty(provider: &ModelProviderSettingsConfig) -> bool {
    let provider_kind = provider.provider_kind.trim();
    (provider_kind.is_empty() || provider_kind == "none")
        && provider.default_model.trim().is_empty()
        && provider.base_url.trim().is_empty()
        && !provider.configured
}

fn normalized_model_provider_settings_id(settings_id: &str) -> String {
    let settings_id = settings_id.trim();
    if settings_id.is_empty() {
        "global".to_string()
    } else {
        settings_id.to_string()
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn ensure_service_project_is_active(store: &SqliteStore, project_id: &str) -> Result<()> {
    let project = store
        .list_projects()?
        .into_iter()
        .find(|project| project.project_id == project_id)
        .ok_or_else(|| crate::ImporterError::internal("project not found"))?;
    if project.status != ProjectStatus::Active {
        return Err(crate::ImporterError::internal("project archived"));
    }
    Ok(())
}

fn prompt_profile_available_to_project(profile: &PromptProfile, project_id: &str) -> bool {
    profile.enabled
        && match profile.scope {
            PromptScope::Global => true,
            PromptScope::Project => profile.project_id.as_deref() == Some(project_id),
        }
}

fn should_schedule_subject_assessment_for_settings(settings: &ProjectEvaluationSettings) -> bool {
    settings.scene_profile == SceneProfile::Portrait
}

fn stable_prompt_hash(output_schema_version: &str, prompt_text: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in output_schema_version
        .as_bytes()
        .iter()
        .copied()
        .chain([0])
        .chain(prompt_text.as_bytes().iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64-{hash:016x}")
}

fn prompt_profile_content_json_from_input(value: &str) -> Result<String> {
    let trimmed = value.trim();
    let content = serde_json::from_str::<PromptProfileContent>(trimmed)
        .unwrap_or_else(|_| PromptProfileContent::new(trimmed));
    serde_json::to_string(&content)
        .map_err(|error| crate::ImporterError::internal(error.to_string()))
}

fn prompt_profile_content_from_json(value: &str) -> Result<PromptProfileContent> {
    serde_json::from_str(value).map_err(|error| {
        crate::ImporterError::internal(format!("invalid prompt profile content: {error}"))
    })
}

fn stable_id_fragment(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    let output = output.trim_matches('-');
    if output.is_empty() {
        "id".to_string()
    } else {
        output.to_string()
    }
}

fn summarize_asset_groups(groups: &[ReceivedAssetGroup]) -> AssetGroupSummary {
    let mut source_counts = BTreeMap::<String, usize>::new();
    let mut remote_addr_counts = BTreeMap::<String, usize>::new();

    for group in groups {
        if let Some(source) = group.primary.display_source.as_ref() {
            *source_counts.entry(source.clone()).or_default() += 1;
        }
        if let Some(remote_addr) = group.primary.remote_addr.as_ref() {
            *remote_addr_counts.entry(remote_addr.clone()).or_default() += 1;
        }
    }

    AssetGroupSummary {
        group_count: groups.len(),
        asset_count: groups.iter().map(|group| group_assets(group).len()).sum(),
        groups_with_jpeg: groups.iter().filter(|group| group.jpeg.is_some()).count(),
        groups_with_raw: groups.iter().filter(|group| group.raw.is_some()).count(),
        groups_with_video: groups.iter().filter(|group| group.video.is_some()).count(),
        source_counts: facet_counts(source_counts),
        remote_addr_counts: facet_counts(remote_addr_counts),
    }
}

fn facet_counts(counts: BTreeMap<String, usize>) -> Vec<AssetFacetCount> {
    counts
        .into_iter()
        .map(|(value, group_count)| AssetFacetCount { value, group_count })
        .collect()
}

fn asset_group_matches(group: &ReceivedAssetGroup, query: &AssetGroupQuery) -> bool {
    let asset_matches = group_assets(group)
        .into_iter()
        .any(|asset| asset_matches(asset, query));
    let favorite_matches = query
        .favorite
        .map(|expected| group.user_marks.favorite == expected)
        .unwrap_or(true);
    let marked_matches = query
        .marked
        .map(|expected| group.user_marks.marked == expected)
        .unwrap_or(true);
    asset_matches && favorite_matches && marked_matches
}

fn group_assets(group: &ReceivedAssetGroup) -> Vec<&ReceivedAsset> {
    let mut assets = Vec::new();
    push_unique_asset(&mut assets, &group.primary);
    if let Some(asset) = group.jpeg.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.raw.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    if let Some(asset) = group.video.as_ref() {
        push_unique_asset(&mut assets, asset);
    }
    assets
}

fn push_unique_asset<'a>(assets: &mut Vec<&'a ReceivedAsset>, asset: &'a ReceivedAsset) {
    if !assets.iter().any(|existing| existing.id == asset.id) {
        assets.push(asset);
    }
}

fn asset_matches(asset: &ReceivedAsset, query: &AssetGroupQuery) -> bool {
    query
        .username
        .as_ref()
        .map(|expected| asset.username.as_ref() == Some(expected))
        .unwrap_or(true)
        && query
            .source_name
            .as_ref()
            .map(|expected| asset.display_source.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .remote_addr
            .as_ref()
            .map(|expected| asset.remote_addr.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .original_path
            .as_ref()
            .map(|expected| {
                asset
                    .original_path
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
        && query
            .format
            .map(|expected| asset.format == expected)
            .unwrap_or(true)
        && query
            .role
            .map(|expected| asset.format.role() == expected)
            .unwrap_or(true)
}

fn asset_from_transfer_record(
    record: TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
    duplicates: &BTreeMap<String, DuplicateInfo>,
) -> ReceivedAsset {
    let storage_location = record.resolved_final_location();
    let display_source = record_display_source(&record, accounts);
    let virtual_display_path = record.virtual_display_path(display_source.as_deref());
    let mut asset = ReceivedAsset::new(
        record.transfer_id.clone(),
        record.final_filename.clone(),
        record.size_bytes,
        import_source_from_protocol(&record.protocol),
    );
    asset.received_time_ms = record.completed_at_ms.or(Some(record.started_at_ms));
    asset.storage_location = storage_location;
    asset.original_path = Some(record.original_path);
    asset.username = record.username;
    asset.display_source = display_source;
    asset.remote_addr = record.remote_addr;
    asset.virtual_display_path = Some(virtual_display_path);
    if let Some(duplicate) = duplicates.get(&record.transfer_id) {
        asset.duplicate_index = Some(duplicate.index);
        asset.duplicate_count = Some(duplicate.count);
    }
    asset
}

fn duplicate_info_by_transfer_id(
    records: &[TransferRecord],
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> BTreeMap<String, DuplicateInfo> {
    let mut duplicate_keys = BTreeMap::<String, Vec<&TransferRecord>>::new();
    for record in records {
        if let Some(key) = duplicate_key(record, accounts) {
            duplicate_keys.entry(key).or_default().push(record);
        }
    }

    let mut duplicates = BTreeMap::new();
    for duplicate_records in duplicate_keys.values_mut() {
        if duplicate_records.len() < 2 {
            continue;
        }
        duplicate_records.sort_by_key(|record| {
            (
                record.completed_at_ms.unwrap_or(record.started_at_ms),
                record.started_at_ms,
                record.transfer_id.clone(),
            )
        });
        let count = duplicate_records.len();
        for (index, record) in duplicate_records.iter().enumerate() {
            duplicates.insert(
                record.transfer_id.clone(),
                DuplicateInfo {
                    index: index + 1,
                    count,
                },
            );
        }
    }
    duplicates
}

fn duplicate_key(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    let original_path = normalized_duplicate_segment(&record.original_path)?;
    let identity = record
        .username
        .as_deref()
        .and_then(normalized_duplicate_segment)
        .or_else(|| {
            record_display_source(record, accounts)
                .and_then(|value| normalized_duplicate_segment(&value))
        })
        .or_else(|| {
            record
                .remote_addr
                .as_deref()
                .and_then(normalized_duplicate_segment)
        })
        .unwrap_or_else(|| "-".to_string());
    Some(format!("{identity}\t{original_path}"))
}

fn normalized_duplicate_segment(value: &str) -> Option<String> {
    let normalized = value.trim().replace('\\', "/").to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn import_source_from_protocol(protocol: &str) -> ImportSource {
    match protocol.to_ascii_lowercase().as_str() {
        "ftp" => ImportSource::FtpPush,
        "sftp" => ImportSource::SftpPush,
        _ => ImportSource::ManualDrop,
    }
}

fn transfer_matches(
    record: &TransferRecord,
    query: &TransferQuery,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> bool {
    query
        .status
        .map(|expected| record.status == expected)
        .unwrap_or(true)
        && query
            .username
            .as_ref()
            .map(|expected| record.username.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .source_name
            .as_ref()
            .map(|expected| record_display_source(record, accounts).as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .remote_addr
            .as_ref()
            .map(|expected| record.remote_addr.as_ref() == Some(expected))
            .unwrap_or(true)
        && query
            .transfer_id
            .as_ref()
            .map(|expected| record.transfer_id.contains(expected))
            .unwrap_or(true)
        && query
            .original_path
            .as_ref()
            .map(|expected| {
                record
                    .original_path
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
        && query
            .final_filename
            .as_ref()
            .map(|expected| {
                record
                    .final_filename
                    .to_ascii_lowercase()
                    .contains(&expected.to_ascii_lowercase())
            })
            .unwrap_or(true)
}

fn record_display_source(
    record: &TransferRecord,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    record
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| record.source_name.clone())
}

fn publish_item_display_source(
    item: &PublishQueueItem,
    accounts: &BTreeMap<String, ReceiverAccountConfig>,
) -> Option<String> {
    item.username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| item.source_name.clone())
}

fn device_matches(device: &ConnectedDevice, username: Option<&str>, online: bool) -> bool {
    (!online || device.online)
        && username
            .map(|expected| device.username.as_deref() == Some(expected))
            .unwrap_or(true)
}

fn device_display_source(
    device: &ConnectedDevice,
    accounts: &BTreeMap<String, crate::ReceiverAccountConfig>,
) -> Option<String> {
    device
        .username
        .as_deref()
        .and_then(|username| accounts.get(username))
        .map(|account| account.device_name.clone())
        .or_else(|| device.source_name.clone())
}

fn remote_addr_display_label(remote_addr: &str) -> String {
    if let Some(last_octet) = remote_addr
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()))
        .and_then(|value| value.parse::<u8>().ok())
    {
        return format!("IP-{last_octet:03}");
    }

    let digits = remote_addr
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    if digits.is_empty() {
        "IP".to_string()
    } else {
        let start = digits.len().saturating_sub(3);
        format!("IP-{:0>3}", &digits[start..])
    }
}
