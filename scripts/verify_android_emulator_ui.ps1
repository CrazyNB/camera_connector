param(
    [string]$Serial = "emulator-5554",
    [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$packageName = "com.cameraconnector.app"
$dumpPath = "/sdcard/camera_connector_window.xml"
$diagnosticsDir = Join-Path $root "target\android-diagnostics\emulator-ui-latest"
$localDumpPath = Join-Path $root "target\android-diagnostics\emulator-ui-window.xml"

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

function U {
    param([int[]]$Codes)
    return -join ($Codes | ForEach-Object { [char]$_ })
}

function Invoke-Adb {
    param([string[]]$Arguments)
    & $adb @("-s", $Serial) @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "adb command failed: adb -s $Serial $($Arguments -join ' ')"
    }
}

function Start-App {
    Invoke-Adb @("shell", "am", "start", "-S", "-n", "$packageName/.MainActivity") | Out-Host
    Start-Sleep -Milliseconds 2000
}

function Get-UiXml {
    for ($attempt = 1; $attempt -le 30; $attempt++) {
        Remove-Item -LiteralPath $localDumpPath -ErrorAction SilentlyContinue
        & $adb -s $Serial shell rm -f $dumpPath 2>$null | Out-Null
        $oldErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            $dumpOutput = & $adb -s $Serial shell uiautomator dump $dumpPath 2>&1
            $dumpExitCode = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($dumpExitCode -ne 0 -or (($dumpOutput -join "`n") -notmatch "dumped to")) {
            Start-Sleep -Milliseconds 800
            continue
        }
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $localDumpPath) | Out-Null
        $oldErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            & $adb -s $Serial pull $dumpPath $localDumpPath 2>&1 | Out-Null
            $pullExitCode = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($pullExitCode -ne 0 -or -not (Test-Path -LiteralPath $localDumpPath -PathType Leaf)) {
            Start-Sleep -Milliseconds 800
            continue
        }
        $xml = [System.IO.File]::ReadAllText($localDumpPath, [System.Text.Encoding]::UTF8)
        if ($xml.Contains("<hierarchy") -and $xml.Contains($packageName)) {
            return $xml
        }
        Start-Sleep -Milliseconds 800
    }
    throw "Unable to read a stable UI hierarchy from $Serial."
}

function Assert-UiContains {
    param([string]$Xml, [string]$Needle, [string]$Label)
    if (-not $Xml.Contains($Needle)) {
        throw "Expected UI to contain '$Label'."
    }
}

function Assert-UiNotContains {
    param([string]$Xml, [string]$Needle, [string]$Label)
    if ($Xml.Contains($Needle)) {
        throw "Expected UI not to contain '$Label'."
    }
}

function Wait-UiContains {
    param([string]$Needle, [string]$Label)
    for ($attempt = 1; $attempt -le 8; $attempt++) {
        $xml = Get-UiXml
        if ($xml.Contains($Needle)) {
            return $xml
        }
        Start-Sleep -Milliseconds 700
    }
    throw "Expected UI to contain '$Label' within timeout."
}

function Tap-UntilUiContains {
    param([int]$X, [int]$Y, [string]$Needle, [string]$Label)
    for ($attempt = 1; $attempt -le 4; $attempt++) {
        Tap $X $Y
        for ($inner = 1; $inner -le 4; $inner++) {
            $xml = Get-UiXml
            if ($xml.Contains($Needle)) {
                return $xml
            }
            Start-Sleep -Milliseconds 500
        }
    }
    throw "Expected UI to contain '$Label' after tapping $X,$Y."
}

function Swipe-UntilUiContains {
    param([int]$StartX, [int]$StartY, [int]$EndX, [int]$EndY, [string]$Needle, [string]$Label)
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        $xml = Get-UiXml
        if ($xml.Contains($Needle)) {
            return $xml
        }
        Invoke-Adb @("shell", "input", "swipe", "$StartX", "$StartY", "$EndX", "$EndY", "500") | Out-Null
        Start-Sleep -Milliseconds 900
    }
    throw "Expected UI to contain '$Label' after swiping."
}

function Scroll-ToTop {
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        Invoke-Adb @("shell", "input", "swipe", "540", "760", "540", "1900", "500") | Out-Null
        Start-Sleep -Milliseconds 500
    }
}

function Tap {
    param([int]$X, [int]$Y)
    Invoke-Adb @("shell", "input", "tap", "$X", "$Y") | Out-Null
    Start-Sleep -Milliseconds 900
}

function Tap-UiNodeByText {
    param([string]$Xml, [string]$Text, [string]$Label)
    $escapedText = [System.Security.SecurityElement]::Escape($Text)
    $pattern = "text=""$([regex]::Escape($escapedText))""[^>]*bounds=""\[(\d+),(\d+)\]\[(\d+),(\d+)\]"""
    $match = [regex]::Match($Xml, $pattern)
    if (-not $match.Success) {
        throw "Unable to find UI node by text '$Label'."
    }
    $left = [int]$match.Groups[1].Value
    $top = [int]$match.Groups[2].Value
    $right = [int]$match.Groups[3].Value
    $bottom = [int]$match.Groups[4].Value
    Tap ([int](($left + $right) / 2)) ([int](($top + $bottom) / 2))
}

function Use-EmulatorBindableReceiverConfig {
    $remoteTempConfig = "/data/local/tmp/camera-connector-emulator-ui-config.json"
    $localConfig = Join-Path $diagnosticsDir "camera-connector-emulator-ui-config.json"
    New-Item -ItemType Directory -Force -Path $diagnosticsDir | Out-Null
    $rawConfig = & $adb -s $Serial shell run-as $packageName cat files/camera-connector.json
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to read Android app config before emulator bind setup."
    }
    $config = ($rawConfig -join "`n") | ConvertFrom-Json
    $config.receiver.protocol = "Ftp"
    $config.receiver.bind_host = "0.0.0.0"
    $config.receiver.ftp_port = 2121
    $config.receiver.sftp_port = 2121
    $json = $config | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText($localConfig, $json, [System.Text.UTF8Encoding]::new($false))
    & $adb -s $Serial push $localConfig $remoteTempConfig | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to push emulator bind config."
    }
    & $adb -s $Serial shell run-as $packageName cp $remoteTempConfig files/camera-connector.json
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to install emulator bind config."
    }
}

$labels = @{
    AppTitle = "Camera Connector"
    ServiceControl = U @(0x670D,0x52A1,0x63A7,0x5236)
    ReceiverService = U @(0x63A5,0x6536,0x670D,0x52A1)
    ReceiverSettings = U @(0x63A5,0x6536,0x8BBE,0x7F6E)
    ListenAddress = U @(0x76D1,0x542C,0x5730,0x5740)
    Start = U @(0x542F,0x52A8)
    Stop = U @(0x505C,0x6B62)
    Running = U @(0x8FD0,0x884C,0x4E2D)
    Stopped = U @(0x5DF2,0x505C,0x6B62)
    Settings = U @(0x8BBE,0x7F6E)
    SettingsSubtitle = U @(0x8D26,0x53F7,0x3001,0x76EE,0x5F55,0x3001,0x901A,0x77E5,0x6743,0x9650)
    DeviceAccounts = U @(0x8BBE,0x5907,0x8D26,0x53F7)
    ImportLocation = U @(0x5BFC,0x5165,0x4F4D,0x7F6E)
    Inbox = U @(0x6536,0x4EF6,0x7BB1)
    Transfers = U @(0x4F20,0x8F93)
    TransferLog = U @(0x4F20,0x8F93,0x8BB0,0x5F55)
    Overview = U @(0x603B,0x89C8)
    EmptyInbox = U @(0x8FD8,0x6CA1,0x6709,0x5BFC,0x5165,0x6587,0x4EF6,0x3002)
}

if (-not $SkipInstall) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "smoke_android_device.ps1") -Serial $Serial
    if ($LASTEXITCODE -ne 0) {
        throw "Smoke install failed before emulator UI verification."
    }
}

Start-App

$xml = Get-UiXml
if ($xml.Contains($labels.SettingsSubtitle)) {
    Tap 75 240
    $xml = Get-UiXml
}
Assert-UiContains $xml $labels.AppTitle "app title"
Assert-UiContains $xml $labels.ServiceControl "service control subtitle"
Assert-UiContains $xml $labels.ReceiverService "receiver service card"
Assert-UiContains $xml $labels.ReceiverSettings "receiver settings card"
Assert-UiContains $xml $labels.ListenAddress "listen address field"
Assert-UiContains $xml $labels.Start "start button"
Assert-UiContains $xml "FTP" "FTP protocol"
Assert-UiContains $xml "SFTP" "SFTP protocol"
Assert-UiContains $xml $labels.Overview "overview tab"
Assert-UiContains $xml $labels.Inbox "inbox tab"
Assert-UiContains $xml $labels.Transfers "transfers tab"
Assert-UiNotContains $xml $labels.DeviceAccounts "device accounts on overview"
Assert-UiNotContains $xml $labels.ImportLocation "import location on overview"

Tap-UiNodeByText $xml "SFTP" "SFTP protocol segment"
$saveReceiverSettingsLabel = U @(0x4FDD,0x5B58,0x63A5,0x6536,0x8BBE,0x7F6E)
$xml = Swipe-UntilUiContains 540 1950 540 1250 $saveReceiverSettingsLabel "save receiver settings"
Tap-UiNodeByText $xml $saveReceiverSettingsLabel "save receiver settings"
Scroll-ToTop
$xml = Get-UiXml
Assert-UiContains $xml "SFTP " "SFTP unified endpoint after save"
Use-EmulatorBindableReceiverConfig
Start-App
$xml = Wait-UiContains "FTP 0.0.0.0:2121" "emulator-bind FTP endpoint after config setup"
Assert-UiContains $xml "FTP 0.0.0.0:2121" "emulator-bind FTP endpoint after save"

if ($xml.Contains($labels.Running)) {
    Tap 540 1080
    $xml = Wait-UiContains $labels.Stopped "receiver stopped state"
}

$xml = Tap-UntilUiContains 540 1080 $labels.Running "receiver running state"
Assert-UiContains $xml $labels.Stop "stop button after running"
$xml = Tap-UntilUiContains 540 1080 $labels.Stopped "receiver stopped state"

Tap 975 240
$xml = Get-UiXml
Assert-UiContains $xml $labels.Settings "settings title"
Assert-UiContains $xml $labels.SettingsSubtitle "settings subtitle"
Assert-UiContains $xml $labels.DeviceAccounts "device accounts section"
Assert-UiNotContains $xml $labels.Overview "bottom overview tab on settings"
Assert-UiNotContains $xml $labels.Transfers "bottom transfers tab on settings"

Invoke-Adb @("shell", "input", "swipe", "540", "1700", "540", "850", "500") | Out-Null
Start-Sleep -Milliseconds 900
$xml = Get-UiXml
Assert-UiContains $xml $labels.ImportLocation "import location section"

Tap 75 240
$xml = Get-UiXml
Assert-UiContains $xml $labels.ServiceControl "overview after back"

Tap 540 2240
$xml = Get-UiXml
Assert-UiContains $xml $labels.Inbox "inbox screen title"

Tap 900 2240
$xml = Get-UiXml
Assert-UiContains $xml $labels.TransferLog "transfer log screen title"

Tap 170 2240
$xml = Get-UiXml
Assert-UiContains $xml $labels.ServiceControl "overview after tab navigation"

& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir $diagnosticsDir
if ($LASTEXITCODE -ne 0) {
    throw "Android diagnostics collection failed for emulator UI verification."
}

Write-Host "Android emulator UI verification passed for $packageName on $Serial"
