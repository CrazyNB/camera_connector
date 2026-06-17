param(
    [string]$Serial = "emulator-5554",
    [string]$Username = "verify",
    [string]$Password = "secret",
    [string]$DeviceName = "Verify Camera",
    [int]$HostControlPort = 12121,
    [string]$RealAssetDirectory,
    [int]$RealPairLimit = 5,
    [switch]$RealImagesOnly
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$sdkRoot = if ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { Join-Path $env:LOCALAPPDATA "Android\Sdk" }
$adb = Join-Path $sdkRoot "platform-tools\adb.exe"
$packageName = "com.cameraconnector.app"
$configPath = Join-Path $root "target\android-emulator-ftp-config.json"
$seedConfigPath = Join-Path $root "target\android-emulator-ftp-seed-config.json"
$seedState = Join-Path $root "target\android-emulator-ftp-seed-state"
$cliExe = Join-Path $root "target\debug\camera-connector.exe"
$controlForward = "tcp:$HostControlPort"
$deviceControl = "tcp:2121"
$androidConfig = "/data/user/0/$packageName/files/camera-connector.json"
$androidOutput = "/data/user/0/$packageName/files/output"
$androidState = "/data/user/0/$packageName/files/state"
$sampleRawName = "VERIFY_9001.NEF"
$sampleJpegName = "VERIFY_9001.JPG"
$sampleRawBytes = [byte[]](0x4E, 0x45, 0x46, 0x21, 0x01, 0x02, 0x03, 0x04)
$sampleJpegBytes = $null
$dumpPath = "/sdcard/camera_connector_ftp_verify_window.xml"
$localDumpPath = Join-Path $root "target\android-diagnostics\emulator-ftp-upload-window.xml"
$rawExtensions = @(".NEF", ".NRW", ".CR3", ".CR2", ".ARW", ".SRF", ".SR2", ".RAF", ".RW2", ".RWL", ".ORF", ".PEF", ".DNG")
$jpegExtensions = @(".JPG", ".JPEG")
$imageExtensions = @($rawExtensions + $jpegExtensions + @(".PNG", ".HEIC", ".HEIF"))

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

function Test-AppForeground {
    $window = & $adb -s $Serial shell dumpsys window 2>$null
    $focus = $window |
        Select-String -Pattern "mCurrentFocus|mFocusedApp" |
        Select-Object -First 10
    return (($focus -join "`n").Contains($packageName))
}

function Bring-AppToForeground {
    for ($attempt = 1; $attempt -le 3; $attempt++) {
        Invoke-Adb @("shell", "am", "start", "-n", "$packageName/.MainActivity") | Out-Null
        Start-Sleep -Milliseconds 2500
        if (Test-AppForeground) {
            return
        }
        Invoke-Adb @(
            "shell",
            "monkey",
            "-p",
            $packageName,
            "-c",
            "android.intent.category.LAUNCHER",
            "1"
        ) | Out-Null
        Start-Sleep -Milliseconds 2500
        if (Test-AppForeground) {
            return
        }
    }
    $window = (& $adb -s $Serial shell dumpsys window 2>$null) -join "`n"
    throw "Unable to bring $packageName to foreground. Window state: $window"
}

function Start-App {
    Invoke-Adb @("shell", "am", "force-stop", $packageName) | Out-Null
    Bring-AppToForeground
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
    $sortedPairs = @($pairs | Sort-Object TotalLength, Stem)
    if ($RealPairLimit -gt 0) {
        return @($sortedPairs | Select-Object -First $RealPairLimit)
    }
    return $sortedPairs
}

function Find-RealImageFiles {
    param([string]$Directory)
    if ([string]::IsNullOrWhiteSpace($Directory)) {
        throw "Real asset directory is required when -RealImagesOnly is set."
    }
    if (-not (Test-Path -LiteralPath $Directory -PathType Container)) {
        throw "Real asset directory does not exist: $Directory"
    }

    $files = @(
        Get-ChildItem -LiteralPath $Directory -Recurse -File |
            Where-Object { $imageExtensions -contains $_.Extension.ToUpperInvariant() } |
            Sort-Object FullName
    )
    if ($files.Count -eq 0) {
        throw "No image files found under: $Directory"
    }
    return $files
}

function New-UploadCase {
    param(
        [string]$Label,
        [string]$RemoteDirectory,
        [string]$Filename,
        [byte[]]$Bytes = $null,
        [string]$ExpectedStoredName = $Filename,
        [bool]$ExpectExactStoredFile = $true,
        [bool]$ExpectInPhotoGrid = $false,
        [string]$SourcePath = $null
    )
    $sizeBytes = if (-not [string]::IsNullOrWhiteSpace($SourcePath)) {
        (Get-Item -LiteralPath $SourcePath).Length
    } elseif ($null -ne $Bytes) {
        $Bytes.Length
    } else {
        0
    }
    return [pscustomobject]@{
        Label = $Label
        RemoteDirectory = $RemoteDirectory
        Filename = $Filename
        Bytes = $Bytes
        SourcePath = $SourcePath
        SizeBytes = $sizeBytes
        ExpectedStoredName = $ExpectedStoredName
        ExpectExactStoredFile = $ExpectExactStoredFile
        ExpectInPhotoGrid = $ExpectInPhotoGrid
    }
}

function New-RealImageUploadCases {
    param([string]$Directory)
    $files = @(Find-RealImageFiles $Directory)
    $cases = @()
    foreach ($file in $files) {
        $remoteDirectory = "DCIM/FULLVERIFY"
        $cases += New-UploadCase `
            -Label "real image $($file.Name)" `
            -RemoteDirectory $remoteDirectory `
            -Filename $file.Name `
            -ExpectedStoredName $file.Name `
            -ExpectExactStoredFile $true `
            -ExpectInPhotoGrid $true `
            -SourcePath $file.FullName
    }
    return $cases
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
    Start-App
    $startText = U @(0x5F00,0x59CB,0x63A5,0x6536)
    $startShortText = U @(0x542F,0x52A8)
    $runningText = U @(0x63A5,0x6536,0x4E2D)
    $stopText = U @(0x505C,0x6B62)

    for ($attempt = 1; $attempt -le 20; $attempt++) {
        if (Test-FtpGreeting) {
            return
        }
        if (-not (Test-AppForeground)) {
            Bring-AppToForeground
        }

        $xml = ""
        try {
            $xml = Get-UiXml
        } catch {
            Start-Sleep -Seconds 1
            continue
        }

        $xml = Enter-ProjectWorkspaceIfNeeded $xml

        if ($xml.Contains($startShortText)) {
            Tap-UiNodeByText $xml $startShortText "start receiver"
        } elseif ($xml.Contains($startText)) {
            Tap-UiNodeByText $xml $startText "start receiving"
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

function Wait-UiContains {
    param([string]$Needle, [string]$Label, [int]$Attempts = 12)
    for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
        $xml = Get-UiXml
        if ($xml.Contains($Needle)) {
            return $xml
        }
        Start-Sleep -Milliseconds 700
    }
    throw "Expected UI to contain '$Label' within timeout."
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

function Test-UiNodeByText {
    param([string]$Xml, [string]$Text)
    $escapedText = [System.Security.SecurityElement]::Escape($Text)
    $pattern = "text=""$([regex]::Escape($escapedText))""[^>]*bounds=""\[(\d+),(\d+)\]\[(\d+),(\d+)\]"""
    return [regex]::Match($Xml, $pattern).Success
}

function Test-ProjectWorkspaceXml {
    param([string]$Xml)
    $cameraConnectIpText = "$(U @(0x76F8,0x673A,0x8FDE,0x63A5)) IP"
    $projectIntelligenceText = U @(0x9879,0x76EE,0x667A,0x80FD)
    $expandReceiverDrawerText = U @(0x5C55,0x5F00,0x63A5,0x6536,0x62BD,0x5C49)
    $allText = U @(0x5168,0x90E8)
    $favoriteText = U @(0x6536,0x85CF)
    $markedText = U @(0x6807,0x8BB0)
    $startShortText = U @(0x542F,0x52A8)
    return $Xml.Contains($cameraConnectIpText) -or
        $Xml.Contains($projectIntelligenceText) -or
        $Xml.Contains($expandReceiverDrawerText) -or (
        $Xml.Contains($startShortText) -and
        $Xml.Contains($allText) -and
        $Xml.Contains($favoriteText) -and
        $Xml.Contains($markedText)
    )
}

function Wait-ProjectWorkspace {
    param([string]$Label)
    for ($attempt = 1; $attempt -le 14; $attempt++) {
        $xml = Get-UiXml
        if (Test-ProjectWorkspaceXml $xml) {
            return $xml
        }
        Start-Sleep -Milliseconds 700
    }
    throw "Project workspace did not open after $Label."
}

function Create-And-EnterVerificationProject {
    param([string]$Xml)
    $newProjectText = U @(0x65B0,0x5EFA,0x9879,0x76EE)
    $createAndEnterText = U @(0x521B,0x5EFA,0x5E76,0x8FDB,0x5165)
    $projectName = "Real Verify"

    if (-not $Xml.Contains($createAndEnterText)) {
        Tap-UiNodeByText $Xml $newProjectText "new project"
        $Xml = Get-UiXml
    }

    $editMatch = [regex]::Match($Xml, 'class="android\.widget\.EditText"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"')
    if (-not $editMatch.Success) {
        throw "Unable to find project name input."
    }
    $editX = [int](([int]$editMatch.Groups[1].Value + [int]$editMatch.Groups[3].Value) / 2)
    $editY = [int](([int]$editMatch.Groups[2].Value + [int]$editMatch.Groups[4].Value) / 2)
    Invoke-Adb @("shell", "input", "tap", "$editX", "$editY") | Out-Null
    Start-Sleep -Milliseconds 300
    Invoke-Adb @("shell", "input", "text", ($projectName -replace " ", "%s")) | Out-Null
    Start-Sleep -Milliseconds 300
    Invoke-Adb @("shell", "input", "keyevent", "BACK") | Out-Null
    Start-Sleep -Milliseconds 500
    $Xml = Get-UiXml
    Tap-UiNodeByText $Xml $createAndEnterText "create and enter verification project"
    return Wait-ProjectWorkspace "creating verification project"
}

function Enter-ProjectWorkspaceIfNeeded {
    param([string]$Xml)
    $enterText = U @(0x8FDB,0x5165)
    $selectText = U @(0x9009,0x62E9)
    $newProjectText = U @(0x65B0,0x5EFA,0x9879,0x76EE)
    if (Test-ProjectWorkspaceXml $Xml) {
        return $Xml
    }
    if (-not (Test-UiNodeByText $Xml $enterText) -and -not (Test-UiNodeByText $Xml $selectText) -and -not $Xml.Contains($newProjectText)) {
        return $Xml
    }
    if (Test-UiNodeByText $Xml $enterText) {
        Tap-UiNodeByText $Xml $enterText "enter selected project"
    } elseif (Test-UiNodeByText $Xml $selectText) {
        Tap-UiNodeByText $Xml $selectText "select project"
    } else {
        return Create-And-EnterVerificationProject $Xml
    }
    return Wait-ProjectWorkspace "project management action"
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

function Collapse-ReceiverLauncherIfExpanded {
    param([string]$Xml)
    $collapseText = U @(0x6536,0x8D77,0x542F,0x52A8,0x9875)
    if ($Xml.Contains($collapseText)) {
        Tap-UiNodeByContentDescription $Xml $collapseText "collapse receiver launcher"
        return Get-UiXml
    }
    return $Xml
}

function Tap-UiNodeByContentDescriptionUntilUiContains {
    param([string]$Xml, [string]$Needle, [string]$Label, [string]$ExpectedNeedle, [string]$ExpectedLabel)
    $currentXml = $Xml
    for ($attempt = 1; $attempt -le 4; $attempt++) {
        Tap-UiNodeByContentDescription $currentXml $Needle $Label
        for ($wait = 1; $wait -le 5; $wait++) {
            $nextXml = Get-UiXml
            if ($nextXml.Contains($ExpectedNeedle)) {
                return $nextXml
            }
            $currentXml = $nextXml
            Start-Sleep -Milliseconds 500
        }
    }
    throw "Expected UI to contain '$ExpectedLabel' after tapping '$Label'."
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

function Send-FtpUploadCase {
    param(
        [System.IO.StreamWriter]$Writer,
        [System.IO.StreamReader]$Reader,
        [object]$UploadCase
    )
    if (-not [string]::IsNullOrWhiteSpace($UploadCase.SourcePath)) {
        $epsv = Send-FtpCommand $Writer $Reader "EPSV" "229"
        if ($epsv -notmatch "\(\|\|\|(\d+)\|\)") {
            throw "EPSV did not include a passive port: $epsv"
        }
        $deviceDataPort = [int]$Matches[1]
        $hostDataPort = $deviceDataPort
        $dataForward = "tcp:$hostDataPort"
        Remove-AdbForward $dataForward
        Invoke-Adb @("forward", $dataForward, "tcp:$deviceDataPort") | Out-Null

        $data = [System.Net.Sockets.TcpClient]::new("127.0.0.1", $hostDataPort)
        $fileStream = $null
        try {
            $Writer.WriteLine("STOR $($UploadCase.Filename)")
            $stor = Read-FtpReply $Reader
            if (-not $stor.StartsWith("150")) { throw "STOR expected 150, got '$stor'" }
            $fileStream = [System.IO.File]::OpenRead($UploadCase.SourcePath)
            $fileStream.CopyTo($data.GetStream())
            $data.GetStream().Close()
            $complete = Read-FtpReply $Reader
            if (-not $complete.StartsWith("226")) { throw "STOR expected 226, got '$complete'" }
        } finally {
            if ($fileStream) { $fileStream.Dispose() }
            $data.Dispose()
            Remove-AdbForward $dataForward
        }
        return
    }

    Send-FtpFile $Writer $Reader $UploadCase.Filename $UploadCase.Bytes
}

$uploadCases = if ($RealImagesOnly) {
    @(New-RealImageUploadCases $RealAssetDirectory)
} else {
    @(New-RealUploadCases $RealAssetDirectory)
}
$photoGridCase = @($uploadCases | Where-Object { $_.Label -like "real pair jpg*" } | Select-Object -First 1)[0]
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

if (Test-Path -LiteralPath $seedState) {
    Remove-Item -LiteralPath $seedState -Recurse -Force
}
if (Test-Path -LiteralPath $seedConfigPath) {
    Remove-Item -LiteralPath $seedConfigPath -Force
}
if (Test-Path -LiteralPath $configPath) {
    Remove-Item -LiteralPath $configPath -Force
}

& cargo build -p camera-connector-cli
if ($LASTEXITCODE -ne 0) { throw "Failed to build current camera-connector CLI." }
if (-not (Test-Path -LiteralPath $cliExe -PathType Leaf)) {
    throw "camera-connector CLI was not built: $cliExe"
}

& $cliExe receiver-settings `
    --config $seedConfigPath `
    --protocol ftp `
    --bind-host "0.0.0.0" `
    --ftp-port 2121 `
    --sftp-port 2222 `
    --output $androidOutput `
    --state $seedState `
    --advertised-host "127.0.0.1" `
    --source-name $DeviceName | Out-Host
if ($LASTEXITCODE -ne 0) { throw "Failed to create verifier seed receiver config." }

& $cliExe account --config $seedConfigPath set --username $Username --password $Password --device-name $DeviceName | Out-Host
if ($LASTEXITCODE -ne 0) { throw "Failed to create verifier account state." }

& $cliExe project --config $seedConfigPath create --name "Real Verify" | Out-Host
if ($LASTEXITCODE -ne 0) { throw "Failed to create verifier project state." }

$seedDatabase = Join-Path $seedState "camera-connector.sqlite3"
if (-not (Test-Path -LiteralPath $seedDatabase -PathType Leaf)) {
    throw "Verifier seed database was not created: $seedDatabase"
}

& $cliExe receiver-settings `
    --config $configPath `
    --protocol ftp `
    --bind-host "0.0.0.0" `
    --ftp-port 2121 `
    --sftp-port 2222 `
    --output $androidOutput `
    --state $androidState `
    --advertised-host "127.0.0.1" `
    --source-name $DeviceName | Out-Host
if ($LASTEXITCODE -ne 0) { throw "Failed to create Android receiver config." }

$configRaw = Get-Content -Raw -LiteralPath $configPath
if ($configRaw -like "*$Password*") {
    throw "Verifier config leaked plaintext password."
}

Invoke-Adb @("shell", "am", "force-stop", $packageName) | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "rm", "-rf", "files/output", "files/state") | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "rm", "-f", "shared_prefs/camera_connector_storage.xml") | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "mkdir", "-p", "files/output", "files/state") | Out-Null
Invoke-Adb @("push", $configPath, "/data/local/tmp/camera-connector.json") | Out-Null
Invoke-Adb @("shell", "chmod", "644", "/data/local/tmp/camera-connector.json") | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "cp", "/data/local/tmp/camera-connector.json", "files/camera-connector.json") | Out-Null
Invoke-Adb @("push", $seedDatabase, "/data/local/tmp/camera-connector.sqlite3") | Out-Null
Invoke-Adb @("shell", "chmod", "644", "/data/local/tmp/camera-connector.sqlite3") | Out-Null
Invoke-Adb @("shell", "run-as", $packageName, "cp", "/data/local/tmp/camera-connector.sqlite3", "files/state/camera-connector.sqlite3") | Out-Null

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
        Write-Host ("Uploading [{0}/{1}] {2}: {3} ({4:n0} bytes)" -f $uploadIndex, $uploadCases.Count, $case.Label, $case.Filename, $case.SizeBytes)
        Send-FtpCommand $writer $reader "CWD /$($case.RemoteDirectory)" "250" | Out-Null
        Send-FtpUploadCase $writer $reader $case
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
                    [int64]$_.size_bytes -eq [int64]$case.SizeBytes
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

if (($sampleRawCase -and $transferLog -notmatch $sampleRawName) -or $transferLog -notmatch $sampleJpegName) {
    throw "Android transfer log did not include uploaded raw/jpeg pair."
}
if ($transferLog -notmatch $Username -or $transferLog -notmatch $DeviceName) {
    throw "Android transfer log did not include account identity."
}

$configAfter = Get-AndroidFileText "files/camera-connector.json"
if ($configAfter -like "*$Password*") {
    throw "Android config leaked plaintext password."
}

if ($RealImagesOnly) {
    $settleSeconds = [Math]::Min(90, [Math]::Max(30, [int][Math]::Ceiling($uploadCases.Count / 4.0)))
    Write-Host "Waiting $settleSeconds seconds for full-image analysis to settle before UI restart."
    Start-Sleep -Seconds $settleSeconds
}

Invoke-Adb @("shell", "am", "start", "-S", "-n", "$packageName/.MainActivity") | Out-Null
Start-Sleep -Seconds 3
$xml = Collapse-ReceiverLauncherIfExpanded (Enter-ProjectWorkspaceIfNeeded (Get-UiXml))
$photoSwipeAttempts = if ($RealImagesOnly) { [Math]::Max(12, $uploadCases.Count + 8) } else { 6 }
if ($RealImagesOnly) {
    $assetGridUi = Swipe-UntilUiContains "DSC_" "uploaded image group in project photos" $photoSwipeAttempts
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir (Join-Path $root "target\android-diagnostics\emulator-ftp-upload-latest")
    if ($LASTEXITCODE -ne 0) {
        throw "Android diagnostics collection failed after FTP upload verification."
    }
    Write-Host "Android emulator FTP full-image upload verification passed for $($uploadCases.Count) real image files"
    return
}
$assetGridUi = Swipe-UntilUiContains $sampleJpegName "uploaded asset in project photos" $photoSwipeAttempts
Assert-UiContains $assetGridUi "RAW" "raw pair tag"
Assert-UiContains $assetGridUi "JPG" "jpeg pair tag"
if (($uploadCases | Where-Object { $_.Filename -eq "EDGE_NOT_IMAGE.TXT" }).Count -gt 0) {
    Assert-UiNotContains $assetGridUi "EDGE_NOT_IMAGE.TXT" "non-image file in photo grid"
}
Invoke-Adb @("shell", "am", "force-stop", $packageName) | Out-Null
Invoke-Adb @("shell", "am", "start", "-n", "$packageName/.MainActivity") | Out-Null
Start-Sleep -Seconds 2
$xml = Collapse-ReceiverLauncherIfExpanded (Enter-ProjectWorkspaceIfNeeded (Get-UiXml))
$assetGridUi = Swipe-UntilUiContains $sampleJpegName "uploaded asset in project photos after restart" $photoSwipeAttempts
$photoDetailText = U @(0x7167,0x7247,0x8BE6,0x60C5)
$detailUi = Tap-UiNodeByContentDescriptionUntilUiContains $assetGridUi "$(U @(0x7167,0x7247,0x0020))$sampleJpegName" "uploaded photo tile" $photoDetailText "photo detail screen"
Assert-UiContains $detailUi (U @(0x7167,0x7247,0x8BE6,0x60C5)) "photo detail screen"
$detailSourceUi = Swipe-UntilUiContains (U @(0x6765,0x6E90,0x4FE1,0x606F)) "photo source information"
Assert-UiContains $detailSourceUi $sampleJpegName "photo detail jpeg file"
if ($sampleRawCase) {
    $detailFilesUi = Swipe-UntilUiContains $sampleRawName "photo detail raw file"
    Assert-UiContains $detailFilesUi $sampleRawName "photo detail raw file"
}
Invoke-Adb @("shell", "input", "keyevent", "4") | Out-Null
Start-Sleep -Milliseconds 700
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "collect_android_diagnostics.ps1") -Serial $Serial -OutputDir (Join-Path $root "target\android-diagnostics\emulator-ftp-upload-latest")
if ($LASTEXITCODE -ne 0) {
    throw "Android diagnostics collection failed after FTP upload verification."
}

Write-Host "Android emulator FTP account login, connection, and RAW/JPEG upload verification passed for $sampleRawName + $sampleJpegName"
