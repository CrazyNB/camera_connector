$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$headerPath = Join-Path $root "core-ffi\include\camera_connector_mobile.h"
$rustSourcePath = Join-Path $root "core-ffi\src"
$nativeMobileCorePath = Join-Path $root "apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt"

if (-not (Test-Path -LiteralPath $headerPath -PathType Leaf)) {
    throw "Missing mobile FFI header: core-ffi\include\camera_connector_mobile.h"
}
if (-not (Test-Path -LiteralPath $nativeMobileCorePath -PathType Leaf)) {
    throw "Missing Android native bridge: apps\android\app\src\main\java\com\cameraconnector\app\core\NativeMobileCore.kt"
}

$header = Get-Content -LiteralPath $headerPath -Raw
$rust = Get-ChildItem -LiteralPath $rustSourcePath -Filter "*.rs" -Recurse |
    Sort-Object FullName |
    ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw } |
    Out-String
$nativeMobileCore = Get-Content -LiteralPath $nativeMobileCorePath -Raw

$requiredFunctions = [regex]::Matches($header, "camera_connector_mobile_core_[A-Za-z0-9_]+(?=\s*\()") |
    ForEach-Object { $_.Value } |
    Sort-Object -Unique
$rustFfiFunctions = [regex]::Matches($rust, "pub\s+unsafe\s+extern\s+`"C`"\s+fn\s+(camera_connector_mobile_core_[A-Za-z0-9_]+)\s*\(") |
    ForEach-Object { $_.Groups[1].Value } |
    Sort-Object -Unique

foreach ($functionName in $requiredFunctions) {
    if ($rust -notmatch [regex]::Escape($functionName)) {
        throw "Rust FFI implementation does not export $functionName"
    }
}
foreach ($functionName in $rustFfiFunctions) {
    if ($requiredFunctions -notcontains $functionName) {
        throw "Rust FFI implementation exports $functionName but the header does not declare it"
    }
}

$requiredJniFunctions = [regex]::Matches($nativeMobileCore, "private\s+external\s+fun\s+([A-Za-z0-9_]+)\s*\(") |
    ForEach-Object { "Java_com_cameraconnector_app_core_NativeMobileCore_$($_.Groups[1].Value)" } |
    Sort-Object -Unique
$rustJniFunctions = [regex]::Matches($rust, "pub\s+extern\s+`"system`"\s+fn\s+(Java_com_cameraconnector_app_core_NativeMobileCore_[A-Za-z0-9_]+)\s*\(") |
    ForEach-Object { $_.Groups[1].Value } |
    Sort-Object -Unique

foreach ($functionName in $requiredJniFunctions) {
    if ($rust -notmatch [regex]::Escape($functionName)) {
        throw "Rust JNI implementation does not export $functionName"
    }
}
foreach ($functionName in $rustJniFunctions) {
    if ($requiredJniFunctions -notcontains $functionName) {
        throw "Rust JNI implementation exports $functionName but NativeMobileCore.kt does not declare it"
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
