param(
    [string]$AvdName = "CameraConnector_API36",
    [switch]$Headless,
    [switch]$InstallAfterBoot,
    [int]$BootTimeoutSeconds = 300
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$emulator = Join-Path $sdkRoot "emulator\emulator.exe"
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"

if (-not (Test-Path -LiteralPath $emulator -PathType Leaf)) {
    throw "Android emulator not found at $emulator. Install the SDK emulator package first."
}
if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Install Android platform-tools first."
}

& $adb start-server | Out-Null

$running = & $adb devices | Select-String "emulator-\d+\s+device" | Select-Object -First 1
if (-not $running) {
    $args = @("-avd", $AvdName, "-no-snapshot-load", "-gpu", "swiftshader_indirect")
    if ($Headless) {
        $args += @("-no-window", "-no-audio", "-no-boot-anim")
        Start-Process -WindowStyle Hidden -FilePath $emulator -ArgumentList $args
    } else {
        Start-Process -FilePath $emulator -ArgumentList $args
    }
}

$deadline = (Get-Date).AddSeconds($BootTimeoutSeconds)
$serial = $null
while ((Get-Date) -lt $deadline) {
    $running = & $adb devices | Select-String "emulator-\d+\s+device" | Select-Object -First 1
    if ($running) {
        $serial = ($running.ToString() -split "\s+")[0]
        $bootValue = & $adb -s $serial shell getprop sys.boot_completed 2>$null
        $booted = ($bootValue -join "").Trim()
        if ($booted -eq "1") {
            break
        }
    }
    Start-Sleep -Seconds 3
}

if (-not $serial) {
    throw "No emulator device appeared within $BootTimeoutSeconds seconds."
}

$bootValue = & $adb -s $serial shell getprop sys.boot_completed 2>$null
$booted = ($bootValue -join "").Trim()
if ($booted -ne "1") {
    throw "Emulator $serial did not finish booting within $BootTimeoutSeconds seconds."
}

if ($InstallAfterBoot) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "install_android_debug.ps1") -Serial $serial -SkipBuild
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

Write-Host "Android emulator ready: $serial ($AvdName)"
