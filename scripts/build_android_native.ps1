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
$targets = @(
    @{
        RustTarget = "aarch64-linux-android"
        LinkerPrefix = "aarch64-linux-android"
        CargoEnv = "AARCH64_LINUX_ANDROID"
        Abi = "arm64-v8a"
    },
    @{
        RustTarget = "x86_64-linux-android"
        LinkerPrefix = "x86_64-linux-android"
        CargoEnv = "X86_64_LINUX_ANDROID"
        Abi = "x86_64"
    }
)

foreach ($target in $targets) {
    $linker = Join-Path $toolchainBin "$($target.LinkerPrefix)$apiLevel-clang.cmd"
    if (-not (Test-Path -LiteralPath $linker)) {
        throw "Android linker not found: $linker"
    }

    Set-Item -Path "env:CARGO_TARGET_$($target.CargoEnv)_LINKER" -Value $linker
    Set-Item -Path "env:CC_$($target.RustTarget.Replace('-', '_'))" -Value $linker
    Set-Item -Path "env:AR_$($target.RustTarget.Replace('-', '_'))" -Value (Join-Path $toolchainBin "llvm-ar.exe")
}

Push-Location $root
try {
    foreach ($target in $targets) {
        cargo build -p camera-connector-ffi --target $target.RustTarget --release
        if ($LASTEXITCODE -ne 0) {
            throw "Android native build failed for $($target.RustTarget)"
        }
    }
} finally {
    Pop-Location
}

foreach ($target in $targets) {
    $source = Join-Path $root "target\$($target.RustTarget)\release\libcamera_connector_ffi.so"
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Expected native library was not built: $source"
    }

    $destinationDir = Join-Path $root "apps\android\app\src\main\jniLibs\$($target.Abi)"
    New-Item -ItemType Directory -Force -Path $destinationDir | Out-Null
    Copy-Item -LiteralPath $source -Destination (Join-Path $destinationDir "libcamera_connector_ffi.so") -Force

    Write-Host "Android native library copied to apps\android\app\src\main\jniLibs\$($target.Abi)\libcamera_connector_ffi.so"
}
