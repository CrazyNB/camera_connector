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
    "camera_connector_mobile_core_dashboard_json",
    "camera_connector_mobile_core_save_receiver_settings_json",
    "camera_connector_mobile_core_save_device_account_json"
)

foreach ($functionName in $requiredFunctions) {
    if ($header -notmatch [regex]::Escape($functionName)) {
        throw "Header does not declare $functionName"
    }
    if ($rust -notmatch [regex]::Escape($functionName)) {
        throw "Rust FFI implementation does not export $functionName"
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
