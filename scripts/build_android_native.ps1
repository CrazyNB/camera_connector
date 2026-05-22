$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ([string]::IsNullOrWhiteSpace($env:ANDROID_SDK_ROOT)) {
    Join-Path $env:LOCALAPPDATA "Android\Sdk"
} else {
    $env:ANDROID_SDK_ROOT
}

$ndkRoot = if ([string]::IsNullOrWhiteSpace($env:ANDROID_NDK_ROOT)) {
    $installedNdks = Get-ChildItem -LiteralPath (Join-Path $sdkRoot "ndk") -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending
    if ($installedNdks.Count -eq 0) {
        throw "Android NDK not found. Install it with sdkmanager first."
    }
    $installedNdks[0].FullName
} else {
    $env:ANDROID_NDK_ROOT
}

$apiLevel = if ([string]::IsNullOrWhiteSpace($env:ANDROID_NATIVE_API_LEVEL)) { "26" } else { $env:ANDROID_NATIVE_API_LEVEL }
$toolchainBin = Join-Path $ndkRoot "toolchains\llvm\prebuilt\windows-x86_64\bin"
$linker = Join-Path $toolchainBin "aarch64-linux-android$apiLevel-clang.cmd"
if (-not (Test-Path -LiteralPath $linker)) {
    throw "Android linker not found: $linker"
}

$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $linker
$env:CC_aarch64_linux_android = $linker
$env:AR_aarch64_linux_android = Join-Path $toolchainBin "llvm-ar.exe"

Push-Location $root
try {
    cargo build -p camera-connector-ffi --target aarch64-linux-android --release
    if ($LASTEXITCODE -ne 0) {
        throw "Android native build failed"
    }
} finally {
    Pop-Location
}

$source = Join-Path $root "target\aarch64-linux-android\release\libcamera_connector_ffi.so"
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
    throw "Expected native library was not built: $source"
}

$destinationDir = Join-Path $root "apps\android\app\src\main\jniLibs\arm64-v8a"
New-Item -ItemType Directory -Force -Path $destinationDir | Out-Null
Copy-Item -LiteralPath $source -Destination (Join-Path $destinationDir "libcamera_connector_ffi.so") -Force

Write-Host "Android native library copied to apps\android\app\src\main\jniLibs\arm64-v8a\libcamera_connector_ffi.so"
