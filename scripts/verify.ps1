$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$vsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$pushInput = Join-Path $root "target\push-input"
$pushOutput = Join-Path $root "target\push-output"

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
New-Item -ItemType Directory -Force -Path $pushInput | Out-Null
New-Item -ItemType Directory -Force -Path $pushOutput | Out-Null

$sample = Join-Path $pushInput "IMG_1234.CR3"
[System.IO.File]::WriteAllBytes($sample, [byte[]](1, 2, 3, 4, 5))

& (Join-Path $root "target\debug\camera-connector.exe") receiver-config --protocol ftp --output $pushOutput --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "receiver-config smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") receive-file --input $sample --output $pushOutput --source ftp --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "receive-file smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") receive-file --input $sample --output $pushOutput --source ftp --source-name "Verify Camera"
if ($LASTEXITCODE -ne 0) { throw "duplicate receive-file smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") inbox --path $pushOutput --source ftp
if ($LASTEXITCODE -ne 0) { throw "inbox smoke failed" }

& (Join-Path $root "target\debug\camera-connector.exe") transfers --path $pushOutput --source-name "Verify Camera" --original-path IMG_1234
if ($LASTEXITCODE -ne 0) { throw "transfers smoke failed" }

$received = Get-Item -LiteralPath (Join-Path $pushOutput "IMG_1234.CR3")
$duplicate = Get-Item -LiteralPath (Join-Path $pushOutput "IMG_1234 (1).CR3")
if ($received.Length -le 0) { throw "received output is empty" }
if ($duplicate.Length -le 0) { throw "duplicate output is empty" }

$transferLog = Join-Path $pushOutput "transfer-log.jsonl"
if (!(Test-Path -LiteralPath $transferLog)) { throw "transfer log was not written" }
$transferRecords = Get-Content -LiteralPath $transferLog | ForEach-Object { $_ | ConvertFrom-Json }
if ($transferRecords.Count -ne 2) { throw "expected 2 transfer log records, found $($transferRecords.Count)" }
if (($transferRecords | Where-Object { $_.source_name -eq "Verify Camera" }).Count -ne 2) {
    throw "transfer log source_name was not recorded"
}

Write-Output "verify.ps1 completed successfully"

