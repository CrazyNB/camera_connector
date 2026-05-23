param(
    [string]$Serial = "emulator-5554",
    [string]$Username = "verify",
    [string]$Password = "secret",
    [string]$DeviceName = "Verify Camera",
    [int]$HostControlPort = 12121
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$packageName = "com.cameraconnector.app"
$configPath = Join-Path $root "target\android-emulator-ftp-config.json"
$controlForward = "tcp:$HostControlPort"
$deviceControl = "tcp:2121"
$androidConfig = "/data/user/0/$packageName/files/camera-connector.json"
$androidInbox = "/data/user/0/$packageName/files/inbox"
$androidState = "/data/user/0/$packageName/files/state"
$sampleName = "VERIFY_9001.NEF"
$sampleBytes = [byte[]](0x4E, 0x45, 0x46, 0x21, 0x01, 0x02, 0x03, 0x04)
$dumpPath = "/sdcard/camera_connector_ftp_verify_window.xml"
$localDumpPath = Join-Path $root "target\android-diagnostics\emulator-ftp-upload-window.xml"

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

function Invoke-Adb {
    param([string[]]$Arguments)
    & $adb @("-s", $Serial) @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "adb command failed: adb -s $Serial $($Arguments -join ' ')"
    }
}

function Remove-AdbForward {
    param([string]$Spec)
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        & $adb -s $Serial forward --remove $Spec 2>$null | Out-Null
    } finally {
        $ErrorActionPreference = $oldErrorActionPreference
    }
}

function Read-FtpReply {
    param([System.IO.StreamReader]$Reader)
    $line = $Reader.ReadLine()
    if ($null -eq $line) {
        throw "FTP server closed the control connection"
    }
    return $line
}

function Send-FtpCommand {
    param(
        [System.IO.StreamWriter]$Writer,
        [System.IO.StreamReader]$Reader,
        [string]$Command,
        [string]$Prefix
    )
    $Writer.WriteLine($Command)
    $reply = Read-FtpReply $Reader
    if (-not $reply.StartsWith($Prefix)) {
        throw "FTP command '$Command' expected $Prefix, got '$reply'"
    }
    return $reply
}

function Wait-LocalPort {
    param([int]$Port)
    for ($i = 0; $i -lt 40; $i++) {
        $client = $null
        try {
            $client = [System.Net.Sockets.TcpClient]::new()
            $connect = $client.BeginConnect("127.0.0.1", $Port, $null, $null)
            if ($connect.AsyncWaitHandle.WaitOne(250)) {
                $client.EndConnect($connect)
                return
            }
        } catch {
        } finally {
            if ($client) { $client.Dispose() }
        }
        Start-Sleep -Milliseconds 250
    }
    throw "Timed out waiting for localhost:$Port"
}

function Test-FtpGreeting {
    $client = $null
    try {
        $client = [System.Net.Sockets.TcpClient]::new("127.0.0.1", $HostControlPort)
        $client.ReceiveTimeout = 2000
        $stream = $client.GetStream()
        $reader = [System.IO.StreamReader]::new($stream, [System.Text.Encoding]::ASCII)
        $line = $reader.ReadLine()
        return ($line -ne $null -and $line.StartsWith("220"))
    } catch {
        return $false
    } finally {
        if ($client) { $client.Dispose() }
    }
}

function Start-ReceiverFromUi {
    Invoke-Adb @("shell", "am", "start", "-S", "-n", "$packageName/.MainActivity") | Out-Null
    Start-Sleep -Seconds 3
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        Invoke-Adb @("shell", "input", "tap", "540", "760") | Out-Null
        Start-Sleep -Seconds 2
        if (Test-FtpGreeting) {
            return
        }
    }
    throw "Android receiver did not expose an FTP greeting after starting from UI."
}

function Get-AndroidFileText {
    param([string]$Path)
    return (& $adb -s $Serial shell run-as $packageName cat $Path) -join "`n"
}

function Get-UiXml {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $localDumpPath) | Out-Null
    Invoke-Adb @("shell", "uiautomator", "dump", $dumpPath) | Out-Null
    Invoke-Adb @("pull", $dumpPath, $localDumpPath) | Out-Null
    return [System.IO.File]::ReadAllText($localDumpPath, [System.Text.Encoding]::UTF8)
}

function Assert-UiContains {
    param([string]$Xml, [string]$Needle, [string]$Label)
    if (-not $Xml.Contains($Needle)) {
        throw "Expected UI to contain '$Label'."
    }
}

function Tap-UntilUiContains {
    param([int]$X, [int]$Y, [string]$Needle, [string]$Label)
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        Invoke-Adb @("shell", "input", "tap", "$X", "$Y") | Out-Null
        Start-Sleep -Milliseconds 900
        $xml = Get-UiXml
        if ($xml.Contains($Needle)) {
            return $xml
        }
    }
    throw "Expected UI to contain '$Label' after tapping $X,$Y."
}

New-Item -ItemType Directory -Force -Path (Join-Path $root "target") | Out-Null

& (Join-Path $root "target\debug\camera-connector.exe") account --config $configPath set --username $Username --password $Password --device-name $DeviceName | Out-Host
if ($LASTEXITCODE -ne 0) { throw "Failed to create verifier account config." }

& (Join-Path $root "target\debug\camera-connector.exe") receiver-settings `
    --config $configPath `
    --protocol ftp `
    --bind-host "0.0.0.0" `
    --ftp-port 2121 `
    --sftp-port 2222 `
    --output $androidInbox `
    --state $androidState `
    --advertised-host "127.0.0.1" `
    --source-name $DeviceName | Out-Host
if ($LASTEXITCODE -ne 0) { throw "Failed to create Android receiver config." }

$configRaw = Get-Content -Raw -LiteralPath $configPath
if ($configRaw -like "*$Password*") {
    throw "Verifier config leaked plaintext password."
}
if ($configRaw -notlike "*password_hash*") {
    throw "Verifier config did not include password hash."
}

Invoke-Adb @("shell", "am", "force-stop", $packageName) | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "rm", "-rf", "files/inbox", "files/state") | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "mkdir", "-p", "files/inbox", "files/state") | Out-Null
Invoke-Adb @("push", $configPath, "/data/local/tmp/camera-connector.json") | Out-Null
Invoke-Adb @("shell", "chmod", "644", "/data/local/tmp/camera-connector.json") | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "cp", "/data/local/tmp/camera-connector.json", "files/camera-connector.json") | Out-Null

Remove-AdbForward $controlForward
Invoke-Adb @("forward", $controlForward, $deviceControl) | Out-Null
& $adb -s $Serial shell pm grant $packageName android.permission.POST_NOTIFICATIONS 2>$null | Out-Null
Start-ReceiverFromUi

$control = [System.Net.Sockets.TcpClient]::new("127.0.0.1", $HostControlPort)
$dataForward = $null
try {
    $stream = $control.GetStream()
    $reader = [System.IO.StreamReader]::new($stream, [System.Text.Encoding]::ASCII)
    $writer = [System.IO.StreamWriter]::new($stream, [System.Text.Encoding]::ASCII)
    $writer.NewLine = "`r`n"
    $writer.AutoFlush = $true

    $hello = Read-FtpReply $reader
    if (-not $hello.StartsWith("220")) { throw "FTP greeting failed: $hello" }
    Send-FtpCommand $writer $reader "USER $Username" "331" | Out-Null
    Send-FtpCommand $writer $reader "PASS $Password" "230" | Out-Null
    Send-FtpCommand $writer $reader "TYPE I" "200" | Out-Null
    Send-FtpCommand $writer $reader "CWD DCIM/100VERIFY" "250" | Out-Null
    $epsv = Send-FtpCommand $writer $reader "EPSV" "229"
    if ($epsv -notmatch "\(\|\|\|(\d+)\|\)") {
        throw "EPSV reply was not parseable: $epsv"
    }
    $deviceDataPort = [int]$Matches[1]
    $hostDataPort = $deviceDataPort
    $dataForward = "tcp:$hostDataPort"
    Remove-AdbForward $dataForward
    Invoke-Adb @("forward", $dataForward, "tcp:$deviceDataPort") | Out-Null

    $data = [System.Net.Sockets.TcpClient]::new("127.0.0.1", $hostDataPort)
    try {
        $writer.WriteLine("STOR $sampleName")
        $stor = Read-FtpReply $reader
        if (-not $stor.StartsWith("150")) { throw "STOR expected 150, got '$stor'" }
        $dataStream = $data.GetStream()
        $dataStream.Write($sampleBytes, 0, $sampleBytes.Length)
        $dataStream.Close()
        $complete = Read-FtpReply $reader
        if (-not $complete.StartsWith("226")) { throw "STOR expected 226, got '$complete'" }
    } finally {
        $data.Dispose()
    }
    Send-FtpCommand $writer $reader "QUIT" "221" | Out-Null
} finally {
    $control.Dispose()
    if ($dataForward) {
        Remove-AdbForward $dataForward
    }
    Remove-AdbForward $controlForward
}

$uploadedSize = ((& $adb -s $Serial shell run-as $packageName stat -c "%s" "files/inbox/$sampleName") -join "`n").Trim()
if ([int64]$uploadedSize -ne $sampleBytes.Length) {
    throw "Uploaded file size mismatch. Expected $($sampleBytes.Length), got $uploadedSize."
}

$transferLog = Get-AndroidFileText "files/state/transfer-log.jsonl"
if ($transferLog -notmatch $sampleName) {
    throw "Android transfer log did not include uploaded sample."
}
if ($transferLog -notmatch $Username -or $transferLog -notmatch $DeviceName) {
    throw "Android transfer log did not include account identity."
}

$configAfter = Get-AndroidFileText "files/camera-connector.json"
if ($configAfter -like "*$Password*") {
    throw "Android config leaked plaintext password."
}

Invoke-Adb @("shell", "am", "start", "-S", "-n", "$packageName/.MainActivity") | Out-Null
Start-Sleep -Seconds 3
$inboxUi = Tap-UntilUiContains 540 2240 $sampleName "uploaded asset in inbox"
$transferUi = Tap-UntilUiContains 900 2240 $sampleName "uploaded transfer row"
Assert-UiContains $transferUi $sampleName "uploaded transfer row"
Assert-UiContains $transferUi $DeviceName "transfer device name"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir (Join-Path $root "target\android-diagnostics\emulator-ftp-upload-latest")
if ($LASTEXITCODE -ne 0) {
    throw "Android diagnostics collection failed after FTP upload verification."
}

Write-Host "Android emulator FTP account login, connection, and upload verification passed for $sampleName"
