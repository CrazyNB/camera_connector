param(
    [string]$Serial,
    [string]$OutputDir,
    [switch]$RunSmoke,
    [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
if (-not $OutputDir) {
    $OutputDir = Join-Path $root "target\android-diagnostics\real-camera-test-latest"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$serialArgs = @()
if ($Serial) {
    $serialArgs = @("-Serial", $Serial)
}

$preflightPath = Join-Path $OutputDir "preflight.txt"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "preflight_android_device.ps1") @serialArgs -ReportPath $preflightPath
if ($LASTEXITCODE -ne 0) {
    throw "Android device preflight failed."
}

if ($RunSmoke) {
    $smokeArgs = @() + $serialArgs
    if ($SkipInstall) {
        $smokeArgs += "-SkipInstall"
    }
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "smoke_android_device.ps1") @smokeArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Android device smoke failed."
    }
}

$preflight = Get-Content -LiteralPath $preflightPath -Raw -Encoding UTF8
$reportPath = Join-Path $OutputDir "android-real-camera-test.md"
$now = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
$smokeStatus = if ($RunSmoke) { "Run" } else { "Not run" }

@"
# Android Real-Camera Test Record

Generated: $now

## Device Preflight

~~~text
$preflight
~~~

## Execution

- Device smoke: $smokeStatus
- Diagnostics directory: $OutputDir

## Compatibility Entry

~~~text
Date:
Tester:
Phone model:
Android version:
Camera vendor/model/firmware:
Network mode: phone hotspot / LAN / camera AP
Phone receiver IP:
Camera IP:
Protocol: FTP
Port:
Authentication:
Selected Android output: SAF tree / app-private fallback
Notification permission:
Foreground service: starts / remains visible / stops cleanly
Camera login:
JPEG upload:
RAW upload:
RAW+JPEG grouping:
SAF publish:
Project photos visibility:
Photo detail visibility:
Project asset visibility:
Transfer record visibility:
Publish queue recovery:
Diagnostics path: $OutputDir
Compatibility result:
Notes:
~~~

Copy the completed entry into docs\compatibility.md after the test.
"@ | Set-Content -LiteralPath $reportPath -Encoding UTF8

Write-Host "Android real-camera test record written to $reportPath"
