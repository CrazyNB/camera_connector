param(
    [string]$Serial = "emulator-5554",
    [string]$Username = "verify",
    [string]$Password = "secret",
    [string]$DeviceName = "Verify Camera",
    [int]$HostControlPort = 12121,
    [string]$RealAssetDirectory,
    [int]$RealPairLimit = 5
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
$rawExtensions = @(".NEF", ".NRW", ".CR3", ".CR2", ".ARW", ".SRF", ".SR2", ".RAF", ".RW2", ".RWL", ".ORF", ".PEF", ".DNG")
$jpegExtensions = @(".JPG", ".JPEG")

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

function Find-RealRawJpegPairs {
    param([string]$Directory)
    if ([string]::IsNullOrWhiteSpace($Directory)) {
        return @()
    }
    if (-not (Test-Path -LiteralPath $Directory -PathType Container)) {
        throw "Real asset directory does not exist: $Directory"
    }

    $files = @(Get-ChildItem -LiteralPath $Directory -Recurse -File)
    $rawByStem = @{}
    $jpegByStem = @{}
    foreach ($file in $files) {
        $extension = $file.Extension.ToUpperInvariant()
        $stem = [System.IO.Path]::GetFileNameWithoutExtension($file.Name).ToUpperInvariant()
        if ($rawExtensions -contains $extension) {
            if (-not $rawByStem.ContainsKey($stem) -or $file.Length -lt $rawByStem[$stem].Length) {
                $rawByStem[$stem] = $file
            }
        }
        if ($jpegExtensions -contains $extension) {
            if (-not $jpegByStem.ContainsKey($stem) -or $file.Length -lt $jpegByStem[$stem].Length) {
                $jpegByStem[$stem] = $file
            }
        }
    }

    $pairs = @()
    foreach ($stem in $rawByStem.Keys) {
        if ($jpegByStem.ContainsKey($stem)) {
            $raw = $rawByStem[$stem]
            $jpeg = $jpegByStem[$stem]
            $pairs += [pscustomobject]@{
                Stem = $stem
                Raw = $raw
                Jpeg = $jpeg
                TotalLength = $raw.Length + $jpeg.Length
            }
        }
    }

    if ($pairs.Count -eq 0) {
        throw "No matching RAW/JPEG pair found under: $Directory"
    }
    return @($pairs | Sort-Object TotalLength, Stem | Select-Object -First $RealPairLimit)
}

function New-UploadCase {
    param(
        [string]$Label,
        [string]$RemoteDirectory,
        [string]$Filename,
        [byte[]]$Bytes,
        [string]$ExpectedStoredName = $Filename,
        [bool]$ExpectExactStoredFile = $true,
        [bool]$ExpectInPhotoGrid = $false
    )
    return [pscustomobject]@{
        Label = $Label
        RemoteDirectory = $RemoteDirectory
        Filename = $Filename
        Bytes = $Bytes
        ExpectedStoredName = $ExpectedStoredName
        ExpectExactStoredFile = $ExpectExactStoredFile
        ExpectInPhotoGrid = $ExpectInPhotoGrid
    }
}

function New-RealUploadCases {
    param([string]$Directory)
    $pairs = @(Find-RealRawJpegPairs $Directory)
    if ($pairs.Count -eq 0) {
        return @(
            New-UploadCase "synthetic raw" "DCIM/100VERIFY" $sampleRawName $sampleRawBytes $sampleRawName $true $true
            New-UploadCase "synthetic jpg" "DCIM/100VERIFY" $sampleJpegName $sampleJpegBytes $sampleJpegName $true $true
        )
    }

    $cases = @()
    foreach ($pair in $pairs) {
        $remoteDirectory = "DCIM/REALPAIR/$($pair.Stem)"
        $cases += New-UploadCase "real pair raw $($pair.Stem)" $remoteDirectory $pair.Raw.Name ([System.IO.File]::ReadAllBytes($pair.Raw.FullName)) $pair.Raw.Name $true $true
        $cases += New-UploadCase "real pair jpg $($pair.Stem)" $remoteDirectory $pair.Jpeg.Name ([System.IO.File]::ReadAllBytes($pair.Jpeg.FullName)) $pair.Jpeg.Name $true $true
    }

    $firstPair = $pairs[0]
    $cases += New-UploadCase "jpg only boundary" "DCIM/EDGE/JPG_ONLY" "EDGE_JPG_ONLY.JPG" ([System.IO.File]::ReadAllBytes($firstPair.Jpeg.FullName)) "EDGE_JPG_ONLY.JPG" $true $true
    $cases += New-UploadCase "raw only boundary" "DCIM/EDGE/RAW_ONLY" "EDGE_RAW_ONLY.NEF" ([System.IO.File]::ReadAllBytes($firstPair.Raw.FullName)) "EDGE_RAW_ONLY.NEF" $true $true
    $duplicateBytes = [System.IO.File]::ReadAllBytes($firstPair.Jpeg.FullName)
    $cases += New-UploadCase "duplicate first boundary" "DCIM/EDGE/DUPLICATE_A" "EDGE_DUPLICATE.JPG" $duplicateBytes "EDGE_DUPLICATE.JPG" $true $true
    $cases += New-UploadCase "duplicate second boundary" "DCIM/EDGE/DUPLICATE_B" "EDGE_DUPLICATE.JPG" $duplicateBytes "EDGE_DUPLICATE.JPG" $false $true
    $cases += New-UploadCase "non image boundary" "DCIM/EDGE/NOT_IMAGE" "EDGE_NOT_IMAGE.TXT" ([System.Text.Encoding]::UTF8.GetBytes("not an image fixture for Camera Connector transfer verification`n")) "EDGE_NOT_IMAGE.TXT" $true $false
    return $cases
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

function Send-FtpCommandAny {
    param(
        [System.IO.StreamWriter]$Writer,
        [System.IO.StreamReader]$Reader,
        [string]$Command,
        [string[]]$Prefixes
    )
    $Writer.WriteLine($Command)
    $reply = Read-FtpReply $Reader
    foreach ($prefix in $Prefixes) {
        if ($reply.StartsWith($prefix)) {
            return $reply
        }
    }
    throw "FTP command '$Command' expected one of $($Prefixes -join ', '), got '$reply'"
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
    $startText = U @(0x5F00,0x59CB,0x63A5,0x6536)
    $startShortText = U @(0x542F,0x52A8)
    $runningText = U @(0x8FD0,0x884C,0x4E2D)
    $stopText = U @(0x505C,0x6B62)

    for ($attempt = 1; $attempt -le 20; $attempt++) {
        if (Test-FtpGreeting) {
            return
        }

        $xml = ""
        try {
            $xml = Get-UiXml
        } catch {
            Start-Sleep -Seconds 1
            continue
        }

        $xml = Enter-ProjectWorkspaceIfNeeded $xml

        if ($xml.Contains($startText) -or $xml.Contains($startShortText)) {
            Invoke-Adb @("shell", "input", "tap", "540", "1200") | Out-Null
        } elseif ($xml.Contains($runningText) -or $xml.Contains($stopText)) {
            Start-Sleep -Seconds 1
        }

        Start-Sleep -Seconds 2
    }
    throw "Android receiver did not expose an FTP greeting after starting from UI."
}

function Get-AndroidFileText {
    param([string]$Path)
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try {
        return (& $adb -s $Serial shell run-as $packageName cat $Path 2>$null) -join "`n"
    } finally {
        $ErrorActionPreference = $oldErrorActionPreference
    }
}

function Wait-AndroidTransferLog {
    param([object[]]$Cases)
    for ($attempt = 1; $attempt -le 90; $attempt++) {
        $log = Get-AndroidFileText "files/state/transfer-log.jsonl"
        $allCasesLogged = $true
        foreach ($case in $Cases) {
            if ($log -notmatch [regex]::Escape($case.Filename)) {
                $allCasesLogged = $false
                break
            }
        }
        if ($allCasesLogged) {
            return $log
        }
        Start-Sleep -Seconds 1
    }
    throw "Android transfer log did not include all uploaded cases within timeout."
}

function Get-UiXml {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $localDumpPath) | Out-Null
    $lastDumpOutput = ""
    $lastDumpExitCode = $null
    $lastPullExitCode = $null
    for ($attempt = 1; $attempt -le 12; $attempt++) {
        if ($attempt -in @(4, 8)) {
            Invoke-Adb @("shell", "input", "keyevent", "KEYCODE_WAKEUP") | Out-Null
            Invoke-Adb @("shell", "wm", "dismiss-keyguard") | Out-Null
        }
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
        $lastDumpOutput = $dumpOutput -join "`n"
        $lastDumpExitCode = $dumpExitCode
        if ($dumpExitCode -ne 0 -or (($dumpOutput -join "`n") -notmatch "dumped to")) {
            Start-Sleep -Milliseconds 750
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
        $lastPullExitCode = $pullExitCode
        if ($pullExitCode -ne 0 -or -not (Test-Path -LiteralPath $localDumpPath -PathType Leaf)) {
            Start-Sleep -Milliseconds 750
            continue
        }
        $xml = [System.IO.File]::ReadAllText($localDumpPath, [System.Text.Encoding]::UTF8)
        if ($xml.Contains("<hierarchy")) {
            return $xml
        }
        Start-Sleep -Milliseconds 750
    }
    throw "Unable to dump Android UI hierarchy. lastDumpExit=$lastDumpExitCode lastPullExit=$lastPullExitCode lastDumpOutput=$lastDumpOutput"
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

function Swipe-UntilUiContains {
    param([string]$Needle, [string]$Label, [int]$Attempts = 6)
    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        $xml = Get-UiXml
        if ($xml.Contains($Needle)) {
            return $xml
        }
        Invoke-Adb @("shell", "input", "swipe", "540", "1900", "540", "900", "400") | Out-Null
        Start-Sleep -Milliseconds 700
    }

    $xml = Get-UiXml
    if ($xml.Contains($Needle)) {
        return $xml
    }
    throw "Expected UI to contain '$Label' after swiping."
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
    Invoke-Adb @("shell", "input", "tap", "$([int](($left + $right) / 2))", "$([int](($top + $bottom) / 2))") | Out-Null
    Start-Sleep -Milliseconds 900
}

function Enter-ProjectWorkspaceIfNeeded {
    param([string]$Xml)
    $projectManagementText = U @(0x9879,0x76EE,0x7BA1,0x7406)
    $projectWorkspaceText = U @(0x9879,0x76EE,0x5DE5,0x4F5C,0x53F0)
    $enterText = U @(0x8FDB,0x5165)
    $selectText = U @(0x9009,0x62E9)
    if ($Xml.Contains($projectWorkspaceText)) {
        return $Xml
    }
    if (-not $Xml.Contains($projectManagementText)) {
        return $Xml
    }
    if ($Xml.Contains($enterText)) {
        Tap-UiNodeByText $Xml $enterText "enter selected project"
    } elseif ($Xml.Contains($selectText)) {
        Tap-UiNodeByText $Xml $selectText "select project"
    } else {
        throw "Project management screen did not expose an enter/select action."
    }
    for ($attempt = 1; $attempt -le 8; $attempt++) {
        $nextXml = Get-UiXml
        if ($nextXml.Contains($projectWorkspaceText)) {
            return $nextXml
        }
        Start-Sleep -Milliseconds 700
    }
    throw "Project workspace did not open from project management."
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

$uploadCases = @(New-RealUploadCases $RealAssetDirectory)
$photoGridCase = @($uploadCases | Where-Object { $_.Label -like "real pair jpg*" } | Select-Object -Last 1)[0]
if ($null -eq $photoGridCase) {
    $photoGridCase = @($uploadCases | Where-Object { $_.ExpectInPhotoGrid -and $_.Filename.ToUpperInvariant().EndsWith(".JPG") } | Select-Object -First 1)[0]
}
if ($null -eq $photoGridCase) {
    throw "No photo-grid JPEG upload case was prepared."
}
$sampleJpegName = $photoGridCase.ExpectedStoredName
$sampleStem = [System.IO.Path]::GetFileNameWithoutExtension($photoGridCase.Filename)
$sampleRawCase = @(
    $uploadCases |
        Where-Object {
            $_.ExpectInPhotoGrid `
                -and ($rawExtensions -contains ([System.IO.Path]::GetExtension($_.Filename).ToUpperInvariant())) `
                -and ([System.IO.Path]::GetFileNameWithoutExtension($_.Filename) -eq $sampleStem)
        } |
        Select-Object -First 1
)[0]
$sampleRawName = if ($sampleRawCase) { $sampleRawCase.ExpectedStoredName } else { $sampleRawName }
if ($RealAssetDirectory) {
    Write-Host "Prepared $($uploadCases.Count) real upload cases from $RealAssetDirectory"
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
Invoke-Adb @("shell", "run-as", $packageName, "rm", "-f", "shared_prefs/camera_connector_storage.xml") | Out-Null
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
    $userReply = Send-FtpCommandAny $writer $reader "USER $Username" @("331", "230")
    if ($userReply.StartsWith("331")) {
        Send-FtpCommand $writer $reader "PASS $Password" "230" | Out-Null
    }
    Send-FtpCommand $writer $reader "TYPE I" "200" | Out-Null
    $uploadIndex = 0
    foreach ($case in $uploadCases) {
        $uploadIndex += 1
        Write-Host ("Uploading [{0}/{1}] {2}: {3} ({4:n0} bytes)" -f $uploadIndex, $uploadCases.Count, $case.Label, $case.Filename, $case.Bytes.Length)
        Send-FtpCommand $writer $reader "CWD /$($case.RemoteDirectory)" "250" | Out-Null
        Send-FtpFile $writer $reader $case.Filename $case.Bytes
    }
    Send-FtpCommand $writer $reader "QUIT" "221" | Out-Null
} finally {
    $control.Dispose()
    Remove-AdbForward $controlForward
}

$transferLog = Wait-AndroidTransferLog $uploadCases
$transferRecords = @(
    $transferLog -split "`n" |
        Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
        ForEach-Object { $_ | ConvertFrom-Json }
)
foreach ($case in $uploadCases) {
    $matchingRecords = @(
        $transferRecords |
            Where-Object {
                $_.original_path -like "*/$($case.Filename)" -and
                    [int64]$_.size_bytes -eq [int64]$case.Bytes.Length
            }
    )
    if ($matchingRecords.Count -eq 0) {
        throw "Android transfer log did not include uploaded case with expected size: $($case.Label)"
    }
    if ($transferLog -notmatch [regex]::Escape($case.Filename)) {
        throw "Android transfer log did not include uploaded case: $($case.Label)"
    }
}
if (([regex]::Matches($transferLog, "EDGE_DUPLICATE\.JPG")).Count -lt 2 -and ($uploadCases | Where-Object { $_.Filename -eq "EDGE_DUPLICATE.JPG" }).Count -gt 0) {
    throw "Duplicate upload boundary did not leave at least two EDGE_DUPLICATE JPG transfer records."
}

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
$xml = Enter-ProjectWorkspaceIfNeeded (Get-UiXml)
$inboxUi = Tap-UntilUiContains 250 680 $sampleJpegName "uploaded asset in project photos"
Assert-UiContains $inboxUi "RAW" "raw pair tag"
Assert-UiContains $inboxUi "JPG" "jpeg pair tag"
Assert-UiContains $inboxUi (U @(0x5168,0x90E8,0x6765,0x6E90)) "source filter"
Assert-UiNotContains $inboxUi "EDGE_NOT_IMAGE.TXT" "non-image file in photo inbox"
Assert-UiContains $inboxUi (U @(0x7167,0x7247,0x0032,0x5217,0x89C6,0x56FE)) "2-column grid control"
Tap-UiNodeByContentDescription $inboxUi (U @(0x7167,0x7247,0x0032,0x5217,0x89C6,0x56FE)) "2-column grid control"
$gridPrefs = Get-AndroidFileText "shared_prefs/camera_connector_storage.xml"
if ($gridPrefs -notmatch 'name="project_photo_grid_columns"\s+value="2"') {
    throw "Android project photo grid preference did not persist 2-column selection."
}
Invoke-Adb @("shell", "am", "force-stop", $packageName) | Out-Null
Invoke-Adb @("shell", "am", "start", "-n", "$packageName/.MainActivity") | Out-Null
Start-Sleep -Seconds 2
$xml = Enter-ProjectWorkspaceIfNeeded (Get-UiXml)
$inboxUi = Tap-UntilUiContains 250 680 $sampleJpegName "uploaded asset in project photos after restart"
Assert-UiContains $inboxUi (U @(0x7167,0x7247,0x0032,0x5217,0x89C6,0x56FE)) "persisted 2-column grid control after restart"
Tap-UiNodeByContentDescription $inboxUi "$(U @(0x7167,0x7247,0x0020))$sampleJpegName" "uploaded photo tile"
$detailUi = Get-UiXml
Assert-UiContains $detailUi (U @(0x7167,0x7247,0x8BE6,0x60C5)) "photo detail screen"
$detailSourceUi = Swipe-UntilUiContains (U @(0x6765,0x6E90,0x4FE1,0x606F)) "photo source information"
Assert-UiContains $detailSourceUi $sampleJpegName "photo detail jpeg file"
$detailFilesUi = Swipe-UntilUiContains $sampleRawName "photo detail raw file"
Assert-UiContains $detailFilesUi $sampleRawName "photo detail raw file"
Invoke-Adb @("shell", "input", "keyevent", "4") | Out-Null
Start-Sleep -Milliseconds 700
$projectUi = Get-UiXml
Tap-UiNodeByText $projectUi (U @(0x8BCA,0x65AD)) "global diagnostics destination"
Start-Sleep -Milliseconds 900
$transferUi = Get-UiXml
Assert-UiContains $transferUi (U @(0x8BCA,0x65AD,0x65E5,0x5FD7)) "diagnostics transfer surface"
Assert-UiContains $transferUi (U @(0x5DF2,0x5B8C,0x6210)) "completed transfer status"
if (($uploadCases | Where-Object { $_.Filename -eq "EDGE_DUPLICATE.JPG" }).Count -gt 0) {
    Assert-UiContains $transferUi "EDGE_DUPLICATE" "visible duplicate transfer row"
}
Assert-UiContains $transferUi $DeviceName "transfer device name"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir (Join-Path $root "target\android-diagnostics\emulator-ftp-upload-latest")
if ($LASTEXITCODE -ne 0) {
    throw "Android diagnostics collection failed after FTP upload verification."
}

Write-Host "Android emulator FTP account login, connection, and RAW/JPEG upload verification passed for $sampleRawName + $sampleJpegName"
