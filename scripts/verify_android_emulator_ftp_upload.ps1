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

. (Join-Path $PSScriptRoot "verify_android_emulator_ftp_upload_helpers.ps1")

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
