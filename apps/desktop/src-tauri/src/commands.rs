use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

use crate::lan_discovery::{
    discover_lan_project_snapshot_sources, LanProjectSnapshotDiscoveryRequest,
    LanProjectSnapshotSource,
};
use camera_connector_core::service::AnalysisDrainSummary;
use camera_connector_core::{
    AssetGroupPage, AssetGroupQuery, BurstGroup, CameraConnectorDashboard, CameraConnectorService,
    CvPolicy, DesktopScanRun, EvaluationRunStatus, ModelProviderKind, ModelProviderSettings,
    ModelSendMode, PreviewSample, Project, ProjectEvaluationSettings, ProjectRecommendationMode,
    ProjectSyncApplySummary, PromptPack, SceneProfile, SelectionRecommendation, StoredAsset,
    SubjectAssessment, TechnicalAssessmentPolicy,
};
use image::metadata::Orientation;
use image::{DynamicImage, GrayImage, ImageBuffer, ImageDecoder, ImageReader, Rgb, RgbImage};
use pico_detect::{
    clusterize::Clusterizer, multiscale::Multiscaler, DetectMultiscale, Detector, Region,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{AppHandle, Emitter, State};

pub struct DesktopState {
    pub service: CameraConnectorService,
}

const DESKTOP_FACE_DETECTOR_MODEL: &[u8] = include_bytes!("../assets/models/face.detector.bin");
const DESKTOP_FACE_DETECTOR_VERSION: &str = "pico-detect-0.7.0/face.detector.bin";
const DESKTOP_FACE_DETECTION_MAX_EDGE: u32 = 1024;
const DESKTOP_FACE_MIN_SIZE_RATIO: f32 = 0.08;
const DESKTOP_FACE_SCORE_THRESHOLD: f32 = 35.0;
const FACE_REGION_MAX_SAMPLES: usize = 4_000;

#[derive(Debug, Clone, Deserialize)]
pub struct AssetPageRequest {
    pub project_id: String,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThumbnailRequest {
    pub source_path: String,
    pub max_edge: Option<u32>,
    pub quality: Option<ThumbnailQuality>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OriginalPreviewRequest {
    pub source_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThumbnailBatchRequest {
    pub source_paths: Vec<String>,
    pub max_edge: Option<u32>,
    pub quality: Option<ThumbnailQuality>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThumbnailQuality {
    Fast,
    Full,
}

impl Default for ThumbnailQuality {
    fn default() -> Self {
        Self::Fast
    }
}

impl ThumbnailQuality {
    fn cache_dir_name(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ThumbnailResponse {
    pub path: String,
    pub cached: bool,
    pub quality: ThumbnailQuality,
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginalPreviewResponse {
    pub path: String,
    pub cached: bool,
    pub direct_source: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThumbnailBatchItem {
    pub source_path: String,
    pub path: Option<String>,
    pub cached: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThumbnailBatchResponse {
    pub thumbnails: Vec<ThumbnailBatchItem>,
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

fn desktop_error(error: camera_connector_core::ImporterError) -> DesktopError {
    DesktopError {
        code: error.code().to_string(),
        message: error.to_string(),
    }
}

fn thumbnail_error(message: impl Into<String>) -> DesktopError {
    DesktopError {
        code: "thumbnail".to_string(),
        message: message.into(),
    }
}

fn project_sync_error(message: impl Into<String>) -> DesktopError {
    DesktopError {
        code: "project_sync".to_string(),
        message: message.into(),
    }
}

fn desktop_model_provider_settings(
    settings: ModelProviderSettings,
) -> DesktopModelProviderSettings {
    DesktopModelProviderSettings {
        settings_id: settings.settings_id,
        provider_kind: settings.provider_kind.as_str().to_string(),
        provider_label: settings.provider_label,
        base_url: settings.base_url,
        default_model: settings.default_model,
        default_max_image_side: settings.default_max_image_side,
        default_send_mode: settings.default_send_mode.as_str().to_string(),
        default_batch_size: settings.default_batch_size,
        configured: settings.configured,
        api_key_configured: settings.api_key_configured,
        key_alias: settings.key_alias,
        updated_at_ms: settings.updated_at_ms,
    }
}

fn model_provider_settings_from_request(
    request: SaveModelProviderSettingsRequest,
) -> ModelProviderSettings {
    ModelProviderSettings {
        settings_id: request.settings_id,
        provider_kind: ModelProviderKind::from_str(request.provider_kind.trim()),
        provider_label: request.provider_label,
        base_url: request.base_url,
        default_model: request.default_model,
        default_max_image_side: request.default_max_image_side.max(1),
        default_send_mode: ModelSendMode::from_str(request.default_send_mode.trim()),
        default_batch_size: request.default_batch_size.max(1),
        configured: request.configured,
        api_key_configured: request
            .api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .is_some(),
        key_alias: request.key_alias,
        updated_at_ms: current_time_ms(),
    }
}

fn desktop_prompt_pack(
    service: &CameraConnectorService,
    pack: PromptPack,
) -> Result<DesktopPromptPack, DesktopError> {
    let shared_preference = service
        .prompt_markdown_for_pack(&pack.prompt_pack_id)
        .map_err(desktop_error)?;
    Ok(DesktopPromptPack {
        prompt_pack_id: pack.prompt_pack_id,
        distribution_folder: pack.distribution_folder,
        name: pack.name,
        version: pack.version,
        author: pack.author,
        style_tags: pack.style_tags,
        scene_profile: pack.scene_profile.as_str().to_string(),
        schema: pack.schema,
        capabilities: pack.capabilities,
        built_in: pack.built_in,
        enabled: pack.enabled,
        shared_preference,
        prompt_hash: pack.prompt_hash,
        updated_at_ms: pack.updated_at_ms,
    })
}

fn desktop_prompt_packs(
    service: &CameraConnectorService,
    packs: Vec<PromptPack>,
) -> Result<Vec<DesktopPromptPack>, DesktopError> {
    packs
        .into_iter()
        .map(|pack| desktop_prompt_pack(service, pack))
        .collect()
}

fn desktop_project_evaluation_settings(
    settings: ProjectEvaluationSettings,
) -> DesktopProjectEvaluationSettings {
    DesktopProjectEvaluationSettings {
        project_id: settings.project_id,
        auto_evaluate_on_upload: settings.auto_evaluate_on_upload,
        auto_burst_recommendation_enabled: settings.auto_burst_recommendation_enabled,
        project_recommendation_mode: settings.project_recommendation_mode.as_str().to_string(),
        prompt_pack_id: settings.prompt_pack_id,
        model_provider_settings_id: settings.model_provider_settings_id,
        scene_profile: settings.scene_profile.as_str().to_string(),
        cv_policy: settings.cv_policy.as_str().to_string(),
        cv_policy_overrides: settings.cv_policy_overrides,
        allow_risky_model_selects: settings.allow_risky_model_selects,
        max_image_side: settings.max_image_side,
        batch_size: settings.batch_size,
        updated_at_ms: settings.updated_at_ms,
    }
}

fn project_evaluation_settings_from_desktop(
    settings: DesktopProjectEvaluationSettings,
) -> ProjectEvaluationSettings {
    ProjectEvaluationSettings {
        project_id: settings.project_id,
        auto_evaluate_on_upload: settings.auto_evaluate_on_upload,
        auto_burst_recommendation_enabled: settings.auto_burst_recommendation_enabled,
        project_recommendation_mode: ProjectRecommendationMode::from_str(
            settings.project_recommendation_mode.trim(),
        ),
        prompt_pack_id: settings.prompt_pack_id,
        model_provider_settings_id: settings.model_provider_settings_id,
        scene_profile: SceneProfile::from_str(settings.scene_profile.trim()),
        cv_policy: CvPolicy::from_str(settings.cv_policy.trim()),
        cv_policy_overrides: settings.cv_policy_overrides,
        allow_risky_model_selects: settings.allow_risky_model_selects,
        max_image_side: settings.max_image_side,
        batch_size: settings.batch_size,
        updated_at_ms: current_time_ms(),
    }
}

#[tauri::command]
pub fn create_project(
    state: State<'_, DesktopState>,
    name: String,
) -> Result<Project, DesktopError> {
    state.service.create_project(name).map_err(desktop_error)
}

#[tauri::command]
pub fn list_projects(state: State<'_, DesktopState>) -> Result<Vec<Project>, DesktopError> {
    state.service.list_projects().map_err(desktop_error)
}

#[tauri::command]
pub fn select_project(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<(), DesktopError> {
    state
        .service
        .set_active_project(&project_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn start_project_scan(
    app: AppHandle,
    state: State<'_, DesktopState>,
    project_id: String,
    root_path: String,
) -> Result<DesktopScanRun, DesktopError> {
    let scan = state
        .service
        .create_desktop_project_scan(&project_id, PathBuf::from(root_path))
        .map_err(desktop_error)?;
    let service = state.service.clone();
    let scan_id = scan.scan_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = service.run_desktop_project_scan(&scan_id);
        let _ = app.emit("desktop-scan-finished", result.is_ok());
    });
    Ok(scan)
}

#[tauri::command]
pub fn get_scan_status(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<Option<DesktopScanRun>, DesktopError> {
    state
        .service
        .latest_desktop_project_scan(&project_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_asset_page(
    state: State<'_, DesktopState>,
    request: AssetPageRequest,
) -> Result<AssetGroupPage, DesktopError> {
    state
        .service
        .project_asset_group_page_with_query(
            &request.project_id,
            AssetGroupQuery::default(),
            request.offset,
            request.limit,
        )
        .map_err(desktop_error)
}

#[tauri::command]
pub async fn get_asset_thumbnail(
    state: State<'_, DesktopState>,
    request: ThumbnailRequest,
) -> Result<ThumbnailResponse, DesktopError> {
    let state_dir = state.service.storage_state_dir().map_err(desktop_error)?;
    tauri::async_runtime::spawn_blocking(move || get_asset_thumbnail_blocking(state_dir, request))
        .await
        .map_err(|error| thumbnail_error(format!("thumbnail task failed: {error}")))?
}

#[tauri::command]
pub async fn get_asset_original_preview(
    state: State<'_, DesktopState>,
    request: OriginalPreviewRequest,
) -> Result<OriginalPreviewResponse, DesktopError> {
    let state_dir = state.service.storage_state_dir().map_err(desktop_error)?;
    tauri::async_runtime::spawn_blocking(move || {
        get_asset_original_preview_blocking(state_dir, request)
    })
    .await
    .map_err(|error| thumbnail_error(format!("original preview task failed: {error}")))?
}

#[tauri::command]
pub async fn get_asset_thumbnails(
    state: State<'_, DesktopState>,
    request: ThumbnailBatchRequest,
) -> Result<ThumbnailBatchResponse, DesktopError> {
    let state_dir = state.service.storage_state_dir().map_err(desktop_error)?;
    let source_paths = request.source_paths;
    let max_edge = request.max_edge;
    tauri::async_runtime::spawn_blocking(move || {
        let thumbnails = source_paths
            .into_iter()
            .map(|source_path| {
                let thumbnail_request = ThumbnailRequest {
                    source_path: source_path.clone(),
                    max_edge,
                    quality: request.quality,
                };
                match get_asset_thumbnail_blocking(state_dir.clone(), thumbnail_request) {
                    Ok(response) => ThumbnailBatchItem {
                        source_path,
                        path: Some(response.path),
                        cached: response.cached,
                        error: None,
                    },
                    Err(error) => ThumbnailBatchItem {
                        source_path,
                        path: None,
                        cached: false,
                        error: Some(error.message),
                    },
                }
            })
            .collect();
        Ok(ThumbnailBatchResponse { thumbnails })
    })
    .await
    .map_err(|error| thumbnail_error(format!("thumbnail batch task failed: {error}")))?
}

fn get_asset_thumbnail_blocking(
    state_dir: PathBuf,
    request: ThumbnailRequest,
) -> Result<ThumbnailResponse, DesktopError> {
    let source_path = PathBuf::from(request.source_path);
    let metadata = fs::metadata(&source_path)
        .map_err(|error| thumbnail_error(format!("source image is not readable: {error}")))?;
    if !metadata.is_file() {
        return Err(thumbnail_error("source image is not a file"));
    }

    let max_edge = request.max_edge.unwrap_or(512).clamp(160, 1280);
    let quality = request.quality.unwrap_or_default();
    let cache_dir = state_dir
        .join("thumb-cache")
        .join("v5")
        .join(quality.cache_dir_name());
    fs::create_dir_all(&cache_dir).map_err(|error| {
        thumbnail_error(format!("thumbnail cache could not be created: {error}"))
    })?;
    let cache_key = thumbnail_cache_key(&source_path, &metadata, max_edge);
    let output_path = cache_dir.join(format!("{cache_key}.jpg"));
    if output_path.is_file() {
        return Ok(ThumbnailResponse {
            path: output_path.to_string_lossy().into_owned(),
            cached: true,
            quality,
        });
    }

    write_thumbnail_with_quality(&source_path, &output_path, max_edge, quality)?;
    Ok(ThumbnailResponse {
        path: output_path.to_string_lossy().into_owned(),
        cached: false,
        quality,
    })
}

fn get_asset_original_preview_blocking(
    state_dir: PathBuf,
    request: OriginalPreviewRequest,
) -> Result<OriginalPreviewResponse, DesktopError> {
    let source_path = PathBuf::from(request.source_path);
    let metadata = fs::metadata(&source_path)
        .map_err(|error| thumbnail_error(format!("source image is not readable: {error}")))?;
    if !metadata.is_file() {
        return Err(thumbnail_error("source image is not a file"));
    }

    if is_browser_original_extension(&source_path) {
        return Ok(OriginalPreviewResponse {
            path: source_path.to_string_lossy().into_owned(),
            cached: true,
            direct_source: true,
        });
    }

    let cache_dir = state_dir.join("preview-cache").join("v1").join("original");
    fs::create_dir_all(&cache_dir).map_err(|error| {
        thumbnail_error(format!(
            "original preview cache could not be created: {error}"
        ))
    })?;
    let cache_key = original_preview_cache_key(&source_path, &metadata);
    let output_path = cache_dir.join(format!("{cache_key}.jpg"));
    if output_path.is_file() {
        return Ok(OriginalPreviewResponse {
            path: output_path.to_string_lossy().into_owned(),
            cached: true,
            direct_source: false,
        });
    }

    let mut image = if is_raw_extension(&source_path) {
        raw_thumbnail_image_from_file(&source_path)?
    } else {
        decoded_image_from_file(&source_path)?
    };
    write_original_preview_image(&mut image, &output_path)?;
    Ok(OriginalPreviewResponse {
        path: output_path.to_string_lossy().into_owned(),
        cached: false,
        direct_source: false,
    })
}

fn run_desktop_cv_assessment_blocking(
    service: &CameraConnectorService,
    request: DesktopCvAssessmentRequest,
    app: Option<AppHandle>,
) -> Result<DesktopCvAssessmentResponse, DesktopError> {
    let limit = request.limit.unwrap_or(1000).clamp(1, 5000);
    let mut offset = 0usize;
    let page_size = 128usize.min(limit);
    let mut assessed_count = 0usize;
    let mut failed_count = 0usize;
    let mut skipped_count = 0usize;
    let mut subject_count = 0usize;
    let scope = if request
        .asset_group_ids
        .as_ref()
        .map(|ids| ids.iter().any(|id| !id.trim().is_empty()))
        .unwrap_or(false)
    {
        "group"
    } else {
        "project"
    };
    let project_settings = service
        .project_evaluation_settings(&request.project_id)
        .map_err(desktop_error)?;
    let subject_policy = technical_policy_for_project_settings(project_settings.as_ref());
    let should_schedule_subjects = service
        .should_schedule_subject_assessment(&request.project_id)
        .unwrap_or(false);

    if let Some(group_ids) = request.asset_group_ids.clone() {
        let requested_ids: Vec<_> = group_ids
            .into_iter()
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .take(limit)
            .collect();
        let total_count = requested_ids.len().max(1);
        for group_id in requested_ids {
            match assess_desktop_cv_group(
                service,
                &request.project_id,
                &group_id,
                subject_policy,
                should_schedule_subjects,
            ) {
                DesktopCvGroupOutcome::Assessed { subjects } => {
                    assessed_count += 1;
                    subject_count += subjects;
                }
                DesktopCvGroupOutcome::Skipped => skipped_count += 1,
                DesktopCvGroupOutcome::Failed => failed_count += 1,
            }
            emit_desktop_cv_progress(
                app.as_ref(),
                &request.project_id,
                scope,
                total_count,
                assessed_count,
                failed_count,
                skipped_count,
                subject_count,
                Some(group_id),
            );
        }
        return Ok(DesktopCvAssessmentResponse {
            assessed_count,
            failed_count,
            skipped_count,
            subject_count,
        });
    }

    while assessed_count + failed_count + skipped_count < limit {
        let page = service
            .project_asset_group_page_with_query(
                &request.project_id,
                AssetGroupQuery::default(),
                offset,
                page_size,
            )
            .map_err(desktop_error)?;
        if page.groups.is_empty() {
            break;
        }
        let total_count = page.total_groups.min(limit).max(1);
        for group in page.groups {
            if assessed_count + failed_count + skipped_count >= limit {
                break;
            }
            let Some(group_id) = group.group_id.as_deref() else {
                skipped_count += 1;
                emit_desktop_cv_progress(
                    app.as_ref(),
                    &request.project_id,
                    scope,
                    total_count,
                    assessed_count,
                    failed_count,
                    skipped_count,
                    subject_count,
                    None,
                );
                continue;
            };
            match assess_desktop_cv_group(
                service,
                &request.project_id,
                group_id,
                subject_policy,
                should_schedule_subjects,
            ) {
                DesktopCvGroupOutcome::Assessed { subjects } => {
                    assessed_count += 1;
                    subject_count += subjects;
                }
                DesktopCvGroupOutcome::Skipped => skipped_count += 1,
                DesktopCvGroupOutcome::Failed => failed_count += 1,
            }
            emit_desktop_cv_progress(
                app.as_ref(),
                &request.project_id,
                scope,
                total_count,
                assessed_count,
                failed_count,
                skipped_count,
                subject_count,
                Some(group_id.to_string()),
            );
        }
        if !page.has_more {
            break;
        }
        offset = offset.saturating_add(page.limit.max(1));
    }

    Ok(DesktopCvAssessmentResponse {
        assessed_count,
        failed_count,
        skipped_count,
        subject_count,
    })
}

enum DesktopCvGroupOutcome {
    Assessed { subjects: usize },
    Skipped,
    Failed,
}

fn assess_desktop_cv_group(
    service: &CameraConnectorService,
    project_id: &str,
    group_id: &str,
    subject_policy: TechnicalAssessmentPolicy,
    should_schedule_subjects: bool,
) -> DesktopCvGroupOutcome {
    let assets = match service.project_group_assets(project_id, group_id) {
        Ok(assets) => assets,
        Err(_) => return DesktopCvGroupOutcome::Failed,
    };
    let Some(asset) = best_asset_for_cv(&assets) else {
        return DesktopCvGroupOutcome::Skipped;
    };
    let Some(source_path) = asset_local_path(asset) else {
        return DesktopCvGroupOutcome::Skipped;
    };
    let sample = match preview_sample_from_source_path(&source_path, 768) {
        Ok(sample) => sample,
        Err(_) => return DesktopCvGroupOutcome::Failed,
    };
    if service
        .assess_asset_group_preview_with_provider_configured(
            group_id,
            sample,
            "desktop-cv-technical-v1",
            false,
        )
        .is_err()
    {
        return DesktopCvGroupOutcome::Failed;
    }
    let mut subjects = 0usize;
    if should_schedule_subjects {
        match save_desktop_face_assessment(
            service,
            project_id,
            group_id,
            &source_path,
            subject_policy,
        ) {
            Ok(saved) => subjects += saved,
            Err(_) => return DesktopCvGroupOutcome::Failed,
        }
    }
    DesktopCvGroupOutcome::Assessed { subjects }
}

#[allow(clippy::too_many_arguments)]
fn emit_desktop_cv_progress(
    app: Option<&AppHandle>,
    project_id: &str,
    scope: &str,
    total_count: usize,
    assessed_count: usize,
    failed_count: usize,
    skipped_count: usize,
    subject_count: usize,
    current_group_id: Option<String>,
) {
    let Some(app) = app else {
        return;
    };
    let _ = app.emit(
        "desktop-cv-assessment-progress",
        DesktopCvAssessmentProgress {
            project_id: project_id.to_string(),
            scope: scope.to_string(),
            total_count,
            assessed_count,
            failed_count,
            skipped_count,
            subject_count,
            current_group_id,
        },
    );
}

#[derive(Debug, Clone)]
struct DesktopFaceDetection {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    score: f32,
    analysis: FaceRegionAnalysis,
}

#[derive(Debug, Clone, Copy, Default)]
struct FaceRegionAnalysis {
    area_ratio: f64,
    shadow_ratio: f64,
    highlight_ratio: f64,
    color_cast_strength: f64,
}

fn technical_policy_for_project_settings(
    settings: Option<&ProjectEvaluationSettings>,
) -> TechnicalAssessmentPolicy {
    if let Some(policy) = settings.and_then(|settings| settings.cv_policy_overrides) {
        return policy;
    }
    match settings
        .map(|settings| settings.cv_policy)
        .unwrap_or(CvPolicy::Standard)
    {
        CvPolicy::Loose => TechnicalAssessmentPolicy::loose(),
        CvPolicy::Strict => TechnicalAssessmentPolicy::strict(),
        CvPolicy::Standard => TechnicalAssessmentPolicy::standard(),
    }
}

fn save_desktop_face_assessment(
    service: &CameraConnectorService,
    project_id: &str,
    group_id: &str,
    source_path: &Path,
    policy: TechnicalAssessmentPolicy,
) -> Result<usize, DesktopError> {
    let image = desktop_face_detection_image(source_path)?;
    let faces = detect_desktop_faces(&image, policy)?;
    let primary = faces
        .iter()
        .max_by(|a, b| a.analysis.area_ratio.total_cmp(&b.analysis.area_ratio))
        .map(|face| face.analysis)
        .unwrap_or_default();
    let exposure_risk = primary.shadow_ratio >= policy.face_exposure_warn_ratio
        || primary.highlight_ratio >= policy.face_exposure_warn_ratio;
    let color_cast_risk = primary.color_cast_strength >= policy.face_color_cast_warn_threshold;
    let gate_status = if faces.is_empty() || exposure_risk || color_cast_risk {
        "warn"
    } else {
        "pass"
    };
    let regions: Vec<_> = faces
        .iter()
        .map(|face| {
            json!({
                "kind": "face",
                "x": face.x,
                "y": face.y,
                "width": face.width,
                "height": face.height,
                "area_ratio": face.analysis.area_ratio,
                "score": face.score,
                "tracking_id": serde_json::Value::Null,
                "left_eye_open_probability": serde_json::Value::Null,
                "right_eye_open_probability": serde_json::Value::Null,
            })
        })
        .collect();
    let signals = json!({
        "face_count": faces.len(),
        "image_width": image.width(),
        "image_height": image.height(),
        "largest_face_area_ratio": primary.area_ratio,
        "eyes_open_probability_min": serde_json::Value::Null,
        "closed_eyes": false,
        "face_shadow_ratio": primary.shadow_ratio,
        "face_highlight_ratio": primary.highlight_ratio,
        "face_color_cast_strength": primary.color_cast_strength,
        "face_exposure_risk": exposure_risk,
        "face_color_cast_risk": color_cast_risk,
    });
    let now = current_time_ms();
    let assessment = SubjectAssessment {
        assessment_id: format!("subject:face:{project_id}:{group_id}"),
        project_id: project_id.to_string(),
        asset_group_id: group_id.to_string(),
        subject_type: "face".to_string(),
        detector_kind: "desktop_pico".to_string(),
        detector_version: DESKTOP_FACE_DETECTOR_VERSION.to_string(),
        status: EvaluationRunStatus::Ready,
        gate_status: gate_status.to_string(),
        regions_json: serde_json::to_string(&regions).map_err(|error| {
            thumbnail_error(format!("face regions could not be encoded: {error}"))
        })?,
        signals_json: serde_json::to_string(&signals).map_err(|error| {
            thumbnail_error(format!("face signals could not be encoded: {error}"))
        })?,
        summary: desktop_face_assessment_summary(
            faces.len(),
            exposure_risk,
            color_cast_risk,
            primary,
            policy,
        ),
        created_at_ms: now,
        updated_at_ms: now,
    };
    service
        .save_subject_assessment(assessment)
        .map(|_| 1)
        .map_err(desktop_error)
}

fn desktop_face_assessment_summary(
    face_count: usize,
    exposure_risk: bool,
    color_cast_risk: bool,
    primary: FaceRegionAnalysis,
    policy: TechnicalAssessmentPolicy,
) -> String {
    if face_count == 0 {
        return "未检测到人脸。".to_string();
    }
    if exposure_risk {
        return format!(
            "检测到人脸，面部死黑/死白占比达到 {:.0}% / {:.0}%，超过 {:.0}% 阈值。",
            primary.shadow_ratio * 100.0,
            primary.highlight_ratio * 100.0,
            policy.face_exposure_warn_ratio * 100.0,
        );
    }
    if color_cast_risk {
        return format!(
            "检测到人脸，面部偏色强度 {:.2}，超过 {:.2} 阈值。",
            primary.color_cast_strength, policy.face_color_cast_warn_threshold,
        );
    }
    "检测到人脸，面部曝光和偏色可用。".to_string()
}

fn desktop_face_detection_image(source_path: &Path) -> Result<RgbImage, DesktopError> {
    let image = decode_cv_image(source_path)?;
    let image = if image.width().max(image.height()) > DESKTOP_FACE_DETECTION_MAX_EDGE {
        image.thumbnail(
            DESKTOP_FACE_DETECTION_MAX_EDGE,
            DESKTOP_FACE_DETECTION_MAX_EDGE,
        )
    } else {
        image
    };
    Ok(image.to_rgb8())
}

fn detect_desktop_faces(
    image: &RgbImage,
    policy: TechnicalAssessmentPolicy,
) -> Result<Vec<DesktopFaceDetection>, DesktopError> {
    let width = image.width();
    let height = image.height();
    let min_dim = width.min(height);
    if min_dim < 40 {
        return Ok(Vec::new());
    }
    let detector = desktop_face_detector()?;
    let gray = rgb_to_luma_image(image);
    let min_size = ((min_dim as f32) * DESKTOP_FACE_MIN_SIZE_RATIO)
        .round()
        .clamp(24.0, min_dim as f32) as u32;
    let max_size = ((min_dim as f32) * 0.85).round().max(min_size as f32) as u32;
    let runner = DetectMultiscale::builder()
        .multiscaler(
            Multiscaler::new(min_size, max_size, 0.1, 1.1).map_err(|error| {
                thumbnail_error(format!("face multiscale setup failed: {error}"))
            })?,
        )
        .clusterizer(Clusterizer::default().score_threshold(DESKTOP_FACE_SCORE_THRESHOLD))
        .build()
        .map_err(|error| thumbnail_error(format!("face detector setup failed: {error}")))?;
    let mut faces: Vec<_> = runner
        .run(detector, &gray)
        .into_iter()
        .filter_map(|detection| {
            let region = detection.region();
            let (x, y, width, height) = clamp_face_bounds(
                region.left(),
                region.top(),
                region.width(),
                region.height(),
                image,
            )?;
            Some(DesktopFaceDetection {
                x,
                y,
                width,
                height,
                score: detection.score(),
                analysis: analyze_face_region(image, x, y, width, height, policy),
            })
        })
        .collect();
    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    faces.truncate(20);
    Ok(faces)
}

fn desktop_face_detector() -> Result<&'static Detector, DesktopError> {
    static DETECTOR: OnceLock<Result<Detector, String>> = OnceLock::new();
    match DETECTOR.get_or_init(|| {
        Detector::load(Cursor::new(DESKTOP_FACE_DETECTOR_MODEL))
            .map_err(|error| format!("face detector model could not be loaded: {error}"))
    }) {
        Ok(detector) => Ok(detector),
        Err(message) => Err(thumbnail_error(message.clone())),
    }
}

fn rgb_to_luma_image(image: &RgbImage) -> GrayImage {
    let mut luma = GrayImage::new(image.width(), image.height());
    for (x, y, pixel) in image.enumerate_pixels() {
        let [r, g, b] = pixel.0;
        let value = (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b))
            .round()
            .clamp(0.0, 255.0) as u8;
        luma.put_pixel(x, y, image::Luma([value]));
    }
    luma
}

fn clamp_face_bounds(
    left: i32,
    top: i32,
    width: u32,
    height: u32,
    image: &RgbImage,
) -> Option<(u32, u32, u32, u32)> {
    let image_width = image.width() as i32;
    let image_height = image.height() as i32;
    let right = left.saturating_add(width as i32).clamp(0, image_width);
    let bottom = top.saturating_add(height as i32).clamp(0, image_height);
    let left = left.clamp(0, image_width);
    let top = top.clamp(0, image_height);
    if right <= left || bottom <= top {
        return None;
    }
    Some((
        left as u32,
        top as u32,
        (right - left) as u32,
        (bottom - top) as u32,
    ))
}

fn analyze_face_region(
    image: &RgbImage,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    policy: TechnicalAssessmentPolicy,
) -> FaceRegionAnalysis {
    if width == 0 || height == 0 || image.width() == 0 || image.height() == 0 {
        return FaceRegionAnalysis::default();
    }
    let stride = face_region_sample_stride(width, height);
    let mut samples = 0usize;
    let mut shadow_pixels = 0usize;
    let mut highlight_pixels = 0usize;
    let mut red_sum = 0.0;
    let mut green_sum = 0.0;
    let mut blue_sum = 0.0;
    let right = left.saturating_add(width).min(image.width());
    let bottom = top.saturating_add(height).min(image.height());
    for y in (top..bottom).step_by(stride) {
        for x in (left..right).step_by(stride) {
            let [r, g, b] = image.get_pixel(x, y).0;
            let luma = 0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b);
            if luma <= f64::from(policy.shadow_clip_threshold) {
                shadow_pixels += 1;
            }
            if luma >= f64::from(policy.highlight_clip_threshold) {
                highlight_pixels += 1;
            }
            red_sum += f64::from(r);
            green_sum += f64::from(g);
            blue_sum += f64::from(b);
            samples += 1;
        }
    }
    let safe_samples = samples.max(1) as f64;
    let red_mean = red_sum / safe_samples;
    let green_mean = green_sum / safe_samples;
    let blue_mean = blue_sum / safe_samples;
    let mean = (red_mean + green_mean + blue_mean) / 3.0;
    let color_cast_strength = if mean <= 1.0 {
        0.0
    } else {
        (red_mean - mean)
            .abs()
            .max((green_mean - mean).abs())
            .max((blue_mean - mean).abs())
            / mean
    };
    let image_area = f64::from(image.width()) * f64::from(image.height());
    FaceRegionAnalysis {
        area_ratio: f64::from(width) * f64::from(height) / image_area.max(1.0),
        shadow_ratio: shadow_pixels as f64 / safe_samples,
        highlight_ratio: highlight_pixels as f64 / safe_samples,
        color_cast_strength,
    }
}

fn face_region_sample_stride(width: u32, height: u32) -> usize {
    let area = f64::from(width.max(1)) * f64::from(height.max(1));
    (area / FACE_REGION_MAX_SAMPLES as f64)
        .sqrt()
        .round()
        .max(1.0) as usize
}

fn best_asset_for_cv(assets: &[StoredAsset]) -> Option<&StoredAsset> {
    assets
        .iter()
        .filter(|asset| is_photo_asset_for_cv(asset))
        .filter(|asset| asset_local_path(asset).is_some())
        .min_by_key(|asset| cv_asset_rank(asset))
}

fn is_photo_asset_for_cv(asset: &StoredAsset) -> bool {
    asset.format.is_photo()
}

fn cv_asset_rank(asset: &StoredAsset) -> u8 {
    match asset.group_role.as_str() {
        "jpeg" => 0,
        "raw" => 1,
        _ if asset.format.role().as_str() == "jpeg" => 0,
        _ if asset.format.is_raw() => 1,
        _ => 2,
    }
}

fn asset_local_path(asset: &StoredAsset) -> Option<PathBuf> {
    asset
        .final_location
        .as_ref()
        .and_then(|location| location.as_local_path())
        .map(PathBuf::from)
        .or_else(|| {
            let original_path = PathBuf::from(&asset.original_path);
            original_path.is_absolute().then_some(original_path)
        })
        .filter(|path| path.is_file())
}

fn preview_sample_from_source_path(
    source_path: &Path,
    max_edge: u32,
) -> Result<PreviewSample, DesktopError> {
    let mut image = decode_cv_image(source_path)?;
    Ok(preview_sample_from_image(
        &mut image,
        Some(source_path.to_string_lossy().into_owned()),
        max_edge,
    ))
}

fn decode_cv_image(source_path: &Path) -> Result<DynamicImage, DesktopError> {
    if is_raw_extension(source_path) {
        if let Ok(image) = raw_thumbnail_image_from_file(source_path) {
            return Ok(image);
        }
        if let Some(image) = embedded_jpeg_preview_image(source_path) {
            return Ok(image);
        }
    }

    let mut decoder = ImageReader::open(source_path)
        .map_err(|error| thumbnail_error(format!("source image could not be opened: {error}")))?
        .with_guessed_format()
        .map_err(|error| {
            thumbnail_error(format!(
                "source image format could not be detected: {error}"
            ))
        })?
        .into_decoder()
        .map_err(|error| thumbnail_error(format!("source image could not be decoded: {error}")))?;
    let orientation = decoder.orientation().map_err(|error| {
        thumbnail_error(format!(
            "source image orientation could not be read: {error}"
        ))
    })?;
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|error| thumbnail_error(format!("source image could not be decoded: {error}")))?;
    image.apply_orientation(orientation);
    Ok(image)
}

fn preview_sample_from_image(
    image: &mut DynamicImage,
    preview_source: Option<String>,
    max_edge: u32,
) -> PreviewSample {
    let source_width = image.width();
    let source_height = image.height();
    let thumbnail = if source_width.max(source_height) > max_edge {
        image.thumbnail(max_edge, max_edge)
    } else {
        image.clone()
    }
    .to_rgb8();
    let width = thumbnail.width() as usize;
    let height = thumbnail.height() as usize;
    let capacity = width.saturating_mul(height);
    let mut luma = Vec::with_capacity(capacity);
    let mut red = Vec::with_capacity(capacity);
    let mut green = Vec::with_capacity(capacity);
    let mut blue = Vec::with_capacity(capacity);
    for pixel in thumbnail.pixels() {
        let [r, g, b] = pixel.0;
        red.push(r);
        green.push(g);
        blue.push(b);
        let y = (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b))
            .round()
            .clamp(0.0, 255.0) as u8;
        luma.push(y);
    }
    PreviewSample {
        width,
        height,
        luma,
        red: Some(red),
        green: Some(green),
        blue: Some(blue),
        preview_source,
    }
}

#[tauri::command]
pub fn get_project_group_detail(
    state: State<'_, DesktopState>,
    project_id: String,
    group_id: String,
) -> Result<Vec<StoredAsset>, DesktopError> {
    state
        .service
        .project_group_assets(&project_id, &group_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub async fn run_desktop_cv_assessment(
    app: AppHandle,
    state: State<'_, DesktopState>,
    request: DesktopCvAssessmentRequest,
) -> Result<DesktopCvAssessmentResponse, DesktopError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_desktop_cv_assessment_blocking(&service, request, Some(app))
    })
    .await
    .map_err(|error| thumbnail_error(format!("desktop cv task failed: {error}")))?
}

#[tauri::command]
pub fn get_subject_assessments_for_asset_groups(
    state: State<'_, DesktopState>,
    request: SubjectAssessmentsRequest,
) -> Result<Vec<SubjectAssessment>, DesktopError> {
    state
        .service
        .subject_assessments_for_asset_groups(&request.project_id, &request.asset_group_ids)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn delete_project_asset_group(
    state: State<'_, DesktopState>,
    project_id: String,
    group_id: String,
) -> Result<bool, DesktopError> {
    state
        .service
        .delete_project_asset_group(&project_id, &group_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn save_group_user_marks(
    state: State<'_, DesktopState>,
    request: UserMarksRequest,
) -> Result<camera_connector_core::AssetUserMarks, DesktopError> {
    state
        .service
        .set_asset_group_user_marks(
            &request.project_id,
            &request.group_id,
            request.favorite,
            request.marked,
        )
        .map_err(desktop_error)
}

fn sync_project_snapshot_from_path_blocking(
    service: &CameraConnectorService,
    request: SyncProjectSnapshotRequest,
) -> Result<SyncProjectSnapshotResponse, DesktopError> {
    let raw = fs::read_to_string(&request.snapshot_path)
        .map_err(camera_connector_core::ImporterError::from)
        .map_err(desktop_error)?;
    let snapshot =
        camera_connector_core::parse_project_sync_snapshot_json(&raw).map_err(desktop_error)?;
    service
        .sync_project_snapshot(&request.project_id, &snapshot)
        .map(SyncProjectSnapshotResponse::from)
        .map_err(desktop_error)
}

fn sync_project_snapshot_from_url_blocking(
    service: &CameraConnectorService,
    request: SyncProjectSnapshotUrlRequest,
) -> Result<SyncProjectSnapshotResponse, DesktopError> {
    let response = reqwest::blocking::get(&request.snapshot_url)
        .map_err(|error| project_sync_error(format!("project snapshot request failed: {error}")))?;
    if !response.status().is_success() {
        return Err(project_sync_error(format!(
            "project snapshot request returned HTTP {}",
            response.status()
        )));
    }
    let raw = response.text().map_err(|error| {
        project_sync_error(format!("project snapshot body could not be read: {error}"))
    })?;
    let snapshot =
        camera_connector_core::parse_project_sync_snapshot_json(&raw).map_err(desktop_error)?;
    service
        .sync_project_snapshot(&request.project_id, &snapshot)
        .map(SyncProjectSnapshotResponse::from)
        .map_err(desktop_error)
}

#[tauri::command]
pub async fn sync_project_snapshot_from_path(
    state: State<'_, DesktopState>,
    request: SyncProjectSnapshotRequest,
) -> Result<SyncProjectSnapshotResponse, DesktopError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || {
        sync_project_snapshot_from_path_blocking(&service, request)
    })
    .await
    .map_err(|error| thumbnail_error(format!("project snapshot sync task failed: {error}")))?
}

#[tauri::command]
pub async fn sync_project_snapshot_from_url(
    state: State<'_, DesktopState>,
    request: SyncProjectSnapshotUrlRequest,
) -> Result<SyncProjectSnapshotResponse, DesktopError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || {
        sync_project_snapshot_from_url_blocking(&service, request)
    })
    .await
    .map_err(|error| project_sync_error(format!("project snapshot sync task failed: {error}")))?
}

#[tauri::command]
pub async fn discover_lan_project_snapshots(
    request: LanProjectSnapshotDiscoveryRequest,
) -> Result<Vec<LanProjectSnapshotSource>, DesktopError> {
    tauri::async_runtime::spawn_blocking(move || {
        discover_lan_project_snapshot_sources(request).map_err(project_sync_error)
    })
    .await
    .map_err(|error| project_sync_error(format!("LAN project discovery task failed: {error}")))?
}

#[tauri::command]
pub fn get_model_provider_settings_list(
    state: State<'_, DesktopState>,
) -> Result<Vec<DesktopModelProviderSettings>, DesktopError> {
    state
        .service
        .model_provider_settings_list()
        .map(|settings| {
            settings
                .into_iter()
                .map(desktop_model_provider_settings)
                .collect()
        })
        .map_err(desktop_error)
}

#[tauri::command]
pub fn save_model_provider_settings(
    state: State<'_, DesktopState>,
    request: SaveModelProviderSettingsRequest,
) -> Result<DesktopModelProviderSettings, DesktopError> {
    let api_key = request.api_key.clone();
    let settings = model_provider_settings_from_request(request);
    state
        .service
        .save_model_provider_settings_with_api_key(settings, api_key)
        .map(desktop_model_provider_settings)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn delete_model_provider_settings(
    state: State<'_, DesktopState>,
    settings_id: String,
) -> Result<bool, DesktopError> {
    state
        .service
        .delete_model_provider_settings(&settings_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_evaluation_settings(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<DesktopProjectEvaluationSettings, DesktopError> {
    state
        .service
        .project_evaluation_settings(&project_id)
        .map(|settings| {
            settings.unwrap_or_else(|| {
                ProjectEvaluationSettings::default_for_project(project_id, current_time_ms())
            })
        })
        .map(desktop_project_evaluation_settings)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn save_project_evaluation_settings(
    state: State<'_, DesktopState>,
    settings: DesktopProjectEvaluationSettings,
) -> Result<DesktopProjectEvaluationSettings, DesktopError> {
    state
        .service
        .save_project_evaluation_settings(project_evaluation_settings_from_desktop(settings))
        .map(desktop_project_evaluation_settings)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn get_global_prompt_packs(
    state: State<'_, DesktopState>,
) -> Result<Vec<DesktopPromptPack>, DesktopError> {
    let packs = state.service.global_prompt_packs().map_err(desktop_error)?;
    desktop_prompt_packs(&state.service, packs)
}

#[tauri::command]
pub fn get_project_prompt_packs(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<Vec<DesktopPromptPack>, DesktopError> {
    let packs = state
        .service
        .prompt_packs_for_project(&project_id)
        .map_err(desktop_error)?;
    desktop_prompt_packs(&state.service, packs)
}

#[tauri::command]
pub fn create_global_prompt_pack(
    state: State<'_, DesktopState>,
    request: CreatePromptPackRequest,
) -> Result<DesktopPromptPack, DesktopError> {
    let pack = state
        .service
        .create_global_prompt_pack(
            request.name,
            request.style_tags,
            SceneProfile::from_str(request.scene_profile.trim()),
            request.distribution_folder,
            request.shared_preference,
            current_time_ms(),
        )
        .map_err(desktop_error)?;
    desktop_prompt_pack(&state.service, pack)
}

#[tauri::command]
pub fn fork_global_prompt_pack(
    state: State<'_, DesktopState>,
    request: ForkPromptPackRequest,
) -> Result<DesktopPromptPack, DesktopError> {
    let pack = state
        .service
        .fork_global_prompt_pack(
            &request.source_prompt_pack_id,
            request.name,
            request.distribution_folder,
            current_time_ms(),
        )
        .map_err(desktop_error)?;
    desktop_prompt_pack(&state.service, pack)
}

#[tauri::command]
pub fn save_global_prompt_pack(
    state: State<'_, DesktopState>,
    request: SavePromptPackRequest,
) -> Result<DesktopPromptPack, DesktopError> {
    let pack = state
        .service
        .save_global_prompt_pack(
            &request.prompt_pack_id,
            request.name,
            request.style_tags,
            SceneProfile::from_str(request.scene_profile.trim()),
            request.shared_preference,
            current_time_ms(),
        )
        .map_err(desktop_error)?;
    desktop_prompt_pack(&state.service, pack)
}

#[tauri::command]
pub fn delete_global_prompt_pack(
    state: State<'_, DesktopState>,
    prompt_pack_id: String,
) -> Result<bool, DesktopError> {
    state
        .service
        .delete_global_prompt_pack(&prompt_pack_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn enqueue_model_evaluation_for_asset_groups(
    state: State<'_, DesktopState>,
    request: EnqueueModelEvaluationRequest,
) -> Result<EnqueueModelEvaluationResponse, DesktopError> {
    state
        .service
        .enqueue_model_evaluation_for_asset_groups(&request.project_id, &request.asset_group_ids)
        .map(|enqueued_count| EnqueueModelEvaluationResponse { enqueued_count })
        .map_err(desktop_error)
}

#[tauri::command]
pub fn drain_analysis_jobs(
    state: State<'_, DesktopState>,
    limit: usize,
) -> Result<AnalysisDrainSummary, DesktopError> {
    state
        .service
        .drain_analysis_jobs(limit)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn recommend_burst_group(
    state: State<'_, DesktopState>,
    burst_group_id: String,
) -> Result<SelectionRecommendation, DesktopError> {
    state
        .service
        .recommend_burst_group_from_model(&burst_group_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn split_burst_member(
    state: State<'_, DesktopState>,
    burst_group_id: String,
    member_group_id: String,
) -> Result<Option<BurstGroup>, DesktopError> {
    state
        .service
        .split_burst_member(&burst_group_id, &member_group_id)
        .map_err(desktop_error)
}

#[tauri::command]
pub fn generate_project_recommendation(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<SelectionRecommendation, DesktopError> {
    state
        .service
        .generate_project_recommendation(&project_id, current_time_ms())
        .map_err(desktop_error)
}

#[tauri::command]
pub fn get_project_dashboard(
    state: State<'_, DesktopState>,
    project_id: String,
) -> Result<CameraConnectorDashboard, DesktopError> {
    state
        .service
        .project_dashboard(&project_id, AssetGroupQuery::default(), 0, 50, false)
        .map_err(desktop_error)
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn thumbnail_cache_key(source_path: &Path, metadata: &fs::Metadata, max_edge: u32) -> String {
    let canonical_path = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    canonical_path.to_string_lossy().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified_at.hash(&mut hasher);
    max_edge.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn original_preview_cache_key(source_path: &Path, metadata: &fs::Metadata) -> String {
    let canonical_path = source_path
        .canonicalize()
        .unwrap_or_else(|_| source_path.to_path_buf());
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    canonical_path.to_string_lossy().hash(&mut hasher);
    metadata.len().hash(&mut hasher);
    modified_at.hash(&mut hasher);
    "original-preview-v1".hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn write_thumbnail_with_quality(
    source_path: &Path,
    output_path: &Path,
    max_edge: u32,
    quality: ThumbnailQuality,
) -> Result<(), DesktopError> {
    if quality == ThumbnailQuality::Fast {
        if let Some(mut image) = embedded_jpeg_preview_image(source_path) {
            return write_thumbnail_image(&mut image, output_path, max_edge);
        }
    }

    if quality == ThumbnailQuality::Full && is_raw_extension(source_path) {
        if let Ok(mut image) = raw_thumbnail_image_from_file(source_path) {
            return write_thumbnail_image(&mut image, output_path, max_edge);
        }
    }

    let mut image = decoded_image_from_file(source_path)?;
    write_thumbnail_image(&mut image, output_path, max_edge)
}

fn decoded_image_from_file(source_path: &Path) -> Result<DynamicImage, DesktopError> {
    let mut decoder = ImageReader::open(source_path)
        .map_err(|error| thumbnail_error(format!("source image could not be opened: {error}")))?
        .with_guessed_format()
        .map_err(|error| {
            thumbnail_error(format!(
                "source image format could not be detected: {error}"
            ))
        })?
        .into_decoder()
        .map_err(|error| thumbnail_error(format!("source image could not be decoded: {error}")))?;
    let orientation = decoder.orientation().map_err(|error| {
        thumbnail_error(format!(
            "source image orientation could not be read: {error}"
        ))
    })?;
    let mut image = DynamicImage::from_decoder(decoder)
        .map_err(|error| thumbnail_error(format!("source image could not be decoded: {error}")))?;
    image.apply_orientation(orientation);
    Ok(image)
}

fn write_thumbnail_image(
    image: &mut DynamicImage,
    output_path: &Path,
    max_edge: u32,
) -> Result<(), DesktopError> {
    let thumbnail = image.thumbnail(max_edge, max_edge);
    let temporary_path = output_path.with_extension(format!("jpg.{}.tmp", current_time_ms()));
    let file = File::create(&temporary_path).map_err(|error| {
        thumbnail_error(format!("thumbnail file could not be created: {error}"))
    })?;
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 78);
    encoder
        .encode_image(&thumbnail)
        .map_err(|error| thumbnail_error(format!("thumbnail could not be encoded: {error}")))?;
    writer.flush().map_err(|error| {
        thumbnail_error(format!("thumbnail file could not be flushed: {error}"))
    })?;
    drop(writer);
    match fs::rename(&temporary_path, output_path) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            fs::copy(&temporary_path, output_path).map_err(|copy_error| {
                thumbnail_error(format!(
                    "thumbnail file could not be stored: {copy_error}; rename failed with {rename_error}"
                ))
            })?;
            let _ = fs::remove_file(&temporary_path);
            Ok(())
        }
    }
}

fn write_original_preview_image(
    image: &mut DynamicImage,
    output_path: &Path,
) -> Result<(), DesktopError> {
    let temporary_path = output_path.with_extension(format!("jpg.{}.tmp", current_time_ms()));
    let file = File::create(&temporary_path).map_err(|error| {
        thumbnail_error(format!(
            "original preview file could not be created: {error}"
        ))
    })?;
    let mut writer = BufWriter::new(file);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 90);
    encoder.encode_image(image).map_err(|error| {
        thumbnail_error(format!("original preview could not be encoded: {error}"))
    })?;
    writer.flush().map_err(|error| {
        thumbnail_error(format!(
            "original preview file could not be flushed: {error}"
        ))
    })?;
    drop(writer);
    match fs::rename(&temporary_path, output_path) {
        Ok(()) => Ok(()),
        Err(rename_error) => {
            fs::copy(&temporary_path, output_path).map_err(|copy_error| {
                thumbnail_error(format!(
                    "original preview file could not be stored: {copy_error}; rename failed with {rename_error}"
                ))
            })?;
            let _ = fs::remove_file(&temporary_path);
            Ok(())
        }
    }
}

fn embedded_jpeg_preview_image(source_path: &Path) -> Option<DynamicImage> {
    let preview = embedded_jpeg_preview_bytes(source_path)?;
    let mut image = image::load_from_memory(&preview).ok()?;
    if let Some(orientation) = embedded_preview_orientation(source_path) {
        image.apply_orientation(orientation);
    }
    Some(image)
}

fn embedded_jpeg_preview_bytes(source_path: &Path) -> Option<Vec<u8>> {
    read_jpeg_exif_payload(source_path)
        .and_then(|payload| embedded_jpeg_from_exif_payload(&payload))
        .or_else(|| read_tiff_embedded_jpeg(source_path))
}

fn embedded_preview_orientation(source_path: &Path) -> Option<Orientation> {
    read_jpeg_exif_payload(source_path)
        .and_then(|payload| tiff_orientation_from_exif_payload(&payload))
        .or_else(|| read_tiff_orientation(source_path))
}

fn read_jpeg_exif_payload(source_path: &Path) -> Option<Vec<u8>> {
    let file = File::open(source_path).ok()?;
    let mut reader = BufReader::new(file);
    let mut marker = [0u8; 2];
    reader.read_exact(&mut marker).ok()?;
    if marker != [0xff, 0xd8] {
        return None;
    }

    loop {
        let mut byte = [0u8; 1];
        if reader.read_exact(&mut byte).is_err() {
            return None;
        }
        if byte[0] != 0xff {
            continue;
        }
        loop {
            reader.read_exact(&mut byte).ok()?;
            if byte[0] != 0xff {
                break;
            }
        }
        let marker = byte[0];
        if marker == 0xda || marker == 0xd9 {
            return None;
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let mut length = [0u8; 2];
        reader.read_exact(&mut length).ok()?;
        let segment_len = u16::from_be_bytes(length) as usize;
        if segment_len < 2 {
            return None;
        }
        let payload_len = segment_len - 2;
        if marker == 0xe1 {
            let mut payload = vec![0u8; payload_len];
            reader.read_exact(&mut payload).ok()?;
            if payload.starts_with(b"Exif\0\0") {
                return Some(payload);
            }
        } else {
            skip_exact(&mut reader, payload_len)?;
        }
    }
}

fn skip_exact(reader: &mut impl Read, mut len: usize) -> Option<()> {
    let mut buffer = [0u8; 4096];
    while len > 0 {
        let chunk_len = len.min(buffer.len());
        reader.read_exact(&mut buffer[..chunk_len]).ok()?;
        len -= chunk_len;
    }
    Some(())
}

fn embedded_jpeg_from_exif_payload(payload: &[u8]) -> Option<Vec<u8>> {
    let tiff = payload.strip_prefix(b"Exif\0\0")?;
    embedded_jpeg_from_tiff_payload(tiff)
}

fn tiff_orientation_from_exif_payload(payload: &[u8]) -> Option<Orientation> {
    let tiff = payload.strip_prefix(b"Exif\0\0")?;
    tiff_orientation_from_payload(tiff)
}

fn read_tiff_orientation(source_path: &Path) -> Option<Orientation> {
    let mut file = File::open(source_path).ok()?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let endian = TiffEndian::from_header(&header)?;
    let first_ifd_offset = endian.read_u32(&header, 4)? as usize;
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut payload = Vec::new();
    file.read_to_end(&mut payload).ok()?;
    tiff_orientation_from_ifd_payload(&payload, endian, first_ifd_offset)
}

fn tiff_orientation_from_payload(tiff: &[u8]) -> Option<Orientation> {
    let endian = TiffEndian::from_header(tiff)?;
    let first_ifd_offset = endian.read_u32(tiff, 4)? as usize;
    tiff_orientation_from_ifd_payload(tiff, endian, first_ifd_offset)
}

fn tiff_orientation_from_ifd_payload(
    tiff: &[u8],
    endian: TiffEndian,
    ifd_offset: usize,
) -> Option<Orientation> {
    let mut orientation = None;
    for_each_ifd_entry(tiff, endian, ifd_offset, |tag, value| {
        if tag == 0x0112 {
            orientation = Orientation::from_exif(value as u8);
        }
    })?;
    orientation
}

fn embedded_jpeg_from_tiff_payload(tiff: &[u8]) -> Option<Vec<u8>> {
    if tiff.len() < 8 {
        return None;
    }
    let endian = TiffEndian::from_header(tiff)?;
    if endian.read_u16(tiff, 2)? != 42 {
        return None;
    }
    let ifd0_offset = usize::try_from(endian.read_u32(tiff, 4)?).ok()?;
    let ifd1_offset = usize::try_from(next_ifd_offset(tiff, endian, ifd0_offset)?).ok()?;
    if ifd1_offset == 0 {
        return None;
    }
    let mut jpeg_offset = None;
    let mut jpeg_len = None;
    for_each_ifd_entry(tiff, endian, ifd1_offset, |tag, value| match tag {
        0x0201 => jpeg_offset = Some(value),
        0x0202 => jpeg_len = Some(value),
        _ => {}
    })?;
    let start = usize::try_from(jpeg_offset?).ok()?;
    let len = usize::try_from(jpeg_len?).ok()?;
    let end = start.checked_add(len)?;
    let data = tiff.get(start..end)?;
    if !data.starts_with(&[0xff, 0xd8]) {
        return None;
    }
    Some(data.to_vec())
}

fn read_tiff_embedded_jpeg(source_path: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(source_path).ok()?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let endian = TiffEndian::from_header(&header)?;
    if endian.read_u16(&header, 2)? != 42 {
        return None;
    }
    let first_ifd_offset = u64::from(endian.read_u32(&header, 4)?);
    read_tiff_ifd_embedded_jpeg(&mut file, endian, first_ifd_offset, 0)
}

fn read_tiff_ifd_embedded_jpeg(
    file: &mut File,
    endian: TiffEndian,
    ifd_offset: u64,
    depth: u8,
) -> Option<Vec<u8>> {
    if depth > 4 || ifd_offset == 0 {
        return None;
    }
    file.seek(SeekFrom::Start(ifd_offset)).ok()?;
    let mut count_bytes = [0u8; 2];
    file.read_exact(&mut count_bytes).ok()?;
    let count = usize::from(endian.read_u16(&count_bytes, 0)?);
    let mut jpeg_offset = None;
    let mut jpeg_len = None;
    let mut child_ifd_offsets = Vec::new();

    for _ in 0..count {
        let mut entry = [0u8; 12];
        file.read_exact(&mut entry).ok()?;
        let tag = endian.read_u16(&entry, 0)?;
        let field_type = endian.read_u16(&entry, 2)?;
        let component_count = endian.read_u32(&entry, 4)?;
        let value_or_offset = endian.read_u32(&entry, 8)?;
        match tag {
            0x0201 if component_count == 1 => {
                jpeg_offset = tiff_entry_first_u32(
                    file,
                    endian,
                    field_type,
                    component_count,
                    value_or_offset,
                );
            }
            0x0202 if component_count == 1 => {
                jpeg_len = tiff_entry_first_u32(
                    file,
                    endian,
                    field_type,
                    component_count,
                    value_or_offset,
                );
            }
            0x014a => {
                child_ifd_offsets.extend(tiff_entry_u32_values(
                    file,
                    endian,
                    field_type,
                    component_count,
                    value_or_offset,
                    8,
                ));
            }
            _ => {}
        }
    }

    if let (Some(offset), Some(len)) = (jpeg_offset, jpeg_len) {
        let data = read_file_range(file, u64::from(offset), usize::try_from(len).ok()?)?;
        if data.starts_with(&[0xff, 0xd8]) {
            return Some(data);
        }
    }

    let mut next_offset = [0u8; 4];
    file.read_exact(&mut next_offset).ok()?;
    let next_ifd_offset = endian.read_u32(&next_offset, 0)?;
    if next_ifd_offset != 0 {
        child_ifd_offsets.push(next_ifd_offset);
    }

    for child_offset in child_ifd_offsets {
        if let Some(data) =
            read_tiff_ifd_embedded_jpeg(file, endian, u64::from(child_offset), depth + 1)
        {
            return Some(data);
        }
    }
    None
}

fn is_raw_extension(source_path: &Path) -> bool {
    matches!(
        source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("nef" | "nrw" | "cr2" | "cr3" | "arw" | "raf" | "rw2" | "orf" | "pef" | "dng")
    )
}

fn is_browser_original_extension(source_path: &Path) -> bool {
    matches!(
        source_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp")
    )
}

fn raw_thumbnail_image_from_file(source_path: &Path) -> Result<DynamicImage, DesktopError> {
    let mut raw = rawloader::decode_file(source_path)
        .map_err(|error| thumbnail_error(format!("raw source could not be decoded: {error}")))?;
    raw_sensor_thumbnail_image(&mut raw)
}

fn raw_sensor_thumbnail_image(raw: &mut rawloader::RawImage) -> Result<DynamicImage, DesktopError> {
    let image = match &raw.data {
        rawloader::RawImageData::Integer(data) => raw_integer_to_rgb_image(raw, data)?,
        rawloader::RawImageData::Float(data) => raw_float_to_rgb_image(raw, data)?,
    };
    let mut image = DynamicImage::ImageRgb8(image);
    image.apply_orientation(rawloader_orientation_to_image(raw.orientation));
    Ok(image)
}

fn raw_integer_to_rgb_image(
    raw: &rawloader::RawImage,
    data: &[u16],
) -> Result<RgbImage, DesktopError> {
    let width = raw.width;
    let height = raw.height;
    if data.len() < width.saturating_mul(height).saturating_mul(raw.cpp.max(1)) {
        return Err(thumbnail_error("raw source has incomplete sensor data"));
    }
    let mut image = ImageBuffer::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let rgb = if raw.cpp >= 3 {
                let base = (y * width + x) * raw.cpp;
                [
                    scale_raw_value(data[base], raw.blacklevels[0], raw.whitelevels[0]),
                    scale_raw_value(data[base + 1], raw.blacklevels[1], raw.whitelevels[1]),
                    scale_raw_value(data[base + 2], raw.blacklevels[2], raw.whitelevels[2]),
                ]
            } else {
                demosaic_raw_pixel(raw, data, x, y)
            };
            image.put_pixel(x as u32, y as u32, Rgb(rgb));
        }
    }
    Ok(image)
}

fn raw_float_to_rgb_image(
    raw: &rawloader::RawImage,
    data: &[f32],
) -> Result<RgbImage, DesktopError> {
    let width = raw.width;
    let height = raw.height;
    if data.len() < width.saturating_mul(height).saturating_mul(raw.cpp.max(1)) {
        return Err(thumbnail_error("raw source has incomplete sensor data"));
    }
    let mut image = ImageBuffer::new(width as u32, height as u32);
    for y in 0..height {
        for x in 0..width {
            let value = if raw.cpp >= 3 {
                let base = (y * width + x) * raw.cpp;
                [
                    scale_float_value(data[base]),
                    scale_float_value(data[base + 1]),
                    scale_float_value(data[base + 2]),
                ]
            } else {
                let gray = scale_float_value(data[y * width + x]);
                [gray, gray, gray]
            };
            image.put_pixel(x as u32, y as u32, Rgb(value));
        }
    }
    Ok(image)
}

fn demosaic_raw_pixel(raw: &rawloader::RawImage, data: &[u16], x: usize, y: usize) -> [u8; 3] {
    if !raw.cfa.is_valid() {
        let gray = scale_raw_value(
            data[y * raw.width + x],
            raw.blacklevels[0],
            raw.whitelevels[0],
        );
        return [gray, gray, gray];
    }
    let mut sum = [0u32; 3];
    let mut count = [0u32; 3];
    let y_start = y.saturating_sub(1);
    let y_end = (y + 1).min(raw.height.saturating_sub(1));
    let x_start = x.saturating_sub(1);
    let x_end = (x + 1).min(raw.width.saturating_sub(1));
    for sample_y in y_start..=y_end {
        for sample_x in x_start..=x_end {
            let color = raw.cfa.color_at(sample_y, sample_x).min(2);
            sum[color] += u32::from(data[sample_y * raw.width + sample_x]);
            count[color] += 1;
        }
    }
    let white = raw.whitelevels[0]
        .max(raw.whitelevels[1])
        .max(raw.whitelevels[2]);
    let black = raw.blacklevels[0]
        .min(raw.blacklevels[1])
        .min(raw.blacklevels[2]);
    [
        scale_raw_value((sum[0] / count[0].max(1)) as u16, black, white),
        scale_raw_value((sum[1] / count[1].max(1)) as u16, black, white),
        scale_raw_value((sum[2] / count[2].max(1)) as u16, black, white),
    ]
}

fn scale_raw_value(value: u16, black: u16, white: u16) -> u8 {
    let black = u32::from(black);
    let white = u32::from(white.max(black as u16 + 1));
    let value = u32::from(value).saturating_sub(black).min(white - black);
    ((value * 255) / (white - black)) as u8
}

fn scale_float_value(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn rawloader_orientation_to_image(orientation: rawloader::Orientation) -> Orientation {
    match orientation {
        rawloader::Orientation::HorizontalFlip => Orientation::FlipHorizontal,
        rawloader::Orientation::Rotate180 => Orientation::Rotate180,
        rawloader::Orientation::VerticalFlip => Orientation::FlipVertical,
        rawloader::Orientation::Transpose => Orientation::Rotate90FlipH,
        rawloader::Orientation::Rotate90 => Orientation::Rotate90,
        rawloader::Orientation::Transverse => Orientation::Rotate270FlipH,
        rawloader::Orientation::Rotate270 => Orientation::Rotate270,
        rawloader::Orientation::Normal | rawloader::Orientation::Unknown => {
            Orientation::NoTransforms
        }
    }
}

fn tiff_entry_first_u32(
    file: &mut File,
    endian: TiffEndian,
    field_type: u16,
    component_count: u32,
    value_or_offset: u32,
) -> Option<u32> {
    tiff_entry_u32_values(
        file,
        endian,
        field_type,
        component_count,
        value_or_offset,
        1,
    )
    .into_iter()
    .next()
}

fn tiff_entry_u32_values(
    file: &mut File,
    endian: TiffEndian,
    field_type: u16,
    component_count: u32,
    value_or_offset: u32,
    max_values: usize,
) -> Vec<u32> {
    let value_size = match field_type {
        3 => 2usize,
        4 => 4usize,
        _ => return Vec::new(),
    };
    let count = usize::try_from(component_count).unwrap_or_default();
    let inline_bytes = count.saturating_mul(value_size);
    if inline_bytes <= 4 {
        return match field_type {
            3 => vec![match endian {
                TiffEndian::Little => value_or_offset & 0xffff,
                TiffEndian::Big => value_or_offset >> 16,
            }],
            4 => vec![value_or_offset],
            _ => Vec::new(),
        };
    }
    let bytes_to_read = count.min(max_values).saturating_mul(value_size);
    let Some(data) = read_file_range(file, u64::from(value_or_offset), bytes_to_read) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for index in 0..count.min(max_values) {
        let offset = index * value_size;
        let value = match field_type {
            3 => data
                .get(offset..offset + 2)
                .and_then(|bytes| endian.read_u16(bytes, 0))
                .map(u32::from),
            4 => data
                .get(offset..offset + 4)
                .and_then(|bytes| endian.read_u32(bytes, 0)),
            _ => None,
        };
        if let Some(value) = value {
            values.push(value);
        }
    }
    values
}

fn read_file_range(file: &mut File, offset: u64, len: usize) -> Option<Vec<u8>> {
    if len > 64 * 1024 * 1024 {
        return None;
    }
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut data = vec![0u8; len];
    file.read_exact(&mut data).ok()?;
    Some(data)
}

fn next_ifd_offset(tiff: &[u8], endian: TiffEndian, ifd_offset: usize) -> Option<u32> {
    let count = usize::from(endian.read_u16(tiff, ifd_offset)?);
    let next_offset_position = ifd_offset
        .checked_add(2)?
        .checked_add(count.checked_mul(12)?)?;
    endian.read_u32(tiff, next_offset_position)
}

fn for_each_ifd_entry(
    tiff: &[u8],
    endian: TiffEndian,
    ifd_offset: usize,
    mut visit: impl FnMut(u16, u32),
) -> Option<()> {
    let count = usize::from(endian.read_u16(tiff, ifd_offset)?);
    let entries_start = ifd_offset.checked_add(2)?;
    for entry_index in 0..count {
        let entry = entries_start.checked_add(entry_index.checked_mul(12)?)?;
        let tag = endian.read_u16(tiff, entry)?;
        let field_type = endian.read_u16(tiff, entry + 2)?;
        let component_count = endian.read_u32(tiff, entry + 4)?;
        if component_count != 1 {
            continue;
        }
        let value = match field_type {
            3 => u32::from(endian.read_u16(tiff, entry + 8)?),
            4 => endian.read_u32(tiff, entry + 8)?,
            _ => continue,
        };
        visit(tag, value);
    }
    Some(())
}

#[derive(Debug, Clone, Copy)]
enum TiffEndian {
    Little,
    Big,
}

impl TiffEndian {
    fn from_header(tiff: &[u8]) -> Option<Self> {
        match tiff.get(0..2)? {
            b"II" => Some(Self::Little),
            b"MM" => Some(Self::Big),
            _ => None,
        }
    }

    fn read_u16(self, bytes: &[u8], offset: usize) -> Option<u16> {
        let value = bytes.get(offset..offset + 2)?;
        Some(match self {
            Self::Little => u16::from_le_bytes([value[0], value[1]]),
            Self::Big => u16::from_be_bytes([value[0], value[1]]),
        })
    }

    fn read_u32(self, bytes: &[u8], offset: usize) -> Option<u32> {
        let value = bytes.get(offset..offset + 4)?;
        Some(match self {
            Self::Little => u32::from_le_bytes([value[0], value[1], value[2], value[3]]),
            Self::Big => u32::from_be_bytes([value[0], value[1], value[2], value[3]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera_connector_core::{ObjectFormat, StoredObjectLocation};
    use image::{GenericImageView, ImageBuffer, Rgb};

    #[test]
    fn write_thumbnail_applies_exif_orientation() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-orientation");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("portrait-with-orientation.jpg");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, jpeg_with_exif_orientation(6)).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        assert_eq!(thumbnail.dimensions(), (64, 43));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_thumbnail_prefers_embedded_jpeg_preview() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-embedded-preview");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("image-with-preview.jpg");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, jpeg_with_embedded_preview()).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        let center = thumbnail
            .to_rgb8()
            .get_pixel(thumbnail.width() / 2, thumbnail.height() / 2)
            .0;
        assert!(
            center[2] > center[0],
            "thumbnail should be generated from the blue embedded preview, got rgb {center:?}"
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_full_thumbnail_ignores_embedded_jpeg_preview() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-full-quality");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("image-with-preview.jpg");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, jpeg_with_embedded_preview()).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Full)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        let center = thumbnail
            .to_rgb8()
            .get_pixel(thumbnail.width() / 2, thumbnail.height() / 2)
            .0;
        assert!(
            center[0] > center[2],
            "full thumbnail should be generated from the red source image, got rgb {center:?}"
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn thumbnail_request_allows_1280_edge() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-1280-edge");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("source.jpg");
        fs::write(&source_path, encode_solid_jpeg(1600, 900, [220, 12, 8]))
            .expect("source should write");

        let response = get_asset_thumbnail_blocking(
            temp_dir.clone(),
            ThumbnailRequest {
                source_path: source_path.to_string_lossy().into_owned(),
                max_edge: Some(1280),
                quality: Some(ThumbnailQuality::Full),
            },
        )
        .expect("thumbnail should write");

        let thumbnail = image::open(response.path).expect("thumbnail should decode");
        assert_eq!(thumbnail.dimensions(), (1280, 720));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn original_preview_reuses_browser_decodable_source() {
        let temp_dir = unique_temp_dir("desktop-original-preview-source");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("source.jpg");
        fs::write(&source_path, encode_solid_jpeg(80, 60, [220, 12, 8]))
            .expect("source should write");

        let response = get_asset_original_preview_blocking(
            temp_dir.clone(),
            OriginalPreviewRequest {
                source_path: source_path.to_string_lossy().into_owned(),
            },
        )
        .expect("browser original should return source path");

        assert_eq!(PathBuf::from(response.path), source_path);
        assert!(response.direct_source);
        assert!(response.cached);
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_original_preview_image_does_not_apply_thumbnail_clamp() {
        let temp_dir = unique_temp_dir("desktop-original-preview-large");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let output_path = temp_dir.join("original.jpg");
        let mut image =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(1600, 900, Rgb([10, 20, 30])));

        write_original_preview_image(&mut image, &output_path)
            .expect("original preview should write");

        let output = image::open(&output_path).expect("original preview should decode");
        assert_eq!(output.dimensions(), (1600, 900));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_thumbnail_reads_embedded_preview_from_raw_tiff_container() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-raw-preview");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("raw-only.nef");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(&source_path, raw_tiff_with_embedded_preview()).expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        let center = thumbnail
            .to_rgb8()
            .get_pixel(thumbnail.width() / 2, thumbnail.height() / 2)
            .0;
        assert!(
            center[1] > center[0] && center[1] > center[2],
            "raw thumbnail should be generated from the green embedded preview, got rgb {center:?}"
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn write_thumbnail_applies_raw_tiff_orientation_to_embedded_preview() {
        let temp_dir = unique_temp_dir("desktop-thumbnail-raw-preview-orientation");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let source_path = temp_dir.join("raw-rotated.nef");
        let output_path = temp_dir.join("thumb.jpg");
        fs::write(
            &source_path,
            raw_tiff_with_embedded_preview_and_orientation(6),
        )
        .expect("source should write");

        write_thumbnail_with_quality(&source_path, &output_path, 64, ThumbnailQuality::Fast)
            .expect("thumbnail should write");

        let thumbnail = image::open(&output_path).expect("thumbnail should decode");
        assert!(
            thumbnail.height() > thumbnail.width(),
            "raw embedded preview orientation should rotate to portrait, got {:?}",
            thumbnail.dimensions()
        );
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn raw_sensor_thumbnail_uses_raw_pixels() {
        let mut image = rawloader::RawImage {
            make: "Test".to_string(),
            model: "Sensor".to_string(),
            clean_make: "Test".to_string(),
            clean_model: "Sensor".to_string(),
            width: 2,
            height: 2,
            cpp: 1,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            whitelevels: [1023, 1023, 1023, 1023],
            blacklevels: [0, 0, 0, 0],
            xyz_to_cam: [[0.0; 3]; 4],
            cfa: rawloader::CFA::new("RGGB"),
            crops: [0, 0, 0, 0],
            blackareas: Vec::new(),
            orientation: rawloader::Orientation::Normal,
            data: rawloader::RawImageData::Integer(vec![1023, 256, 512, 768]),
        };

        let thumbnail = raw_sensor_thumbnail_image(&mut image).expect("raw sensor should render");

        assert_eq!(thumbnail.dimensions(), (2, 2));
        let center = thumbnail.to_rgb8().get_pixel(0, 0).0;
        assert!(
            center[0] > center[1] && center[0] > center[2],
            "red pixel should stay red, got {center:?}"
        );
    }

    #[test]
    fn preview_sample_from_image_preserves_luma_and_rgb_channels() {
        let mut image = DynamicImage::ImageRgb8(ImageBuffer::from_fn(2, 1, |x, _y| {
            if x == 0 {
                Rgb([255, 0, 0])
            } else {
                Rgb([0, 255, 0])
            }
        }));

        let sample = preview_sample_from_image(&mut image, Some("unit".to_string()), 16);

        assert_eq!(sample.width, 2);
        assert_eq!(sample.height, 1);
        assert_eq!(sample.luma, vec![54, 182]);
        assert_eq!(sample.red.as_deref(), Some([255, 0].as_slice()));
        assert_eq!(sample.green.as_deref(), Some([0, 255].as_slice()));
        assert_eq!(sample.blue.as_deref(), Some([0, 0].as_slice()));
        assert_eq!(sample.preview_source.as_deref(), Some("unit"));
    }

    #[test]
    fn best_asset_for_cv_uses_core_format_role_and_prefers_jpeg() {
        let temp_dir = unique_temp_dir("desktop-cv-photo-media-kind");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let raw_path = temp_dir.join("sample.nef");
        let jpeg_path = temp_dir.join("sample.jpg");
        fs::write(&raw_path, b"raw").expect("raw placeholder should write");
        fs::write(&jpeg_path, b"jpeg").expect("jpeg placeholder should write");

        let assets = vec![
            stored_cv_asset("raw", "raw", ObjectFormat::Nef, &raw_path),
            stored_cv_asset("jpeg", "jpeg", ObjectFormat::Jpeg, &jpeg_path),
        ];

        let selected = best_asset_for_cv(&assets).expect("photo asset should be selected");
        assert_eq!(selected.asset_id, "jpeg");
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn desktop_face_detector_loads_bundled_model() {
        desktop_face_detector().expect("bundled PICO model should load");
    }

    #[test]
    fn sync_project_snapshot_from_path_returns_compact_counts() {
        let temp_dir = unique_temp_dir("desktop-project-sync-snapshot");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let root = temp_dir.join("photos");
        fs::create_dir_all(&root).expect("photo root should create");
        fs::write(root.join("IMG_5100.JPG"), [1_u8, 2, 3, 4]).expect("jpeg should write");

        let service = CameraConnectorService::new(Some(temp_dir.join("config.json")));
        let project = service
            .create_project("Desktop Snapshot Sync")
            .expect("project should create");
        let scan = service
            .create_desktop_project_scan(&project.project_id, &root)
            .expect("scan should queue");
        service
            .run_desktop_project_scan(&scan.scan_id)
            .expect("scan should complete");

        let snapshot_path = temp_dir.join("snapshot.json");
        fs::write(
            &snapshot_path,
            r#"{
              "schema_version": 1,
              "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
              "project": {"project_id": "phone-project", "name": "Phone Project", "exported_at_ms": 1781800000000},
              "assets": [{
                "asset_id": "remote-asset",
                "group_id": "remote-group",
                "original_filename": "IMG_5100.JPG",
                "final_filename": "IMG_5100.JPG",
                "normalized_stem": "IMG_5100",
                "original_path": "Android/DCIM/IMG_5100.JPG",
                "original_parent_path": "Android/DCIM",
                "format": "jpeg",
                "size_bytes": 4,
                "capture_at_ms": null,
                "received_at_ms": null,
                "source_identity": null
              }],
              "groups": [{
                "group_id": "remote-group",
                "display_key": "IMG_5100",
                "source_identity": null,
                "original_parent_path": "Android/DCIM",
                "member_asset_ids": ["remote-asset"],
                "primary_asset_id": "remote-asset",
                "preview_asset_id": "remote-asset",
                "has_raw": false,
                "has_jpeg": true,
                "has_video": false
              }],
              "model_evaluations": [],
              "selection_recommendations": [],
              "user_marks": [{"group_id": "remote-group", "favorite": true, "marked": null}]
            }"#,
        )
        .expect("snapshot should write");

        let response = sync_project_snapshot_from_path_blocking(
            &service,
            SyncProjectSnapshotRequest {
                project_id: project.project_id,
                snapshot_path,
            },
        )
        .expect("snapshot sync should return counts");

        assert_eq!(response.matched_assets, 1);
        assert_eq!(response.matched_groups, 1);
        assert_eq!(response.applied_user_marks, 1);
        assert_eq!(response.unresolved_records, 0);
        assert_eq!(response.ambiguous_records, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn sync_project_snapshot_from_url_fetches_snapshot_and_returns_compact_counts() {
        let temp_dir = unique_temp_dir("desktop-project-sync-snapshot-url");
        fs::create_dir_all(&temp_dir).expect("temp dir should create");
        let root = temp_dir.join("photos");
        fs::create_dir_all(&root).expect("photo root should create");
        fs::write(root.join("IMG_5200.JPG"), [1_u8, 2, 3, 4]).expect("jpeg should write");

        let service = CameraConnectorService::new(Some(temp_dir.join("config.json")));
        let project = service
            .create_project("Desktop Snapshot URL Sync")
            .expect("project should create");
        let scan = service
            .create_desktop_project_scan(&project.project_id, &root)
            .expect("scan should queue");
        service
            .run_desktop_project_scan(&scan.scan_id)
            .expect("scan should complete");

        let snapshot = r#"{
          "schema_version": 1,
          "source_device": {"device_id": "phone", "device_label": "Phone", "platform": "android"},
          "project": {"project_id": "phone-project", "name": "Phone Project", "exported_at_ms": 1781800000000},
          "assets": [{
            "asset_id": "remote-asset",
            "group_id": "remote-group",
            "original_filename": "IMG_5200.JPG",
            "final_filename": "IMG_5200.JPG",
            "normalized_stem": "IMG_5200",
            "original_path": "Android/DCIM/IMG_5200.JPG",
            "original_parent_path": "Android/DCIM",
            "format": "jpeg",
            "size_bytes": 4,
            "capture_at_ms": null,
            "received_at_ms": null,
            "source_identity": null
          }],
          "groups": [{
            "group_id": "remote-group",
            "display_key": "IMG_5200",
            "source_identity": null,
            "original_parent_path": "Android/DCIM",
            "member_asset_ids": ["remote-asset"],
            "primary_asset_id": "remote-asset",
            "preview_asset_id": "remote-asset",
            "has_raw": false,
            "has_jpeg": true,
            "has_video": false
          }],
          "model_evaluations": [],
          "selection_recommendations": [],
          "user_marks": [{"group_id": "remote-group", "favorite": true, "marked": null}]
        }"#;
        let url = serve_once(snapshot);

        let response = sync_project_snapshot_from_url_blocking(
            &service,
            SyncProjectSnapshotUrlRequest {
                project_id: project.project_id,
                snapshot_url: url,
            },
        )
        .expect("snapshot URL sync should return counts");

        assert_eq!(response.matched_assets, 1);
        assert_eq!(response.matched_groups, 1);
        assert_eq!(response.applied_user_marks, 1);
        assert_eq!(response.unresolved_records, 0);
        assert_eq!(response.ambiguous_records, 0);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn analyze_face_region_uses_project_clip_thresholds() {
        let image = ImageBuffer::from_fn(2, 1, |x, _y| {
            if x == 0 {
                Rgb([5, 5, 5])
            } else {
                Rgb([250, 250, 250])
            }
        });

        let standard =
            analyze_face_region(&image, 0, 0, 2, 1, TechnicalAssessmentPolicy::standard());
        let relaxed = analyze_face_region(
            &image,
            0,
            0,
            2,
            1,
            TechnicalAssessmentPolicy {
                shadow_clip_threshold: 0,
                highlight_clip_threshold: 255,
                ..TechnicalAssessmentPolicy::standard()
            },
        );

        assert_eq!(standard.shadow_ratio, 0.5);
        assert_eq!(standard.highlight_ratio, 0.5);
        assert_eq!(relaxed.shadow_ratio, 0.0);
        assert_eq!(relaxed.highlight_ratio, 0.0);
    }

    fn jpeg_with_exif_orientation(orientation: u16) -> Vec<u8> {
        let image = ImageBuffer::from_fn(2, 3, |x, y| {
            Rgb([
                (40 + x * 80) as u8,
                (50 + y * 50) as u8,
                (120 + x * 20 + y * 5) as u8,
            ])
        });
        let mut jpeg = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg should encode");
        insert_exif_orientation(&mut jpeg, orientation);
        jpeg
    }

    fn jpeg_with_embedded_preview() -> Vec<u8> {
        let image = ImageBuffer::from_fn(48, 32, |_x, _y| Rgb([220u8, 12, 8]));
        let mut jpeg = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg should encode");
        let preview = encode_solid_jpeg(16, 12, [8, 30, 220]);
        insert_exif_orientation_and_preview(&mut jpeg, 1, &preview);
        jpeg
    }

    fn encode_solid_jpeg(width: u32, height: u32, color: [u8; 3]) -> Vec<u8> {
        let image = ImageBuffer::from_pixel(width, height, Rgb(color));
        let mut jpeg = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("jpeg should encode");
        jpeg
    }

    fn raw_tiff_with_embedded_preview() -> Vec<u8> {
        raw_tiff_with_embedded_preview_and_orientation(1)
    }

    fn raw_tiff_with_embedded_preview_and_orientation(orientation: u16) -> Vec<u8> {
        let preview = encode_solid_jpeg(18, 12, [10, 220, 30]);
        let ifd_offset = 8u32;
        let entry_count = 3u16;
        let ifd_size = 2u32 + u32::from(entry_count) * 12 + 4;
        let preview_offset = ifd_offset + ifd_size;

        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&ifd_offset.to_le_bytes());
        tiff.extend_from_slice(&entry_count.to_le_bytes());
        tiff.extend_from_slice(&0x0112u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&orientation.to_le_bytes());
        tiff.extend_from_slice(&0u16.to_le_bytes());
        tiff.extend_from_slice(&0x0201u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&preview_offset.to_le_bytes());
        tiff.extend_from_slice(&0x0202u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&(preview.len() as u32).to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        tiff.extend_from_slice(&preview);
        tiff
    }

    fn insert_exif_orientation(jpeg: &mut Vec<u8>, orientation: u16) {
        assert!(jpeg.starts_with(&[0xff, 0xd8]));
        let mut payload = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(b"II");
        payload.extend_from_slice(&42u16.to_le_bytes());
        payload.extend_from_slice(&8u32.to_le_bytes());
        payload.extend_from_slice(&1u16.to_le_bytes());
        payload.extend_from_slice(&0x0112u16.to_le_bytes());
        payload.extend_from_slice(&3u16.to_le_bytes());
        payload.extend_from_slice(&1u32.to_le_bytes());
        payload.extend_from_slice(&orientation.to_le_bytes());
        payload.extend_from_slice(&0u16.to_le_bytes());
        payload.extend_from_slice(&0u32.to_le_bytes());

        let length = (payload.len() + 2) as u16;
        let mut segment = vec![0xff, 0xe1];
        segment.extend_from_slice(&length.to_be_bytes());
        segment.extend_from_slice(&payload);
        jpeg.splice(2..2, segment);
    }

    fn insert_exif_orientation_and_preview(jpeg: &mut Vec<u8>, orientation: u16, preview: &[u8]) {
        assert!(jpeg.starts_with(&[0xff, 0xd8]));
        let ifd0_offset = 8u32;
        let ifd0_entry_count = 1u16;
        let ifd0_size = 2u32 + u32::from(ifd0_entry_count) * 12 + 4;
        let ifd1_offset = ifd0_offset + ifd0_size;
        let ifd1_entry_count = 2u16;
        let ifd1_size = 2u32 + u32::from(ifd1_entry_count) * 12 + 4;
        let preview_offset = ifd1_offset + ifd1_size;

        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42u16.to_le_bytes());
        tiff.extend_from_slice(&ifd0_offset.to_le_bytes());
        tiff.extend_from_slice(&ifd0_entry_count.to_le_bytes());
        tiff.extend_from_slice(&0x0112u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&orientation.to_le_bytes());
        tiff.extend_from_slice(&0u16.to_le_bytes());
        tiff.extend_from_slice(&ifd1_offset.to_le_bytes());
        tiff.extend_from_slice(&ifd1_entry_count.to_le_bytes());
        tiff.extend_from_slice(&0x0201u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&preview_offset.to_le_bytes());
        tiff.extend_from_slice(&0x0202u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&(preview.len() as u32).to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes());
        tiff.extend_from_slice(preview);

        let mut payload = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(&tiff);
        let length = (payload.len() + 2) as u16;
        let mut segment = vec![0xff, 0xe1];
        segment.extend_from_slice(&length.to_be_bytes());
        segment.extend_from_slice(&payload);
        jpeg.splice(2..2, segment);
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}-{}", current_time_ms()))
    }

    fn serve_once(body: &'static str) -> String {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("test HTTP listener should bind");
        let url = format!("http://{}/project-snapshot", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("test HTTP request should arrive");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("test HTTP response should write");
        });
        url
    }

    fn stored_cv_asset(
        asset_id: &str,
        group_role: &str,
        format: ObjectFormat,
        path: &Path,
    ) -> StoredAsset {
        StoredAsset {
            asset_id: asset_id.to_string(),
            project_id: "project".to_string(),
            group_id: Some("group".to_string()),
            transfer_id: asset_id.to_string(),
            group_role: group_role.to_string(),
            media_kind: "photo".to_string(),
            format,
            original_filename: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(asset_id)
                .to_string(),
            final_filename: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(asset_id)
                .to_string(),
            normalized_stem: "sample".to_string(),
            original_path: path.display().to_string(),
            original_parent_path: path.parent().map(|parent| parent.display().to_string()),
            final_location: Some(StoredObjectLocation::local_path(path)),
            size_bytes: 1,
            capture_at_ms: None,
            received_at_ms: None,
            published_at_ms: None,
            source_identity: None,
            username: None,
            remote_addr: None,
            source_status: "available".to_string(),
            source_modified_at_ms: None,
            last_seen_scan_id: None,
            duplicate_index: None,
            duplicate_count: None,
        }
    }
}
