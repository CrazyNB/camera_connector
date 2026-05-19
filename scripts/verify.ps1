$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$vsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$mockOutput = Join-Path $root "target\mock-output"

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

if (Test-Path $mockOutput) {
    Remove-Item -LiteralPath $mockOutput -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $mockOutput | Out-Null

$mock = Start-Process `
    -FilePath (Join-Path $root "target\debug\mock-camera.exe") `
    -ArgumentList "--host", "127.0.0.1", "--port", "15740" `
    -WindowStyle Hidden `
    -PassThru `
    -RedirectStandardOutput (Join-Path $root "target\mock-camera.out.log") `
    -RedirectStandardError (Join-Path $root "target\mock-camera.err.log")

try {
    Start-Sleep -Seconds 1

    & (Join-Path $root "target\debug\nikon-importer.exe") scan --subnet 127.0.0.1/32 --port 15740 --timeout-ms 500 --concurrency 1
    if ($LASTEXITCODE -ne 0) { throw "scan smoke failed" }

    & (Join-Path $root "target\debug\nikon-importer.exe") info --host 127.0.0.1 --port 15740
    if ($LASTEXITCODE -ne 0) { throw "info smoke failed" }

    & (Join-Path $root "target\debug\nikon-importer.exe") list --host 127.0.0.1 --port 15740
    if ($LASTEXITCODE -ne 0) { throw "list smoke failed" }

    & (Join-Path $root "target\debug\nikon-importer.exe") thumb --host 127.0.0.1 --port 15740 --handle 101 --output (Join-Path $mockOutput "thumb.jpg")
    if ($LASTEXITCODE -ne 0) { throw "thumb smoke failed" }

    & (Join-Path $root "target\debug\nikon-importer.exe") pull --host 127.0.0.1 --port 15740 --handle 102 --output $mockOutput
    if ($LASTEXITCODE -ne 0) { throw "pull smoke failed" }

    $thumb = Get-Item -LiteralPath (Join-Path $mockOutput "thumb.jpg")
    $raw = Get-Item -LiteralPath (Join-Path $mockOutput "DSC_1234.NEF")
    if ($thumb.Length -le 0) { throw "thumb output is empty" }
    if ($raw.Length -le 0) { throw "pull output is empty" }

    Write-Output "verify.ps1 completed successfully"
} finally {
    if ($mock -and -not $mock.HasExited) {
        Stop-Process -Id $mock.Id -Force
    }
}
