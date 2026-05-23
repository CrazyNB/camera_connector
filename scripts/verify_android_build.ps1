$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$androidRoot = Join-Path $root "apps\android"
$buildNativeScript = Join-Path $root "scripts\build_android_native.ps1"
$inspectApkScript = Join-Path $root "scripts\inspect_android_apk.ps1"

$defaultJavaHome = "C:\Program Files\ojdkbuild\java-17-openjdk-17.0.3.0.6-1"
$defaultSdkRoot = Join-Path $env:LOCALAPPDATA "Android\Sdk"
$defaultGradle = Join-Path $env:LOCALAPPDATA "CameraConnectorToolchains\gradle-9.5.1\bin\gradle.bat"

if ([string]::IsNullOrWhiteSpace($env:JAVA_HOME) -or -not (Test-Path -LiteralPath (Join-Path $env:JAVA_HOME "bin\java.exe"))) {
    if (Test-Path -LiteralPath (Join-Path $defaultJavaHome "bin\java.exe")) {
        $env:JAVA_HOME = $defaultJavaHome
    } else {
        throw "JDK 17 not found. Install JDK 17 or set JAVA_HOME."
    }
}

if ([string]::IsNullOrWhiteSpace($env:ANDROID_SDK_ROOT)) {
    if (Test-Path -LiteralPath $defaultSdkRoot) {
        $env:ANDROID_SDK_ROOT = $defaultSdkRoot
    } else {
        throw "Android SDK not found. Install it or set ANDROID_SDK_ROOT."
    }
}
$env:ANDROID_HOME = $env:ANDROID_SDK_ROOT

$gradle = $defaultGradle
if (-not (Test-Path -LiteralPath $gradle)) {
    $gradleCommand = Get-Command gradle -ErrorAction SilentlyContinue
    if ($null -eq $gradleCommand) {
        throw "Gradle not found. Install Gradle 9.5.1 or put gradle on PATH."
    }
    $gradle = $gradleCommand.Source
}

$env:Path = "$env:JAVA_HOME\bin;$env:ANDROID_SDK_ROOT\cmdline-tools\latest\bin;$env:ANDROID_SDK_ROOT\platform-tools;$env:Path"

& $buildNativeScript
if ($LASTEXITCODE -ne 0) {
    throw "Android native library build failed"
}

Push-Location $androidRoot
try {
    & $gradle ":app:assembleDebug" "--no-daemon"
    if ($LASTEXITCODE -ne 0) {
        throw "Android debug build failed"
    }
} finally {
    Pop-Location
}

$apk = Join-Path $androidRoot "app\build\outputs\apk\debug\app-debug.apk"
if (-not (Test-Path -LiteralPath $apk -PathType Leaf)) {
    throw "Android debug APK not found: $apk"
}

& powershell -NoProfile -ExecutionPolicy Bypass -File $inspectApkScript -ApkPath $apk
if ($LASTEXITCODE -ne 0) {
    throw "Android APK inspection failed"
}

Write-Host "Android debug build passed with arm64 native core packaged."
