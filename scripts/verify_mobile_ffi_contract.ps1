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
    "camera_connector_mobile_core_archive_project_json",
    "camera_connector_mobile_core_restore_project_json",
    "camera_connector_mobile_core_active_project_json",
    "camera_connector_mobile_core_ensure_active_project_json",
    "camera_connector_mobile_core_project_dashboard_json",
    "camera_connector_mobile_core_project_group_assets_json",
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
    "Java_com_cameraconnector_app_core_NativeMobileCore_archiveProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_restoreProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_activeProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_ensureActiveProjectJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_projectDashboardJson",
    "Java_com_cameraconnector_app_core_NativeMobileCore_projectGroupAssetsJson",
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
