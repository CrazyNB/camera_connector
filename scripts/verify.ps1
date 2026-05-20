$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$vsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$pushInput = Join-Path $root "target\push-input"
$pushOutput = Join-Path $root "target\push-output"
$ftpSmokeOutput = Join-Path $root "target\ftp-smoke-output"
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

if (Test-Path $pushInput) {
    Remove-Item -LiteralPath $pushInput -Recurse -Force
}
if (Test-Path $pushOutput) {
    Remove-Item -LiteralPath $pushOutput -Recurse -Force
}
if (Test-Path $ftpSmokeOutput) {
    Remove-Item -LiteralPath $ftpSmokeOutput -Recurse -Force
}
if (Test-Path $configPath) {
    Remove-Item -LiteralPath $configPath -Force
}
New-Item -ItemType Directory -Force -Path $pushInput | Out-Null
New-Item -ItemType Directory -Force -Path $pushOutput | Out-Null
New-Item -ItemType Directory -Force -Path $ftpSmokeOutput | Out-Null

$sample = Join-Path $pushInput "IMG_1234.CR3"
[System.IO.File]::WriteAllBytes($sample, [byte[]](1, 2, 3, 4, 5))

& (Join-Path $root "target\debug\camera-connector.exe") account --config $configPath set --username "verify" --password "secret" --device-name "Verify Camera" --ip "192.168.137.56"
if ($LASTEXITCODE -ne 0) { throw "account set smoke failed" }

$accountList = & (Join-Path $root "target\debug\camera-connector.exe") account --config $configPath list
if ($LASTEXITCODE -ne 0) { throw "account list smoke failed" }
if (($accountList | Where-Object { $_ -like "*verify*Verify Camera*192.168.137.56*" }).Count -lt 1) {
    throw "account list did not include Verify Camera"
}
Write-Output $accountList

& (Join-Path $root "target\debug\camera-connector.exe") receiver-config --config $configPath --protocol ftp --output $pushOutput
if ($LASTEXITCODE -ne 0) { throw "receiver-config smoke failed" }

$serverOut = Join-Path $root "target\ftp-smoke.out.log"
$serverErr = Join-Path $root "target\ftp-smoke.err.log"
$server = Start-Process -FilePath (Join-Path $root "target\debug\camera-connector.exe") `
    -ArgumentList @("serve-ftp", "--config", $configPath, "--bind-host", "127.0.0.1", "--port", "2221", "--output", $ftpSmokeOutput) `
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
$ftpTransferLog = Join-Path $ftpSmokeOutput "transfer-log.jsonl"
$ftpTransferRecords = @(Get-Content -LiteralPath $ftpTransferLog | ForEach-Object { $_ | ConvertFrom-Json })
if (@($ftpTransferRecords | Where-Object { $_.source_name -eq "Verify Camera" -and $_.remote_addr -eq "127.0.0.1" }).Count -ne 1) {
    throw "FTP smoke transfer log did not record account source and remote address"
}

& (Join-Path $root "target\debug\camera-connector.exe") receive-file --input $sample --output $pushOutput --source ftp --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "receive-file smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") receive-file --input $sample --output $pushOutput --source ftp --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "duplicate receive-file smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") inbox --path $pushOutput --source ftp
if ($LASTEXITCODE -ne 0) { throw "inbox smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") devices --config $configPath --path $pushOutput
if ($LASTEXITCODE -ne 0) { throw "devices smoke failed" }

$transfersOutput = & (Join-Path $root "target\debug\camera-connector.exe") transfers --config $configPath --path $pushOutput --source-name "Verify Camera" --original-path IMG_1234
if ($LASTEXITCODE -ne 0) { throw "transfers smoke failed" }
if (($transfersOutput | Where-Object { $_ -like "*display=Verify Camera/IMG_1234.CR3*" }).Count -lt 1) {
    throw "transfers display path smoke failed"
}
Write-Output $transfersOutput

$received = Get-Item -LiteralPath (Join-Path $pushOutput "IMG_1234.CR3")
$duplicate = Get-Item -LiteralPath (Join-Path $pushOutput "IMG_1234 (1).CR3")
if ($received.Length -le 0) { throw "received output is empty" }
if ($duplicate.Length -le 0) { throw "duplicate output is empty" }

$transferLog = Join-Path $pushOutput "transfer-log.jsonl"
if (!(Test-Path -LiteralPath $transferLog)) { throw "transfer log was not written" }
$transferRecords = @(Get-Content -LiteralPath $transferLog | ForEach-Object { $_ | ConvertFrom-Json })
if ($transferRecords.Count -ne 2) { throw "expected 2 transfer log records, found $($transferRecords.Count)" }
if (($transferRecords | Where-Object { $_.source_name -eq "Verify Camera" }).Count -ne 2) {
    throw "transfer log source_name was not recorded"
}

Write-Output "verify.ps1 completed successfully"

