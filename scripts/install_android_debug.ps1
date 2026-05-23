param(
    [string]$Serial,
    [switch]$SkipBuild,
    [switch]$NoLaunch,
    [int]$InstallTimeoutSeconds = 120
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$apk = Join-Path $root "apps\android\app\build\outputs\apk\debug\app-debug.apk"
$packageName = "com.cameraconnector.app"

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

if (-not $SkipBuild) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "verify_android_build.ps1")
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

if (-not (Test-Path -LiteralPath $apk -PathType Leaf)) {
    throw "APK not found at $apk. Run scripts\verify_android_build.ps1 first or omit -SkipBuild."
}

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

$installOut = Join-Path $root "target\android-install.out.txt"
$installErr = Join-Path $root "target\android-install.err.txt"
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $installOut) | Out-Null
Remove-Item -LiteralPath $installOut, $installErr -ErrorAction SilentlyContinue

$install = Start-Process `
    -FilePath $adb `
    -ArgumentList ($deviceArgs + @("install", "-r", $apk)) `
    -RedirectStandardOutput $installOut `
    -RedirectStandardError $installErr `
    -NoNewWindow `
    -PassThru

if (-not $install.WaitForExit($InstallTimeoutSeconds * 1000)) {
    Stop-Process -Id $install.Id -Force
    throw "adb install timed out after $InstallTimeoutSeconds seconds. Check the phone for USB install prompts, enable Install via USB if required, then retry."
}
$install.WaitForExit()
Start-Sleep -Milliseconds 300
$installOutput = ""
if (Test-Path -LiteralPath $installOut) {
    $installOutput += (Get-Content -LiteralPath $installOut -Raw)
}
if (Test-Path -LiteralPath $installErr) {
    $installOutput += "`n" + (Get-Content -LiteralPath $installErr -Raw)
}
if ($install.ExitCode -ne 0 -or $installOutput -match "failed to install|INSTALL_FAILED|Failure \\[") {
    if (Test-Path -LiteralPath $installOut) { Get-Content -LiteralPath $installOut | ForEach-Object { Write-Host $_ } }
    if (Test-Path -LiteralPath $installErr) { Get-Content -LiteralPath $installErr | ForEach-Object { Write-Host $_ } }
    throw "adb install failed. Check the phone for USB install prompts or enable Install via USB, then retry."
}

if (-not $NoLaunch) {
    & $adb @deviceArgs shell monkey -p $packageName -c android.intent.category.LAUNCHER 1 | Out-Host
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

Write-Host "Installed Camera Connector debug APK to $($deviceArgs -join ' ')"
