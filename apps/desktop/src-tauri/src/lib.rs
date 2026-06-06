mod commands;

use camera_connector_core::CameraConnectorService;
use commands::DesktopState;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(DesktopState {
            service: CameraConnectorService::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::select_project,
            commands::start_project_scan,
            commands::get_scan_status,
            commands::get_project_asset_page,
            commands::get_project_group_detail,
            commands::save_group_user_marks,
            commands::drain_analysis_jobs,
            commands::recommend_burst_group,
            commands::generate_project_recommendation,
            commands::get_project_dashboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running camera connector desktop");
}
