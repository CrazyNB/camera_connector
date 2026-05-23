param(
    [string]$ApkPath,
    [string]$ReportPath
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
if (-not $ApkPath) {
    $ApkPath = Join-Path $root "apps\android\app\build\outputs\apk\debug\app-debug.apk"
}
if (-not $ReportPath) {
    $ReportPath = Join-Path $root "target\android-apk-report.txt"
}

if (-not (Test-Path -LiteralPath $ApkPath -PathType Leaf)) {
    throw "Android APK not found: $ApkPath"
}

Add-Type -AssemblyName System.IO.Compression.FileSystem

$requiredEntries = @(
    "AndroidManifest.xml",
    "classes.dex",
    "lib/arm64-v8a/libcamera_connector_ffi.so",
    "lib/x86_64/libcamera_connector_ffi.so"
)

$apkItem = Get-Item -LiteralPath $ApkPath
$apkHash = Get-FileHash -Algorithm SHA256 -LiteralPath $ApkPath
$report = New-Object System.Collections.Generic.List[string]
$report.Add("apk: $ApkPath")
$report.Add("size_bytes: $($apkItem.Length)")
$report.Add("sha256: $($apkHash.Hash)")

$zip = [System.IO.Compression.ZipFile]::OpenRead($ApkPath)
try {
    foreach ($entryName in $requiredEntries) {
        $entry = $zip.Entries | Where-Object { $_.FullName -eq $entryName } | Select-Object -First 1
        if ($null -eq $entry -or $entry.Length -le 0) {
            throw "Android APK is missing required entry: $entryName"
        }
        $report.Add("entry: $entryName size_bytes=$($entry.Length)")
    }
} finally {
    $zip.Dispose()
}

$reportDir = Split-Path -Parent $ReportPath
if ($reportDir) {
    New-Item -ItemType Directory -Force -Path $reportDir | Out-Null
}
$report | Set-Content -LiteralPath $ReportPath -Encoding utf8

Write-Host "Android APK report written to $ReportPath"
