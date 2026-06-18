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
            commands::get_asset_thumbnail,
            commands::get_asset_original_preview,
            commands::get_asset_thumbnails,
            commands::get_project_group_detail,
            commands::run_desktop_cv_assessment,
            commands::get_subject_assessments_for_asset_groups,
            commands::delete_project_asset_group,
            commands::save_group_user_marks,
            commands::get_model_provider_settings_list,
            commands::save_model_provider_settings,
            commands::delete_model_provider_settings,
            commands::get_project_evaluation_settings,
            commands::save_project_evaluation_settings,
            commands::get_global_prompt_packs,
            commands::get_project_prompt_packs,
            commands::create_global_prompt_pack,
            commands::fork_global_prompt_pack,
            commands::save_global_prompt_pack,
            commands::delete_global_prompt_pack,
            commands::enqueue_model_evaluation_for_asset_groups,
            commands::drain_analysis_jobs,
            commands::recommend_burst_group,
            commands::split_burst_member,
            commands::generate_project_recommendation,
            commands::get_project_dashboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running camera connector desktop");
}
