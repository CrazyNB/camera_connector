param(
    [string]$Serial,
    [string]$ReportPath
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$packageName = "com.cameraconnector.app"
$minimumSdk = 26

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

if (-not $ReportPath) {
    $ReportPath = Join-Path $root "target\android-device-preflight.txt"
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
}

$deviceSerial = (& $adb @adbArgs get-serialno).Trim()
$sdk = ((& $adb @adbArgs shell getprop ro.build.version.sdk) -join "`n").Trim()
$release = ((& $adb @adbArgs shell getprop ro.build.version.release) -join "`n").Trim()
$model = ((& $adb @adbArgs shell getprop ro.product.model) -join "`n").Trim()
$abis = ((& $adb @adbArgs shell getprop ro.product.cpu.abilist) -join "`n").Trim()
$ipAddr = ((& $adb @adbArgs shell ip addr) -join "`n").Trim()
$notificationPermission = ((& $adb @adbArgs shell dumpsys package $packageName 2>$null | Select-String -Pattern "POST_NOTIFICATIONS" -Context 0,2) -join "`n").Trim()

if ([int]$sdk -lt $minimumSdk) {
    throw "Android SDK $sdk is below Camera Connector minimum SDK $minimumSdk."
}
if ($abis -notmatch "arm64-v8a" -and $abis -notmatch "x86_64") {
    throw "Connected Android device does not advertise a packaged Camera Connector ABI (arm64-v8a or x86_64). ABI list: $abis"
}

$report = @(
    "serial: $deviceSerial",
    "model: $model",
    "android_release: $release",
    "sdk: $sdk",
    "abis: $abis",
    "notification_permission: $notificationPermission",
    "ip_addr:",
    $ipAddr
)

$reportDir = Split-Path -Parent $ReportPath
if ($reportDir) {
    New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
}
$report | Set-Content -LiteralPath $ReportPath -Encoding utf8

Write-Host "Android device preflight written to $ReportPath"
