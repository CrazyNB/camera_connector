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
$sampleRawName = "VERIFY_9001.NEF"
$sampleJpegName = "VERIFY_9001.JPG"
$sampleRawBytes = [byte[]](0x4E, 0x45, 0x46, 0x21, 0x01, 0x02, 0x03, 0x04)
$sampleJpegBytes = $null
$dumpPath = "/sdcard/camera_connector_ftp_verify_window.xml"
$localDumpPath = Join-Path $root "target\android-diagnostics\emulator-ftp-upload-window.xml"

if (-not (Test-Path -LiteralPath $adb -PathType Leaf)) {
    throw "adb not found at $adb. Set ANDROID_SDK_ROOT or install Android platform-tools."
}

Add-Type -AssemblyName System.Drawing
$sampleBitmap = [System.Drawing.Bitmap]::new(96, 72)
$sampleGraphics = [System.Drawing.Graphics]::FromImage($sampleBitmap)
$sampleStream = [System.IO.MemoryStream]::new()
try {
    $sampleGraphics.Clear([System.Drawing.Color]::FromArgb(38, 132, 255))
    $sampleGraphics.FillEllipse([System.Drawing.Brushes]::White, 26, 14, 44, 44)
    $sampleBitmap.Save($sampleStream, [System.Drawing.Imaging.ImageFormat]::Jpeg)
    $sampleJpegBytes = $sampleStream.ToArray()
} finally {
    $sampleGraphics.Dispose()
    $sampleBitmap.Dispose()
    $sampleStream.Dispose()
}

function Invoke-Adb {
    param([string[]]$Arguments)
    & $adb @("-s", $Serial) @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "adb command failed: adb -s $Serial $($Arguments -join ' ')"
    }
}

function U {
    param([int[]]$Codes)
    return -join ($Codes | ForEach-Object { [char]$_ })
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
    for ($attempt = 1; $attempt -le 5; $attempt++) {
        if (Test-Path -LiteralPath $localDumpPath) {
            Remove-Item -LiteralPath $localDumpPath -Force
        }
        Invoke-Adb @("shell", "rm", "-f", $dumpPath) | Out-Null
        $oldErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            $dumpOutput = & $adb -s $Serial shell uiautomator dump $dumpPath 2>&1
            $dumpExitCode = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($dumpExitCode -ne 0 -or (($dumpOutput -join "`n") -notmatch "dumped to")) {
            Start-Sleep -Milliseconds 400
            continue
        }
        $oldErrorActionPreference = $ErrorActionPreference
        $ErrorActionPreference = "Continue"
        try {
            & $adb -s $Serial pull $dumpPath $localDumpPath 2>&1 | Out-Null
            $pullExitCode = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($pullExitCode -ne 0 -or -not (Test-Path -LiteralPath $localDumpPath -PathType Leaf)) {
            Start-Sleep -Milliseconds 400
            continue
        }
        $xml = [System.IO.File]::ReadAllText($localDumpPath, [System.Text.Encoding]::UTF8)
        if ($xml.Contains("<hierarchy")) {
            return $xml
        }
        Start-Sleep -Milliseconds 400
    }
    throw "Unable to dump Android UI hierarchy."
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

function Tap-UiNodeByContentDescription {
    param([string]$Xml, [string]$Needle, [string]$Label)
    $pattern = 'content-desc="' + [regex]::Escape($Needle) + '[^"]*"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"'
    $match = [regex]::Match($Xml, $pattern)
    if (-not $match.Success) {
        throw "Expected UI to contain tappable content description '$Label'."
    }
    $x = [int](([int]$match.Groups[1].Value + [int]$match.Groups[3].Value) / 2)
    $y = [int](([int]$match.Groups[2].Value + [int]$match.Groups[4].Value) / 2)
    Invoke-Adb @("shell", "input", "tap", "$x", "$y") | Out-Null
    Start-Sleep -Milliseconds 900
}

function Send-FtpFile {
    param(
        [System.IO.StreamWriter]$Writer,
        [System.IO.StreamReader]$Reader,
        [string]$Filename,
        [byte[]]$Bytes
    )
    $epsv = Send-FtpCommand $Writer $Reader "EPSV" "229"
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
        $Writer.WriteLine("STOR $Filename")
        $stor = Read-FtpReply $Reader
        if (-not $stor.StartsWith("150")) { throw "STOR expected 150, got '$stor'" }
        $dataStream = $data.GetStream()
        $dataStream.Write($Bytes, 0, $Bytes.Length)
        $dataStream.Close()
        $complete = Read-FtpReply $Reader
        if (-not $complete.StartsWith("226")) { throw "STOR expected 226, got '$complete'" }
    } finally {
        $data.Dispose()
        Remove-AdbForward $dataForward
    }
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
    Send-FtpFile $writer $reader $sampleRawName $sampleRawBytes
    Send-FtpFile $writer $reader $sampleJpegName $sampleJpegBytes
    Send-FtpCommand $writer $reader "QUIT" "221" | Out-Null
} finally {
    $control.Dispose()
    Remove-AdbForward $controlForward
}

$uploadedRawSize = ((& $adb -s $Serial shell run-as $packageName stat -c "%s" "files/inbox/$sampleRawName") -join "`n").Trim()
if ([int64]$uploadedRawSize -ne $sampleRawBytes.Length) {
    throw "Uploaded raw file size mismatch. Expected $($sampleRawBytes.Length), got $uploadedRawSize."
}
$uploadedJpegSize = ((& $adb -s $Serial shell run-as $packageName stat -c "%s" "files/inbox/$sampleJpegName") -join "`n").Trim()
if ([int64]$uploadedJpegSize -ne $sampleJpegBytes.Length) {
    throw "Uploaded jpeg file size mismatch. Expected $($sampleJpegBytes.Length), got $uploadedJpegSize."
}

$transferLog = Get-AndroidFileText "files/state/transfer-log.jsonl"
if ($transferLog -notmatch $sampleRawName -or $transferLog -notmatch $sampleJpegName) {
    throw "Android transfer log did not include uploaded raw/jpeg pair."
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
$inboxUi = Tap-UntilUiContains 540 2240 $sampleJpegName "uploaded asset in inbox"
Assert-UiContains $inboxUi "RAW" "raw pair tag"
Assert-UiContains $inboxUi "JPG" "jpeg pair tag"
Assert-UiContains $inboxUi (U @(0x5168,0x90E8,0x6765,0x6E90)) "source filter"
Assert-UiNotContains $inboxUi (U @(0x0052,0x0041,0x0057,0x0020,0x9884,0x89C8,0x5F85,0x751F,0x6210)) "raw preview placeholder"
Assert-UiContains $inboxUi (U @(0x6536,0x4EF6,0x7BB1,0x0032,0x5217,0x89C6,0x56FE)) "2-column grid control"
Tap-UiNodeByContentDescription $inboxUi (U @(0x6536,0x4EF6,0x7BB1,0x0032,0x5217,0x89C6,0x56FE)) "2-column grid control"
$gridPrefs = Get-AndroidFileText "shared_prefs/camera_connector_storage.xml"
if ($gridPrefs -notmatch 'name="inbox_grid_columns"\s+value="2"') {
    throw "Android inbox grid preference did not persist 2-column selection."
}
Invoke-Adb @("shell", "am", "force-stop", $packageName) | Out-Null
Invoke-Adb @("shell", "am", "start", "-n", "$packageName/.MainActivity") | Out-Null
Start-Sleep -Seconds 2
$inboxUi = Tap-UntilUiContains 540 2240 $sampleJpegName "uploaded asset in inbox after restart"
Assert-UiContains $inboxUi (U @(0x6536,0x4EF6,0x7BB1,0x0032,0x5217,0x89C6,0x56FE)) "persisted 2-column grid control after restart"
Tap-UiNodeByContentDescription $inboxUi "$(U @(0x7167,0x7247,0x0020))$sampleJpegName" "uploaded photo tile"
$detailUi = Get-UiXml
Assert-UiContains $detailUi (U @(0x7167,0x7247,0x8BE6,0x60C5)) "photo detail screen"
Assert-UiContains $detailUi (U @(0x6765,0x6E90,0x4FE1,0x606F)) "photo source information"
Assert-UiContains $detailUi $sampleJpegName "photo detail jpeg file"
Assert-UiNotContains $detailUi (U @(0x0052,0x0041,0x0057,0x0020,0x9884,0x89C8,0x5F85,0x751F,0x6210)) "raw preview placeholder in detail"
Invoke-Adb @("shell", "input", "swipe", "540", "1900", "540", "900", "400") | Out-Null
Start-Sleep -Milliseconds 700
$detailFilesUi = Get-UiXml
Assert-UiContains $detailFilesUi $sampleRawName "photo detail raw file"
Invoke-Adb @("shell", "input", "keyevent", "4") | Out-Null
Start-Sleep -Milliseconds 700
$transferUi = Tap-UntilUiContains 900 2240 $sampleJpegName "uploaded transfer row"
Assert-UiContains $transferUi $sampleRawName "uploaded raw transfer row"
Assert-UiContains $transferUi $sampleJpegName "uploaded jpeg transfer row"
Assert-UiContains $transferUi $DeviceName "transfer device name"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir (Join-Path $root "target\android-diagnostics\emulator-ftp-upload-latest")
if ($LASTEXITCODE -ne 0) {
    throw "Android diagnostics collection failed after FTP upload verification."
}

Write-Host "Android emulator FTP account login, connection, and RAW/JPEG upload verification passed for $sampleRawName + $sampleJpegName"
