$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$headerPath = Join-Path $root "core-ffi\include\camera_connector_mobile.h"
$rustPath = Join-Path $root "core-ffi\src\lib.rs"

if (-not (Test-Path -LiteralPath $headerPath -PathType Leaf)) {
    throw "Missing mobile FFI header: core-ffi\include\camera_connector_mobile.h"
}

$header = Get-Content -LiteralPath $headerPath -Raw
$rust = Get-Content -LiteralPath $rustPath -Raw

$requiredFunctions = @(
    "camera_connector_mobile_core_create",
    "camera_connector_mobile_core_destroy",
    "camera_connector_mobile_core_free_string",
    "camera_connector_mobile_core_config_path",
    "camera_connector_mobile_core_default_state_dir",
    "camera_connector_mobile_core_create_project_json",
    "camera_connector_mobile_core_list_projects_json",
    "camera_connector_mobile_core_set_active_project_json",
    "camera_connector_mobile_core_rename_project_json",
    "camera_connector_mobile_core_archive_project_json",
    "camera_connector_mobile_core_restore_project_json",
    "camera_connector_mobile_core_active_project_json",
    "camera_connector_mobile_core_project_dashboard_json",
    "camera_connector_mobile_core_project_asset_group_page_json",
    "camera_connector_mobile_core_project_group_assets_json",
    "camera_connector_mobile_core_move_project_group_json",
    "camera_connector_mobile_core_set_asset_group_user_marks_json",
    "camera_connector_mobile_core_claim_next_publish_item_json",
    "camera_connector_mobile_core_mark_publish_completed_json",
    "camera_connector_mobile_core_complete_publish_json",
    "camera_connector_mobile_core_mark_publish_failed_json",
    "camera_connector_mobile_core_release_failed_publish_retries_json",
    "camera_connector_mobile_core_drain_analysis_jobs_json",
    "camera_connector_mobile_core_drain_analysis_jobs_with_provider_configured_json",
    "camera_connector_mobile_core_assess_asset_group_preview_json",
    "camera_connector_mobile_core_assess_asset_group_preview_with_provider_configured_json",
    "camera_connector_mobile_core_split_burst_member_json",
    "camera_connector_mobile_core_merge_burst_member_json",
    "camera_connector_mobile_core_model_provider_settings_json",
    "camera_connector_mobile_core_save_model_provider_settings_json",
    "camera_connector_mobile_core_project_evaluation_settings_json",
    "camera_connector_mobile_core_save_project_evaluation_settings_json",
    "camera_connector_mobile_core_prompt_packs_for_project_json",
    "camera_connector_mobile_core_global_prompt_packs_json",
    "camera_connector_mobile_core_create_global_prompt_pack_json",
    "camera_connector_mobile_core_save_global_prompt_pack_json",
    "camera_connector_mobile_core_delete_global_prompt_pack_json",
    "camera_connector_mobile_core_delete_global_prompt_package_json",
    "camera_connector_mobile_core_fork_prompt_pack_json",
    "camera_connector_mobile_core_save_prompt_pack_json",
    "camera_connector_mobile_core_generate_project_recommendation_json",
    "camera_connector_mobile_core_latest_project_recommendation_run_status_json",
    "camera_connector_mobile_core_should_schedule_subject_assessment_json",
    "camera_connector_mobile_core_save_subject_assessment_json",
    "camera_connector_mobile_core_subject_assessments_for_asset_groups_json",
    "camera_connector_mobile_core_save_receiver_settings_json",
    "camera_connector_mobile_core_save_device_account_json",
    "camera_connector_mobile_core_remove_device_account_json",
    "camera_connector_mobile_core_start_receiver_json",
    "camera_connector_mobile_core_stop_receiver_json"
)

$requiredJniFunctions = @(
    "Java_com_cameraconnector_app_core_NativeMobileCore_create",
    "Java_com_cameraconnector_app_core_NativeMobileCore_destroy",
    "Java_com_cameraconnector_app_core_NativeMobileCore_createProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_listProjectsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_setActiveProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_renameProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_archiveProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_restoreProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_activeProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_projectDashboardJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_projectAssetGroupPageJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_projectGroupAssetsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_moveProjectGroupJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_setAssetGroupUserMarksJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_claimNextPublishItemJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_markPublishCompletedJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_completePublishJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_markPublishFailedJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_releaseFailedPublishRetriesJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_drainAnalysisJobsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_drainAnalysisJobsWithProviderConfiguredJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_assessAssetGroupPreviewJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_assessAssetGroupPreviewWithProviderConfiguredJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_splitBurstMemberJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_mergeBurstMemberJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_modelProviderSettingsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_saveModelProviderSettingsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_projectEvaluationSettingsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_saveProjectEvaluationSettingsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_PromptPacksForProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_globalPromptPacksJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_forkGlobalPromptPackJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_createGlobalPromptPackJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_saveGlobalPromptPackJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_deleteGlobalPromptPackJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_deleteGlobalPromptPackageJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_forkPromptPackJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_savePromptPackJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_generateProjectRecommendationJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_latestProjectRecommendationRunStatusJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_shouldScheduleSubjectAssessmentJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_saveSubjectAssessmentJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_subjectAssessmentsForAssetGroupsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_saveReceiverSettingsJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_saveDeviceAccountJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_removeDeviceAccountJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_startReceiverJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_stopReceiverJson"
)

foreach ($functionName in $requiredFunctions) {
    if ($header -notmatch [regex]::Escape($functionName)) {
        throw "Header does not declare $functionName"
    }
    if ($rust -notmatch [regex]::Escape($functionName)) {
        throw "Rust FFI implementation does not export $functionName"
    }
}

foreach ($functionName in $requiredJniFunctions) {
    if ($rust -notmatch [regex]::Escape($functionName)) {
        throw "Rust JNI implementation does not export $functionName"
    }
}

if ($header -notmatch "#ifndef CAMERA_CONNECTOR_MOBILE_H") {
    throw "Header guard is missing or incorrect"
}
if ($header -notmatch "extern `"C`"") {
    throw "Header does not expose C++ compatible extern C declarations"
}
if ($header -notmatch "CameraConnectorMobileCore") {
    throw "Header does not declare the opaque core handle"
}
if ($header -notmatch "JSON envelope") {
    throw "Header does not document the JSON envelope contract"
}

Write-Host "Mobile FFI contract checks passed."
