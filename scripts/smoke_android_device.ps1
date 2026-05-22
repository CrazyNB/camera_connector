param(
    [string]$Serial,
    [switch]$SkipBuild,
    [switch]$NoDiagnostics
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$packageName = "com.cameraconnector.app"

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

function Collect-Diagnostics {
    param([string]$Name)

    if ($NoDiagnostics) {
        return
    }

    $diagnosticsDir = Join-Path $root "target\android-diagnostics\$Name"
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") @serialArgs -OutputDir $diagnosticsDir
    if ($LASTEXITCODE -ne 0) {
        throw "Android diagnostics collection failed for $Name."
    }
}

$serialArgs = @()
if ($Serial) {
    $serialArgs = @("-Serial", $Serial)
}

$adbArgs = @()
if ($Serial) {
    $adbArgs = @("-s", $Serial)
} else {
    $devices = @(& $adb devices | Select-Object -Skip 1 | Where-Object { $_ -match "\tdevice$" })
    if ($devices.Count -eq 0) {
        throw "No adb device is connected. Enable USB debugging or connect with adb tcpip."
    }
    if ($devices.Count -gt 1) {
        $serials = $devices | ForEach-Object { ($_ -split "\s+")[0] }
        throw "Multiple adb devices are connected. Re-run with -Serial <serial>. Connected: $($serials -join ', ')"
    }
    $adbArgs = @("-s", (($devices[0] -split "\s+")[0]))
    $serialArgs = @("-Serial", (($devices[0] -split "\s+")[0]))
}

$installArgs = @("-NoLaunch") + $serialArgs
if ($SkipBuild) {
    $installArgs += "-SkipBuild"
}

& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "install_android_debug.ps1") @installArgs
if ($LASTEXITCODE -ne 0) {
    Collect-Diagnostics "smoke-install-failed"
    exit $LASTEXITCODE
}

& $adb @adbArgs shell pm grant $packageName android.permission.POST_NOTIFICATIONS 2>$null | Out-Null

$packagePath = (& $adb @adbArgs shell pm path $packageName) -join "`n"
if ($packagePath -notmatch "package:") {
    Collect-Diagnostics "smoke-package-missing"
    throw "Installed package was not found by 'pm path $packageName'."
}

& $adb @adbArgs logcat -c

& $adb @adbArgs shell monkey -p $packageName -c android.intent.category.LAUNCHER 1 | Out-Host
if ($LASTEXITCODE -ne 0) {
    Collect-Diagnostics "smoke-launch-failed"
    exit $LASTEXITCODE
}

Start-Sleep -Seconds 2

$appPid = (& $adb @adbArgs shell pidof $packageName) -join "`n"
if ($appPid.Trim().Length -eq 0) {
    Collect-Diagnostics "smoke-process-missing"
    throw "App process is not running after launch."
}

Collect-Diagnostics "smoke-latest"

Write-Host "Android device smoke passed for $packageName"
