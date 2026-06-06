use std::path::PathBuf;

use camera_connector_core::service::AnalysisDrainSummary;
use camera_connector_core::{
    AssetGroupPage, AssetGroupQuery, CameraConnectorDashboard, CameraConnectorService,
    DesktopScanRun, Project, SelectionRecommendation, StoredAsset,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

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
