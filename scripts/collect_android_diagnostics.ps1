param(
    [string]$Serial,
    [string]$OutputDir,
    [int]$LogLines = 600
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$packageName = "com.cameraconnector.app"
$tag = "CameraConnectorReceiver"

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

if (-not $OutputDir) {
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $OutputDir = Join-Path $root "target\android-diagnostics\$stamp"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$deviceArgs = @()
if ($Serial) {
    $deviceArgs = @("-s", $Serial)
} else {
    $devices = @(& $adb devices | Select-Object -Skip 1 | Where-Object { $_ -match "\tdevice$" })
    if ($devices.Count -eq 0) {
        throw "No adb device is connected. Enable USB debugging or connect with adb tcpip."
    }
    if ($devices.Count -gt 1) {
        $serials = $devices | ForEach-Object { ($_ -split "\s+")[0] }
        throw "Multiple adb devices are connected. Re-run with -Serial <serial>. Connected: $($serials -join ', ')"
    }
    $deviceArgs = @("-s", (($devices[0] -split "\s+")[0]))
}

& $adb devices | Out-File -LiteralPath (Join-Path $OutputDir "adb-devices.txt") -Encoding utf8
& $adb @deviceArgs shell dumpsys package $packageName | Out-File -LiteralPath (Join-Path $OutputDir "package.txt") -Encoding utf8
& $adb @deviceArgs shell dumpsys activity services $packageName | Out-File -LiteralPath (Join-Path $OutputDir "services.txt") -Encoding utf8
& $adb @deviceArgs logcat -d -t $LogLines AndroidRuntime:E "$tag`:V" "*:S" | Out-File -LiteralPath (Join-Path $OutputDir "receiver-logcat.txt") -Encoding utf8

Write-Host "Android diagnostics written to $OutputDir"
