$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$vsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$pushInput = Join-Path $root "target\push-input"
$pushOutput = Join-Path $root "target\push-output"
$pushState = Join-Path $root "target\push-state"
$ftpSmokeOutput = Join-Path $root "target\ftp-smoke-output"
$ftpSmokeState = Join-Path $root "target\ftp-smoke-state"
$sftpSmokeOutput = Join-Path $root "target\sftp-smoke-output"
$sftpSmokeState = Join-Path $root "target\sftp-smoke-state"
$configPath = Join-Path $root "target\push-config.json"

function Invoke-CargoDev {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Command
    )

    cmd /c "call `"$vsDevCmd`" -arch=x64 && set PATH=$cargoBin;%PATH% && cd /d `"$root`" && $Command"
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed: $Command"
    }
}

Invoke-CargoDev "cargo fmt --all -- --check"
Invoke-CargoDev "cargo clippy --workspace -- -D warnings"
Invoke-CargoDev "cargo test --workspace"
Invoke-CargoDev "cargo build --workspace"

& (Join-Path $root "scripts\verify_android_skeleton.ps1")
if ($LASTEXITCODE -ne 0) {
    throw "Android skeleton verification failed"
}
& (Join-Path $root "scripts\verify_mobile_ffi_contract.ps1")
if ($LASTEXITCODE -ne 0) {
    throw "Mobile FFI contract verification failed"
}

if (Test-Path $pushInput) {
    Remove-Item -LiteralPath $pushInput -Recurse -Force
}
if (Test-Path $pushOutput) {
    Remove-Item -LiteralPath $pushOutput -Recurse -Force
}
if (Test-Path $pushState) {
    Remove-Item -LiteralPath $pushState -Recurse -Force
}
if (Test-Path $ftpSmokeOutput) {
    Remove-Item -LiteralPath $ftpSmokeOutput -Recurse -Force
}
if (Test-Path $ftpSmokeState) {
    Remove-Item -LiteralPath $ftpSmokeState -Recurse -Force
}
if (Test-Path $sftpSmokeOutput) {
    Remove-Item -LiteralPath $sftpSmokeOutput -Recurse -Force
}
if (Test-Path $sftpSmokeState) {
    Remove-Item -LiteralPath $sftpSmokeState -Recurse -Force
}
if (Test-Path $configPath) {
    Remove-Item -LiteralPath $configPath -Force
}
New-Item -ItemType Directory -Force -Path $pushInput | Out-Null
New-Item -ItemType Directory -Force -Path $pushOutput | Out-Null
New-Item -ItemType Directory -Force -Path $pushState | Out-Null
New-Item -ItemType Directory -Force -Path $ftpSmokeOutput | Out-Null
New-Item -ItemType Directory -Force -Path $ftpSmokeState | Out-Null
New-Item -ItemType Directory -Force -Path $sftpSmokeOutput | Out-Null
New-Item -ItemType Directory -Force -Path $sftpSmokeState | Out-Null

$sample = Join-Path $pushInput "IMG_1234.CR3"
[System.IO.File]::WriteAllBytes($sample, [byte[]](1, 2, 3, 4, 5))

& (Join-Path $root "target\debug\camera-connector.exe") account --config $configPath set --username "verify" --password "secret" --device-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "account set smoke failed" }
$configRaw = Get-Content -Raw -LiteralPath $configPath
if ($configRaw -like "*secret*") { throw "account config leaked plaintext password" }
if ($configRaw -notlike "*password_hash*") { throw "account config did not store a password hash" }
$configJson = $configRaw | ConvertFrom-Json
if ($configJson.receiver.protocol -ne "Ftp" -or $configJson.receiver.ftp_port -ne 2121 -or $configJson.receiver.sftp_port -ne 2222) {
    throw "receiver settings were not persisted in config"
}

$accountList = & (Join-Path $root "target\debug\camera-connector.exe") account --config $configPath list
if ($LASTEXITCODE -ne 0) { throw "account list smoke failed" }
if (($accountList | Where-Object { $_ -like "*verify*Verify Camera*" }).Count -lt 1) {
    throw "account list did not include Verify Camera"
}
Write-Output $accountList

$receiverSettings = & (Join-Path $root "target\debug\camera-connector.exe") receiver-settings --config $configPath --protocol ftp --bind-host "0.0.0.0" --ftp-port 2121 --sftp-port 2222 --output $pushOutput --state $pushState --advertised-host "192.168.137.1" --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "receiver-settings smoke failed" }
if (($receiverSettings | Where-Object { $_ -like "config: $configPath*" }).Count -lt 1) {
    throw "receiver-settings did not report config path"
}
Write-Output $receiverSettings
$configJson = Get-Content -Raw -LiteralPath $configPath | ConvertFrom-Json
if ($configJson.receiver.output_dir -ne $pushOutput -or $configJson.receiver.state_dir -ne $pushState -or $configJson.receiver.source_name -ne "Verify Camera") {
    throw "receiver-settings did not persist updated receiver paths"
}

$projectCreate = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath create --name "Verify Shoot"
if ($LASTEXITCODE -ne 0) { throw "project create smoke failed" }
$projectCreateLine = @($projectCreate | Where-Object { $_ -like "project*name=Verify Shoot*active=true*" })
if ($projectCreateLine.Count -lt 1) {
    throw "project create did not create an active project"
}
if ($projectCreateLine[0] -notmatch "id=([^\t]+)") {
    throw "project create did not print a project id"
}
$projectId = $Matches[1]
Write-Output $projectCreate

$projectActive = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath active
if ($LASTEXITCODE -ne 0) { throw "project active smoke failed" }
if (($projectActive | Where-Object { $_ -like "project*id=$projectId*name=Verify Shoot*active=true*" }).Count -lt 1) {
    throw "project active did not return the created project"
}
Write-Output $projectActive

$projectList = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath list
if ($LASTEXITCODE -ne 0) { throw "project list smoke failed" }
if (($projectList | Where-Object { $_ -like "project*id=$projectId*name=Verify Shoot*active=true*" }).Count -lt 1) {
    throw "project list did not include the active project"
}
Write-Output $projectList

$projectArchive = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath archive --id $projectId
if ($LASTEXITCODE -ne 0) { throw "project archive smoke failed" }
if (($projectArchive | Where-Object { $_ -like "project*id=$projectId*status=archived*active=false*" }).Count -lt 1) {
    throw "project archive did not mark project archived"
}
Write-Output $projectArchive

$projectActiveAfterArchive = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath list
if ($LASTEXITCODE -ne 0) { throw "project list after archive smoke failed" }
if (($projectActiveAfterArchive | Where-Object { $_ -like "project*id=$projectId*status=archived*active=true*" }).Count -gt 0) {
    throw "project archive should clear active project selection"
}
Write-Output $projectActiveAfterArchive

$projectRestore = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath restore --id $projectId
if ($LASTEXITCODE -ne 0) { throw "project restore smoke failed" }
if (($projectRestore | Where-Object { $_ -like "project*id=$projectId*status=active*" }).Count -lt 1) {
    throw "project restore did not reactivate project status"
}
Write-Output $projectRestore

$projectSelect = & (Join-Path $root "target\debug\camera-connector.exe") project --config $configPath select --id $projectId
if ($LASTEXITCODE -ne 0) { throw "project select smoke failed" }
if (($projectSelect | Where-Object { $_ -like "project*id=$projectId*active=true*" }).Count -lt 1) {
    throw "project select did not mark the project active"
}
Write-Output $projectSelect

& (Join-Path $root "target\debug\camera-connector.exe") receiver-config --config $configPath --protocol ftp --output $pushOutput --state $pushState
if ($LASTEXITCODE -ne 0) { throw "receiver-config smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") receiver-config --config $configPath --protocol sftp --port 2222 --output $sftpSmokeOutput --state $sftpSmokeState
if ($LASTEXITCODE -ne 0) { throw "receiver-config sftp smoke failed" }

$serverOut = Join-Path $root "target\ftp-smoke.out.log"
$serverErr = Join-Path $root "target\ftp-smoke.err.log"
$server = Start-Process -FilePath (Join-Path $root "target\debug\camera-connector.exe") `
    -ArgumentList @("serve-ftp", "--config", $configPath, "--bind-host", "127.0.0.1", "--port", "2221", "--output", $ftpSmokeOutput, "--state", $ftpSmokeState) `
    -WindowStyle Hidden `
    -PassThru `
    -RedirectStandardOutput $serverOut `
    -RedirectStandardError $serverErr
try {
    $ready = $false
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 200
        if (Test-NetConnection -ComputerName 127.0.0.1 -Port 2221 -InformationLevel Quiet) {
            $ready = $true
            break
        }
    }
    if (!$ready) {
        throw "FTP smoke server did not start"
    }

    function Read-FtpReply {
        param([System.IO.StreamReader] $Reader)
        $line = $Reader.ReadLine()
        if ($null -eq $line) { throw "FTP server closed the control connection" }
        return $line
    }

    function Send-FtpCommand {
        param(
            [System.IO.StreamWriter] $Writer,
            [System.IO.StreamReader] $Reader,
            [string] $Command,
            [string] $Prefix
        )
        $Writer.WriteLine($Command)
        $reply = Read-FtpReply $Reader
        if (!$reply.StartsWith($Prefix)) {
            throw "FTP command '$Command' expected $Prefix, got '$reply'"
        }
        return $reply
    }

    $control = [System.Net.Sockets.TcpClient]::new("127.0.0.1", 2221)
    try {
        $stream = $control.GetStream()
        $reader = [System.IO.StreamReader]::new($stream, [System.Text.Encoding]::ASCII)
        $writer = [System.IO.StreamWriter]::new($stream, [System.Text.Encoding]::ASCII)
        $writer.NewLine = "`r`n"
        $writer.AutoFlush = $true

        $hello = Read-FtpReply $reader
        if (!$hello.StartsWith("220")) { throw "FTP greeting failed: $hello" }
        Send-FtpCommand $writer $reader "USER verify" "331" | Out-Null
        Send-FtpCommand $writer $reader "PASS secret" "230" | Out-Null
        Send-FtpCommand $writer $reader "TYPE I" "200" | Out-Null
        $pasv = Send-FtpCommand $writer $reader "PASV" "227"
        if ($pasv -notmatch "\((\d+),(\d+),(\d+),(\d+),(\d+),(\d+)\)") {
            throw "PASV reply was not parseable: $pasv"
        }
        $dataPort = ([int]$Matches[5] * 256) + [int]$Matches[6]
        $data = [System.Net.Sockets.TcpClient]::new("127.0.0.1", $dataPort)
        try {
            Send-FtpCommand $writer $reader "CWD DCIM" "250" | Out-Null
            $writer.WriteLine("STOR IMG_5678.NEF")
            $stor = Read-FtpReply $reader
            if (!$stor.StartsWith("150")) { throw "STOR expected 150, got '$stor'" }
            $dataStream = $data.GetStream()
            $bytes = [byte[]](9, 8, 7, 6)
            $dataStream.Write($bytes, 0, $bytes.Length)
            $dataStream.Close()
            $complete = Read-FtpReply $reader
            if (!$complete.StartsWith("226")) { throw "STOR expected 226, got '$complete'" }
        } finally {
            $data.Dispose()
        }
        Send-FtpCommand $writer $reader "QUIT" "221" | Out-Null
    } finally {
        $control.Dispose()
    }
} finally {
    if (!$server.HasExited) {
        Stop-Process -Id $server.Id -Force
    }
}

$ftpReceived = Get-Item -LiteralPath (Join-Path $ftpSmokeOutput "IMG_5678.NEF")
if ($ftpReceived.Length -ne 4) { throw "FTP smoke upload length mismatch" }
$receiverStatusOutput = & (Join-Path $root "target\debug\camera-connector.exe") receiver-status --state $ftpSmokeState
if ($LASTEXITCODE -ne 0) { throw "receiver-status smoke failed" }
if (($receiverStatusOutput | Where-Object { $_ -like "phase: Stopped*" }).Count -lt 1) {
    throw "receiver-status did not report stopped phase after smoke shutdown"
}
Write-Output $receiverStatusOutput
$ftpTransferLog = Join-Path $ftpSmokeState "transfer-log.jsonl"
$ftpTransferRecords = @(Get-Content -LiteralPath $ftpTransferLog | ForEach-Object { $_ | ConvertFrom-Json })
if (@($ftpTransferRecords | Where-Object { $_.source_name -eq "Verify Camera" -and $_.remote_addr -eq "127.0.0.1" }).Count -ne 1) {
    throw "FTP smoke transfer log did not record account source and remote address"
}

$sftpServerOut = Join-Path $root "target\sftp-smoke.out.log"
$sftpServerErr = Join-Path $root "target\sftp-smoke.err.log"
$sftpServer = Start-Process -FilePath (Join-Path $root "target\debug\camera-connector.exe") `
    -ArgumentList @("serve-sftp", "--config", $configPath, "--bind-host", "127.0.0.1", "--port", "2222", "--output", $sftpSmokeOutput, "--state", $sftpSmokeState) `
    -WindowStyle Hidden `
    -PassThru `
    -RedirectStandardOutput $sftpServerOut `
    -RedirectStandardError $sftpServerErr
try {
    $sftpReady = $false
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 200
        if (Test-NetConnection -ComputerName 127.0.0.1 -Port 2222 -InformationLevel Quiet) {
            $sftpReady = $true
            break
        }
    }
    if (!$sftpReady) {
        throw "SFTP smoke server did not start"
    }

    $sftpRunningStatus = & (Join-Path $root "target\debug\camera-connector.exe") receiver-status --state $sftpSmokeState
    if ($LASTEXITCODE -ne 0) { throw "sftp receiver-status smoke failed" }
    if (($sftpRunningStatus | Where-Object { $_ -like "phase: Running*" }).Count -lt 1) {
        throw "sftp receiver-status did not report running phase"
    }
    if (($sftpRunningStatus | Where-Object { $_ -like "protocol: sftp*" }).Count -lt 1) {
        throw "sftp receiver-status did not report sftp protocol"
    }
} finally {
    if (!$sftpServer.HasExited) {
        Stop-Process -Id $sftpServer.Id -Force
    }
}

$sftpStoppedStatus = & (Join-Path $root "target\debug\camera-connector.exe") receiver-status --state $sftpSmokeState
if ($LASTEXITCODE -ne 0) { throw "sftp stopped receiver-status smoke failed" }
if (($sftpStoppedStatus | Where-Object { $_ -like "phase: Stopped*" }).Count -lt 1) {
    throw "sftp receiver-status did not report stopped phase after smoke shutdown"
}
Write-Output $sftpStoppedStatus

& (Join-Path $root "target\debug\camera-connector.exe") receive-file --input $sample --output $pushOutput --state $pushState --source ftp --username "verify" --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "receive-file smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") receive-file --input $sample --output $pushOutput --state $pushState --source ftp --username "verify" --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "duplicate receive-file smoke failed" }

$transferLog = Join-Path $pushState "transfer-log.jsonl"
$nowMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$failedTransfer = [ordered]@{
    transfer_id = "ftp:verify-failed"
    protocol = "ftp"
    status = "Failed"
    original_path = "IMG_9999.CR3"
    final_filename = "IMG_9999.CR3"
    final_path = $null
    final_location = $null
    size_bytes = 0
    username = "verify"
    remote_addr = "192.168.137.56"
    source_name = "Verify Camera"
    started_at_ms = $nowMs
    completed_at_ms = $nowMs
    error = "simulated failure"
}
($failedTransfer | ConvertTo-Json -Compress) | Add-Content -LiteralPath $transferLog

& (Join-Path $root "target\debug\camera-connector.exe") inbox --path $pushOutput --source ftp
if ($LASTEXITCODE -ne 0) { throw "inbox smoke failed" }

$logBackedInboxOutput = & (Join-Path $root "target\debug\camera-connector.exe") inbox --path $pushState --from-transfers --summary
if ($LASTEXITCODE -ne 0) { throw "log-backed inbox smoke failed" }
if (($logBackedInboxOutput | Where-Object { $_ -like "summary*groups=2*raw_groups=2*sources=Verify Camera:2*" }).Count -lt 1) {
    throw "log-backed inbox did not expose summary counts"
}
if (($logBackedInboxOutput | Where-Object { $_ -like "IMG_1234*primary=IMG_1234.CR3*" }).Count -lt 1) {
    throw "log-backed inbox did not group transfers"
}
if (($logBackedInboxOutput | Where-Object { $_ -like "*primary_location_kind=local_path*" }).Count -lt 1) {
    throw "log-backed inbox did not expose primary storage location"
}
if (($logBackedInboxOutput | Where-Object { $_ -like "*username=verify*source=Verify Camera*display=Verify Camera/IMG_1234.CR3*" }).Count -lt 1) {
    throw "log-backed inbox did not expose transfer metadata"
}
if (($logBackedInboxOutput | Where-Object { $_ -like "*IMG_1234 (1)*duplicate=2/2*" }).Count -lt 1) {
    throw "log-backed inbox did not expose duplicate metadata"
}
Write-Output $logBackedInboxOutput

$pagedLogBackedInboxOutput = & (Join-Path $root "target\debug\camera-connector.exe") inbox --path $pushState --from-transfers --summary --offset 1 --limit 1
if ($LASTEXITCODE -ne 0) { throw "paged log-backed inbox smoke failed" }
if (($pagedLogBackedInboxOutput | Where-Object { $_ -like "summary*offset=1*limit=1*total_groups=2*has_more=False*" }).Count -lt 1) {
    throw "paged log-backed inbox did not expose paging state"
}
if (($pagedLogBackedInboxOutput | Where-Object { $_ -like "IMG_1234*primary=IMG_1234.CR3*" }).Count -lt 1) {
    throw "paged log-backed inbox did not return expected second page"
}
Write-Output $pagedLogBackedInboxOutput

$filteredLogBackedInboxOutput = & (Join-Path $root "target\debug\camera-connector.exe") inbox --config $configPath --path $pushState --from-transfers --username "verify" --source-name "Verify Camera" --original-path IMG_1234 --format cr3
if ($LASTEXITCODE -ne 0) { throw "filtered log-backed inbox smoke failed" }
if (($filteredLogBackedInboxOutput | Where-Object { $_ -like "IMG_1234*username=verify*source=Verify Camera*" }).Count -lt 1) {
    throw "filtered log-backed inbox did not include expected group"
}
Write-Output $filteredLogBackedInboxOutput

$projectInboxOutput = & (Join-Path $root "target\debug\camera-connector.exe") inbox --config $configPath --project-id $projectId --summary --username "verify" --limit 1
if ($LASTEXITCODE -ne 0) { throw "project inbox smoke failed" }
if (($projectInboxOutput | Where-Object { $_ -like "summary*groups=2*offset=0*limit=1*total_groups=2*has_more=true*" }).Count -lt 1) {
    throw "project inbox did not expose paged project asset summary"
}
if (($projectInboxOutput | Where-Object { $_ -like "IMG_1234*username=verify*source=Verify Camera*" }).Count -lt 1) {
    throw "project inbox did not expose project asset rows"
}
Write-Output $projectInboxOutput

$projectTransfersOutput = & (Join-Path $root "target\debug\camera-connector.exe") transfers --config $configPath --project-id $projectId --status completed --username "verify"
if ($LASTEXITCODE -ne 0) { throw "project transfers smoke failed" }
if (($projectTransfersOutput | Where-Object { $_ -like "ftp:*Completed*username=verify*source=Verify Camera*" }).Count -lt 2) {
    throw "project transfers did not expose completed SQLite transfers"
}
if (($projectTransfersOutput | Where-Object { $_ -like "*ftp:verify-failed*" }).Count -gt 0) {
    throw "project transfers should not include transfer-log-only failure"
}
Write-Output $projectTransfersOutput

$projectDashboardOutput = & (Join-Path $root "target\debug\camera-connector.exe") dashboard --config $configPath --project-id $projectId --username "verify" --limit 1
if ($LASTEXITCODE -ne 0) { throw "project dashboard smoke failed" }
if (($projectDashboardOutput | Where-Object { $_ -like "paths*config=$configPath*state=$pushState*" }).Count -lt 1) {
    throw "project dashboard did not use configured state path"
}
if (($projectDashboardOutput | Where-Object { $_ -like "transfers*total=2*completed=2*failed=0*" }).Count -lt 1) {
    throw "project dashboard did not expose SQLite transfer summary"
}
if (($projectDashboardOutput | Where-Object { $_ -like "summary*groups=2*offset=0*limit=1*total_groups=2*has_more=true*" }).Count -lt 1) {
    throw "project dashboard did not expose paged project asset summary"
}
if (($projectDashboardOutput | Where-Object { $_ -like "asset*username=verify*source=Verify Camera*" }).Count -lt 1) {
    throw "project dashboard did not expose project asset rows"
}
Write-Output $projectDashboardOutput

$dashboardOutput = & (Join-Path $root "target\debug\camera-connector.exe") dashboard --config $configPath --state $pushState --username "verify" --limit 1
if ($LASTEXITCODE -ne 0) { throw "dashboard smoke failed" }
if (($dashboardOutput | Where-Object { $_ -like "summary*groups=2*offset=0*limit=1*total_groups=2*has_more=true*" }).Count -lt 1) {
    throw "dashboard did not expose paged asset summary"
}
if (($dashboardOutput | Where-Object { $_ -like "account*username=verify*device=Verify Camera*password_configured=true*online=*connections=*remote=*" }).Count -lt 1) {
    throw "dashboard did not expose account summary"
}
if (($dashboardOutput | Where-Object { $_ -like "paths*config=$configPath*state=$pushState*" }).Count -lt 1) {
    throw "dashboard did not expose system paths"
}
if (($dashboardOutput | Where-Object { $_ -like "transfers*total=3*completed=2*failed=1*" }).Count -lt 1) {
    throw "dashboard did not expose transfer summary"
}
if (($dashboardOutput | Where-Object { $_ -like "failure*ftp:verify-failed*Failed*error=simulated failure*" }).Count -lt 1) {
    throw "dashboard did not expose recent failure rows"
}
if (($dashboardOutput | Where-Object { $_ -like "asset*username=verify*source=Verify Camera*" }).Count -lt 1) {
    throw "dashboard did not expose filtered asset rows"
}
Write-Output $dashboardOutput

$dashboardJsonOutput = & (Join-Path $root "target\debug\camera-connector.exe") dashboard --config $configPath --state $pushState --username "verify" --limit 1 --json
if ($LASTEXITCODE -ne 0) { throw "dashboard json smoke failed" }
$dashboardJson = $dashboardJsonOutput -join "`n" | ConvertFrom-Json
if ($dashboardJson.assets.summary.group_count -ne 2) {
    throw "dashboard json did not expose asset summary"
}
if ($dashboardJson.accounts[0].username -ne "verify" -or $dashboardJson.accounts[0].password_configured -ne $true) {
    throw "dashboard json did not expose safe account summary"
}
if ($null -eq $dashboardJson.accounts[0].online -or $null -eq $dashboardJson.accounts[0].active_connections) {
    throw "dashboard json did not expose account connection state"
}
if ($dashboardJson.paths.config_path -ne $configPath -or $dashboardJson.paths.state_dir -ne $pushState) {
    throw "dashboard json did not expose system paths"
}
if ($dashboardJson.transfers.total_count -ne 3 -or $dashboardJson.transfers.completed_count -ne 2 -or $dashboardJson.transfers.failed_count -ne 1) {
    throw "dashboard json did not expose transfer summary"
}
if ($dashboardJson.recent_failures.Count -ne 1 -or $dashboardJson.recent_failures[0].record.error -ne "simulated failure") {
    throw "dashboard json did not expose recent failures"
}
if (($dashboardJsonOutput -join "`n") -like "*password_hash*") {
    throw "dashboard json exposed password hash"
}
if ($dashboardJson.assets.groups[0].primary.username -ne "verify") {
    throw "dashboard json did not expose filtered asset username"
}
if ($dashboardJson.assets.groups[0].primary.duplicate_index -ne 2 -or $dashboardJson.assets.groups[0].primary.duplicate_count -ne 2) {
    throw "dashboard json did not expose duplicate metadata"
}
Write-Output $dashboardJsonOutput

& (Join-Path $root "target\debug\camera-connector.exe") devices --config $configPath --state $pushState
if ($LASTEXITCODE -ne 0) { throw "devices smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") devices --config $configPath --state $ftpSmokeState --username "verify"
if ($LASTEXITCODE -ne 0) { throw "devices username filter smoke failed" }

$transfersOutput = & (Join-Path $root "target\debug\camera-connector.exe") transfers --config $configPath --state $pushState --username "verify" --source-name "Verify Camera" --original-path IMG_1234
if ($LASTEXITCODE -ne 0) { throw "transfers smoke failed" }
if (($transfersOutput | Where-Object { $_ -like "*username=verify*display=Verify Camera/IMG_1234.CR3*" }).Count -lt 1) {
    throw "transfers display path smoke failed"
}
Write-Output $transfersOutput

$failedTransfersOutput = & (Join-Path $root "target\debug\camera-connector.exe") transfers --config $configPath --state $pushState --username "verify" --status failed
if ($LASTEXITCODE -ne 0) { throw "failed transfers status smoke failed" }
if (($failedTransfersOutput | Where-Object { $_ -like "*ftp:verify-failed*Failed*error=simulated failure*" }).Count -ne 1) {
    throw "failed transfers status filter did not return expected failure"
}
Write-Output $failedTransfersOutput

$received = Get-Item -LiteralPath (Join-Path $pushOutput "IMG_1234.CR3")
$duplicate = Get-Item -LiteralPath (Join-Path $pushOutput "IMG_1234 (1).CR3")
if ($received.Length -le 0) { throw "received output is empty" }
if ($duplicate.Length -le 0) { throw "duplicate output is empty" }

if (!(Test-Path -LiteralPath $transferLog)) { throw "transfer log was not written" }
$transferRecords = @(Get-Content -LiteralPath $transferLog | ForEach-Object { $_ | ConvertFrom-Json })
if ($transferRecords.Count -ne 3) { throw "expected 3 transfer log records, found $($transferRecords.Count)" }
if (($transferRecords | Where-Object { $_.username -eq "verify" -and $_.source_name -eq "Verify Camera" }).Count -ne 3) {
    throw "transfer log username/source_name was not recorded"
}

Write-Output "verify.ps1 completed successfully"

