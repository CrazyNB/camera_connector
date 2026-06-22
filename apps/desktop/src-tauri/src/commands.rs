use std::fs;
use std::path::PathBuf;

use crate::lan_discovery::{
    discover_lan_project_snapshot_sources, LanProjectSnapshotDiscoveryRequest,
    LanProjectSnapshotSource,
};
use camera_connector_core::{
    AssetGroupPage, AssetGroupQuery, CameraConnectorService, DesktopScanRun, Project, StoredAsset,
    SubjectAssessment,
};
use tauri::{AppHandle, Emitter, State};

#[path = "desktop_cv.rs"]
mod desktop_cv;

#[path = "thumbnails.rs"]
pub mod thumbnails;

#[path = "command_types.rs"]
mod command_types;
pub use command_types::*;

#[path = "command_models.rs"]
mod command_models;

#[path = "intelligence_commands.rs"]
mod intelligence_commands;
pub use intelligence_commands::*;

#[cfg(test)]
use thumbnails::{
    get_asset_original_preview_blocking, get_asset_thumbnail_blocking, raw_sensor_thumbnail_image,
    write_original_preview_image, write_thumbnail_with_quality, OriginalPreviewRequest,
    ThumbnailQuality, ThumbnailRequest,
};

pub(super) fn desktop_error(error: camera_connector_core::ImporterError) -> DesktopError {
    DesktopError {
        code: error.code().to_string(),
        message: error.to_string(),
    }
}

fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
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
pub async fn run_desktop_cv_assessment(
    app: AppHandle,
    state: State<'_, DesktopState>,
    request: DesktopCvAssessmentRequest,
) -> Result<DesktopCvAssessmentResponse, DesktopError> {
    let service = state.service.clone();
    tauri::async_runtime::spawn_blocking(move || {
        desktop_cv::run_desktop_cv_assessment_blocking(&service, request, Some(app))
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

#[cfg(test)]
mod tests;
