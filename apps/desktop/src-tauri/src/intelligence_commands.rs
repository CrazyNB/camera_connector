use camera_connector_core::service::AnalysisDrainSummary;
use camera_connector_core::{
    AssetGroupQuery, BurstGroup, CameraConnectorDashboard, ProjectEvaluationSettings, SceneProfile,
    SelectionRecommendation,
};
use tauri::State;

use super::command_models::{
    desktop_model_provider_settings, desktop_project_evaluation_settings, desktop_prompt_pack,
    desktop_prompt_packs, model_provider_settings_from_request,
    project_evaluation_settings_from_desktop,
};
use super::{
    current_time_ms, desktop_error, CreatePromptPackRequest, DesktopError,
    DesktopModelProviderSettings, DesktopProjectEvaluationSettings, DesktopPromptPack,
    DesktopState, EnqueueModelEvaluationRequest, EnqueueModelEvaluationResponse,
    ForkPromptPackRequest, SaveModelProviderSettingsRequest, SavePromptPackRequest,
};

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
