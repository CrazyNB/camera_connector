$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$androidRoot = Join-Path $root "apps\android"

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

Push-Location $androidRoot
try {
    & $gradle ":app:assembleDebug" "--no-daemon"
    if ($LASTEXITCODE -ne 0) {
        throw "Android debug build failed"
    }
} finally {
    Pop-Location
}

Write-Host "Android debug build passed."
