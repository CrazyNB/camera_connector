param(
    [string]$Version = "9.5.1",
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA "CameraConnectorToolchains"),
    [switch]$Force
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($InstallRoot)) {
    throw "InstallRoot is required."
}

$resolvedInstallRoot = [System.IO.Path]::GetFullPath($InstallRoot)
$gradleHome = [System.IO.Path]::GetFullPath((Join-Path $resolvedInstallRoot "gradle-$Version"))
$gradleBat = Join-Path $gradleHome "bin\gradle.bat"
$downloadDir = [System.IO.Path]::GetFullPath((Join-Path $resolvedInstallRoot "downloads"))
$zipPath = Join-Path $downloadDir "gradle-$Version-bin.zip"
$extractDir = [System.IO.Path]::GetFullPath((Join-Path $resolvedInstallRoot ".gradle-$Version-extract"))

foreach ($path in @($gradleHome, $downloadDir, $extractDir)) {
    if (-not $path.StartsWith($resolvedInstallRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to operate outside install root: $path"
    }
}

if ((Test-Path -LiteralPath $gradleBat -PathType Leaf) -and -not $Force) {
    Write-Host "Gradle $Version already available at $gradleBat"
    Write-Output $gradleBat
    exit 0
}

New-Item -ItemType Directory -Force -Path $downloadDir | Out-Null

if ((-not (Test-Path -LiteralPath $zipPath -PathType Leaf)) -or $Force) {
    $url = "https://services.gradle.org/distributions/gradle-$Version-bin.zip"
    Write-Host "Downloading Gradle $Version from $url"
    Invoke-WebRequest -Uri $url -OutFile $zipPath
}

if (Test-Path -LiteralPath $extractDir) {
    Remove-Item -LiteralPath $extractDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null

Write-Host "Extracting Gradle $Version to $resolvedInstallRoot"
Expand-Archive -LiteralPath $zipPath -DestinationPath $extractDir -Force

$expandedHome = Join-Path $extractDir "gradle-$Version"
$expandedGradleBat = Join-Path $expandedHome "bin\gradle.bat"
if (-not (Test-Path -LiteralPath $expandedGradleBat -PathType Leaf)) {
    throw "Downloaded Gradle archive did not contain bin\gradle.bat."
}

if (Test-Path -LiteralPath $gradleHome) {
    Remove-Item -LiteralPath $gradleHome -Recurse -Force
}
Move-Item -LiteralPath $expandedHome -Destination $gradleHome
Remove-Item -LiteralPath $extractDir -Recurse -Force

if (-not (Test-Path -LiteralPath $gradleBat -PathType Leaf)) {
    throw "Gradle install failed: $gradleBat was not created."
}

Write-Host "Gradle $Version installed at $gradleBat"
Write-Output $gradleBat
