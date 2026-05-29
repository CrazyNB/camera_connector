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

function Enter-ProjectWorkspaceIfNeeded {
    param([string]$Xml)
    if ($Xml.Contains($labels.ProjectWorkspace)) {
        return $Xml
    }
    if (-not $Xml.Contains($labels.ProjectManagement)) {
        return $Xml
    }
    if ($Xml.Contains($labels.Enter)) {
        Tap-UiNodeByText $Xml $labels.Enter "enter selected project"
    } else {
        Tap-UiNodeByText $Xml $labels.Select "select project"
    }
    return Wait-UiContains $labels.ProjectWorkspace "project workspace after selecting project"
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
    ProjectManagement = U @(0x9879,0x76EE,0x7BA1,0x7406)
    ProjectWorkspace = U @(0x9879,0x76EE,0x5DE5,0x4F5C,0x53F0)
    CurrentProject = U @(0x5F53,0x524D,0x62CD,0x6444,0x9879,0x76EE)
    NewProject = U @(0x65B0,0x5EFA,0x9879,0x76EE)
    Enter = U @(0x8FDB,0x5165)
    Select = U @(0x9009,0x62E9)
    ServiceControl = U @(0x63A5,0x6536,0x670D,0x52A1,0x63A7,0x5236)
    ReceiverSettings = U @(0x63A5,0x6536,0x8BBE,0x7F6E)
    ListenAddress = U @(0x76D1,0x542C,0x5730,0x5740)
    Start = U @(0x542F,0x52A8)
    Stop = U @(0x505C,0x6B62)
    Running = U @(0x8FD0,0x884C,0x4E2D)
    Stopped = U @(0x5DF2,0x505C,0x6B62)
    Project = U @(0x9879,0x76EE)
    Account = U @(0x8D26,0x53F7)
    Settings = U @(0x8BBE,0x7F6E)
    Diagnostics = U @(0x8BCA,0x65AD)
    Overview = U @(0x6982,0x89C8)
    Photos = U @(0x7167,0x7247)
    ImportLocation = U @(0x5BFC,0x5165,0x4F4D,0x7F6E)
    ProjectPhotos = U @(0x9879,0x76EE,0x7167,0x7247)
    AccountsTitle = U @(0x8D26,0x53F7,0x7BA1,0x7406)
    SettingsTitle = U @(0x7CFB,0x7EDF,0x8BBE,0x7F6E)
    DiagnosticsTitle = U @(0x8BCA,0x65AD,0x65E5,0x5FD7)
    OnlineAccounts = U @(0x5728,0x7EBF,0x8D26,0x53F7)
    SaveReceiverSettings = U @(0x4FDD,0x5B58,0x63A5,0x6536,0x8BBE,0x7F6E)
}

if (-not $SkipInstall) {
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "smoke_android_device.ps1") -Serial $Serial
    if ($LASTEXITCODE -ne 0) {
        throw "Smoke install failed before emulator UI verification."
    }
}

& $adb -s $Serial shell pm grant $packageName android.permission.POST_NOTIFICATIONS 2>$null | Out-Null

Start-App

$xml = Wait-UiContains $labels.ProjectManagement "project management"
Assert-UiContains $xml $labels.ProjectManagement "project management title"
Assert-UiContains $xml $labels.NewProject "new project action"
Assert-UiContains $xml $labels.Project "global projects destination"
Assert-UiContains $xml $labels.Account "global accounts destination"
Assert-UiContains $xml $labels.Settings "global settings destination"
Assert-UiNotContains $xml $labels.Diagnostics "diagnostics is not a global destination"
Assert-UiNotContains $xml $labels.Overview "project overview tab before entering project"
Assert-UiNotContains $xml $labels.Photos "project photos tab before entering project"
if ($xml.Contains($labels.Enter)) {
    Tap-UiNodeByText $xml $labels.Enter "enter selected project"
} else {
    Tap-UiNodeByText $xml $labels.Select "select project"
}

$xml = Wait-UiContains $labels.ProjectWorkspace "project workspace"
Assert-UiContains $xml $labels.ProjectWorkspace "project workspace title"
Assert-UiContains $xml $labels.ServiceControl "service control subtitle"
Assert-UiContains $xml $labels.ReceiverSettings "receiver settings card"
Assert-UiContains $xml $labels.ListenAddress "listen address field"
Assert-UiContains $xml $labels.Start "start button"
Assert-UiContains $xml "FTP" "FTP protocol"
Assert-UiContains $xml "SFTP" "SFTP protocol"
Assert-UiContains $xml $labels.Project "global projects destination"
Assert-UiContains $xml $labels.Account "global accounts destination"
Assert-UiContains $xml $labels.Settings "global settings destination"
Assert-UiNotContains $xml $labels.Diagnostics "diagnostics is not a global destination"
Assert-UiContains $xml $labels.Overview "project overview tab"
Assert-UiContains $xml $labels.Photos "project photos tab"
Assert-UiNotContains $xml (U @(0x6536,0x4EF6,0x7BB1)) "project inbox tab"
Assert-UiNotContains $xml (U @(0x4F20,0x8F93)) "project transfers tab"
Assert-UiNotContains $xml (U @(0x53D1,0x5E03)) "project publish tab"

Tap-UiNodeByText $xml "SFTP" "SFTP protocol segment"
$xml = Get-UiXml
Assert-UiNotContains $xml $labels.SaveReceiverSettings "receiver settings save button is not shown"
Use-EmulatorBindableReceiverConfig
Start-App
$xml = Wait-UiContains "FTP 192.168.50.1:2121" "emulator-bind FTP endpoint after config setup"
$xml = Enter-ProjectWorkspaceIfNeeded $xml
Assert-UiContains $xml "FTP 192.168.50.1:2121" "emulator-bind FTP endpoint after save"

if ($xml.Contains($labels.Running)) {
    Tap 540 1200
    $xml = Wait-UiContains $labels.Stopped "receiver stopped state"
}

$xml = Enter-ProjectWorkspaceIfNeeded $xml
$xml = Tap-UntilUiContains 540 1200 $labels.Running "receiver running state"
Assert-UiContains $xml $labels.Stop "stop button after running"
$xml = Tap-UntilUiContains 540 1200 $labels.Stopped "receiver stopped state"

Tap-UiNodeByText $xml $labels.Photos "project photos tab"
$xml = Wait-UiContains $labels.ProjectPhotos "project photos screen title"
Assert-UiContains $xml $labels.ProjectPhotos "project photos screen title"

Tap-UiNodeByText $xml $labels.Account "global accounts destination"
$xml = Wait-UiContains $labels.AccountsTitle "accounts screen title"
Assert-UiContains $xml $labels.AccountsTitle "accounts screen title"

Tap-UiNodeByText $xml $labels.Settings "global settings destination"
$xml = Wait-UiContains $labels.SettingsTitle "settings screen title"
Assert-UiContains $xml $labels.SettingsTitle "settings screen title"
Assert-UiContains $xml $labels.ImportLocation "settings import location section"
Assert-UiContains $xml $labels.DiagnosticsTitle "settings diagnostics menu"

Tap-UiNodeByText $xml $labels.DiagnosticsTitle "settings diagnostics menu"
$xml = Wait-UiContains $labels.OnlineAccounts "diagnostics screen metrics"
Assert-UiContains $xml $labels.DiagnosticsTitle "diagnostics screen title"
Assert-UiContains $xml $labels.OnlineAccounts "diagnostics screen metrics"

Tap-UiNodeByText $xml $labels.Project "global projects destination"
$xml = Wait-UiContains $labels.ProjectManagement "project management after global navigation"
Assert-UiContains $xml $labels.ProjectManagement "project management after global navigation"
Assert-UiContains $xml $labels.NewProject "new project action after global navigation"
Assert-UiNotContains $xml $labels.Overview "project overview tab after returning to project management"
Assert-UiNotContains $xml $labels.Photos "project photos tab after returning to project management"

& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir $diagnosticsDir
if ($LASTEXITCODE -ne 0) {
    throw "Android diagnostics collection failed for emulator UI verification."
}

Write-Host "Android emulator UI verification passed for $packageName on $Serial"
